#![allow(unused)]

use std::{
    error::Error,
    num,
    sync::{Arc, atomic::AtomicBool},
    thread::sleep,
    time::{Duration, Instant},
};

use rppal::gpio::{Gpio, OutputPin};
const CUR: u64 = 140;
const SHORT: u64 = 300 - CUR; //220-380 for 1 220-420
const LONG: u64 = 950 - CUR; //580-1600

//each bit is 1,25us

const T0H: Duration = Duration::from_nanos(SHORT);
const T1H: Duration = Duration::from_nanos(LONG);
const T0L: Duration = Duration::from_nanos(LONG);
const T1L: Duration = Duration::from_nanos(SHORT);
const RES: Duration = Duration::from_micros(300); // > 280us

pub struct LEDStripe {
    pin: OutputPin,
    number_of_leds: usize,
}

impl LEDStripe {
    pub fn new(pin: u8, number_of_leds: usize) -> Result<Self, Box<dyn Error>> {
        if number_of_leds == 0 {
            panic!("number cannot be zero")
        }
        Ok(Self {
            pin: Gpio::new()?.get(pin)?.into_output_low(),
            number_of_leds: number_of_leds,
        })
    }
    #[inline]
    fn send_0_code(&mut self) {
        self.pin.set_high();
        let target = Instant::now() + T0H;
        while Instant::now() < target {
            std::hint::spin_loop();
        }
        //sleep(T0H);
        self.pin.set_low();
        let target = Instant::now() + T0L;
        while Instant::now() < target {
            std::hint::spin_loop();
        }
        //sleep(T0L);
    }
    #[inline]
    fn send_1_code(&mut self) {
        self.pin.set_high();
        let target = Instant::now() + T1H;
        while Instant::now() < target {
            std::hint::spin_loop();
        }
        //sleep(T1H);
        self.pin.set_low();
        let target = Instant::now() + T1L;
        while Instant::now() < target {
            std::hint::spin_loop();
        }
        //sleep(T1L);
    }
    #[inline]
    fn send_ret_code(&mut self) {
        self.pin.set_low();
        let target = Instant::now() + RES;
        while Instant::now() < target {
            std::hint::spin_loop();
        }
    }
    fn activate_frame(&mut self, frame: &String) {
        //dbg!(frame);
        //let start = Instant::now();
        for bit in frame.chars() {
            match bit {
                '0' => self.send_0_code(),
                '1' => self.send_1_code(),
                _ => {}
            }
        }
        //dbg!(Instant::now() - start);
        self.send_ret_code();
    }
    pub fn reset(&mut self) {
        self.send_ret_code();
        for _ in 0..(self.number_of_leds * 24) {
            self.send_0_code();
        }
        self.send_ret_code();
    }
    pub fn activate_sequenz(&mut self, sequenz: Sequenz, repeat: Arc<AtomicBool>) {
        self.reset();
        let frame_rate = sequenz.framerate;
        let wait = Duration::from_secs_f32(1.0 / frame_rate);
        let refino = sequenz.refine();

        for rframe in &refino.frames {
            let target = Instant::now() + wait;
            self.activate_frame(rframe);
            if !repeat.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            while Instant::now() < target {
                std::hint::spin_loop();
            }
        }

        while repeat.load(std::sync::atomic::Ordering::SeqCst) {
            for rframe in &refino.frames {
                self.activate_frame(rframe);
                let target = Instant::now() + wait;
                while Instant::now() < target {
                    std::hint::spin_loop();
                }
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
}

pub struct SequenzGenerator;

impl SequenzGenerator {
    pub fn create_static(num_of_leds: usize, color: (u8, u8, u8)) -> Sequenz {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(LED::new(color.0, color.1, color.2))
        }
        Sequenz {
            frames: vec![Frame(on)],
            framerate: 1.0,
        }
    }
    pub fn create_blink(num_of_leds: usize, color: (u8, u8, u8), frequenz: f32) -> Sequenz {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(LED::new(color.0, color.1, color.2))
        }
        let mut off = Vec::new();
        for _ in 0..num_of_leds {
            off.push(LED::new(0, 0, 0))
        }
        Sequenz {
            frames: vec![Frame(on), Frame(off)],
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
            let mut frame_vec = vec![LED::default(); num_of_leds];
            frame_vec[i] = LED::new(color.0, color.1, color.2);

            if blur_trail > 0 {
                let ii = i as i32;
                for j in 1..=blur_trail {
                    let dev = blur_trail as u8 + 1;
                    let mul = blur_trail as u8 + 1 - j as u8;
                    frame_vec[(ii - j as i32).rem_euclid(num_of_leds as i32) as usize] = LED::new(
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
                    frame_vec[(ii + j as i32).rem_euclid(num_of_leds as i32) as usize] = LED::new(
                        color.0 / dev * mul,
                        color.1 / dev * mul,
                        color.2 / dev * mul,
                    );
                }
            }

            v.push(Frame(frame_vec));
        }

        Sequenz {
            frames: v,
            framerate: frequenz,
        }
    }
    pub fn interpolate_frames(sequenz: Sequenz, steps: usize) -> Sequenz {
        todo!()
    }
}

pub struct Sequenz {
    frames: Vec<Frame>,
    framerate: f32,
}
impl Sequenz {
    pub fn new(frames: Vec<Frame>, framerate: f32) -> Self {
        Sequenz { frames, framerate }
    }
    fn refine(self) -> RefinedSequenz {
        let mut rfframes = Vec::new();
        for frame in self.frames {
            rfframes.push(frame.refine().0);
        }
        RefinedSequenz {
            frames: rfframes,
            framerate: self.framerate,
        }
    }
}

struct RefinedSequenz {
    frames: Vec<String>,
    framerate: f32,
}
pub struct Frame(pub Vec<LED>);

impl Frame {
    fn refine(self) -> RefinedFrame {
        let mut rf = String::new();
        for led in &self.0 {
            rf.push_str(&format!("{:08b}", led.g));
            rf.push_str(&format!("{:08b}", led.r));
            rf.push_str(&format!("{:08b}", led.b));
        }
        // rf = rf
        //     .as_bytes()
        //     .chunks(24)
        //     .map(|chunk| {
        //         if chunk.iter().all(|&b| b == b'0') {
        //             "1".repeat(24)
        //         } else {
        //             std::str::from_utf8(chunk).unwrap().to_string()
        //         }
        //     })
        //     .collect();
        RefinedFrame(rf)
    }
}

struct RefinedFrame(String);
#[derive(Debug, Clone)]
pub struct LED {
    r: u8,
    g: u8,
    b: u8,
}

impl LED {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
impl Default for LED {
    fn default() -> Self {
        LED { r: 0, g: 0, b: 0 }
    }
}

fn lerp_rgb(color1: (u8, u8, u8), color2: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0); // Sicherstellen, dass t im Bereich [0, 1] liegt
    let r = (color1.0 as f32 * (1.0 - t) + color2.0 as f32 * t) as u8;
    let g = (color1.1 as f32 * (1.0 - t) + color2.1 as f32 * t) as u8;
    let b = (color1.2 as f32 * (1.0 - t) + color2.2 as f32 * t) as u8;
    (r, g, b)
}
#[cfg(test)]
mod tests {
    use crate::led_stripe::{Frame, LED, LEDStripe, Sequenz, SequenzGenerator};

    //#[test]
    fn test_frame_refining() {
        let led1 = LED::new(0, 255, 170);
        let led2 = LED::new(255, 0, 85);

        let frame1 = Frame(vec![led1, led2]);

        let rframe1 = frame1.refine();
        assert_eq!(
            &rframe1.0,
            "111111110000000010101010000000001111111101010101"
        );
        let led1 = LED::new(0, 255, 170);
        let led2 = LED::new(255, 0, 85);
        let frame2 = Frame(vec![led2, led1]);
        assert_eq!(
            &frame2.refine().0,
            "000000001111111101010101111111110000000010101010"
        );
    }
    // #[test]
    fn test_create_dot_normal() {
        let seq = SequenzGenerator::create_dot(3, (255, 255, 255), 0.0, 0, 0);

        let rseq = seq.refine();
        let rf1 = &rseq.frames[0];
        let rf2 = &rseq.frames[1];
        let rf3 = &rseq.frames[2];
        assert_eq!(
            rf1,
            "111111111111111111111111000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            rf2,
            "000000000000000000000000111111111111111111111111000000000000000000000000"
        );
        assert_eq!(
            rf3,
            "000000000000000000000000000000000000000000000000111111111111111111111111"
        );
    }
    //#[test]
    fn test_create_dot_blur_trail() {
        let seq = SequenzGenerator::create_dot(5, (255, 255, 255), 0.0, 2, 0);

        let rseq = seq.refine();
        let rf1 = &rseq.frames[0];
        let rf2 = &rseq.frames[1];
        let rf3 = &rseq.frames[2];
        let rf4 = &rseq.frames[3];
        let rf5 = &rseq.frames[4];
        assert_eq!(
            rf1,
            "111111111111111111111111000000000000000000000000000000000000000000000000010101010101010101010101101010101010101010101010"
        );

        assert_eq!(
            rf3,
            "010101010101010101010101101010101010101010101010111111111111111111111111000000000000000000000000000000000000000000000000"
        );
    }
    //#[test]
    fn test_create_dot_blur_head() {
        let seq = SequenzGenerator::create_dot(5, (255, 255, 255), 0.0, 0, 2);

        let rseq = seq.refine();
        let rf1 = &rseq.frames[0];
        let rf2 = &rseq.frames[1];
        let rf3 = &rseq.frames[2];
        let rf4 = &rseq.frames[3];
        let rf5 = &rseq.frames[4];
        assert_eq!(
            rf1,
            "111111111111111111111111101010101010101010101010010101010101010101010101000000000000000000000000000000000000000000000000"
        );

        assert_eq!(
            rf3,
            "000000000000000000000000000000000000000000000000111111111111111111111111101010101010101010101010010101010101010101010101"
        );
    }
    //#[test]
    fn test_create_dot_blur_both() {
        let seq = SequenzGenerator::create_dot(5, (255, 255, 255), 0.0, 2, 2);

        let rseq = seq.refine();
        let rf1 = &rseq.frames[0];
        let rf2 = &rseq.frames[1];
        let rf3 = &rseq.frames[2];
        let rf4 = &rseq.frames[3];
        let rf5 = &rseq.frames[4];
        assert_eq!(
            rf1,
            "111111111111111111111111101010101010101010101010010101010101010101010101010101010101010101010101101010101010101010101010"
        );

        assert_eq!(
            rf3,
            "010101010101010101010101101010101010101010101010111111111111111111111111101010101010101010101010010101010101010101010101"
        );
    }
}
