use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nannou::prelude::Point2;
use opencv::core::{Point, Ptr, Rect, Size, Vector, CV_8UC3};
use opencv::imgproc::{bounding_rect, contour_area, rectangle, threshold, LINE_AA};
use opencv::prelude::*;
use opencv::tracking::TrackerKCF;
use opencv::video::{create_background_subtractor_mog2, BackgroundSubtractorMOG2};
use opencv::videoio::{
    CAP_FFMPEG, CAP_GSTREAMER, CAP_PROP_AUTO_EXPOSURE, CAP_PROP_BUFFERSIZE, CAP_PROP_FPS,
    CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_PROP_POS_FRAMES,
};
use opencv::{core, highgui, imgproc, types, videoio, Result};
use parry3d::math::Real;

const URL2: &str = "rtspsrc location=rtsp://schatter:titstits@192.168.1.177:554/stream2 \
                    latency=100 !  rtph264depay ! h264parse ! avdec_h264 ! videoconvert ! \
                    video/x-raw,format=BGRx !  appsink";
const URL: &str = "rtspsrc location=rtsp://schatter:titstits@192.168.1.178:554/stream2 \
                   latency=100 ! rtph264depay ! h264parse ! avdec_h264 ! videoconvert ! \
                   video/x-raw,format=BGR ! appsink max-buffers=1 drop=true";
const MAX_TRACKER_COUNT: usize = 4;
const TIME_TO_KILL: u64 = 10;
const MIN_CONTOUR_AREA: f64 = 400.0;

pub struct VideoProcessor {
    mask_window: String,
    output_window: String,
    cam: videoio::VideoCapture,
    background_subtractor: Ptr<BackgroundSubtractorMOG2>,
    frame: Mat,
    output: Mat,
    mask: Mat,
    trackers: Vec<(Ptr<TrackerKCF>, core::Rect, Instant)>,
    pub position: Arc<Mutex<Point2>>,
}

impl VideoProcessor {
    fn calculate_position(&self) -> Point2 {
        match self.trackers.get(0) {
            Some((_, rect, _)) => Point2::new(
                (rect.x + rect.width / 2) as f32,
                (rect.y + rect.height / 2) as f32,
            ),
            None => Point2::new(450.0, 129.0),
        }
    }

    pub fn new(position: Arc<Mutex<Point2>>) -> Result<Self, opencv::Error> {
        println!("This will take some time");
        // let param = opencv::tracking::TrackerKCF_Params::default().unwrap();
        // let mut tracker = TrackerKCF::create(param).unwrap();
        // let mut frame = Mat::new_size_with_default(
        //     Size::new(640, 480),
        //     CV_8UC3,
        //     opencv::core::Scalar::all(0.0),
        // )?;
        println!("Done");

        let rect = core::Rect::new(50, 50, 200, 200);
        // tracker.init(&frame, rect).unwrap();

        highgui::named_window(&"Mask".to_string(), highgui::WINDOW_AUTOSIZE)?;
        highgui::named_window(&"Output".to_string(), highgui::WINDOW_AUTOSIZE)?;
        // let params: Vector<i32> = Vector::from_slice(&[CAP_PROP_AUTO_EXPOSURE, 3]);
        let params: Vector<i32> = Vector::from_slice(&[
            CAP_PROP_BUFFERSIZE,
            1,
            CAP_PROP_FRAME_WIDTH,
            640,
            CAP_PROP_FRAME_HEIGHT,
            480,
        ]);
        let params: Vector<i32> = Vector::new();
        let cam = videoio::VideoCapture::from_file_with_params(URL, CAP_GSTREAMER, &params)?;
        let opened = videoio::VideoCapture::is_opened(&cam)?;
        if !opened {
            panic!("Unable to open default camera!");
        }

        Ok(Self {
            mask_window: "Mask".to_string(),
            output_window: "Output".to_string(),
            cam,
            background_subtractor: create_background_subtractor_mog2(1000, 50.0, true)?,
            frame: Mat::default(),
            output: Mat::default(),
            mask: Mat::default(),
            trackers: Vec::new(),
            position,
        })
    }

    pub fn process_frames(&mut self) {
        loop {
            self.process_frame().unwrap();
            {
                let mut position = self.position.lock().unwrap();
                *position = self.calculate_position(); // Assuming calculate_position() is a method that calculates the position
            } // The lock is dropped here, allowing other threads to access `position`
              // println!("{:?}", self.position);
        }
    }

    fn subtract_background(&mut self) {
        BackgroundSubtractorMOG2Trait::apply(
            &mut self.background_subtractor,
            &self.frame,
            &mut self.mask,
            -1.0,
        )
        .unwrap();
        let mut thresh = Mat::default();
        threshold(&self.mask, &mut thresh, 245.0, 255.0, 0).unwrap();
        self.mask = thresh;
        let rect = Rect::new(274, 108, 50, 140);
        rectangle(
            &mut self.mask,
            rect,
            core::Scalar::new(0., 0., 0., 0.),
            -1,
            LINE_AA,
            0,
        )
        .unwrap();
    }

    fn process_trackers(&mut self) {
        let mut i = 0;
        while i < self.trackers.len() {
            let (tracker, rect, time) = &mut self.trackers[i];
            if tracker.update(&self.frame, rect).unwrap()
                && time.elapsed() < Duration::from_secs(TIME_TO_KILL)
            {
                rectangle(
                    &mut self.mask,
                    *rect,
                    core::Scalar::new(0., 0., 0., 0.),
                    -1,
                    LINE_AA,
                    0,
                )
                .unwrap();
                if i == 0 {
                    rectangle(
                        &mut self.output,
                        *rect,
                        core::Scalar::new(255., 255., 255., 255.),
                        1,
                        LINE_AA,
                        0,
                    )
                    .unwrap();
                } else {
                    rectangle(
                        &mut self.output,
                        *rect,
                        core::Scalar::new(255., 0., 0., 255.),
                        1,
                        LINE_AA,
                        0,
                    )
                    .unwrap();
                }

                i += 1;
            } else {
                self.trackers.remove(i);
            }
        }
    }

    fn process_contours(&mut self) {
        let mut contours = types::VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &self.mask,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_TC89_L1,
            Point::default(),
        )
        .unwrap();

        let mut i = 0;
        while i < contours.len() {
            let contour = &contours.get(i).unwrap();
            if contour_area(contour, false).unwrap_or(0.0) >= MIN_CONTOUR_AREA
                && self.trackers.len() < MAX_TRACKER_COUNT
            {
                let rectangl = bounding_rect(contour).unwrap();
                let param = opencv::tracking::TrackerKCF_Params::default().unwrap();
                let mut tracker = TrackerKCF::create(param).unwrap();
                tracker.init(&self.frame, rectangl).unwrap();
                self.trackers.push((tracker, rectangl, Instant::now()));
                rectangle(
                    &mut self.output,
                    rectangl,
                    core::Scalar::new(0., 255., 0., 255.),
                    1,
                    LINE_AA,
                    0,
                )
                .unwrap();
                i += 1;
            } else {
                contours.remove(i).unwrap();
            }
        }

        imgproc::draw_contours(
            &mut self.output,
            &contours,
            -1,
            core::Scalar::new(0., 0., 255., 0.),
            1,
            imgproc::LINE_8,
            &Mat::default(),
            100,
            Point::default(),
        )
        .unwrap();
    }

    fn process_frame(&mut self) -> Result<(), opencv::Error> {
        self.cam.read(&mut self.frame)?;
        self.output = self.frame.clone();
        if self.frame.size()?.width > 0 {
            self.subtract_background();

            self.process_trackers();

            self.process_contours();

            highgui::imshow(&self.mask_window, &self.output)?;
            highgui::imshow(&self.output_window, &self.mask)?;
        }

        let _key = highgui::wait_key(10)?;
        Ok(())
    }
}
