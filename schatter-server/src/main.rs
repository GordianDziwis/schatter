mod camera_wrapper;
mod collision_detector;
mod monolith;
mod motion_tracker;

use std::net::TcpStream;

use nannou::prelude::{Key, Update};
use nannou::{App, Frame, LoopMode};
use parry3d::na::{Isometry3, Point3, Vector3};
use parry3d::query::PointQuery;
use parry3d::shape::{Cone, Cuboid};

use crate::camera_wrapper::CameraWrapper;
use crate::monolith::Monolith;
use crate::motion_tracker::VideoProcessor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

const RASPBERRY_PI_ADDRESS: &str = "127.0.0.1:34254";

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    monolith: Monolith,
}

fn model(app: &App) -> Model {
    let window_id_monolith = app
        .new_window()
        .msaa_samples(1)
        .title("Monolith")
        .resizable(false)
        .view(view2)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    Model {
        monolith: Monolith::new(app, window_id_monolith),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // model.camera.update(app);
    model.monolith.update(app, &update);
}

fn view(app: &App, model: &Model, frame: Frame) {
}

fn key_pressed(app: &App, _model: &mut Model, _key: Key) {
    #[cfg(debug_assertions)]
    {
        match app.loop_mode() {
            LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::RefreshSync),
            LoopMode::RefreshSync { .. } => app.set_loop_mode(LoopMode::rate_fps(5.0)),
            LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::loop_once()),
            LoopMode::NTimes { .. } => app.set_loop_mode(LoopMode::Wait),
        }
        println!("Loop mode switched to: {:?}", app.loop_mode());
    }
}

fn view2(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model
        .monolith
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}
