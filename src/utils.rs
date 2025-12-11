use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Dword = usize;

pub fn find_library(library_name: &str) -> Option<Dword> {
    let maps_file = File::open("/proc/self/maps").ok()?;
    let reader = BufReader::new(maps_file);

    for line in reader.lines() {
        if let Ok(line_content) = line {
            if line_content.contains(library_name) {
                let address_str = line_content.split('-').next()?;
                return usize::from_str_radix(address_str, 16).ok();
            }
        }
    }

    None
}

pub fn get_absolute_address(library_name: &str, relative_addr: Dword) -> Option<Dword> {
    let lib_base = find_library(library_name)?;
    Some(lib_base + relative_addr)
}

pub fn is_library_loaded(library_name: &str) -> bool {
    if let Ok(maps_file) = File::open("/proc/self/maps") {
        let reader = BufReader::new(maps_file);
        for line in reader.lines().flatten() {
            if line.contains(library_name) {
                return true;
            }
        }
    }
    false
}

pub fn string_to_offset(hex_string: &str) -> Option<usize> {
    let cleaned = hex_string.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(cleaned, 16).ok()
}

pub fn wait_for_library(library_name: &str, timeout_ms: u64) -> bool {
    use std::thread;
    use std::time::Duration;

    let sleep_duration = Duration::from_millis(100);
    let max_iterations = timeout_ms / 100;

    for _ in 0..max_iterations {
        if is_library_loaded(library_name) {
            return true;
        }
        thread::sleep(sleep_duration);
    }

    false
}
