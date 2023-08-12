use nannou::image::DynamicImage;
use nannou::image::RgbImage;
use nannou::wgpu;
use nannou::App;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::CameraFormat;
use nokhwa::utils::CameraIndex;
use nokhwa::utils::FrameFormat;
use nokhwa::utils::RequestedFormat;
use nokhwa::utils::RequestedFormatType;
use nokhwa::Camera;

const CAMERA_INDEX: u32 = 0;
const X_RES: u32 = 1280;
const Y_RES: u32 = 720;
const FRAME_FORMAT: FrameFormat = FrameFormat::MJPEG;
const FRAME_RATE: u32 = 30;

pub struct MotionTracker {
    pub camera: Camera,
    pub actual_x_res: u32,
    pub actual_y_res: u32,
    pub texture: wgpu::Texture,
}

impl MotionTracker {
    pub fn new(app: &App) -> Self {
        let camera_index = CameraIndex::Index(CAMERA_INDEX);
        let camera_format = CameraFormat::new_from(X_RES, Y_RES, FRAME_FORMAT, FRAME_RATE);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format));
        let mut camera = Camera::new(camera_index, format).unwrap();
        camera.open_stream().unwrap();
        let frame = camera.frame().unwrap();
        let resolution = frame.resolution();

        let actual_x_res = resolution.width_x;
        let actual_y_res = resolution.height_y;
        let image = DynamicImage::new_rgb8(actual_x_res, actual_y_res);
        let texture = wgpu::Texture::from_image(app, &image);
        MotionTracker {
            camera,
            actual_x_res,
            actual_y_res,
            texture,
        }
    }

    pub fn update(&mut self, app: &App) {
        self.texture = wgpu::Texture::from_image(app, &self.frame());
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
