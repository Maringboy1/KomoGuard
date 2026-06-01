use std::fs;

use crate::alerts::Alert;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct CpuMeasure {
    pub total: u64,
    pub idle: u64,
}

impl CpuMeasure {
    pub fn new() -> Self {
        CpuMeasure { total: 0, idle: 0 }
    }
}

pub fn read_cpu() -> Option<CpuMeasure> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let first = content.lines().next()?;

    let vals: Vec<u64> = first
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();

    if vals.len() < 5 {
        return None;
    }

    let total: u64 = vals.iter().sum();
    let idle = vals[3];

    Some(CpuMeasure { total, idle })
}

pub fn cpu_usage(prev: &CpuMeasure, curr: &CpuMeasure) -> f32 {
    let total_delta = curr.total.saturating_sub(prev.total);
    let idle_delta = curr.idle.saturating_sub(prev.idle);

    if total_delta == 0 {
        return 0.0;
    }

    100.0 * (1.0 - idle_delta as f32 / total_delta as f32)
}

pub struct MemoryInfo {
    pub total_kb: u64,
    pub available_kb: u64,
}

pub fn get_memory() -> Option<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1)?.parse().ok()?;
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1)?.parse().ok()?;
        }
    }

    if total == 0 {
        return None;
    }

    Some(MemoryInfo { total_kb: total, available_kb: available })
}

pub struct DiskInfo {
    pub mount: String,
    pub used_percent: f32,
}

pub fn get_disk() -> Option<Vec<DiskInfo>> {
    let output = std::process::Command::new("df")
        .arg("-hP")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts[5];
            let pcent_str = parts[4].trim_end_matches('%');
            if let Ok(pcent) = pcent_str.parse::<f32>() {
                let fstype = parts[1];
                let relevant = ["ext4", "ext3", "ext2", "btrfs", "xfs", "zfs", "tmpfs"]
                    .contains(&fstype)
                    || mount == "/";

                if relevant {
                    disks.push(DiskInfo {
                        mount: mount.to_string(),
                        used_percent: pcent,
                    });
                }
            }
        }
    }

    Some(disks)
}

pub fn check_resources(config: &Config, prev_cpu: &CpuMeasure, curr_cpu: &CpuMeasure) -> Vec<Alert> {
    let mut alerts = Vec::new();

    let cpu = cpu_usage(prev_cpu, curr_cpu);
    if cpu > config.cpu_threshold_percent {
        alerts.push(Alert::warning(
            "High CPU Usage",
            &format!("CPU at {:.1}% (threshold: {}%)", cpu, config.cpu_threshold_percent),
        ));
    }

    if let Some(mem) = get_memory() {
        let usage = 100.0 * (1.0 - mem.available_kb as f32 / mem.total_kb as f32);
        if usage > config.memory_threshold_percent {
            alerts.push(Alert::warning(
                "High Memory Usage",
                &format!("Memory at {:.1}% (threshold: {}%)", usage, config.memory_threshold_percent),
            ));
        }
    }

    if let Some(disks) = get_disk() {
        for disk in disks {
            if disk.used_percent > (100.0 - config.disk_threshold_percent) {
                alerts.push(Alert::warning(
                    "Low Disk Space",
                    &format!("{}: {:.0}% used", disk.mount, disk.used_percent),
                ));
            }
        }
    }

    alerts
}
