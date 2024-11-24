use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nannou::prelude::Point2;
use opencv::core::{Point, Ptr, Rect, Vector};
use opencv::imgproc::{bounding_rect, contour_area, rectangle, threshold, LINE_AA};
use opencv::prelude::*;
use opencv::tracking::TrackerKCF;
use opencv::video::{create_background_subtractor_mog2, BackgroundSubtractorMOG2};
use opencv::videoio::CAP_GSTREAMER;
use opencv::{core, highgui, imgproc, types, videoio, Result};

const URL: &str = "rtspsrc location=rtsp://schatter:titstits@192.168.1.178:554/stream2 \
                   latency=100 ! rtph264depay ! h264parse ! avdec_h264 ! videoconvert ! \
                   video/x-raw,format=BGR ! appsink max-buffers=1 drop=true";
const MAX_TRACKER_COUNT: usize = 1;
const TIME_TO_KILL: u64 = 3;
const MIN_CONTOUR_AREA: f64 = 500.0;
const BACKGROUND_THRESHOLD: f64 = 30.0;
const BACKGROUND_LEARNING_RATE: f64 = -1.0;

pub struct VideoProcessor {
    mask_window: String,
    output_window: String,
    cam: videoio::VideoCapture,
    background_subtractor: Ptr<BackgroundSubtractorMOG2>,
    frame: Mat,
    output: Mat,
    mask: Mat,
    trackers: Vec<(Ptr<TrackerKCF>, core::Rect, Instant)>,
    pub position: Arc<Mutex<Option<Point2>>>,
    pub new: Arc<Mutex<bool>>,
}

impl VideoProcessor {
    fn calculate_position(&self) -> Option<Point2> {
        match self.trackers.get(0) {
            Some((_, rect, _)) => Some(Point2::new(
                (rect.x + rect.width / 2) as f32,
                (rect.y + rect.height / 2) as f32,
            )),
            None => None,
        }
    }

    pub fn new(
        position: Arc<Mutex<Option<Point2>>>,
        new: Arc<Mutex<bool>>,
    ) -> Result<Self, opencv::Error> {
        highgui::named_window(&"Mask".to_string(), highgui::WINDOW_AUTOSIZE)?;
        highgui::named_window(&"Output".to_string(), highgui::WINDOW_AUTOSIZE)?;
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
            background_subtractor: create_background_subtractor_mog2(
                1000,
                BACKGROUND_THRESHOLD,
                true,
            )?,
            frame: Mat::default(),
            output: Mat::default(),
            mask: Mat::default(),
            trackers: Vec::new(),
            position,
            new,
        })
    }

    pub fn process_frames(&mut self) {
        loop {
            self.process_frame().unwrap();
            {
                let mut position = self.position.lock().unwrap();
                *position = self.calculate_position();
            }
        }
    }

    fn subtract_background(&mut self) {
        BackgroundSubtractorMOG2Trait::apply(
            &mut self.background_subtractor,
            &self.frame,
            &mut self.mask,
            BACKGROUND_LEARNING_RATE,
        )
        .unwrap();
        let mut thresh = Mat::default();
        threshold(&self.mask, &mut thresh, 245.0, 255.0, 0).unwrap();
        self.mask = thresh;
        let rect = Rect::new(104, 93, 180, 140);
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

                let mut new = self.new.lock().unwrap();
                *new = true;
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
