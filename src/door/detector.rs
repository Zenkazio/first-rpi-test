use std::{f32::consts::PI, io::Read, os::unix::net::UnixStream, thread::spawn};

const AVERAGE_SIZE: usize = 2;

#[derive(Debug, Default)]
pub struct Target {
    x_values: [i16; AVERAGE_SIZE],
    prev_x: [f32; AVERAGE_SIZE],
    y_values: [i16; AVERAGE_SIZE],
    prev_y: [f32; AVERAGE_SIZE],
    speed: i16,
    resolution: u16,
    c: usize,
    prev_point: (f32, f32),
    prev_vec: (f32, f32),
}
impl Target {
    pub fn update(&mut self, x: i16, y: i16, speed: i16, resolution: u16) {
        self.speed = speed;
        self.resolution = resolution;
        self.x_values[self.c] = x;
        self.y_values[self.c] = y;

        let sum_x: i16 = self.x_values.iter().sum();
        let x = sum_x as f32 / AVERAGE_SIZE as f32;
        let sum_y: i16 = self.y_values.iter().sum();
        let y = sum_y as f32 / AVERAGE_SIZE as f32;

        let prev_x = self.prev_x[self.c];
        let prev_y = self.prev_y[self.c];

        self.prev_x[self.c] = x;
        self.prev_y[self.c] = y;

        self.c = (self.c + 1) % AVERAGE_SIZE;

        self.prev_point = (x, y);
        self.prev_vec = (x - prev_x, y - prev_y)
    }
    pub fn get_point(&self) -> (f32, f32) {
        self.prev_point
    }
    pub fn get_vec(&self) -> (f32, f32) {
        self.prev_vec
    }
    pub fn is_alive(&self) -> bool {
        self.prev_point.0 != 0.0 || self.prev_point.1 != 0.0
    }
    pub fn get_speed(&self) -> i16 {
        self.speed
    }
    pub fn is_door_open(&self) -> bool {
        calculate_angle(
            self.prev_point.0,
            self.prev_point.1,
            self.prev_vec.0,
            self.prev_vec.1,
        ) >= 175.0
            && calculate_vector_length(self.prev_point.0, self.prev_point.1) < 1500.0
            && self.speed < -30
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
        F: FnMut(&[Target]) + Send + 'static,
    {
        let socket_path = format!("/tmp/ld2450_{}.sock", uart_num);

        spawn(move || {
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
                            targets_array[i].update(-x, y, speed, res);
                        }
                    }
                }
                callback(&targets_array);
            }
        });
    }
}
pub fn calculate_angle(vec1_x: f32, vec1_y: f32, vec2_x: f32, vec2_y: f32) -> f32 {
    // Calculate the dot product
    let dot_product = vec1_x * vec2_x + vec1_y * vec2_y;

    // Calculate the magnitudes
    let magnitude1 = (vec1_x.powi(2) + vec1_y.powi(2)).sqrt();
    let magnitude2 = (vec2_x.powi(2) + vec2_y.powi(2)).sqrt();

    // Calculate the cosine of the angle (with clamping to avoid numerical errors)
    let cos_theta = dot_product / (magnitude1 * magnitude2);
    let cos_theta = cos_theta.clamp(-1.0, 1.0); // Clamp for acos domain

    // Convert the angle from radians to degrees
    let angle_radians = cos_theta.acos();
    let angle_degrees = angle_radians * (180.0 / PI);

    angle_degrees
}
pub fn calculate_vector_length(x: f32, y: f32) -> f32 {
    // Calculate the magnitude (length) of the vector
    (x.powi(2) + y.powi(2)).sqrt()
}
