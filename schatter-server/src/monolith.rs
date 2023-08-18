use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

use ::rgb::RGB8;
use csv::Reader;
use nannou::image::{ImageBuffer, Pixel};
use nannou::prelude::*;
use nannou::wgpu::CommandEncoder;
use nannou_osc as osc;
use nannou_osc::Type;
use osc::Color;
use parry3d::math::Real;
use parry3d::na::{Point3, Rotation3};

use crate::collision_detector::CollisionDetector;

const WIDTH: f32 = 1460.0;
const DEPTH: f32 = 335.0;
const WIDTH_NET: f32 = (WIDTH + DEPTH) * 2.0;
const HEIGHT: f32 = 3350.0;
const SCALE_TEXTURE: f32 = 0.5;

const PATH_LED_POINTS_FILE: &str = "./points.csv";
const NUM_LED_FRONT: usize = 1173;
const NUM_LED_SIDE: usize = 1310;

// const RASPBERRY_PI_ADDRESS: &str = "127.0.0.1:34254";
const RASPBERRY_PI_ADDRESS: &str = "192.168.1.186:34254";
const NUM_LEDS_TO_SEND: usize = 2 * NUM_LED_SIDE;

const STRIPES_NUM_LEDS: [usize; 58] = [
    0,
    20,
    63,
    120,
    229,
    344,
    416,
    491,
    545,
    626,
    712,
    754,
    843,
    911,
    929,
    954,
    974,
    998,
    1029,
    1067,
    1079,
    1107,
    1118,
    1141,
    1164,
    1173,
    1197,
    1230,
    1266,
    1310,
    20 + 1310,
    63 + 1310,
    120 + 1310,
    229 + 1310,
    344 + 1310,
    416 + 1310,
    491 + 1310,
    545 + 1310,
    626 + 1310,
    712 + 1310,
    754 + 1310,
    843 + 1310,
    911 + 1310,
    929 + 1310,
    954 + 1310,
    974 + 1310,
    998 + 1310,
    1029 + 1310,
    1067 + 1310,
    1079 + 1310,
    1107 + 1310,
    1118 + 1310,
    1141 + 1310,
    1164 + 1310,
    1173 + 1310,
    1197 + 1310,
    1230 + 1310,
    1266 + 1310,
];
pub struct Monolith {
    pub texture: wgpu::Texture,
    pub texture_reshaper: wgpu::TextureReshaper,
    window_id: WindowId,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    texture_capturer: wgpu::TextureCapturer,
    led_coordinates: LedCoordinates,
    collision_detector: CollisionDetector,
}

pub struct Cone {}

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
            .take(NUM_LED_FRONT)
            .map(|c| {
                (
                    Point3::new(c.x + (WIDTH * 0.5) + DEPTH, c.y + HEIGHT / 2.0, DEPTH / 2.0),
                    Point3::new(
                        -(c.x + (WIDTH * 0.5) + DEPTH),
                        c.y + HEIGHT / 2.0,
                        -DEPTH / 2.0,
                    ),
                )
            })
            .unzip();
        let (led_3d_e, led_3d_w): (Vec<Point3<f32>>, Vec<Point3<f32>>) = led_2d_so
            .iter()
            .skip(NUM_LED_FRONT)
            .map(|c| {
                (
                    Point3::new(-(WIDTH + DEPTH) / 2.0, c.y + HEIGHT / 2.0, c.x),
                    Point3::new(-WIDTH + (DEPTH / 2.0), c.y + HEIGHT / 2.0, c.x),
                )
            })
            .unzip();
        let led_2d = [led_2d_so, led_2d_nw].concat();
        let led_2d_image: Vec<Point2> = led_2d
            .iter()
            .map(|c| Monolith::from_nannou_to_image(*c))
            .collect();
        let led_3d = [led_3d_n, led_3d_e, led_3d_s, led_3d_w].concat();
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
            collision_detector: CollisionDetector::new(),
        }
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

    pub fn update(&mut self, app: &App, update: &Update) {
        let window = &app.window(self.window_id).unwrap();
        self.draw(update);
        #[cfg(debug_assertions)]
        self.draw_debug(update);
        self.render(window, true);
    }

    fn draw(&mut self, update: &Update) {
        self.draw.reset();

        self.draw.background().color(BLACK);

        let time_since_update = update.since_last.secs();
        let time_since_start = update.since_start.secs();
        let view = Point3::new(-000.0, 2320.0, -3000.0);
        self.draw_viewcone(time_since_start, view);
        self.draw
            .line()
            .color(WHITE)
            .start(Vec2::new(-WIDTH_NET / 2.0, time_since_start as f32));
    }

    fn draw_viewcone(&self, update: f64, view: Point3<Real>) {
        let rotation =
            Rotation3::from_axis_angle(&parry3d::na::Vector3::y_axis(), (update * 3.10) as f32);
        let rotation2 =
            Rotation3::from_axis_angle(&parry3d::na::Vector3::x_axis(), (update * 2.10) as f32);
        let view = rotation * rotation2 * view;

        for (i, coordinate) in self.led_coordinates.led_2d.iter().enumerate() {
            if self
                .collision_detector
                .detect_collison(view, &self.led_coordinates.led_3d[i])
            {
                self.draw
                    .ellipse()
                    .color(WHITE)
                    .w(10.0)
                    .h(10.0)
                    .x_y(coordinate.x as f32, coordinate.y as f32);
            }
        }
    }

    fn draw_debug(&mut self, update: &Update) {
        self.draw
            .rect()
            .w_h(WIDTH_NET, HEIGHT)
            .no_fill()
            .stroke(HOTPINK)
            .stroke_weight(4.0);

        let time_since_update = update.since_last.secs();
        let string = format!("{:.2}", time_since_update);
        self.draw
            .text(&string)
            .color(HOTPINK)
            .font_size(96)
            .align_text_bottom()
            .left_justify()
            .w_h(WIDTH_NET - 96.0, HEIGHT - 96.0);

        for num_leds in &STRIPES_NUM_LEDS[1..58] {
            self.render_led(*num_leds - 2, RED);
            self.render_led(*num_leds - 1, RED);
            self.render_led(*num_leds, GREEN);
            self.render_led(*num_leds + 1, GREEN);
        }
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

    fn render_led(&self, index: usize, color: Srgb<u8>) {
        self.draw.ellipse().color(color).w(15.0).h(15.0).x_y(
            self.led_coordinates.led_2d[index].x as f32,
            self.led_coordinates.led_2d[index].y as f32,
        );
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
        let led_coordinates = self.led_coordinates.led_2d_image[0..NUM_LEDS_TO_SEND].to_vec();
        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory").to_owned();
                let addr = "/";
                let args = Monolith::get_pixels(led_coordinates, &image);
                let sender = osc::sender()
                    .expect("Could not bind to default socket")
                    .connect(RASPBERRY_PI_ADDRESS)
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
