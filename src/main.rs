use sysinfo::{System, SystemExt};
use serde::{Serialize};
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum AppError {
    FileCreation(std::io::Error),
    FileWrite(std::io::Error),
    JsonSerialization(serde_json::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FileCreation(e) => write!(f, "Failed to create file: {}", e),
            AppError::FileWrite(e) => write!(f, "Failed to write to file: {}", e),
            AppError::JsonSerialization(e) => write!(f, "Failed to serialize data to JSON: {}", e),
        }
    }
}

impl Error for AppError {}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log(THRESHOLD).floor() as usize).min(UNITS.len() - 1);
    let value = bytes_f / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", value, UNITS[unit_index])
    }
}

#[derive(Serialize)]
struct SystemInfo {
    os_name: String,
    os_version: String,
    cpu_cores: usize,
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

fn run() -> Result<(), AppError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let info = SystemInfo {
        os_name: sys.name().unwrap_or_else(|| "N/A".to_string()),
        os_version: sys.os_version().unwrap_or_else(|| "N/A".to_string()),
        cpu_cores: sys.physical_core_count().unwrap_or(0),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    };

    println!("System Information:");
    println!("  OS Name: {}", info.os_name);
    println!("  OS Version: {}", info.os_version);
    println!("  CPU Cores: {}", info.cpu_cores);
    println!("  Total Memory: {}", format_bytes(info.total_memory));
    println!("  Used Memory: {}", format_bytes(info.used_memory));
    println!("  Total Swap: {}", format_bytes(info.total_swap));
    println!("  Used Swap: {}", format_bytes(info.used_swap));

    let json = serde_json::to_string_pretty(&info)
        .map_err(AppError::JsonSerialization)?;

    let mut file = File::create("system_info.json")
        .map_err(AppError::FileCreation)?;

    file.write_all(json.as_bytes())
        .map_err(AppError::FileWrite)?;

    println!("System information saved to system_info.json");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}