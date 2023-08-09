use nannou_osc as osc;
use osc::Packet;
use schatter_client::{display, Led};
use smart_leds::colors::*;
use smart_leds::gamma;
use smart_leds::Gamma;
use smart_leds::{SmartLedsWrite, RGB8};
use std::env;
use std::net::SocketAddrV4;
use std::ops::Deref;
use std::str::FromStr;
use std::{default, fmt, thread, time};

#[cfg(target_arch = "arm")]
use ws281x_rpi::Ws2812Rpi;

const PIN: i32 = 10;
const NUM_LEDS: i32 = 1300;
const PORT: u16 = 34254;
const MTU: usize = 50000;

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP", &args[0]);
    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    // let addr = match SocketAddrV4::from_str(&args[1]) {
    //     Ok(_) => &args[1],
    //     Err(_) => panic!("{}", usage),
    // };
    let target_addr = format!("{}:{}", "192.168.1.186", PORT);
    let receiver = osc::Receiver::bind_with_mtu(PORT, 50000)
        .expect("Could not bind to socket");

    #[cfg(target_arch = "arm")]
    let mut ws = Ws2812Rpi::new(NUM_LEDS, PIN).unwrap();
    #[cfg(target_arch = "arm")]
    test(&mut ws);

    loop {
        const DELAY: time::Duration = time::Duration::from_millis(10);
        thread::sleep(DELAY);

        // Receive any pending osc packets.
        for (packet, _) in receiver.try_iter() {
            let stripe = get_rgb(packet);

            let stripe = stripe.iter().map(RGB8::from).collect::<Vec<_>>();
            let stripe = gamma(stripe.iter().cloned()).collect::<Vec<_>>();
            let stripe = stripe.iter().map(Led::from).collect::<Vec<_>>();

            #[cfg(debug_assertions)]
            display(&stripe);

            #[cfg(target_arch = "arm")]
            ws.write(stripe.iter().cloned()).unwrap();
        }
    }
}

fn get_rgb(packet: Packet) -> Vec<Led> {
    packet
        .into_msgs()
        .into_iter()
        .flat_map(|message| message.args.unwrap_or_default())
        .filter_map(|arg| match arg {
            osc::Type::Color(color) => Some(color.into()),
            _ => None,
        })
        .collect()
}

#[cfg(target_arch = "arm")]
fn test(ws: &mut Ws2812Rpi) {
    let pattern: Vec<Led> = vec![
        RED.into(),
        GREEN.into(),
        BLUE.into(),
        MAGENTA.into(),
        YELLOW.into(),
        WHITE.into(),
    ];
    let mut stripe = Vec::default();
    for i in 0..(NUM_LEDS) {
        let n = (i as usize) % pattern.len();
        stripe.push(pattern[n]);
    }

    // loop {
    //     const DELAY: time::Duration = time::Duration::from_millis(10);
    //     thread::sleep(DELAY);
    //     #[cfg(debug_assertions)]
    //     display(&stripe);
    //     stripe.rotate_right(1);
    //     match ws.write(stripe.iter().cloned()) {
    //         Ok(_) => (),
    //         Err(e) => println!("{}", e),
    //     }
    // }
    // const DELAY: time::Duration = time::Duration::from_millis(10000);
    // thread::sleep(DELAY);
}

// fn test() {

//     // A vec for collecting packets and their source address.
//     let mut received_packets = vec![];
//     loop {
//         const DELAY: time::Duration = time::Duration::from_millis(100);
//         thread::sleep(DELAY);
//
//         // Receive any pending osc packets.
//         for (packet, addr) in receiver.try_iter() {
//             println!("hi2");
//             received_packets.push((addr, packet));
//         }

//         // We'll display 10 packets at a time, so remove any excess.
//         let max_packets = 10;
//         while received_packets.len() > max_packets {
//             println!("hi2");
//             received_packets.remove(0);
//         }
//         let mut packets_text = format!("Listening on port {}\nReceived packets:\n", PORT);
//
