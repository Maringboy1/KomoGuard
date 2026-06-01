use std::fs;
use std::io::Write;

#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub timestamp: String,
    pub severity: Severity,
    pub title: String,
    pub message: String,
}

impl Alert {
    fn now() -> String {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn info(title: &str, message: &str) -> Self {
        Alert {
            timestamp: Self::now(),
            severity: Severity::Info,
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    pub fn warning(title: &str, message: &str) -> Self {
        Alert {
            timestamp: Self::now(),
            severity: Severity::Warning,
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    pub fn critical(title: &str, message: &str) -> Self {
        Alert {
            timestamp: Self::now(),
            severity: Severity::Critical,
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    fn to_line(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.timestamp,
            self.severity.as_str(),
            self.title,
            self.message.replace('|', "/").replace('\n', " ")
        )
    }
}

fn alert_file() -> String {
    "/dev/shm/komoguard.alerts".to_string()
}

pub fn record(alert: &Alert) {
    let path = alert_file();
    let line = alert.to_line();
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{}", line);
    }

}

pub fn recent(count: usize) -> Vec<String> {
    let path = alert_file();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > count { lines.len() - count } else { 0 };
    lines[start..]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn total_alerts() -> usize {
    let path = alert_file();
    match fs::read_to_string(&path) {
        Ok(c) => c.lines().count(),
        Err(_) => 0,
    }
}
