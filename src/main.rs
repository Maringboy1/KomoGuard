mod config;
mod alerts;
mod processes;
mod network;
mod resources;
mod watcher;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const PID_FILE: &str = "/tmp/komoguard.pid";
const ALERT_FILE: &str = "/dev/shm/komoguard.alerts";
const MAX_ALERT_LINES: usize = 5000;

fn cmd_scan() {
    println!("\n  ╔══════════════════════════════════════════╗");
    println!("  ║     KomoGuard Security Scan              ║");
    println!("  ╚══════════════════════════════════════════╝\n");

    let cfg = config::load();
    let started = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("  Started: {}\n", started);

    alert(" Processes", &format!("Scanning {} process names...", process_count()));
    alert(" Network", "Checking listening ports and connections...");
    alert(" Resources", "Checking CPU, memory, and disk...\n");

    let findings = watcher::scan_once(&cfg);

    if findings.is_empty() {
        println!("  \x1b[1;32m✓ No issues detected. System looks clean.\x1b[0m\n");
    } else {
        println!("  \x1b[1;33m── Findings ──────────────────────────────────\x1b[0m\n");
        for f in &findings {
            println!("  {}", f);
        }
        println!();
    }

    let elapsed = chrono::Local::now().format("%H:%M:%S").to_string();
    println!("  Finished: {}", elapsed);
    println!("  Total findings: {}\n", findings.len());
}

fn process_count() -> usize {
    std::fs::read_dir("/proc")
        .map(|d| d.flatten().filter(|e| e.file_name().to_string_lossy().parse::<u32>().is_ok()).count())
        .unwrap_or(0)
}

fn alert(category: &str, msg: &str) {
    println!("  \x1b[1;36m{}\x1b[0m {}", category, msg);
}

fn cmd_alerts() {
    let alerts_list = alerts::recent(30);
    if alerts_list.is_empty() {
        println!("\n  \x1b[1;33m[i]\x1b[0m No alerts recorded. Run 'komoguard scan' first.\n");
        return;
    }

    println!("\n  Recent KomoGuard Alerts:\n");
    for alert_line in &alerts_list {
        let parts: Vec<&str> = alert_line.splitn(4, '|').collect();
        if parts.len() >= 4 {
            let ts = parts[0];
            let sev = parts[1];
            let title = parts[2];
            let msg = parts[3];

            let color = match sev {
                "critical" => "\x1b[1;31m",
                "warning" => "\x1b[1;33m",
                _ => "\x1b[1;34m",
            };
            let reset = "\x1b[0m";

            println!("  {}{}{} [{}]", color, sev.to_uppercase(), reset, ts);
            println!("  {} {} {}", color, title, reset);
            println!("  {}\n", msg);
        }
    }
}

fn cmd_status() {
    let running = is_daemon_running();
    let total = alerts::total_alerts();
    println!();
    println!("  KomoGuard v{} — Security Scanner", env!("CARGO_PKG_VERSION"));
    if running {
        println!("  Status:  \x1b[1;32mRunning (PID {})\x1b[0m", read_pid().unwrap_or(0));
    } else {
        println!("  Status:  \x1b[1;33mStopped\x1b[0m");
    }
    println!("  Alerts:  {}", total);
    println!("  Config:  {}", config::path().display());
    println!();
    println!("  Commands:");
    println!("    komoguard scan     Run a security scan now");
    println!("    komoguard start    Start daemon (background monitoring)");
    println!("    komoguard stop     Stop daemon");
    println!("    komoguard alerts   View alerts from last scan");
    println!("    komoguard status   Show this info");
    println!("    komoguard export   Export alerts to JSON file");
    println!("    komoguard clear    Clear alert log");
    println!();
}

fn cmd_clear() {
    let _ = std::fs::remove_file(ALERT_FILE);
    println!("[i] Alert log cleared.\n");
}

fn cmd_start() {
    if is_daemon_running() {
        let pid = read_pid().unwrap_or(0);
        println!("[i] KomoGuard daemon is already running (PID {}).\n", pid);
        return;
    }

    let binary = std::env::current_exe().unwrap_or_else(|_| Path::new("komoguard").to_path_buf());

    let child = Command::new(&binary)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(proc) => {
            let pid = proc.id();
            if let Ok(mut f) = std::fs::File::create(PID_FILE) {
                let _ = writeln!(f, "{}", pid);
            }
            println!("  \x1b[1;32m✓ KomoGuard daemon started (PID {})\x1b[0m\n", pid);
        }
        Err(e) => {
            eprintln!("  \x1b[1;31m✗ Failed to start daemon: {}\x1b[0m\n", e);
            std::process::exit(1);
        }
    }
}

fn cmd_stop() {
    let pid = match read_pid() {
        Some(p) => p,
        None => {
            println!("[i] KomoGuard daemon is not running.\n");
            return;
        }
    };

    if !is_pid_alive(pid) {
        let _ = std::fs::remove_file(PID_FILE);
        println!("[i] KomoGuard daemon was not running (stale PID cleaned up).\n");
        return;
    }

    println!("[i] Stopping KomoGuard daemon (PID {})...", pid);

    unsafe { libc::kill(pid as i32, libc::SIGTERM); }

    for _ in 0..20 {
        if !is_pid_alive(pid) {
            let _ = std::fs::remove_file(PID_FILE);
            println!("  \x1b[1;32m✓ KomoGuard daemon stopped\x1b[0m\n");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    unsafe { libc::kill(pid as i32, libc::SIGKILL); }
    std::thread::sleep(Duration::from_millis(200));
    let _ = std::fs::remove_file(PID_FILE);
    println!("  \x1b[1;33m⚠ KomoGuard daemon force-killed\x1b[0m\n");
}

fn cmd_daemon() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting signal handler");

    println!("[i] KomoGuard daemon started. Monitoring every {}s.\n", {
        let cfg = config::load();
        cfg.check_interval_seconds
    });

    let cfg = config::load();
    let interval = Duration::from_secs(cfg.check_interval_seconds);

    while running.load(Ordering::SeqCst) {
        let findings = watcher::scan_once(&cfg);

        if !findings.is_empty() {
            for f in &findings {
                if f.contains("CRITICAL") && cfg.notify_desktop {
                    let _ = Command::new("notify-send")
                        .arg("-u")
                        .arg("critical")
                        .arg("KomoGuard Alert")
                        .arg(f.trim())
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn();
                }
            }
        }

        trim_alert_log();

        std::thread::sleep(interval);
    }

    println!("\n[i] KomoGuard daemon exiting gracefully.");
}

fn trim_alert_log() {
    let content = match std::fs::read_to_string(ALERT_FILE) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > MAX_ALERT_LINES {
        let trimmed = lines[lines.len() - MAX_ALERT_LINES..].join("\n");
        let _ = std::fs::write(ALERT_FILE, trimmed + "\n");
    }
}

fn cmd_export() {
    let alerts_list = alerts::recent(usize::MAX);
    if alerts_list.is_empty() {
        println!("\n  \x1b[1;33m[i]\x1b[0m No alerts to export. Run 'komoguard scan' first.\n");
        return;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}/komoguard_report_{}.json", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()), timestamp);

    let mut entries = Vec::new();
    for line in &alerts_list {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() >= 4 {
            entries.push(serde_json::json!({
                "timestamp": parts[0],
                "severity": parts[1],
                "title": parts[2],
                "message": parts[3],
            }));
        }
    }

    let report = serde_json::json!({
        "tool": "KomoGuard",
        "version": env!("CARGO_PKG_VERSION"),
        "exported_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "total_alerts": entries.len(),
        "alerts": entries,
    });

    match std::fs::write(&filename, serde_json::to_string_pretty(&report).unwrap()) {
        Ok(_) => println!("  \x1b[1;32m✓ Exported {} alerts to {}\x1b[0m\n", entries.len(), filename),
        Err(e) => eprintln!("  \x1b[1;31m✗ Failed to export: {}\x1b[0m\n", e),
    }
}

fn read_pid() -> Option<u32> {
    let content = std::fs::read_to_string(PID_FILE).ok()?;
    content.trim().parse().ok()
}

fn is_pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

fn is_daemon_running() -> bool {
    match read_pid() {
        Some(pid) => is_pid_alive(pid),
        None => false,
    }
}

fn print_usage() {
    println!();
    println!("KomoGuard v{} — Security Scanner", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: komoguard [COMMAND]");
    println!();
    println!("Commands:");
    println!("  scan      Run a one-time security scan (default)");
    println!("  start     Start daemon (background monitoring)");
    println!("  stop      Stop daemon");
    println!("  alerts    Show alerts from last scan");
    println!("  status    Show scanner info");
    println!("  export    Export alerts to JSON file");
    println!("  clear     Clear alert log");
    println!("  help      Show this help");
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("scan");

    match cmd {
        "scan" => cmd_scan(),
        "start" => cmd_start(),
        "stop" => cmd_stop(),
        "daemon" => cmd_daemon(),
        "alerts" | "alert" => cmd_alerts(),
        "status" | "state" => cmd_status(),
        "export" => cmd_export(),
        "clear" => cmd_clear(),
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-V" => {
            println!("KomoGuard v{}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("[-] Unknown command: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }
}