use nannou::image::DynamicImage;
use nannou::image::ImageBuffer;
use nannou::image::Pixel;
use nannou::image::RgbImage;
use nannou::prelude::*;
use nannou_osc as osc;
use nannou_osc::Color;
use nannou_osc::Type;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::CameraFormat;
use nokhwa::utils::CameraIndex;
use nokhwa::utils::FrameFormat;
use nokhwa::utils::RequestedFormat;
use nokhwa::utils::RequestedFormatType;
use nokhwa::Camera;

const TARGET_PORT: u16 = 34254;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    led_xy: Vec<(u32, u32)>,
    texture_capturer: wgpu::TextureCapturer,
    model_motion_tracker: ModelMotionTracker,
}

struct ModelMotionTracker {
    output: wgpu::Texture,
    motion_tracker: MotionTracker,
}

impl ModelMotionTracker {
    fn new(app: &App) -> Self {
        let motion_tracker = MotionTracker::new();
        let image =
            DynamicImage::new_rgb8(motion_tracker.actual_x_res, motion_tracker.actual_y_res);
        let texture = wgpu::Texture::from_image(app, &image);
        ModelMotionTracker {
            output: texture,
            motion_tracker,
        }
    }
    fn update(&mut self, app: &App) {
        self.output = wgpu::Texture::from_image(app, &self.motion_tracker.frame());
    }
}

const CAMERA_INDEX: u32 = 0;
const X_RES: u32 = 1280;
const Y_RES: u32 = 720;
const FRAME_FORMAT: FrameFormat = FrameFormat::MJPEG;
const FRAME_RATE: u32 = 30;

struct MotionTracker {
    camera: Camera,
    actual_x_res: u32,
    actual_y_res: u32,
}

impl MotionTracker {
    fn new() -> Self {
        let camera_index = CameraIndex::Index(CAMERA_INDEX);
        let camera_format = CameraFormat::new_from(X_RES, Y_RES, FRAME_FORMAT, FRAME_RATE);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format));
        let mut camera = Camera::new(camera_index, format).unwrap();
        camera.open_stream().unwrap();
        let frame = camera.frame().unwrap();
        let resolution = frame.resolution();
        MotionTracker {
            camera,
            actual_x_res: resolution.width_x,
            actual_y_res: resolution.height_y,
        }
    }

    fn frame(&mut self) -> DynamicImage {
        let frame = self.camera.frame().unwrap();
        let decoded_frame = frame.decode_image::<RgbFormat>().unwrap();
        let image = RgbImage::from_raw(
            self.actual_x_res,
            self.actual_y_res,
            decoded_frame.into_vec(),
        )
        .unwrap();
        DynamicImage::ImageRgb8(image)
    }
}

fn model(app: &App) -> Model {
    let _window_id = app
        .new_window()
        .title("OSC Sender")
        .size(680, 480)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    let path = Path::new("./points.csv");

    println!("{:?}", read_csv_to_tuples(path, X_RES.into(), Y_RES.into()));

    Model {
        led_xy: read_csv_to_tuples(path, X_RES.into(), Y_RES.into()),
        model_motion_tracker: ModelMotionTracker::new(app),
        texture_capturer: wgpu::TextureCapturer::default(),
    }
}
fn get_pixels(
    led_xy: Vec<(u32, u32)>,
    image: &ImageBuffer<nannou::image::Rgba<u8>, Vec<u8>>,
) -> Vec<Type> {
    let mut pixels: Vec<Type> = Vec::new();
    for (x, y) in led_xy {
        let p: &[u8] = image.get_pixel(x, y).channels();
        let color = Type::Color(Color {
            red: p[0],
            green: p[1],
            blue: p[2],
            alpha: p[3],
        });
        pixels.push(color);
    }
    pixels
}

fn update(app: &App, model: &mut Model, _update: Update) {
    const DELAY: time::Duration = time::Duration::from_millis(30);
    thread::sleep(DELAY);
    model.model_motion_tracker.update(app);

    let window = app.main_window();
    let device = window.device();
    let ce_description = wgpu::CommandEncoderDescriptor {
        label: Some("Capturer for reading texture to CPU"),
    };
    let mut encoder = device.create_command_encoder(&ce_description);
    let texture = &model.model_motion_tracker.output;
    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, texture);
    window.queue().submit(Some(encoder.finish()));
    let mut fuck = model.led_xy.clone();
    fuck.truncate(1300);
    snapshot
        .read(move |result| {
            let image = result.expect("failed to map texture memory").to_owned();
            let addr = "/";
            // let p: &[u8] = image.get_pixel(700, 700).channels();
            // let color = Type::Color(Color {
            //     red: p[0],
            //     green: p[1],
            //     blue: p[2],
            //     alpha: p[3],
            // });

            // let args = vec![color];
            let args = get_pixels(fuck, &image);
            let sender = osc::sender()
                .expect("Could not bind to default socket")
                .connect(format!("{}:{}", "192.168.1.186", TARGET_PORT))
                .expect("Could not connect to socket at address");

            println!("{:?}", args);

            sender.send((addr, args)).ok();
        })
        .unwrap();
}
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::path::Path;
use std::thread;
use std::time;

fn read_csv_to_tuples(path: &Path, w: f64, h: f64) -> Vec<(u32, u32)> {
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let mut tuples: Vec<(f64, f64)> = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let mut split = line.split(",");
        let x: f64 = split.next().unwrap().parse::<f64>().unwrap();
        let y: f64 = split.next().unwrap().parse::<f64>().unwrap();
        tuples.push((x, y));
    }

    let max_x = tuples.iter().map(|(x, _)| *x).fold(f64::MIN, f64::max);
    let max_y = tuples.iter().map(|(_, y)| *y).fold(f64::MIN, f64::max);

    tuples
        .iter()
        .map(|(x, y)| {
            (
                ((x / max_x * w) - 1.0) as u32,
                ((y / max_y * h) - 1.0) as u32,
            )
        })
        .collect()
}
fn event(_app: &App, _model: &mut Model, event: WindowEvent) {
    match event {
        // MouseMoved(pos) => {
        //     let addr = "/example/mouse_moved/";
        //     let args = vec![Type::Float(pos.x), Type::Float(pos.y)];
        //     model.sender.send((addr, args)).ok();
        // }

        // MousePressed(button) => {
        //     let addr = "/example/mouse_pressed/";
        //     let color = Type::Color(Color {
        //         red: 255,
        //         green: 0,
        //         blue: 0,
        //         alpha: 255,
        //     });
        //     let args = vec![color];
        //     model.sender.send((addr, args)).ok();
        // }

        // MouseReleased(button) => {
        //     let addr = "/example/mouse_released/";
        //     let button = format!("{:?}", button);
        //     let args = vec![Type::String(button)];
        //     model.sender.send((addr, args)).ok();
        // }
        _other => (),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let texture = &model.model_motion_tracker.output;
    draw.texture(texture);
    for (x, y) in model.led_xy.iter() {
        draw.ellipse()
            .color(WHITE)
            .stroke_color(BLACK)
            .w(5.0)
            .h(5.0)
            .x_y(*x as f32 - 1280.0 / 2.0, 720.0 / 2.0 - *y as f32);
    }
    draw.to_frame(app, &frame).unwrap();
}

fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
}
