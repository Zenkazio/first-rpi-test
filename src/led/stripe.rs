use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use ws2818_rgb_led_spi_driver::{
    adapter_gen::WS28xxAdapter, adapter_spi::WS28xxSpiAdapter, encoding::encode_rgb,
};

use crate::led::{
    frame::Frame, led::LED, sequence::Sequence, sequence_generator::SequenzGenerator,
};

unsafe impl Send for Stripe {}
unsafe impl Sync for Stripe {}
pub struct Stripe {
    adapter: WS28xxSpiAdapter,
    number_of_leds: usize,
    running: Arc<AtomicBool>,
}

impl Stripe {
    pub fn new(number_of_leds: usize) -> Self {
        assert!(number_of_leds > 0);

        Self {
            adapter: WS28xxSpiAdapter::new("/dev/spidev0.0").unwrap(),
            number_of_leds: number_of_leds,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn get_number_of_leds(&self) -> usize {
        self.number_of_leds
    }

    pub fn reset(&mut self) {
        let mut v = vec![];
        for _ in 0..self.number_of_leds {
            v.push((0, 0, 0));
        }
        self.adapter.write_rgb(&v).unwrap();
    }
    pub fn activate_sequenz(&mut self, sequence: Sequence) {
        self.reset();
        let wait = Duration::from_secs_f32(1.0 / sequence.get_framerate());

        let rs = refine_sequence(&sequence);

        for frame in &rs {
            self.adapter
                .write_encoded_rgb(frame)
                .expect("write encoded rgb");
            sleep(wait);
            if !self.running.load(Ordering::SeqCst) {
                break;
            }
        }
        while self.running.load(Ordering::SeqCst) {
            for frame in &rs {
                self.adapter
                    .write_encoded_rgb(frame)
                    .expect("write encoded rgb");
                sleep(wait);
                if !self.running.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }
    pub fn strength(&mut self, strength: f32, color: (u8, u8, u8)) -> Frame {
        let fac = strength.clamp(0.0, 1.0);
        let end = (self.number_of_leds as f32 * fac) as usize;
        let mut v = Vec::new();
        for i in 0..self.number_of_leds {
            if i + 1 <= end {
                v.push(LED::from_color(color));
            } else {
                v.push(LED::from_color((0, 0, 0)));
            }
        }
        Frame(v)
    }
    pub fn activate_frame(&mut self, frame: &Frame) {
        self.running.store(false, Ordering::SeqCst);
        self.adapter
            .write_rgb(&frame.to_vec())
            .expect("in activate frame");
    }

    pub fn get_running_clone(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    pub fn create_static(&self, color: (u8, u8, u8)) -> Sequence {
        SequenzGenerator::create_static(self.number_of_leds, color)
    }
    pub fn create_blink(&self, color: (u8, u8, u8), frequenz: f32) -> Sequence {
        SequenzGenerator::create_blink(self.number_of_leds, color, frequenz)
    }
    pub fn create_dot(
        &self,
        color: (u8, u8, u8),
        frequenz: f32,
        blur_trail: usize,
        blur_head: usize,
    ) -> Sequence {
        SequenzGenerator::create_dot(self.number_of_leds, color, frequenz, blur_trail, blur_head)
    }
    pub fn custom(&self) -> Sequence {
        SequenzGenerator::custom(self.number_of_leds)
    }

    pub fn red_alert(&self) -> Sequence {
        SequenzGenerator::red_alert(self.number_of_leds)
    }
}
fn refine_sequence(seq: &Sequence) -> Vec<Vec<u8>> {
    let mut v = Vec::new();
    for frame in seq.get_frames() {
        let mut w = Vec::new();
        for led in &frame.0 {
            w.extend_from_slice(&encode_rgb(led.0, led.1, led.2));
        }
        v.push(w);
    }
    v
}
// #[cfg(test)]
// #[test]
// fn test_seq_shift() {
//     let mut frame1 = vec![];
//     frame1.push((255, 0, 0));
//     frame1.push((0, 0, 255));
//     let mut frame2 = vec![];
//     frame2.push((0, 255, 0));
//     frame2.push((0, 255, 0));
//     let mut frame3 = vec![];
//     frame3.push((0, 0, 255));
//     frame3.push((255, 0, 0));
//     let frames = vec![frame1, frame2, frame3];
//     let s = Sequence::new(frames, 1.0);
//     dbg!(&s.get_frames());
//     let sl = s << 1;
//     dbg!(&sl.get_frames());
//     let sr = sl >> 1;
//     dbg!(&sr.get_frames());
//     assert!(true);
// }
