use std::{io::Read, os::unix::net::UnixStream};

use tokio::task::spawn_blocking;

#[derive(Debug, Default)]
pub struct Target {
    pub x: i16,
    pub y: i16,
    pub speed: i16,
    pub resolution: u16,
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
        F: FnMut() + Send + 'static,
    {
        let socket_path = format!("/tmp/ld2450_{}.sock", uart_num);

        spawn_blocking(move || {
            let mut stream =
                UnixStream::connect(socket_path).expect("Socket-Verbindung fehlgeschlagen");
            let mut buffer = [0u8; 30];

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
                            if speed <= -50 {
                                // Hier wird die übergebene Funktion aufgerufen
                                // callback();
                            }
                        }
                    }
                }
            }
        });
    }
}
