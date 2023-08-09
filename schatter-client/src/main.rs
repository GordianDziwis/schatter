use nannou_osc as osc;
use osc::Packet;
use schatter_client::{display, osc_color_to_rgb8};
use smart_leds::colors::*;
use smart_leds::gamma;
use smart_leds::RGB8;
use std::env;
use std::{thread, time};

#[cfg(target_arch = "arm")]
use ws281x_rpi::Ws2812Rpi;

use smart_leds::SmartLedsWrite;

const PIN: i32 = 10;
const NUM_LEDS: i32 = 1300;
const PORT: u16 = 34254;
const MTU: usize = 50000;

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!("usage: {} {{ test,stream }}", &args[0]);
    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }

    match args[1].as_str() {
        "test" => {
            #[cfg(target_arch = "arm")]
            test();
        }
        "stream" => {
            stream();
        }
        _ => {
            println!("{}", usage);
            ::std::process::exit(1);
        }
    }
}

fn stream() {
    #[cfg(target_arch = "arm")]
    let mut ws = Ws2812Rpi::new(NUM_LEDS, PIN).unwrap();
    let receiver = osc::Receiver::bind_with_mtu(PORT, 50000).expect("Could not bind to socket");
    loop {
        const DELAY: time::Duration = time::Duration::from_millis(10);
        thread::sleep(DELAY);

        // Receive any pending osc packets.
        for (packet, _) in receiver.try_iter() {
            let stripe = get_rgb(packet);
            let stripe_gamma_corrected = gamma(stripe.iter().cloned()).collect();
            #[cfg(debug_assertions)]
            display(&stripe_gamma_corrected);
            #[cfg(target_arch = "arm")]
            ws.write(stripe_gamma_corrected.iter().cloned()).unwrap();
        }
    }
}

#[cfg(target_arch = "arm")]
fn test() {
    let mut ws = Ws2812Rpi::new(NUM_LEDS, PIN).unwrap();
    let pattern: Vec<RGB8> = vec![
        RED,
        GREEN,
        BLUE,
        MAGENTA,
        YELLOW,
        WHITE,
    ];
    let mut stripe = Vec::default();
    for i in 0..(NUM_LEDS) {
        let n = (i as usize) % pattern.len();
        stripe.push(pattern[n]);
    }

    loop {
        const DELAY: time::Duration = time::Duration::from_millis(10);
        thread::sleep(DELAY);
        #[cfg(debug_assertions)]
        display(&stripe);
        stripe.rotate_right(1);
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
