use std::io::Write;
use std::net::TcpStream;
use std::ops::{Add, Range};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use csv::Reader;
use nannou::draw::properties::ColorScalar;
use nannou::image::{ImageBuffer, Pixel};
use nannou::lyon::geom::euclid::Trig;
use nannou::prelude::float::FloatCore;
use nannou::prelude::*;
use nannou::rand;
use nannou::rand::Rng;
use nannou::wgpu::CommandEncoder;
use nannou_osc as osc;
use nannou_osc::Type;
use osc::{Color, Connected, Sender};
use parry3d::math::{Real, Vector};
use parry3d::na::{Point3, Rotation3};

use crate::collision_detector::CollisionDetector;
use crate::motion_tracker::VideoProcessor;

const WIDTH: f32 = 1460.0;
const DEPTH: f32 = 335.0;
const WIDTH_NET: f32 = (WIDTH + DEPTH) * 2.0;
const HEIGHT: f32 = 3350.0;
const SCALE_TEXTURE: f32 = 0.5;

const PATH_LED_POINTS_FILE: &str = "./points.csv";
const NUM_LED_FRONT: usize = 1173;
const NUM_LED_SIDE: usize = 1310;
const FLEEING_TIME: Duration = Duration::new(3, 0);

// const RASPBERRY_PI_ADDRESS: &str = "127.0.0.1:34254";
const RASPBERRY_PI_ADDRESS: &str = "192.168.1.186:34254";
const NUM_LEDS_TO_SEND: usize = 2 * NUM_LED_SIDE;

struct Client {
    sender: Sender<Connected>,
    led_range: Range<usize>,
}

impl Client {
    fn new(address: &str, led_range: Range<usize>) -> Self {
        let sender = osc::sender()
            .expect("Could not bind to default socket")
            .connect(address)
            .expect("Could not connect to socket at address");
        Client { sender, led_range }
    }
}

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
    754 + 1,
    843,
    911 + 1,
    929 + 1,
    954 + 1,
    974 + 1,
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
    491 + 1310 - 1,
    545 + 1310,
    626 + 1310 - 1,
    712 + 1310,
    754 + 1310 - 2,
    843 + 1310,
    911 + 1310 - 2,
    929 + 1310 - 2,
    954 + 1310 - 2,
    974 + 1310 - 2,
    998 + 1310 - 2,
    1029 + 1310 - 3,
    1067 + 1310 - 3,
    1079 + 1310 - 2,
    1107 + 1310 - 3,
    1118 + 1310 - 3,
    1141 + 1310 - 3,
    1164 + 1310 - 3,
    1173 + 1310 - 3,
    1197 + 1310 - 3,
    1230 + 1310 - 3,
    1266 + 1310 - 4,
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
    viewpoint: Arc<Mutex<Option<Point2>>>,
    time_animation: Instant,
    stripe: Vec<usize>,
    client_configs: Arc<Mutex<Vec<Client>>>,
    cones: Cones,
    new: Arc<Mutex<bool>>,
}

pub struct LedCoordinates {
    led_2d: Vec<Point2>,
    led_2d_image: Vec<Point2>,
    led_3d: Vec<Point3<f32>>,
}

pub struct Cones {
    positions: Vec<Vector<f32>>,
    tracking_time_left: Duration,
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
                    Point3::new(-WIDTH / 2.0, c.y + HEIGHT / 2.0, c.x),
                    Point3::new(WIDTH / 2.0, c.y + HEIGHT / 2.0, -c.x - DEPTH / 2.0),
                )
            })
            .unzip();
        let led_2d = [led_2d_so, led_2d_nw].concat();
        let led_2d_image: Vec<Point2> = led_2d
            .iter()
            .map(|c| Monolith::from_nannou_to_image(*c))
            .collect();
        let led_3d = [led_3d_n, led_3d_w, led_3d_s, led_3d_e].concat();

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

        let position = Arc::new(Mutex::new(Some(Point2::default())));
        let position_clone = Arc::clone(&position);
        let new = Arc::new(Mutex::new(false));
        let new_clone = Arc::clone(&new);
        thread::spawn(move || {
            let mut motion_tracker = VideoProcessor::new(position_clone, new_clone).unwrap();
            motion_tracker.process_frames();
        });

        let client_configs = vec![
            Client::new("192.168.1.186:34254", 0..626),
            Client::new("192.168.1.186:34255", 626..1310),
            Client::new("192.168.1.219:34254", 1310..1936),
            Client::new("192.168.1.219:34255", 1936..2620),
        ];

        Monolith {
            window_id,
            draw: nannou::Draw::new().scale(SCALE_TEXTURE),
            texture,
            renderer,
            texture_reshaper,
            texture_capturer: wgpu::TextureCapturer::default(),
            led_coordinates: LedCoordinates::new(),
            collision_detector: CollisionDetector::new(),
            viewpoint: position,
            time_animation: Instant::now(),
            stripe: Vec::new(),
            client_configs: Arc::new(Mutex::new(client_configs)),
            cones: Cones {
                positions: vec![
                    Vector::new(3000.0, 2000.0, 0.0),
                    Vector::new(3000.0, 2000.0, 3000.0),
                    Vector::new(3000.0, 2000.0, 0.0),
                ],
                tracking_time_left: FLEEING_TIME,
            },
            new,
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
        let mut rng = rand::thread_rng();
        let time_since_update = update.since_last.secs();
        let time_since_start = update.since_start.secs();
        self.draw.reset();
        self.draw.background().color(BLACK);

        //         for i in 0..((NUM_LED_SIDE - 1) * 2) {
        //             self.render_led(
        //                 i,
        //                 Srgb::new(
        //                     rng.gen_range(230..255),
        //                     rng.gen_range(230..255),
        //                     rng.gen_range(230..255),
        //                 ),
        //             )
        //         }
        let steps: usize = 300;
        let stroke_weight = HEIGHT / (HEIGHT / steps as f32);
        // for i in (-(HEIGHT * 2.0) as i32..(HEIGHT * 2.0) as i32).step_by(steps) {
        //     self.draw_line(stroke_weight, time_since_start as f32, i as f32)
        // }

        // println!("{:?}", &view); // Print the viewpoint_clone
        // self.draw
        //     .line()
        //     .color(RED)
        //     .stroke_weight(200.0)
        //     .start(Point2::new(
        //         -WIDTH_NET / 2.0,
        //         (time_since_start.sin() * HEIGHT as f64) as f32 / 2.0,
        //     ))
        //     .end(Point2::new(
        //         WIDTH_NET / 2.0,
        //         (time_since_start.sin() * HEIGHT as f64) as f32 / 2.0,
        //     ));
        // self.draw
        //     .line()
        //     .color(RED)
        //     .stroke_weight(100.0)
        //     .start(Point2::new(
        //         (time_since_start.sin() * (WIDTH_NET as f64)) as f32 / 2.0,
        //         -HEIGHT / 2.0,
        //     ))
        //     .end(Point2::new(
        //         (time_since_start.sin() * (WIDTH_NET as f64)) as f32 / 2.0,
        //         HEIGHT / 2.0,
        //     ));
        // if self.time_animation.elapsed() > Duration::from_millis(1000) {
        //     self.time_animation = Instant::now();
        //     self.stripe = (0..15).map(|_| rng.gen_range(14..25)).collect();
        // }

        // for stripe_index in self.stripe.iter() {
        //     for i in STRIPES_NUM_LEDS[*stripe_index]..STRIPES_NUM_LEDS[stripe_index + 1] {
        //         self.render_led(
        //             i,
        //             Srgb::new(
        //                 255,
        //                 255,
        //                 255,
        //             ),
        //         )
        //     }
        // }
        //
        let pos = (*self.viewpoint.lock().unwrap()).clone();
        let mut new = self.new.lock().unwrap();
        *new = match *new {
            true => {
                self.cones.tracking_time_left = Duration::new(3, 0);
                false
            }
            false => false,
        };

        let view = match pos {
            Some(expr) => Monolith::from_camera_to_col(expr),
            None => Vector::default(),
        };
        let range = 200.0;

        for i in 0..self.cones.positions.len() {
            let x_offset = rng.gen_range(-range..range);
            let y_offset = rng.gen_range(-range..range);
            let z_offset = rng.gen_range(-range..range);
            if self.cones.tracking_time_left.is_zero() {
                self.cones.positions[i] = Vector::new(
                    self.cones.positions[i].x + x_offset * 10.0,
                    self.cones.positions[i].y + y_offset * 10.0,
                    self.cones.positions[i].z + z_offset * 10.0,
                );
                self.cones.positions[i] = self.cones.positions[i].normalize() * 3000.0;
            } else {
                let distance: Vector<Real> = (view - self.cones.positions[i]).into();
                self.cones.positions[i] = (distance * 0.1) + self.cones.positions[i];
                self.cones.tracking_time_left = self
                    .cones
                    .tracking_time_left
                    .saturating_sub(Duration::from_secs_f64(time_since_update / 3.0))
            }
            self.cones.positions[i] = Vector::new(
                self.cones.positions[i].x + x_offset,
                self.cones.positions[i].y + y_offset,
                self.cones.positions[i].z + z_offset,
            );
        }

        // self.draw_viewcone(time_since_start, view, 400.0, 50.0, 1.0, 1.0);

        self.draw_viewcone2(
            time_since_start,
            self.cones.positions[0].into(),
            200.0,
            CYAN,
        );
        self.draw_viewcone2(
            time_since_start,
            self.cones.positions[1].into(),
            200.0,
            YELLOW,
        );
        self.draw_viewcone2(
            time_since_start,
            self.cones.positions[2].into(),
            200.0,
            MAGENTA,
        );

        let time_since_start = update.since_last.secs();
        let string = format!("{:.2}", time_since_start);
        self.draw
            .text(&string)
            .color(HOTPINK)
            .font_size(96)
            .align_text_bottom()
            .left_justify()
            .w_h(WIDTH_NET - 96.0, HEIGHT - 96.0);
    }

    fn draw_line(&self, stroke_weight: f32, time_since_update: f32, y: f32) {
        let t = time_since_update * 0.5;
        let k = PI / HEIGHT;
        let y = y + (HEIGHT / 2.0) * (k * y).cos() * (t).cos();

        self.draw
            .line()
            .hsv(0.55, 1.0, 1.0)
            .stroke_weight(y.abs() / 10.0 + 60.0)
            .start(Point2::new(-WIDTH_NET / 2.0, y))
            .end(Point2::new(WIDTH_NET / 2.0, y));
    }

    fn draw_viewcone(
        &self,
        update: f64,
        view: Point3<Real>,
        circle_radius: f32,
        h: ColorScalar,
        s: ColorScalar,
        v: ColorScalar,
    ) {
        for (i, coordinate) in self.led_coordinates.led_2d.iter().enumerate() {
            if self.collision_detector.detect_collison(
                view,
                &self.led_coordinates.led_3d[i],
                circle_radius,
            ) {
                self.draw
                    .ellipse()
                    .hsv(h, s, v)
                    .w(10.0)
                    .h(10.0)
                    .x_y(coordinate.x as f32, coordinate.y as f32);
            }
        }
    }

    fn draw_viewcone2(&self, update: f64, view: Point3<Real>, circle_radius: f32, color: Srgb<u8>) {
        for (i, coordinate) in self.led_coordinates.led_2d.iter().enumerate() {
            if self.collision_detector.detect_collison(
                view,
                &self.led_coordinates.led_3d[i],
                circle_radius,
            ) {
                self.draw
                    .blend(BLEND_ADD)
                    .ellipse()
                    .color(color)
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

    fn from_camera_to_col(point: Point2) -> Vector<Real> {
        let mm_per_px_x = 2340.0 / 90.0;
        let mm_per_px_y = 6220.0 / 265.0;
        Vector::new(
            point.y * mm_per_px_x - 3000.0,
            1880.0,
            -point.x * mm_per_px_y + 5400.0,
        )
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
        let sender_clone = self.client_configs.clone();

        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory").to_owned();
                let addr = "/";
                let sender = sender_clone.lock().unwrap();
                for client in sender.iter() {
                    let args =
                        Monolith::get_pixels(&led_coordinates[client.led_range.clone()], &image);
                    client.sender.send((addr, args)).ok();
                }
            })
            .unwrap();
    }

    fn get_pixels(
        led_coordinates: &[Point2],
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
