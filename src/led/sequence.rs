use std::ops::{Add, Div, Shl, Shr};

use crate::led::{frame::Frame, led::LED};

#[derive(Debug, Clone)]
pub struct Sequence {
    frames: Vec<Frame>,
    framerate: f32,
}

impl Sequence {
    pub fn new(frames: Vec<Frame>, framerate: f32) -> Self {
        assert!(!frames.is_empty());
        assert!(framerate > 0.0);

        Sequence { frames, framerate }
    }
    pub fn get_framerate(&self) -> &f32 {
        &self.framerate
    }
    pub fn get_frames(&self) -> &Vec<Frame> {
        &self.frames
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn reverse(&self) -> Sequence {
        let mut rev = self.frames.clone();
        rev.reverse();
        Sequence {
            frames: rev,
            framerate: self.framerate,
        }
    }

    pub fn pulse(&self, steps: usize, lows: f32) -> Self {
        // for smooth look have at least 30Hz
        let seq = match self.framerate >= 30.0 {
            true => self.clone(),
            false => self.change_framerate(30.0),
        };

        let mut v = vec![];

        for (i, frame) in seq.get_frames().iter().enumerate() {
            let current_phase = i % (2 * steps);
            let step = match current_phase <= steps {
                true => current_phase,
                false => (2 * steps) - current_phase,
            };
            let fac = ((lows - 1.0) / steps as f32) * step as f32 + 1.0;
            // dbg!(fac);
            v.push(frame.scale(fac));
        }

        Sequence {
            frames: v,
            framerate: seq.framerate,
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
    pub fn change_framerate(&self, new_framerate: f32) -> Self {
        assert!(new_framerate.is_finite() && new_framerate > 0.0);
        assert!(self.framerate.is_finite() && self.framerate > 0.0);

        let duration = self.frames.len() as f32 / self.framerate;
        let new_len = (duration * new_framerate).round() as usize;

        let mut new_frames = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let t = i as f32 / new_framerate;
            let src_index = (t * self.framerate)
                .floor()
                .clamp(0.0, (self.frames.len() - 1) as f32) as usize;

            new_frames.push(self.frames[src_index].clone());
        }

        Self {
            frames: new_frames,
            framerate: new_framerate,
        }
    }
    pub fn add(&self, other: &Self) -> Self {
        let framerate = self.framerate.max(other.framerate);
        let first_seq = self.change_framerate(framerate);
        let second_seq = other.change_framerate(framerate);

        let new_len = first_seq.len().max(second_seq.len());
        let mut v = Vec::new();

        let empty = Frame(vec![]);
        for i in 0..new_len {
            let f1 = first_seq.get_frames().get(i).unwrap_or(&empty);
            let f2 = second_seq.get_frames().get(i).unwrap_or(&empty);

            v.push(f1.add(f2));
        }

        Self {
            frames: v,
            framerate: framerate,
        }
    }
    pub fn concat(&self, other: &Self) -> Self {
        let framerate = self.framerate.max(other.framerate);
        let mut first_seq = self.change_framerate(framerate);
        let second_seq = other.change_framerate(framerate);

        first_seq.frames.extend(second_seq.frames);

        Self {
            frames: first_seq.frames,
            framerate,
        }
    }
    pub fn shl(&self, mid: usize) -> Self {
        let mut t = self.frames.clone();
        t.rotate_left(mid);
        Self {
            frames: t,
            framerate: self.framerate,
        }
    }
    pub fn shr(&self, mid: usize) -> Self {
        let mut t = self.frames.clone();
        t.rotate_right(mid);
        Self {
            frames: t,
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
                    .unwrap_or(&LED(0, 0, 0));
                let color_from_second = second
                    .get(i)
                    .and_then(|frame| frame.get(j))
                    .unwrap_or(&LED(0, 0, 0));
                new_frame.push(color_from_first.add(color_from_second));
            }
            v.push(Frame(new_frame));
        }
        Sequence::new(v, self.framerate.max(rhs.framerate))
    }
}
impl Div for Sequence {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.concat(&rhs)
    }
}

impl Shl<usize> for Sequence {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self::Output {
        let len = self.frames.len();
        let mut frs = self.frames;
        frs.rotate_right(rhs % len);
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
        frs.rotate_left(rhs % len);
        Sequence {
            frames: frs,
            framerate: self.framerate,
        }
    }
}
