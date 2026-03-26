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

const SIZE: usize = 3;

#[derive(Debug, Default, Serialize, Clone)]
pub struct Target {
    points: [(i16, i16); SIZE],
    vecs: [(i16, i16); SIZE],
    speed: i16,
    resolution: u16,
    done: bool,
}
impl Target {
    pub fn update(&mut self, x: i16, y: i16, speed: i16, resolution: u16) {
        self.speed = speed;
        self.resolution = resolution;

        let (x1, y1) = self.points[0];

        self.vecs.rotate_right(1);
        self.vecs[0] = (x - x1, y - y1);

        self.points.rotate_right(1);
        self.points[0] = (x, y)
    }
    pub fn get_point(&self) -> (i16, i16) {
        self.points[0]
    }
    pub fn get_vec(&self) -> (i16, i16) {
        self.vecs[0]
    }
    pub fn is_alive(&self) -> bool {
        self.points[0] != (0, 0) && self.speed != 0
    }
    pub fn get_speed(&self) -> i16 {
        self.speed
    }
    pub fn is_door_open(&mut self) -> bool {
        let dis = Target::calculate_vector_length(self.points[0]);
        let angle = Target::calculate_angle(self.points[0], self.vecs[0]);
        let angle2 = Target::calculate_angle(self.points[1], self.vecs[1]);
        // let m = 1.0 / 240.0;
        // let n = 167.0 + 11.0 / 12.0;
        // linear function welche 175 auf 1700 und 170 auf 500 mapped
        self.done = (angle >= 177.0 && angle2 >= 177.0 && dis < 1700.0) || dis < 500.0;
        self.done
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
}
// #[derive(Serialize)]
// pub struct Row {
//     timestamp: u128,
//     id: u8,
//     distance: f32,
//     speed: i16,
//     angle: f32,
// }
// impl Row {
//     pub fn new(target: &Target, id: u8) -> Self {
//         Self {
//             timestamp: SystemTime::now()
//                 .duration_since(UNIX_EPOCH)
//                 .unwrap()
//                 .as_millis(),
//             id: id,
//             distance: calculate_vector_length(target.prev_point.0, target.prev_point.1),
//             speed: target.get_speed(),
//             angle: calculate_angle(
//                 target.prev_point.0,
//                 target.prev_point.1,
//                 target.prev_vec.0,
//                 target.prev_vec.1,
//             ),
//         }
//     }
// }
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
                }
                callback(targets_array.clone());
            }
        });
    }
}
