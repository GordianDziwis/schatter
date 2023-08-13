use opencv::core::{Point, Ptr, Vector};
use opencv::imgproc::{bounding_rect, contour_area, rectangle, threshold, LINE_AA};
use opencv::prelude::*;
use opencv::tracking::TrackerKCF;
use opencv::video::{create_background_subtractor_mog2, BackgroundSubtractorMOG2};
use opencv::videoio::CAP_PROP_AUTO_EXPOSURE;
use opencv::{core, highgui, imgproc, types, videoio, Result};

pub struct VideoProcessor {
    mask_window: String,
    output_window: String,
    cam: videoio::VideoCapture,
    background_subtractor: Ptr<BackgroundSubtractorMOG2>,
    frame: Mat,
    output: Mat,
    mask: Mat,
    trackers: Vec<(Ptr<TrackerKCF>, core::Rect)>,
}

impl VideoProcessor {
    pub fn new() -> Result<Self, opencv::Error> {
        highgui::named_window(&"Mask".to_string(), highgui::WINDOW_AUTOSIZE)?;
        highgui::named_window(&"Output".to_string(), highgui::WINDOW_AUTOSIZE)?;
        let params: Vector<i32> = Vector::from_slice(&[CAP_PROP_AUTO_EXPOSURE, 3]);
        Ok(Self {
            mask_window: "Mask".to_string(),
            output_window: "Output".to_string(),
            cam: videoio::VideoCapture::new_with_params(0, videoio::CAP_ANY, &params)?,
            background_subtractor: create_background_subtractor_mog2(10000, 40.0, true)?,
            frame: Mat::default(),
            output: Mat::default(),
            mask: Mat::default(),
            trackers: Vec::new(),
        })
    }

    pub fn process_frames(&mut self) {
        loop {
            self.process_frame().unwrap();
        }
    }

    fn subtract_background(&mut self) {
        BackgroundSubtractorMOG2Trait::apply(
            &mut self.background_subtractor,
            &self.frame,
            &mut self.mask,
            0.0001,
        )
        .unwrap();
        let mut thresh = Mat::default();
        threshold(&self.mask, &mut thresh, 245.0, 255.0, 0).unwrap();
        self.mask = thresh;
    }

    fn clean_trackers(&mut self) {
        let mut i = 0;
        while i < self.trackers.len() {
            let (tracker, rect) = &mut self.trackers[i];
            if tracker.update(&self.frame, rect).unwrap() {
                rectangle(
                    &mut self.mask,
                    *rect,
                    core::Scalar::new(0., 0., 0., 0.),
                    -1,
                    LINE_AA,
                    0,
                )
                .unwrap();
                rectangle(
                    &mut self.output,
                    *rect,
                    core::Scalar::new(255., 0., 0., 255.),
                    1,
                    LINE_AA,
                    0,
                )
                .unwrap();
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
            if contour_area(contour, false).unwrap_or(0.0) >= 8000.0 {
                let rectangl = bounding_rect(contour).unwrap();
                let param = opencv::tracking::TrackerKCF_Params::default().unwrap();
                let mut tracker = TrackerKCF::create(param).unwrap();
                tracker.init(&self.frame, rectangl).unwrap();
                self.trackers.push((tracker, rectangl));
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

            self.clean_trackers();

            self.process_contours();

            highgui::imshow(&self.mask_window, &self.output)?;
            highgui::imshow(&self.output_window, &self.mask)?;
        }

        let _key = highgui::wait_key(10)?;
        Ok(())
    }
}
