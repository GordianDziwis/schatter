mod camera_wrapper;
mod monolith;
mod motion_tracker;

use nannou::prelude::*;

use crate::camera_wrapper::CameraWrapper;
use crate::monolith::Monolith;
use crate::motion_tracker::VideoProcessor;

fn main() {
    let mut processor = VideoProcessor::new().unwrap();
    processor.process_frames();
    nannou::app(model).update(update).run();
}

struct Model {
    monolith: Monolith,
    camera: CameraWrapper,
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
        camera: CameraWrapper::new(app),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // model.camera.update(app);
    model.monolith.update(app, &model.camera);
    println!("{}", app.fps());
}

fn view(app: &App, model: &Model, frame: Frame) {
    // let draw = app.draw();
    // let texture = &model.camera.texture;
    // draw.texture(texture);
    // draw.to_frame(app, &frame).unwrap();
}

fn view2(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model
        .monolith
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}
