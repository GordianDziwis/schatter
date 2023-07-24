use nannou::prelude::*;
use nannou_osc as osc;

use std::{thread, time};

#[cfg(target_arch = "arm")]
use {
    smart_leds::{SmartLedsWrite, RGB8},
    ws281x_rpi::Ws2812Rpi,
};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    receiver: osc::Receiver,
    received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
}

// Make sure this matches the `TARGET_PORT` in the `osc_sender.rs` example.
const PORT: u16 = 34254;

fn model(app: &App) -> Model {
    let _w_id = app
        .new_window()
        .title("OSC Receiver")
        .size(1400, 480)
        .view(view)
        .build()
        .unwrap();

    // Bind an `osc::Receiver` to a port.
    let receiver = osc::receiver(PORT).unwrap();

    // A vec for collecting packets and their source address.
    let received_packets = vec![];

    Model {
        receiver,
        received_packets,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Receive any pending osc packets.
    for (packet, addr) in model.receiver.try_iter() {
        model.received_packets.push((addr, packet));
    }

    // We'll display 10 packets at a time, so remove any excess.
    let max_packets = 10;
    while model.received_packets.len() > max_packets {
        model.received_packets.remove(0);
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(DARKBLUE);

    // Create a string showing all the packets.
    let mut packets_text = format!("Listening on port {}\nReceived packets:\n", PORT);
    for &(addr, ref packet) in model.received_packets.iter().rev() {
        packets_text.push_str(&format!("{}: {:?}\n", addr, packet));
    }
    let rect = frame.rect().pad(10.0);
    draw.text(&packets_text)
        .font_size(16)
        .align_text_top()
        .line_spacing(10.0)
        .left_justify()
        .wh(rect.wh());

    draw.to_frame(app, &frame).unwrap();
}

#[cfg(target_arch = "arm")]
fn blink() {
    // GPIO Pin 10 is SPI
    // Other modes and PINs are available depending on the Raspberry Pi revision
    // Additional OS configuration might be needed for any mode.
    // Check https://github.com/jgarff/rpi_ws281x for more information.
    const PIN: i32 = 10;
    const NUM_LEDS: usize = 8;
    const DELAY: time::Duration = time::Duration::from_millis(1000);

    let mut ws = Ws2812Rpi::new(NUM_LEDS as i32, PIN).unwrap();

    let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
    let empty: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    // Blink the LED's in a blue-green-red-white pattern.
    for led in data.iter_mut().step_by(4) {
        led.b = 32;
    }

    if NUM_LEDS > 1 {
        for led in data.iter_mut().skip(1).step_by(4) {
            led.g = 32;
        }
    }

    if NUM_LEDS > 2 {
        for led in data.iter_mut().skip(2).step_by(4) {
            led.r = 32;
        }
    }

    if NUM_LEDS > 3 {
        for led in data.iter_mut().skip(3).step_by(4) {
            led.r = 32;
            led.g = 32;
            led.b = 32;
        }
    }

    loop {
        // On
        println!("LEDS on");
        ws.write(data.iter().cloned()).unwrap();
        thread::sleep(DELAY);

        // Off
        println!("LEDS off");
        ws.write(empty.iter().cloned()).unwrap();
        thread::sleep(DELAY);
    }
}
