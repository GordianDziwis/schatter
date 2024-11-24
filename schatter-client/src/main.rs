use core::time;
use std::{env, thread};

use nannou_osc as osc;
use osc::Packet;
use schatter_client::{display, osc_color_to_rgb8};
use smart_leds::colors::*;
use smart_leds::{SmartLedsWrite, RGB8};
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use ws281x_rpi::Ws2812Rpi;

const NUM_LEDS: i32 = 700;
const MTU: usize = 10000;

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!(
        "usage: {} {{ test,stream }} {{ port }} {{ pin }} {{ dma }}",
        &args[0]
    );

    let port: u16 = match args.get(2) {
        Some(p) => p.parse().expect("Invalid port number"),
        None => 12345,
    };
    let pin: i32 = match args.get(3) {
        Some(p) => p.parse().expect("Invalid pin"),
        None => 18,
    };
    let dma: i32 = match args.get(4) {
        Some(p) => p.parse().expect("Invalid dma"),
        None => 10,
    };

    match args[1].as_str() {
        "test" => {
            test(pin, dma);
        }
        "stream" => {
            stream(port, pin, dma);
        }
        _ => {
            println!("{}", usage);
            ::std::process::exit(1);
        }
    }
}

fn stream(port: u16, pin: i32, dma: i32) {
    let receiver = osc::Receiver::bind_with_mtu(port, MTU).expect("Could not bind to socket");
    let (packet, _) = receiver.recv().unwrap();
    let len = get_rgb(packet).len();
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let mut ws = Ws2812Rpi::new(len.try_into().unwrap(), pin, dma).unwrap();
    loop {
        for (packet, _) in receiver.iter() {
            // const DELAY: time::Duration = time::Duration::from_millis(30);
            // thread::sleep(DELAY);
            let stripe = get_rgb(packet);
            #[cfg(debug_assertions)]
            display(&stripe);
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            ws.write(stripe.iter().cloned()).unwrap();
        }
    }
}

fn test(pin: i32, dma: i32) {
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let mut ws = Ws2812Rpi::new(NUM_LEDS, pin, dma).unwrap();
    let pattern: Vec<RGB8> = vec![
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE,
        WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
    ];
    let mut stripe = Vec::default();
    for i in 0..(NUM_LEDS) {
        let n = (i as usize) % pattern.len();
        stripe.push(pattern[n]);
    }
    loop {
        const DELAY: time::Duration = time::Duration::from_millis(5);
        thread::sleep(DELAY);
        #[cfg(debug_assertions)]
        display(&stripe);
        stripe.rotate_right(1);
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        match ws.write(stripe.iter().cloned()) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
    }
}

fn get_rgb(packet: Packet) -> Vec<RGB8> {
    packet
        .into_msgs()
        .into_iter()
        .flat_map(|message| message.args.unwrap_or_default())
        .filter_map(|arg| match arg {
            osc::Type::Color(color) => Some(osc_color_to_rgb8(color)),
            _ => None,
        })
        .collect()
}
