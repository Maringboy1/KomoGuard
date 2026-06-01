# KomoGuard

Real-time system security monitor for Kali Linux. Detects suspicious processes, network anomalies, and resource abuse.

## Features

- **Process Monitoring** — detects miners, malware, reverse shells, and pentest tools (msfconsole, mimikatz, xmrig, etc.)
- **Network Monitoring** — watches listening ports on high-risk ports and established connections
- **Resource Monitoring** — alerts on high CPU / memory / disk usage
- **Daemon Mode** — runs in the background, checks every N seconds
- **Terminal Integration** — shows outstanding alerts when you open your shell
- **Export** — saves alerts to JSON reports

## Quick Install

```bash
git clone https://github.com/your-username/KomoGuard
cd KomoGuard
./setup.sh
```

The script builds the binary, installs it to `~/.cargo/bin/`, copies the config, and optionally installs terminal enhancements (zsh plugins, Starship prompt, fzf).

## Usage

| Command | Description |
|---------|-------------|
| `komoguard scan` | Run a one-time security scan (default) |
| `komoguard start` | Start background daemon |
| `komoguard stop` | Stop daemon |
| `komoguard status` | Show daemon status and alert count |
| `komoguard alerts` | View recent alerts |
| `komoguard export` | Export alerts to a JSON file |
| `komoguard clear` | Clear alert log |

### Example

```
$ komoguard scan

  ╔══════════════════════════════════════════╗
  ║     KomoGuard Security Scan              ║
  ╚══════════════════════════════════════════╝

  Started: 2026-06-01 12:30:00

   Processes    Scanning 342 processes...
   Network      Checking listening ports and connections...
   Resources    Checking CPU, memory, and disk...

  ✓ No issues detected. System looks clean.

  Finished: 12:30:01
  Total findings: 0
```

## Daemon Mode

Start the daemon for continuous monitoring:

```bash
komoguard start
```

It checks system health every 30 seconds (configurable) and logs alerts to `/dev/shm/komoguard.alerts`. The terminal shows alert status on shell startup via `activate.sh`.

## Configuration

Edit `~/.config/komoguard/config.json`:

```json
{
  "check_interval_seconds": 30,
  "cpu_threshold_percent": 90,
  "memory_threshold_percent": 90,
  "disk_threshold_percent": 10,
  "notify_desktop": false,
  "suspicious_names": ["xmrig", "msfconsole", "mimikatz", ...],
  "suspicious_dirs": ["/tmp", "/dev/shm", ...],
  "high_risk_ports": [4444, 1337, 31337, ...]
}
```

## Building from Source

```bash
cargo build --release
./target/release/komoguard scan
```

## Project Structure

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point and command dispatch |
| `src/config.rs` | Load/save JSON configuration |
| `src/alerts.rs` | Alert data model and persistent logging |
| `src/processes.rs` | Process scanning (names, directories, reverse shells) |
| `src/network.rs` | Network scanning from `/proc/net/tcp` |
| `src/resources.rs` | CPU, memory, and disk monitoring |
| `src/watcher.rs` | Orchestrates a full scan cycle |
| `activate.sh` | Terminal integration — shows alerts on shell start |
| `setup.sh` | Build, install, and optional terminal enhancements |
| `cleanup.sh` | Frees disk space (trash, caches, browser cache) |
| `config.json` | Default configuration file |

## Cleanup

```bash
./cleanup.sh
```

Removes trash, Gradle cache, npm cache, thumbnails, and Chromium cache — no system files touched.
