use std::collections::HashSet;
use std::time::Duration;

use crate::alerts;
use crate::config::Config;
use crate::network;
use crate::processes;
use crate::resources;

pub fn scan_once(config: &Config) -> Vec<String> {
    let mut findings = Vec::new();
    let empty_pids = HashSet::new();

    // ── Processes ──
    let (proc_alerts, _) = processes::check_new(config, &empty_pids);
    for a in &proc_alerts {
        findings.push(format!("  \x1b[1;31m{}\x1b[0m [{}] {} — {}",
            a.severity.as_str().to_uppercase(), a.timestamp, a.title, a.message));
        alerts::record(a);
    }

    // ── Network: listening ports ──
    if let Some(ports) = network::get_listening_ports() {
        for p in &ports {
            if *p > 0 && config.high_risk_ports.contains(p) {
                let msg = format!("High-risk port {} is listening", p);
                findings.push(format!("  \x1b[1;34mINFO\x1b[0m {} — {}", "", msg));
                alerts::record(&alerts::Alert::info("Listening Port", &msg));
            }
        }
    }

    // ── Network: established connections ──
    for a in &network::check_connections(config) {
        findings.push(format!("  \x1b[1;34mINFO\x1b[0m [{}] {} — {}",
            a.timestamp, a.title, a.message));
        alerts::record(a);
    }

    // ── CPU (two samples with short delay) ──
    if let Some(c1) = resources::read_cpu() {
        std::thread::sleep(Duration::from_millis(500));
        if let Some(c2) = resources::read_cpu() {
            let usage = resources::cpu_usage(&c1, &c2);
            if usage > config.cpu_threshold_percent {
                let msg = format!("CPU at {:.1}% (threshold: {}%)", usage, config.cpu_threshold_percent);
                findings.push(format!("  \x1b[1;33mWARNING\x1b[0m {} — {}", "", msg));
                alerts::record(&alerts::Alert::warning("High CPU", &msg));
            } else {
                findings.push(format!("  \x1b[1;32mOK\x1b[0m     CPU at {:.1}%", usage));
            }
        }
    }

    // ── Memory ──
    if let Some(mem) = resources::get_memory() {
        let usage = 100.0 * (1.0 - mem.available_kb as f32 / mem.total_kb as f32);
        if usage > config.memory_threshold_percent {
            let msg = format!("Memory at {:.1}% (threshold: {}%)", usage, config.memory_threshold_percent);
            findings.push(format!("  \x1b[1;33mWARNING\x1b[0m {} — {}", "", msg));
            alerts::record(&alerts::Alert::warning("High Memory", &msg));
        } else {
            findings.push(format!("  \x1b[1;32mOK\x1b[0m     Memory at {:.1}%", usage));
        }
    }

    // ── Disk ──
    if let Some(disks) = resources::get_disk() {
        for d in disks {
            if d.used_percent > (100.0 - config.disk_threshold_percent) {
                let msg = format!("{}: {:.0}% used", d.mount, d.used_percent);
                findings.push(format!("  \x1b[1;33mWARNING\x1b[0m {} — {}", "", msg));
                alerts::record(&alerts::Alert::warning("Low Disk Space", &msg));
            } else {
                findings.push(format!("  \x1b[1;32mOK\x1b[0m     Disk {} at {:.0}%", d.mount, d.used_percent));
            }
        }
    }

    findings
}
