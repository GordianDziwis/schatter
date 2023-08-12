mod monolith;
mod motion_tracker;
use monolith::Monolith;
use motion_tracker::MotionTracker;
use nannou::prelude::*;
use std::thread;
use std::time;

fn main() {
    nannou::app(model).update(update).run();
}

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

    const DELAY: time::Duration = time::Duration::from_millis(5);
    thread::sleep(DELAY);
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
