use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub check_interval_seconds: u64,
    pub cpu_threshold_percent: f32,
    pub memory_threshold_percent: f32,
    pub disk_threshold_percent: f32,
    pub notify_desktop: bool,
    pub suspicious_names: Vec<String>,
    pub suspicious_dirs: Vec<String>,
    pub high_risk_ports: Vec<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            check_interval_seconds: 30,
            cpu_threshold_percent: 90.0,
            memory_threshold_percent: 90.0,
            disk_threshold_percent: 10.0,
            notify_desktop: false,
            suspicious_names: vec![
                "nc".into(), "ncat".into(), "msfconsole".into(), "empire".into(),
                "mimikatz".into(), "beef-xss".into(), "xmrig".into(), "ccminer".into(),
                "ethminer".into(), "lolMiner".into(), "t-rex".into(), "minerd".into(),
                "cryptonight".into(), "stratum".into(), "meltdown".into(),
                "shellter".into(), "veil".into(), "cobaltstrike".into(),
                "meterpreter".into(), "bypassuac".into(), "psexec".into(),
                "wmiexec".into(), "smbexec".into(), "kerberoast".into(),
                "hashcat".into(), "john".into(), "hydra".into(),
                "medusa".into(), "sqlmap".into(), "nikto".into(),
                "gobuster".into(), "dirb".into(), "burpsuite".into(),
                "ettercap".into(), "bettercap".into(), "tshark".into(),
                "ransomware".into(), "lockbit".into(), "blackcat".into(),
                "stealer".into(), "redline".into(), "vidar".into(),
            ],
            suspicious_dirs: vec![
                "/tmp".into(), "/dev/shm".into(), "/var/tmp".into(),
                "/run".into(), "/var/run".into(),
            ],
            high_risk_ports: vec![
                22, 23, 3389, 5900, 4444, 6667, 1337, 31337, 5555, 9999,
                8443, 6666, 2222, 1080, 9050, 8080, 10000,
            ],
        }
    }
}

pub fn path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/komo".into());
    PathBuf::from(home).join(".config/komoguard").join("config.json")
}

pub fn legacy_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/komo".into());
    PathBuf::from(home).join("KomoGuard").join("config.json")
}

pub fn load() -> Config {
    let p = path();
    let lp = legacy_path();

    let config_path = if p.exists() { p } else if lp.exists() { lp } else { p };

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        eprintln!("[-] Config parse error ({}), using defaults: {}", config_path.display(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[-] Config read error ({}), using defaults: {}", config_path.display(), e);
            }
        }
    }

    let cfg = Config::default();
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&cfg) {
        Ok(json) => {
            let _ = fs::write(&config_path, json);
        }
        Err(e) => {
            eprintln!("[-] Failed to serialize default config: {}", e);
        }
    }
    cfg
}
