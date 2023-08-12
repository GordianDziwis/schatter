mod monolith;
mod motion_tracker;

use nannou::prelude::*;
use opencv::core::{Point, Ptr, Vector};
use opencv::imgproc::{
    bounding_rect, contour_area, find_contours, rectangle, threshold, LINE_AA, RETR_TREE,
    THRESH_BINARY,
};
use opencv::prelude::*;
use opencv::tracking::{TrackerKCF, TrackerKCF_Params};
use opencv::video::{create_background_subtractor_mog2, BackgroundSubtractorMOG2};
use opencv::videoio::{VideoCaptureProperties, CAP_PROP_FPS, CAP_PROP_MONOCHROME};
use opencv::{core, features2d, highgui, imgproc, types, videoio, Result};

use crate::monolith::Monolith;
use crate::motion_tracker::MotionTracker;

fn main() -> Result<()> {
    let input_window = "input";
    let output_window = "video capture";
    highgui::named_window(output_window, highgui::WINDOW_AUTOSIZE)?;
    highgui::named_window(output_window, highgui::WINDOW_AUTOSIZE)?;
    let test: opencv::core::Vector<i32> =
        opencv::core::Vector::from_slice(&[CAP_PROP_FPS, 10, 21, 3]);
    let mut cam = videoio::VideoCapture::new_with_params(0, videoio::CAP_ANY, &test)?; // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }

    let brightness = cam.get(21)?;
    println!("{}", brightness);
    let x = 21;
    for i in 0..255 {
        if cam.set(x, i as f64)? {
            println!("{} can be set to {}", x, i);
        }
    }
    let mut background_subtractor: Ptr<BackgroundSubtractorMOG2> =
        create_background_subtractor_mog2(10000, 40.0, true)?;
    let mut frame = Mat::default();
    let mut debug_frame = Mat::default();
    let mut mask = Mat::default();
    let mut thresh = Mat::default();
    let mut vec = types::VectorOfVectorOfPoint::new();
    let mut trackers: Vec<(Ptr<TrackerKCF>, core::Rect)> = Vec::new();
    loop {
        cam.read(&mut frame)?;
        debug_frame = frame.clone();
        if frame.size()?.width > 0 {
            BackgroundSubtractorMOG2Trait::apply(
                &mut background_subtractor,
                &frame,
                &mut mask,
                0.005,
            )?;

            let mut to_remove = Vec::new();
            for (i, (tracker, rect)) in trackers.iter_mut().enumerate() {
                let lost = !tracker.update(&frame, rect)?;
                if lost {
                    to_remove.push(i);
                } else {
                    rectangle(
                        &mut mask,
                        *rect,
                        core::Scalar::new(0., 0., 0., 0.),
                        -1,
                        LINE_AA,
                        0,
                    )?;
                    rectangle(
                        &mut debug_frame,
                        *rect,
                        core::Scalar::new(255., 0., 0., 255.),
                        1,
                        LINE_AA,
                        0,
                    )?;
                }
            }
            for i in to_remove.iter().rev() {
                trackers.remove(*i);
            }

            threshold(&mask, &mut thresh, 245.0, 255.0, 0)?;
            imgproc::find_contours(
                &thresh,
                &mut vec,
                imgproc::RETR_EXTERNAL,
                imgproc::CHAIN_APPROX_TC89_L1,
                Point::default(),
            )?;
            // for cnt in vec.iter() {
            //     println!("Cols: {}", cnt.);
            // ... and so on for the other fields you're interested in
            // }

            let mut temp_vec = types::VectorOfVectorOfPoint::new();
            let mut rect = types::VectorOfRect::new();

            for cnt in vec.iter() {
                if contour_area(&cnt, false).unwrap_or(0.0) >= 2000.0 {
                    temp_vec.push(cnt.clone());
                    let rectangl = bounding_rect(&cnt)?;
                    rect.push(rectangl);
                    let param: opencv::tracking::TrackerKCF_Params =
                        opencv::tracking::TrackerKCF_Params::default().unwrap();
                    let mut tracker = TrackerKCF::create(param)?;
                    tracker.init(&frame, rectangl)?;
                    trackers.push((tracker, rectangl));
                    rectangle(
                        &mut debug_frame,
                        rectangl,
                        core::Scalar::new(0., 255., 0., 255.),
                        1,
                        LINE_AA,
                        0,
                    )?;
                }
            }

            vec = temp_vec;
            println!("contours: {}", vec.len());
            imgproc::draw_contours(
                &mut frame,
                &vec,
                -1,
                core::Scalar::new(0., 0., 255., 0.),
                1,
                imgproc::LINE_8,
                &Mat::default(),
                100,
                Point::default(),
            )?;
            highgui::imshow(input_window, &debug_frame)?;
            highgui::imshow(output_window, &thresh)?;
        }

        let key = highgui::wait_key(10)?;
    }
    Ok(())
}
// fn main() {
//     nannou::app(model).update(update).run();
// }

struct Model {
    monolith: Monolith,
    motion_tracker: MotionTracker,
}

fn model(app: &App) -> Model {
    let _window_id_motiontracker = app
        .new_window()
        .title("MotionTracker")
        .resizable(false)
        .view(view)
        .build()
        .unwrap();

    let window_id_monolith = app
        .new_window()
        .title("Monolith")
        .size(449, 838)
        .resizable(false)
        .view(view2)
        .build()
        .unwrap();

    Model {
        monolith: Monolith::new(app, window_id_monolith),
        motion_tracker: MotionTracker::new(app),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.motion_tracker.update(app);
    model.monolith.update(app, &model.motion_tracker);
    println!("{}", app.fps());
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let texture = &model.motion_tracker.texture;
    draw.texture(texture);
    draw.to_frame(app, &frame).unwrap();
}

fn view2(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model
        .monolith
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}
