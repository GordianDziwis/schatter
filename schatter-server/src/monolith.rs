use std::path::Path;

use csv::Reader;
use nannou::image::{ImageBuffer, Pixel};
use nannou::prelude::*;
use nannou::wgpu::{CommandEncoder, Device};
use nannou_osc as osc;
use nannou_osc::Type;
use osc::{encoder, Color};

use crate::motion_tracker::MotionTracker;

const WIDTH: u32 = 1795;
const HEIGHT: u32 = 3350;

const SCALE_TEXTURE: u32 = 2;
const SCALED_WIDTH: u32 = WIDTH / SCALE_TEXTURE;
const SCALED_HEIGHT: u32 = HEIGHT / SCALE_TEXTURE;
const PATH_LED_POINTS_FILE: &str = "./points.csv";

pub struct Monolith {
    pub texture: wgpu::Texture,
    pub texture_reshaper: wgpu::TextureReshaper,
    window_id: WindowId,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    texture_capturer: wgpu::TextureCapturer,
    led_coordinates: Vec<Point2>,
}

impl Monolith {
    pub fn new(app: &App, window_id: WindowId) -> Monolith {
        let window = app.window(window_id).unwrap();
        let device = window.device();
        let sample_count = window.msaa_samples();
        let texture = wgpu::TextureBuilder::new()
            .size([WIDTH / SCALE_TEXTURE, HEIGHT / SCALE_TEXTURE])
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .sample_count(sample_count)
            .format(wgpu::TextureFormat::Rgba16Float)
            .build(device);
        let descriptor = texture.descriptor();
        let renderer =
            nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);
        let texture_reshaper = wgpu::TextureReshaper::new(
            device,
            &texture.view().build(),
            sample_count,
            texture.sample_type(),
            sample_count,
            Frame::TEXTURE_FORMAT,
        );
        Monolith {
            window_id,
            draw: nannou::Draw::new(),
            texture,
            renderer,
            texture_reshaper,
            led_coordinates: Monolith::parse_led_coordinates(Path::new(PATH_LED_POINTS_FILE)),
            texture_capturer: wgpu::TextureCapturer::default(),
        }
    }

    pub fn update(&mut self, app: &App, motion_tracker: &MotionTracker) {
        let window = &app.window(self.window_id).unwrap();
        self.draw(motion_tracker);
        self.render(window, true);
        self.draw(motion_tracker);
        self.draw_debug();
        self.render(window, false);
    }

    fn parse_led_coordinates(path: &Path) -> Vec<Point2> {
        let mut rdr = Reader::from_path(path).unwrap();
        let mut points: Vec<Point2> = Vec::new();
        for result in rdr.records() {
            let record = result.unwrap();
            let x: f32 = record[0].parse().unwrap();
            let y: f32 = record[1].parse().unwrap();
            points.push(Monolith::from_inkscape_to_nannou(Point2::new(x, y)));
        }
        points
    }

    fn from_inkscape_to_nannou(point: Point2) -> Point2 {
        let transform =
            Mat3::from_translation(Vec2::new(-(WIDTH as f32) / 2.0, (HEIGHT as f32) / 2.0))
                * Mat3::from_scale(Vec2::new(1.0, -1.0));

        transform.transform_point2(point) / SCALE_TEXTURE as f32
    }

    fn from_nannou_to_image(point: Point2) -> Point2 {
        let transform =
            Mat3::from_translation(Vec2::new((WIDTH as f32) / 2.0, (HEIGHT as f32) / 2.0))
                * Mat3::from_scale(Vec2::new(1.0, -1.0));

        transform.transform_point2(point) / SCALE_TEXTURE as f32
    }

    fn draw(&mut self, motion_tracker: &MotionTracker) {
        self.draw.reset();
        self.draw.background().color(BLACK);
        self.draw.texture(&motion_tracker.texture);
        self.draw
            .rect()
            .x_y((SCALED_WIDTH as f32) / 4.0, (SCALED_HEIGHT as f32) / 4.0)
            .w_h((SCALED_WIDTH as f32) / 2.0, (SCALED_HEIGHT as f32) / 2.0)
            .color(BLUE);
    }

    fn draw_debug(&mut self) {
        self.draw
            .rect()
            .w_h(SCALED_WIDTH as f32, SCALED_HEIGHT as f32)
            .no_fill()
            .stroke(HOTPINK)
            .stroke_weight(1.0);

        for coordinate in self.led_coordinates.iter() {
            self.draw
                .ellipse()
                .color(WHITE)
                .w(1.0)
                .h(1.0)
                .x_y(coordinate.x as f32, coordinate.y as f32);
        }
    }

    fn render(&mut self, window: &Window, snapshot: bool) {
        let device = window.device();
        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);
        self.renderer
            .render_to_texture(device, &mut encoder, &self.draw, &self.texture);
        if snapshot {
            self.snapshot(window, encoder)
        } else {
            window.queue().submit(Some(encoder.finish()));
        }
    }

    fn snapshot(&mut self, window: &Window, mut encoder: CommandEncoder) {
        let device = window.device();
        let snapshot = self
            .texture_capturer
            .capture(device, &mut encoder, &self.texture);
        window.queue().submit(Some(encoder.finish()));
        let led_coordinates = self.led_coordinates.clone();
        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory").to_owned();
                let addr = "/";
                let args = Monolith::get_pixels(led_coordinates, &image);
                let sender = osc::sender()
                    .expect("Could not bind to default socket")
                    .connect("127.0.0.1:34254")
                    .expect("Could not connect to socket at address");
                // println!("{:?}", args[20]);
                sender.send((addr, args)).ok();
            })
            .unwrap();
    }

    fn get_pixels(
        led_coordinates: Vec<Point2>,
        image: &ImageBuffer<nannou::image::Rgba<u8>, Vec<u8>>,
    ) -> Vec<Type> {
        let mut pixels: Vec<Type> = Vec::new();
        for coordinate in led_coordinates {
            let transformed_coordinate = Monolith::from_nannou_to_image(coordinate);
            let p: &[u8] = image
                .get_pixel(
                    transformed_coordinate.x as u32,
                    transformed_coordinate.y as u32,
                )
                .channels();
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
}
