use std::ops::{Add, Div, Shl, Shr};

use crate::led::frame::Frame;

#[derive(Debug, Clone)]
pub struct Sequence {
    frames: Vec<Frame>,
    framerate: f32,
}
impl Sequence {
    pub fn new(frames: Vec<Vec<(u8, u8, u8)>>, framerate: f32) -> Self {
        if frames.len() == 0 {
            panic!("empty frames");
        }
        Sequence { frames, framerate }
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
        Sequence {
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

impl Add for Sequence {
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
        Sequence::new(v, self.framerate.max(rhs.framerate))
    }
}
impl Div for Sequence {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shl<usize> for Sequence {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self::Output {
        let len = self.frames.len();
        let mut frs = self.frames;
        frs.rotate_left(rhs % len);
        Sequence {
            frames: frs,
            framerate: self.framerate,
        }
    }
}
impl Shr<usize> for Sequence {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self::Output {
        let len = self.frames.len();
        let mut frs = self.frames;
        frs.rotate_right(rhs % len);
        Sequence {
            frames: frs,
            framerate: self.framerate,
        }
    }
}
fn add_colors(color1: &(u8, u8, u8), color2: &(u8, u8, u8)) -> (u8, u8, u8) {
    (
        (color1.0 as u16 + color2.0 as u16).min(255) as u8,
        (color1.1 as u16 + color2.1 as u16).min(255) as u8,
        (color1.2 as u16 + color2.2 as u16).min(255) as u8,
    )
}
