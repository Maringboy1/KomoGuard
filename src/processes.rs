use std::collections::HashSet;
use std::fs;

use crate::alerts::Alert;
use crate::config::Config;

struct ProcInfo {
    pid: u32,
    cmdline: String,
    exe: String,
}

fn read_proc_info(pid: u32) -> Option<ProcInfo> {
    let dir = format!("/proc/{}", pid);
    if !std::path::Path::new(&dir).exists() {
        return None;
    }

    let cmdline_raw = fs::read_to_string(format!("{}/cmdline", dir)).ok()?;
    let cmdline = cmdline_raw.replace('\0', " ").trim().to_string();
    if cmdline.is_empty() {
        return None;
    }

    let exe = match fs::read_link(format!("{}/exe", dir)) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => String::new(),
    };

    Some(ProcInfo { pid, cmdline, exe })
}

fn exe_basename(exe: &str) -> String {
    std::path::Path::new(exe)
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn cmdline_basename(cmdline: &str) -> String {
    cmdline
        .split_whitespace()
        .next()
        .unwrap_or("")
        .split('/')
        .last()
        .unwrap_or("")
        .to_lowercase()
}

fn matches_name(target: &str, name: &str) -> bool {
    let target_lower = target.to_lowercase();
    let name_lower = name.to_lowercase();

    if target_lower == name_lower {
        return true;
    }

    if name_lower.len() <= 3 {
        return target_lower == name_lower
            || target_lower.starts_with(&format!("{} ", name_lower))
            || target_lower.contains(&format!("/{}", name_lower));
    }

    target_lower.contains(&name_lower)
}

fn check_suspicious_names(proc: &ProcInfo, names: &[String]) -> Option<String> {
    let exe_name = if !proc.exe.is_empty() {
        exe_basename(&proc.exe)
    } else {
        cmdline_basename(&proc.cmdline)
    };

    for name in names {
        if matches_name(&exe_name, name) {
            return Some(format!("PID {}: {} (matched: {})", proc.pid, proc.cmdline, name));
        }
    }
    None
}

fn check_suspicious_dirs(proc: &ProcInfo, dirs: &[String]) -> Option<String> {
    for dir in dirs {
        if proc.exe.starts_with(dir) {
            return Some(format!(
                "PID {} running from {}: {}",
                proc.pid, dir, proc.cmdline
            ));
        }
    }
    None
}

fn check_reverse_shell(proc: &ProcInfo) -> Option<String> {
    let cmd_lower = proc.cmdline.to_lowercase();
    let patterns = ["socket", "connect", "bash -i", "sh -i", "python -c", "perl -e"];
    for pat in &patterns {
        if cmd_lower.contains(pat) && (cmd_lower.contains("/tcp/") || cmd_lower.contains("socket")) {
            return Some(format!(
                "PID {} possible reverse shell: {}",
                proc.pid, proc.cmdline
            ));
        }
    }
    None
}

pub fn check_new(
    config: &Config,
    known_pids: &HashSet<u32>,
) -> (Vec<Alert>, HashSet<u32>) {
    let mut alerts = Vec::new();
    let mut current_pids = HashSet::new();

    let dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return (alerts, current_pids),
    };

    for entry in dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let pid: u32 = match name_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        current_pids.insert(pid);

        if known_pids.contains(&pid) {
            continue;
        }

        let proc = match read_proc_info(pid) {
            Some(p) => p,
            None => continue,
        };

        if let Some(msg) = check_suspicious_names(&proc, &config.suspicious_names) {
            alerts.push(Alert::critical("Suspicious Process", &msg));
        }

        if let Some(msg) = check_suspicious_dirs(&proc, &config.suspicious_dirs) {
            alerts.push(Alert::warning("Process in Temp Directory", &msg));
        }

        if let Some(msg) = check_reverse_shell(&proc) {
            alerts.push(Alert::critical("Possible Reverse Shell", &msg));
        }
    }

    (alerts, current_pids)
}
