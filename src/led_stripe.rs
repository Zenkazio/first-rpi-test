//#![allow(unused)]

use std::{
    ops::{Add, Div, Shl, Shr},
    sync::{Arc, atomic::AtomicBool},
    thread::sleep,
    time::Duration,
};

use ws2818_rgb_led_spi_driver::{adapter_gen::WS28xxAdapter, adapter_spi::WS28xxSpiAdapter};
unsafe impl Send for LEDStripe {}
unsafe impl Sync for LEDStripe {}
pub struct LEDStripe {
    adapter: WS28xxSpiAdapter,
    number_of_leds: usize,
}

impl LEDStripe {
    pub fn new(number_of_leds: usize) -> Self {
        if number_of_leds == 0 {
            panic!("number cannot be zero")
        }
        Self {
            adapter: WS28xxSpiAdapter::new("/dev/spidev0.0").unwrap(),
            number_of_leds: number_of_leds,
        }
    }

    pub fn reset(&mut self) {
        let mut v = vec![];
        for _ in 0..self.number_of_leds {
            v.push((0, 0, 0));
        }
        self.adapter.write_rgb(&v).unwrap();
    }
    pub fn activate_sequenz(&mut self, sequenz: Sequenz, repeat: Arc<AtomicBool>) {
        self.reset();
        let wait = Duration::from_secs_f32(1.0 / sequenz.framerate);

        for frame in &sequenz.frames {
            self.adapter.write_rgb(frame).unwrap();
            sleep(wait);
            if !repeat.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
        }
        while repeat.load(std::sync::atomic::Ordering::SeqCst) {
            for frame in &sequenz.frames {
                self.adapter.write_rgb(&frame).unwrap();
                sleep(wait);
                if !repeat.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
            }
        }
    }
    pub fn create_static(&self, color: (u8, u8, u8)) -> Sequenz {
        SequenzGenerator::create_static(self.number_of_leds, color)
    }
    pub fn create_blink(&self, color: (u8, u8, u8), frequenz: f32) -> Sequenz {
        SequenzGenerator::create_blink(self.number_of_leds, color, frequenz)
    }
    pub fn create_dot(
        &self,
        color: (u8, u8, u8),
        frequenz: f32,
        blur_trail: usize,
        blur_head: usize,
    ) -> Sequenz {
        SequenzGenerator::create_dot(self.number_of_leds, color, frequenz, blur_trail, blur_head)
    }
    pub fn custom(&self) -> Sequenz {
        SequenzGenerator::custom(self.number_of_leds)
    }
}

pub struct SequenzGenerator;

impl SequenzGenerator {
    pub fn create_static(num_of_leds: usize, color: (u8, u8, u8)) -> Sequenz {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(color)
        }
        Sequenz {
            frames: vec![on],
            framerate: 1.0,
        }
    }
    pub fn create_blink(num_of_leds: usize, color: (u8, u8, u8), frequenz: f32) -> Sequenz {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(color)
        }
        let mut off = Vec::new();
        for _ in 0..num_of_leds {
            off.push((0, 0, 0))
        }
        Sequenz {
            frames: vec![on, off],
            framerate: frequenz,
        }
    }
    pub fn create_dot(
        num_of_leds: usize,
        color: (u8, u8, u8),
        frequenz: f32,
        blur_trail: usize,
        blur_head: usize,
    ) -> Sequenz {
        let mut v = vec![];
        for i in 0..num_of_leds {
            let mut frame_vec = vec![(0, 0, 0); num_of_leds];
            frame_vec[i] = color;

            if blur_trail > 0 {
                let ii = i as i32;
                for j in 1..=blur_trail {
                    let dev = blur_trail as u8 + 1;
                    let mul = blur_trail as u8 + 1 - j as u8;
                    frame_vec[(ii - j as i32).rem_euclid(num_of_leds as i32) as usize] = (
                        color.0 / dev * mul,
                        color.1 / dev * mul,
                        color.2 / dev * mul,
                    );
                }
            }
            if blur_head > 0 {
                let ii = i as i32;
                for j in 1..=blur_head {
                    let dev = blur_head as u8 + 1;
                    let mul = blur_head as u8 + 1 - j as u8;
                    frame_vec[(ii + j as i32).rem_euclid(num_of_leds as i32) as usize] = (
                        color.0 / dev * mul,
                        color.1 / dev * mul,
                        color.2 / dev * mul,
                    );
                }
            }

            v.push(frame_vec);
        }

        Sequenz {
            frames: v,
            framerate: frequenz,
        }
    }
    pub fn interpolate_frames(sequenz: Sequenz, steps: usize) -> Sequenz {
        todo!()
    }
    pub fn custom(num_of_leds: usize) -> Sequenz {
        let freq = 20.0;
        let length = 20;
        let mut green =
            SequenzGenerator::create_dot(num_of_leds, (0, 255, 0), freq, length, length);
        let mut red = SequenzGenerator::create_dot(num_of_leds, (255, 0, 0), freq, length, length);
        red = red << 50;
        let mut blue = SequenzGenerator::create_dot(num_of_leds, (0, 0, 255), freq, length, length);
        blue = blue << 100;
        let mut yellow =
            SequenzGenerator::create_dot(num_of_leds, (255, 255, 0), freq, length, length);
        yellow.reverse();
        let mut magenta =
            SequenzGenerator::create_dot(num_of_leds, (255, 0, 255), freq, length, length);
        magenta = magenta << 50;
        magenta.reverse();
        let mut cyan =
            SequenzGenerator::create_dot(num_of_leds, (0, 255, 255), freq, length, length);
        cyan = cyan << 100;
        cyan.reverse();
        green + red + blue + yellow + magenta + cyan
    }
}

pub struct Sequenz {
    frames: Vec<Vec<(u8, u8, u8)>>,
    framerate: f32,
}

impl Sequenz {
    pub fn new(frames: Vec<Vec<(u8, u8, u8)>>, framerate: f32) -> Self {
        if frames.len() == 0 {
            panic!("empty frames");
        }
        Sequenz { frames, framerate }
    }
    pub fn reverse(&mut self) {
        let mut rev = self.frames.clone();
        rev.reverse();
        self.frames = rev;
    }
    pub fn pulse(&self, steps: usize, lows: f32) -> Self {
        todo!();
        let range = 1.0 - lows;
        let mut v = vec![];
        for (i, frame) in self.frames.iter().enumerate() {
            let indu = i % (steps + steps - 2);
            if indu == 0 {
                v.push(frame.clone());
            } else if indu == steps - 1 {
            }
        }
        Sequenz {
            frames: v,
            framerate: self.framerate,
        }
    }
    pub fn repeat(&self, num: usize) -> Self {
        let mut v = vec![];
        for _ in 0..num {
            for frame in &self.frames {
                v.push(frame.clone());
            }
        }

        Self {
            frames: v,
            framerate: self.framerate,
        }
    }
}

impl Add for Sequenz {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let first = self.frames;
        let second = rhs.frames;
        let slen = first.len().max(second.len());
        let f1len = first.iter().map(|frame| frame.len()).max().unwrap_or(0);
        let f2len = second.iter().map(|frame| frame.len()).max().unwrap_or(0);
        let flen = f1len.max(f2len);
        let mut v = vec![];
        for i in 0..slen {
            let mut new_frame = vec![];
            for j in 0..flen {
                let color_from_first = first
                    .get(i)
                    .and_then(|frame| frame.get(j))
                    .unwrap_or(&(0, 0, 0));
                let color_from_second = second
                    .get(i)
                    .and_then(|frame| frame.get(j))
                    .unwrap_or(&(0, 0, 0));
                new_frame.push(add_colors(color_from_first, color_from_second));
            }
            v.push(new_frame);
        }
        Sequenz::new(v, self.framerate.max(rhs.framerate))
    }
}
impl Div for Sequenz {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shl<usize> for Sequenz {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self::Output {
        let len = self.frames.len();
        let mut frs = self.frames;
        frs.rotate_left(rhs % len);
        Sequenz {
            frames: frs,
            framerate: self.framerate,
        }
    }
}
impl Shr<usize> for Sequenz {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self::Output {
        let len = self.frames.len();
        let mut frs = self.frames;
        frs.rotate_right(rhs % len);
        Sequenz {
            frames: frs,
            framerate: self.framerate,
        }
    }
}
fn scale_color(color: &(u8, u8, u8), fac: f32) -> (u8, u8, u8) {
    (
        (color.0 as f32 * fac) as u8,
        (color.1 as f32 * fac) as u8,
        (color.2 as f32 * fac) as u8,
    )
}
fn add_colors(color1: &(u8, u8, u8), color2: &(u8, u8, u8)) -> (u8, u8, u8) {
    (
        (color1.0 as u16 + color2.0 as u16).min(255) as u8,
        (color1.1 as u16 + color2.1 as u16).min(255) as u8,
        (color1.2 as u16 + color2.2 as u16).min(255) as u8,
    )
}

fn lerp_rgb(color1: (u8, u8, u8), color2: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0); // Sicherstellen, dass t im Bereich [0, 1] liegt
    let r = (color1.0 as f32 * (1.0 - t) + color2.0 as f32 * t) as u8;
    let g = (color1.1 as f32 * (1.0 - t) + color2.1 as f32 * t) as u8;
    let b = (color1.2 as f32 * (1.0 - t) + color2.2 as f32 * t) as u8;
    (r, g, b)
}

#[cfg(test)]
#[test]
fn test_seq_shift() {
    let mut frame1 = vec![];
    frame1.push((255, 0, 0));
    frame1.push((0, 0, 255));
    let mut frame2 = vec![];
    frame2.push((0, 255, 0));
    frame2.push((0, 255, 0));
    let mut frame3 = vec![];
    frame3.push((0, 0, 255));
    frame3.push((255, 0, 0));
    let mut frames = vec![frame1, frame2, frame3];
    let s = Sequenz::new(frames, 1.0);
    dbg!(&s.frames);
    let sl = s << 1;
    dbg!(&sl.frames);
    let sr = sl >> 1;
    dbg!(&sr.frames);
    assert!(true);
}
