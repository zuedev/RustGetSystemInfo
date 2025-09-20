//! # RustGetSystemInfo
//!
//! A system information utility that collects and displays system metrics
//! including OS details, CPU cores, memory usage, and swap information.
//!
//! The program displays information in a human-readable format to the console
//! and exports the raw data as JSON to a file for programmatic use.

use sysinfo::{System, SystemExt, NetworkExt, DiskExt};
use serde::{Serialize};
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::fmt;
use std::collections::HashMap;

/// Custom error types for application-specific error handling.
///
/// Provides descriptive error messages for common failure scenarios
/// when collecting system information and writing output files.
#[derive(Debug)]
enum AppError {
    /// Failed to create the output JSON file
    FileCreation(std::io::Error),
    /// Failed to write data to the output file
    FileWrite(std::io::Error),
    /// Failed to serialize system information to JSON format
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

/// Converts raw byte values to human-readable format with appropriate units.
///
/// Uses binary prefixes (1024-based) to convert bytes into the most appropriate
/// unit (B, KB, MB, GB, TB) with 2 decimal places for units larger than bytes.
///
/// # Arguments
///
/// * `bytes` - The number of bytes to format
///
/// # Returns
///
/// A formatted string with the value and appropriate unit
///
/// # Examples
///
/// ```
/// assert_eq!(format_bytes(0), "0 B");
/// assert_eq!(format_bytes(1024), "1.00 KB");
/// assert_eq!(format_bytes(1536), "1.50 KB");
/// assert_eq!(format_bytes(1073741824), "1.00 GB");
/// ```
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

/// Disk usage information for a single disk/partition.
#[derive(Serialize)]
struct DiskInfo {
    /// Disk name or mount point
    name: String,
    /// File system type (e.g., "NTFS", "ext4", "APFS")
    file_system: String,
    /// Total disk space in bytes
    total_space: u64,
    /// Available disk space in bytes
    available_space: u64,
}

/// Network interface information.
#[derive(Serialize)]
struct NetworkInfo {
    /// Interface name (e.g., "eth0", "wlan0", "Ethernet")
    name: String,
    /// Total bytes received since boot
    bytes_received: u64,
    /// Total bytes transmitted since boot
    bytes_transmitted: u64,
    /// Total packets received since boot
    packets_received: u64,
    /// Total packets transmitted since boot
    packets_transmitted: u64,
}

/// System information data structure for serialization and display.
///
/// Contains comprehensive system metrics including operating system details,
/// CPU information, memory/swap usage statistics, disk usage, and network interfaces.
/// All memory and disk values are stored as raw bytes for accuracy and consistency.
#[derive(Serialize)]
struct SystemInfo {
    /// Operating system name (e.g., "Windows", "Linux", "macOS")
    os_name: String,
    /// Operating system version string
    os_version: String,
    /// Number of physical CPU cores
    cpu_cores: usize,
    /// Total system memory in bytes
    total_memory: u64,
    /// Currently used memory in bytes
    used_memory: u64,
    /// Total swap space in bytes
    total_swap: u64,
    /// Currently used swap space in bytes
    used_swap: u64,
    /// Disk usage information for all detected disks
    disks: Vec<DiskInfo>,
    /// Network interface statistics
    networks: Vec<NetworkInfo>,
}

/// Core application logic for collecting and outputting system information.
///
/// Gathers system metrics using the sysinfo crate, displays them in a
/// human-readable format to the console, and exports the raw data as JSON.
///
/// # Returns
///
/// * `Ok(())` - If system information was successfully collected and saved
/// * `Err(AppError)` - If file creation, writing, or JSON serialization fails
///
/// # Errors
///
/// This function will return an error if:
/// * The output JSON file cannot be created
/// * Writing to the JSON file fails
/// * System information cannot be serialized to JSON
fn run() -> Result<(), AppError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Collect disk information
    let disks: Vec<DiskInfo> = sys.disks().iter().map(|disk| {
        DiskInfo {
            name: disk.mount_point().to_string_lossy().to_string(),
            file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        }
    }).collect();

    // Collect network information
    let networks: Vec<NetworkInfo> = sys.networks().iter().map(|(name, network)| {
        NetworkInfo {
            name: name.clone(),
            bytes_received: network.received(),
            bytes_transmitted: network.transmitted(),
            packets_received: network.packets_received(),
            packets_transmitted: network.packets_transmitted(),
        }
    }).collect();

    let info = SystemInfo {
        os_name: sys.name().unwrap_or_else(|| "N/A".to_string()),
        os_version: sys.os_version().unwrap_or_else(|| "N/A".to_string()),
        cpu_cores: sys.physical_core_count().unwrap_or(0),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
        disks,
        networks,
    };

    println!("System Information:");
    println!("  OS Name: {}", info.os_name);
    println!("  OS Version: {}", info.os_version);
    println!("  CPU Cores: {}", info.cpu_cores);
    println!("  Total Memory: {}", format_bytes(info.total_memory));
    println!("  Used Memory: {}", format_bytes(info.used_memory));
    println!("  Total Swap: {}", format_bytes(info.total_swap));
    println!("  Used Swap: {}", format_bytes(info.used_swap));

    println!("\nDisk Usage:");
    if info.disks.is_empty() {
        println!("  No disks detected");
    } else {
        for disk in &info.disks {
            let used_space = disk.total_space - disk.available_space;
            let usage_percent = if disk.total_space > 0 {
                (used_space as f64 / disk.total_space as f64) * 100.0
            } else {
                0.0
            };
            println!("  {}: {} / {} ({:.1}% used, {} available) [{}]",
                disk.name,
                format_bytes(used_space),
                format_bytes(disk.total_space),
                usage_percent,
                format_bytes(disk.available_space),
                disk.file_system
            );
        }
    }

    println!("\nNetwork Interfaces:");
    if info.networks.is_empty() {
        println!("  No network interfaces detected");
    } else {
        for network in &info.networks {
            println!("  {}:", network.name);
            println!("    Received: {} ({} packets)",
                format_bytes(network.bytes_received),
                network.packets_received
            );
            println!("    Transmitted: {} ({} packets)",
                format_bytes(network.bytes_transmitted),
                network.packets_transmitted
            );
        }
    }

    let json = serde_json::to_string_pretty(&info)
        .map_err(AppError::JsonSerialization)?;

    let mut file = File::create("system_info.json")
        .map_err(AppError::FileCreation)?;

    file.write_all(json.as_bytes())
        .map_err(AppError::FileWrite)?;

    println!("System information saved to system_info.json");
    Ok(())
}

/// Application entry point.
///
/// Executes the main program logic and handles any errors that occur during
/// system information collection or file operations. If an error occurs,
/// it prints the error message to stderr and exits with code 1.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}