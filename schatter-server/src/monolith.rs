use std::path::Path;

use csv::Reader;
use nannou::image::{ImageBuffer, Pixel};
use nannou::prelude::*;
use nannou::wgpu::CommandEncoder;
use nannou_osc as osc;
use nannou_osc::Type;
use osc::Color;
use parry3d::na::Point3;

use crate::camera_wrapper::CameraWrapper;

const WIDTH: f32 = 1460.0;
const DEPTH: f32 = 335.0;
const WIDTH_NET: f32 = (WIDTH + DEPTH) * 2.0;
const HEIGHT: f32 = 3350.0;
const SCALE_TEXTURE: f32 = 2.0;
const PATH_LED_POINTS_FILE: &str = "./points.csv";
const LAST_LED_FRONT: usize = 1171;
const LAST_LED_SIDE: usize = 1308;

pub struct Monolith {
    pub texture: wgpu::Texture,
    pub texture_reshaper: wgpu::TextureReshaper,
    window_id: WindowId,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    texture_capturer: wgpu::TextureCapturer,
    led_coordinates: LedCoordinates,
    counter: usize,
}

pub struct LedCoordinates {
    led_2d: Vec<Point2>,
    led_2d_image: Vec<Point2>,
    led_3d: Vec<Point3<f32>>,
}

impl LedCoordinates {
    fn new() -> LedCoordinates {
        let led_2d_so = Monolith::parse_led_coordinates(Path::new(PATH_LED_POINTS_FILE));
        let led_2d_nw: Vec<Point2> = led_2d_so
            .iter()
            .map(|c| Point2::new(c.x + WIDTH + DEPTH, c.y))
            .collect();
        let (led_3d_n, led_3d_s): (Vec<Point3<f32>>, Vec<Point3<f32>>) = led_2d_so
            .iter()
            .take(LAST_LED_FRONT)
            .map(|c| {
                (
                    Point3::new(c.x + (WIDTH * 0.5) + DEPTH, c.y + HEIGHT / 2.0, DEPTH / 2.0),
                    Point3::new(c.x, c.y + HEIGHT / 2.0, -DEPTH / 2.0),
                )
            })
            .unzip();
        let (led_3d_e, led_3d_w): (Vec<Point3<f32>>, Vec<Point3<f32>>) = led_2d_so
            .iter()
            .skip(LAST_LED_FRONT)
            .map(|c| {
                (
                    Point3::new(-(WIDTH + DEPTH) / 2.0, c.y + HEIGHT / 2.0, c.x),
                    Point3::new(-WIDTH + (DEPTH / 2.0), c.y + HEIGHT / 2.0, -c.x),
                )
            })
            .unzip();
        let led_2d = [led_2d_so, led_2d_nw].concat();
        let led_2d_image: Vec<Point2> = led_2d
            .iter()
            .map(|c| Monolith::from_nannou_to_image(*c))
            .collect();
        let led_3d = [led_3d_n, led_3d_e, led_3d_s, led_3d_w].concat();
        println!("{:?}", led_3d[0]);
        println!("{:?}", led_3d[1]);
        println!("{:?}", led_3d[2]);
        println!("{:?}", led_3d[18]);
        println!("{:?}", led_3d[19]);
        println!("xxx");
        println!("{:?}", led_3d[20]);
        println!("{:?}", led_3d[21]);
        LedCoordinates {
            led_2d,
            led_2d_image,
            led_3d,
        }
    }
}

impl Monolith {
    pub fn new(app: &App, window_id: WindowId) -> Monolith {
        let window = app.window(window_id).unwrap();
        let device = window.device();
        let sample_count = window.msaa_samples();
        let texture = wgpu::TextureBuilder::new()
            .size([
                (WIDTH_NET * SCALE_TEXTURE) as u32,
                (HEIGHT * SCALE_TEXTURE) as u32,
            ])
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
            draw: nannou::Draw::new().scale(SCALE_TEXTURE),
            texture,
            renderer,
            texture_reshaper,
            texture_capturer: wgpu::TextureCapturer::default(),
            led_coordinates: LedCoordinates::new(),
            counter: 0,
        }
    }

    pub fn update(&mut self, app: &App, motion_tracker: &CameraWrapper) {
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
        let transform = Mat3::from_translation(Vec2::new(-WIDTH_NET / 2.0, HEIGHT / 2.0))
            * Mat3::from_scale(Vec2::new(1.0, -1.0));
        transform.transform_point2(point)
    }

    fn from_nannou_to_image(point: Point2) -> Point2 {
        let transform =
            Mat3::from_translation(Vec2::new((WIDTH_NET as f32) / 2.0, (HEIGHT as f32) / 2.0))
                * Mat3::from_scale(Vec2::new(1.0, -1.0));

        transform.transform_point2(point) * SCALE_TEXTURE
    }

    fn draw(&mut self, motion_tracker: &CameraWrapper) {
        self.draw.reset();
        self.draw.background().color(BLACK);
        self.draw.texture(&motion_tracker.texture);
        self.draw
            .rect()
            .x_y((WIDTH as f32) / 4.0, (HEIGHT as f32) / 4.0)
            .w_h((WIDTH as f32) / 2.0, (HEIGHT as f32) / 2.0)
            .color(BLUE);
    }

    fn draw_debug(&mut self) {
        self.draw
            .rect()
            .w_h(WIDTH_NET, HEIGHT)
            .no_fill()
            .stroke(HOTPINK)
            .stroke_weight(4.0);

        for coordinate in self.led_coordinates.led_2d.iter() {
            self.draw
                .ellipse()
                .color(WHITE)
                .w(2.0)
                .h(2.0)
                .x_y(coordinate.x as f32, coordinate.y as f32);
        }
        let index = self.counter % self.led_coordinates.led_2d.len();
        self.draw.ellipse().color(RED).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[0].x as f32,
            self.led_coordinates.led_2d[0].y as f32,
        );
        self.draw.ellipse().color(RED).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[17].x as f32,
            self.led_coordinates.led_2d[17].y as f32,
        );
        self.draw.ellipse().color(RED).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[18].x as f32,
            self.led_coordinates.led_2d[18].y as f32,
        );
        self.draw.ellipse().color(RED).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[19].x as f32,
            self.led_coordinates.led_2d[19].y as f32,
        );
        self.draw.ellipse().color(BLUE).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[20].x as f32,
            self.led_coordinates.led_2d[20].y as f32,
        );
        self.draw.ellipse().color(GREEN).w(10.0).h(10.0).x_y(
            self.led_coordinates.led_2d[1171].x as f32,
            self.led_coordinates.led_2d[1171].y as f32,
        );
        self.counter += 1;
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
        let led_coordinates = self.led_coordinates.led_2d_image.clone();
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
            let p: &[u8] = image
                .get_pixel(coordinate.x as u32, coordinate.y as u32)
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
