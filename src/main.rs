use sysinfo::{System, SystemExt};
use serde::{Serialize};
use std::fs::File;
use std::io::Write;

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

fn main() {
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
    println!("  Total Memory: {} bytes", info.total_memory);
    println!("  Used Memory: {} bytes", info.used_memory);
    println!("  Total Swap: {} bytes", info.total_swap);
    println!("  Used Swap: {} bytes", info.used_swap);

    let json = serde_json::to_string_pretty(&info).unwrap();
    let mut file = File::create("system_info.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();

    println!("
System information saved to system_info.json");
}