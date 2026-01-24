use crate::led::{frame::Frame, led::LED, sequence::Sequence};

pub struct SequenzGenerator;

impl SequenzGenerator {
    pub fn create_static(num_of_leds: usize, color: (u8, u8, u8)) -> Sequence {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(LED::from_color(color))
        }
        Sequence::new(vec![Frame(on)], 1.0)
    }
    pub fn create_blink(num_of_leds: usize, color: (u8, u8, u8), frequenz: f32) -> Sequence {
        let mut on = Vec::new();
        for _ in 0..num_of_leds {
            on.push(LED::from_color(color));
        }
        let mut off = Vec::new();
        for _ in 0..num_of_leds {
            off.push(LED::default())
        }
        Sequence::new(vec![Frame(on), Frame(off)], frequenz)
    }
    pub fn create_dot(
        num_of_leds: usize,
        color: (u8, u8, u8),
        frequenz: f32,
        blur_trail: usize,
        blur_head: usize,
    ) -> Sequence {
        let mut v = vec![];
        for i in 0..num_of_leds {
            let mut frame_vec = vec![LED::default(); num_of_leds];
            frame_vec[i] = LED::from_color(color);

            if blur_trail > 0 {
                let ii = i as i32;
                for j in 1..=blur_trail {
                    let dev = blur_trail as u8 + 1;
                    let mul = blur_trail as u8 + 1 - j as u8;
                    frame_vec[(ii - j as i32).rem_euclid(num_of_leds as i32) as usize] = LED(
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
                    frame_vec[(ii + j as i32).rem_euclid(num_of_leds as i32) as usize] = LED(
                        color.0 / dev * mul,
                        color.1 / dev * mul,
                        color.2 / dev * mul,
                    );
                }
            }

            v.push(Frame(frame_vec));
        }
        Sequence::new(v, frequenz)
    }

    pub fn custom(num_of_leds: usize) -> Sequence {
        let freq = 30.0;
        let length = 10;
        let green = SequenzGenerator::create_dot(num_of_leds, (0, 255, 0), freq, length, length);
        let mut red = SequenzGenerator::create_dot(num_of_leds, (255, 0, 0), freq, length, length);
        red = red << 50;
        let mut blue = SequenzGenerator::create_dot(num_of_leds, (0, 0, 255), freq, length, length);
        blue = blue << 100;
        let mut yellow =
            SequenzGenerator::create_dot(num_of_leds, (255, 255, 0), freq, length, length);
        yellow = yellow.reverse();
        let mut magenta =
            SequenzGenerator::create_dot(num_of_leds, (255, 0, 255), freq, length, length);
        magenta = magenta << 50;
        magenta = magenta.reverse();
        let mut cyan =
            SequenzGenerator::create_dot(num_of_leds, (0, 255, 255), freq, length, length);
        cyan = cyan << 100;
        cyan = cyan.reverse();
        green + red + blue + yellow + magenta + cyan
    }

    pub fn red_alert(num_of_leds: usize) -> Sequence {
        let length = 13;
        let seq =
            SequenzGenerator::create_dot(num_of_leds, (255, 0, 0), 20.0, length, length) >> 15;
        let f1 = &seq.get_frames()[0];
        let mut n = f1.add(&f1.shr(30));
        n = n.add(&f1.shr(60));
        n = n.add(&f1.shr(90));
        n = n.add(&f1.shr(120));
        // Sequence::new(vec![n], 20.0)
        //let s = SequenzGenerator::create_scrolling_frame(num_of_leds, &n, 15.0);
        let s1 = Sequence::new(vec![n], 1.0);
        let s = s1.repeat(150);
        s.pulse(30, 0.2)
    }
    pub fn create_scrolling_frame(num_of_leds: usize, frame: &Frame, framerate: f32) -> Sequence {
        let mut v = Vec::new();
        for i in 0..num_of_leds {
            let mut t = frame.clone();
            t = t.shr(i);
            v.push(t);
        }

        Sequence::new(v, framerate)
    }
}
