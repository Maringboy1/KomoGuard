use std::collections::HashSet;
use std::fs;

use crate::alerts::Alert;
use crate::config::Config;

fn parse_tcp_table(path: &str) -> Vec<(String, u16, String, String, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let state = parts[3];
        let local = parts[1];
        let remote = parts[2];

        let local_parts: Vec<&str> = local.split(':').collect();
        let remote_parts: Vec<&str> = remote.split(':').collect();

        if local_parts.len() < 2 || remote_parts.len() < 2 {
            continue;
        }

        let local_ip_hex = local_parts[0];
        let local_port_hex = local_parts[1];
        let remote_ip_hex = remote_parts[0];

        let local_port = u16::from_str_radix(local_port_hex, 16).unwrap_or(0);

        entries.push((local_ip_hex.to_string(), local_port, state.to_string(), remote_ip_hex.to_string(), remote_parts.get(1).unwrap_or(&"0").to_string()));
    }

    entries
}

fn hex_ip_to_string(hex: &str) -> String {
    let hex = hex.trim();
    if hex.len() < 8 {
        return hex.to_string();
    }

    let bytes: Vec<u8> = (0..4)
        .filter_map(|i| {
            let start = i * 2;
            if start + 2 <= hex.len() {
                u8::from_str_radix(&hex[start..start + 2], 16).ok()
            } else {
                None
            }
        })
        .collect();

    if bytes.len() == 4 {
        format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
    } else {
        hex.to_string()
    }
}

pub fn get_listening_ports() -> Option<HashSet<u16>> {
    let mut ports = HashSet::new();

    for table in &["/proc/net/tcp", "/proc/net/tcp6"] {
        for (_, port, state, _, _) in parse_tcp_table(table) {
            if state == "0A" {
                ports.insert(port);
            }
        }
    }

    Some(ports)
}

pub fn check_new_ports(
    current: &HashSet<u16>,
    previous: &HashSet<u16>,
) -> Vec<Alert> {
    let mut alerts = Vec::new();
    for port in current.difference(previous) {
        if *port > 0 {
            alerts.push(Alert::warning(
                "New Listening Port",
                &format!("Port {} is now listening on this system", port),
            ));
        }
    }
    alerts
}

pub fn check_connections(config: &Config) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for table in &["/proc/net/tcp", "/proc/net/tcp6"] {
        for (_local_ip, local_port, state, remote_ip, _) in parse_tcp_table(table) {
            if state == "01" {
                // Established connection
                let remote_str = hex_ip_to_string(&remote_ip);
                if remote_str != "0.0.0.0" && remote_str != "::0" {
                    if config.high_risk_ports.contains(&local_port) {
                        alerts.push(Alert::info(
                            "Connection on Risk Port",
                            &format!("Port {} connected to {}", local_port, remote_str),
                        ));
                    }
                }
            }
        }
    }

    alerts
}
