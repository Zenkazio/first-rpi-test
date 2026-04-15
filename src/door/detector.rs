#![allow(unused)]
use std::{
    f32::consts::PI,
    fs::{File, OpenOptions},
    io::Read,
    os::unix::net::UnixStream,
    thread::spawn,
    time::{SystemTime, UNIX_EPOCH},
};

use csv::Writer;
use serde::Serialize;

const SIZE: usize = 5;

#[derive(Debug, Default, Serialize, Clone)]
pub struct Target {
    points: [(i16, i16); SIZE],
    vecs: [(i16, i16); SIZE],
    angles: [f32; SIZE],
    distances: [f32; SIZE],
    speeds: [i16; SIZE],
    calc_speeds: [f32; SIZE],
    two_second_points: [(f32, f32); SIZE],
    timestamps: [u128; 1],
    resolution: u16,
    is_alive: bool,
    is_open_door: bool,
    is_close_door: bool,
    opening_angle: f32,
}
impl Target {
    pub fn update(&mut self, x: i16, y: i16, speed: i16, resolution: u16) {
        self.speeds.rotate_right(1);
        self.speeds[0] = speed;

        self.resolution = resolution;

        let (x1, y1) = self.points[0];

        self.vecs.rotate_right(1);
        self.vecs[0] = (x - x1, y - y1);

        self.points.rotate_right(1);
        self.points[0] = (x, y);

        self.angles.rotate_right(1);
        self.angles[0] = Target::calculate_angle(self.points[0], self.vecs[0]);

        self.distances.rotate_right(1);
        self.distances[0] = Target::calculate_vector_length(self.points[0]);

        let t1 = self.timestamps[0];
        self.timestamps.rotate_right(1);
        self.timestamps[0] = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let t_diff = self.timestamps[0] - t1;
        self.calc_speeds.rotate_right(1);
        self.calc_speeds[0] =
            Target::calculate_vector_length(self.vecs[0]) / (t_diff as f32 / 1000.0);

        let fac = 2.5;
        self.opening_angle = 10.0;
        self.two_second_points.rotate_right(1);
        self.two_second_points[0] = (
            self.vecs[0].0 as f32 / (t_diff as f32 / 1000.0) * fac,
            self.vecs[0].1 as f32 / (t_diff as f32 / 1000.0) * fac,
        );

        self.is_alive =
            (x, y) != (0, 0) && (self.speeds[0].abs() > 12 || self.distances[0] < 650.0);

        // self.is_open_door = self
        //     .angles
        //     .iter()
        //     .take(if self.distances[0] < 1000.0 { 2 } else { 3 })
        //     .all(|&x| x >= 175.0_f32)
        //     && self.distances[0] < 2000.0
        //     && self.speeds[0].abs() > 30
        //     || self.distances[0] < 500.0;
        self.is_open_door = self
            .calc_speeds
            .iter()
            .take(3)
            .all(|&x| x * fac > self.distances[0])
            && self
                .angles
                .iter()
                .take(3)
                .all(|&x| x >= 180.0 - self.opening_angle)
            || self.distances[0] < 500.0;
        self.is_close_door = self
            .calc_speeds
            .iter()
            .take(1)
            .all(|&x| 1000.0 < self.distances[0] && self.distances[0] < 1500.0)
            && self.angles.iter().take(1).all(|&x| x <= self.opening_angle)
    }
    pub fn get_points(&self) -> [(i16, i16); SIZE] {
        self.points
    }
    pub fn get_vecs(&self) -> [(i16, i16); SIZE] {
        self.vecs
    }
    pub fn is_alive(&self) -> bool {
        self.is_alive
    }
    pub fn get_speeds(&self) -> [i16; SIZE] {
        self.speeds
    }
    pub fn is_door_open(&self) -> bool {
        self.is_open_door
    }
    pub fn calculate_vector_length(vec: (i16, i16)) -> f32 {
        let (x, y) = vec;
        ((x as f32).powi(2) + (y as f32).powi(2)).sqrt()
    }
    pub fn calculate_angle(vec1: (i16, i16), vec2: (i16, i16)) -> f32 {
        let (x1, y1) = vec1;
        let (x2, y2) = vec2;
        let dot_product = x1 as f32 * x2 as f32 + y1 as f32 * y2 as f32;

        let mag1 = Target::calculate_vector_length(vec1);
        let mag2 = Target::calculate_vector_length(vec2);

        let cos_theta = dot_product / (mag1 * mag2);
        let cos_theta = cos_theta.clamp(-1.0, 1.0);

        let angle_radians = cos_theta.acos();
        let angle_degrees = angle_radians * (180.0 / PI);

        angle_degrees
    }

    pub fn is_close_door(&self) -> bool {
        self.is_close_door
    }
}

fn parse_ld2450_value(low: u8, high: u8) -> i16 {
    // 15-Bit Wert extrahieren, 16. Bit (0x80) ist das Vorzeichen
    let val = (((high & 0x7F) as i16) << 8) | (low as i16);
    if (high & 0x80) == 0 { -val } else { val }
}

pub struct Detector {}

impl Detector {
    pub fn start<F>(uart_num: u8, mut callback: F)
    where
        F: FnMut([Target; 3]) + Send + 'static,
    {
        let socket_path = format!("/tmp/ld2450_{}.sock", uart_num);

        spawn(move || {
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("data.csv")
                .unwrap();

            let mut wtr = csv::WriterBuilder::new()
                .has_headers(file.metadata().unwrap().len() == 0)
                .from_writer(file);

            let mut stream =
                UnixStream::connect(socket_path).expect("Socket-Verbindung fehlgeschlagen");
            let mut buffer = [0u8; 30];
            let mut targets_array: [Target; 3] =
                [Target::default(), Target::default(), Target::default()];
            loop {
                if stream.read_exact(&mut buffer).is_ok() {
                    if buffer[28] == 0x55 && buffer[29] == 0xCC {
                        for i in 0..3 {
                            let offset = 4 + (i * 8);
                            let x = parse_ld2450_value(buffer[offset], buffer[offset + 1]);
                            let y = parse_ld2450_value(buffer[offset + 2], buffer[offset + 3]);
                            let speed = parse_ld2450_value(buffer[offset + 4], buffer[offset + 5]);
                            let res =
                                ((buffer[offset + 7] as u16) << 8) | (buffer[offset + 6] as u16);
                            targets_array[i].update(x, y, speed, res);
                        }
                    }
                    // dbg!(&targets_array);
                    callback(targets_array.clone());
                }
            }
        });
    }
}
