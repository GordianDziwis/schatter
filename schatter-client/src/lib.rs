use std::fmt;
use std::io::{Write, stdout};

use colored::*;
use nannou_osc as osc;
use smart_leds::RGB8;
use termion::cursor::Goto;

struct RGB8Wrapper(RGB8);

pub fn osc_color_to_rgb8(color: osc::Color) -> RGB8 {
    RGB8 {
        r: color.red,
        g: color.green,
        b: color.blue,
    }
}
impl fmt::Display for RGB8Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "█".truecolor(self.0.r, self.0.g, self.0.b))
    }
}

pub fn display(leds: &Vec<RGB8>) {
    print!("{}", Goto(1, 1)); // Move the cursor to the beginning of the line
    for led in leds {
        print!("{}", RGB8Wrapper(*led));
    }
    stdout().flush().unwrap();
}
