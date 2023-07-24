use nannou::image::DynamicImage;
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
    sender: osc::Sender<osc::Connected>,

    texture_capturer: wgpu::TextureCapturer,

    model_motion_tracker: ModelMotionTracker,
}

struct ModelMotionTracker {
    window_id: WindowId,
    output: wgpu::Texture,
    motion_tracker: MotionTracker,
}

impl ModelMotionTracker {
    fn new(app: &App, window_id: WindowId) -> Self {
        let motion_tracker = MotionTracker::new();
        let image =
            DynamicImage::new_rgb8(motion_tracker.actual_x_res, motion_tracker.actual_y_res);
        let texture = wgpu::Texture::from_image(app, &image);
        ModelMotionTracker {
            window_id,
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
// Make sure this matches `PORT` in the `osc_receiver.rs` example.

fn target_address_string() -> String {
    format!("{}:{}", "127.0.0.1", TARGET_PORT)
}

fn model(app: &App) -> Model {
    let motion_tracker_window_id = app
        .new_window()
        .title("OSC Sender")
        .size(680, 480)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    // The address to which the `Sender` will send messages.
    let target_addr = target_address_string();

    // Bind an `osc::Sender` and connect it to the target address.
    let sender = osc::sender()
        .expect("Could not bind to default socket")
        .connect(target_addr)
        .expect("Could not connect to socket at address");

    // let display = wgpu::Texture::from_image(app, &DynamicImage::ImageRgb8(img));
    Model {
        sender,
        model_motion_tracker: ModelMotionTracker::new(app, motion_tracker_window_id),
        texture_capturer: wgpu::TextureCapturer::default(),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.model_motion_tracker.update(app);

    let window = app.main_window();
    let device = window.device();
    let ce_description = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_description);
    let texture = &model.model_motion_tracker.output;
    let elapsed_frames = app.main_window().elapsed_frames();

    let path = capture_directory(app)
        .join(elapsed_frames.to_string())
        .with_extension("png");
    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, texture);
    snapshot
        .read(move |result| {
            let image = result.expect("failed to map texture memory").to_owned();
            image.save(&path).expect("failed to save texture to png image");

        })
        .unwrap();
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        MouseMoved(pos) => {
            let addr = "/example/mouse_moved/";
            let args = vec![Type::Float(pos.x), Type::Float(pos.y)];
            model.sender.send((addr, args)).ok();
        }

        MousePressed(button) => {
            let addr = "/example/mouse_pressed/";
            let color = Type::Color(Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            });
            let args = vec![color];
            model.sender.send((addr, args)).ok();
        }

        MouseReleased(button) => {
            let addr = "/example/mouse_released/";
            let button = format!("{:?}", button);
            let args = vec![Type::String(button)];
            model.sender.send((addr, args)).ok();
        }

        _other => (),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let texture = &model.model_motion_tracker.output;
    draw.texture(texture);
    draw.to_frame(app, &frame).unwrap();
}

fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
}
