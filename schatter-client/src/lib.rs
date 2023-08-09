use colored::*;
use nannou_osc as osc;
use rand::Rng;
use smart_leds::RGB8;
use std::fmt;
use std::io::Write;

pub fn display(leds: &Vec<Led>) {
    for led in leds {
        print!("{}", led);
    }
    print!("\r");
    std::io::stdout().flush().unwrap();
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Led {
    r: u8,
    g: u8,
    b: u8,
}

impl Led {
    pub fn next_color(&mut self) {
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..3);

        fn increment_or_reset(color: &mut u8) {
            if *color < 255 {
                *color += 1;
            } else {
                *color = 0;
            }
        }

        match num {
            0 => increment_or_reset(&mut self.r),
            1 => increment_or_reset(&mut self.g),
            _ => increment_or_reset(&mut self.b),
        }
    }
}

impl From<RGB8> for Led {
    fn from(rgb: RGB8) -> Self {
        Led {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}
impl From<Led> for RGB8 {
    fn from(led: Led) -> Self {
        RGB8 {
            r: led.r,
            g: led.g,
            b: led.b,
        }
    }
}
impl<'a> From<&'a Led> for RGB8 {
    fn from(led: &'a Led) -> Self {
        RGB8 {
            r: led.r,
            g: led.g,
            b: led.b,
        }
    }
}
impl<'a> From<&'a RGB8> for Led {
    fn from(rgb: &'a RGB8) -> Self {
        Led {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}
impl From<osc::Color> for Led {
    fn from(color: osc::Color) -> Self {
        Led {
            r: color.red,
            g: color.green,
            b: color.blue,
        }
    }
}

impl fmt::Display for Led {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "█".truecolor(self.r, self.g, self.b))
    }
}
