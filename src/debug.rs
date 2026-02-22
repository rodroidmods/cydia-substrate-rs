use crate::log::*;
use core::sync::atomic::{AtomicBool, Ordering};

pub static MS_DEBUG: AtomicBool = AtomicBool::new(false);

pub fn is_debug() -> bool {
    MS_DEBUG.load(Ordering::Relaxed)
}

pub fn set_debug(enabled: bool) {
    MS_DEBUG.store(enabled, Ordering::Relaxed);
}

fn hex_char(value: u8) -> char {
    if (0x20..0x80).contains(&value) {
        value as char
    } else {
        '.'
    }
}

const HEX_WIDTH: usize = 16;

pub fn log_hex_ex(data: &[u8], stride: usize, mark: Option<&str>) {
    let size = data.len();
    let mut i = 0;

    while i < size {
        let mut line = String::with_capacity(256);

        if i % HEX_WIDTH == 0 {
            if let Some(m) = mark {
                line.push_str(&format!("\n[{}] ", m));
            }
            line.push_str(&format!("0x{:03x}:", i));
        }

        line.push(' ');

        for q in 0..stride {
            if i + stride - q - 1 < size {
                line.push_str(&format!("{:02x}", data[i + stride - q - 1]));
            }
        }

        i += stride;

        for _ in 1..stride {
            line.push(' ');
        }

        if i % 4 == 0 {
            line.push(' ');
        }

        if i % HEX_WIDTH == 0 {
            line.push(' ');
            let start = i - HEX_WIDTH;
            for j in start..i {
                if j < size {
                    line.push(hex_char(data[j]));
                }
            }
            info!("{}", line);
        }
    }

    if i % HEX_WIDTH != 0 {
        let mut line = String::with_capacity(256);
        let remainder = i % HEX_WIDTH;
        for _ in remainder..HEX_WIDTH {
            line.push_str("   ");
        }
        for _ in 0..(HEX_WIDTH - remainder + 3) / 4 {
            line.push(' ');
        }
        line.push(' ');
        let row_start = i / HEX_WIDTH * HEX_WIDTH;
        for j in row_start..i {
            if j < size {
                line.push(hex_char(data[j]));
            }
        }
        info!("{}", line);
    }
}

pub fn log_hex(data: &[u8], mark: Option<&str>) {
    log_hex_ex(data, 1, mark);
}
