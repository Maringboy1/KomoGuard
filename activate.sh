#!/bin/bash
# KomoGuard - Terminal activation script (alert display only)
# Does NOT auto-start the daemon. Use `komoguard start` to start manually.

KOMOGUARD_BIN="$HOME/.cargo/bin/komoguard"
ALERT_FILE="/dev/shm/komoguard.alerts"
PID_FILE="/tmp/komoguard.pid"

# Only proceed if the binary exists
if [ ! -f "$KOMOGUARD_BIN" ]; then
    return 0
fi

# Check if daemon is running
DAEMON_RUNNING=0
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE" 2>/dev/null)
    if [ -n "$PID" ] && [ -d "/proc/$PID" ]; then
        DAEMON_RUNNING=1
    else
        rm -f "$PID_FILE" 2>/dev/null
    fi
fi

# Show alerts if any exist
if [ -f "$ALERT_FILE" ] && [ -s "$ALERT_FILE" ]; then
    TOTAL=$(wc -l < "$ALERT_FILE" 2>/dev/null || echo 0)
    if [ "$TOTAL" -gt 0 ]; then
        echo ""
        echo "  ╭──────────────────────────────────────────╮"
        printf "  │  \033[1;32mKomoGuard\033[0m - System Security Monitor  │\n"
        printf "  │  Total alerts: %-27d│\n" "$TOTAL"
        if [ "$DAEMON_RUNNING" -eq 1 ]; then
            printf "  │  Daemon: \033[1;32mrunning\033[0m                     │\n"
        else
            printf "  │  Daemon: \033[1;33mstopped\033[0m (start: komoguard start) │\n"
        fi
        echo "  ├──────────────────────────────────────────┤"
        echo "  │  Recent alerts:                          │"

        tail -3 "$ALERT_FILE" 2>/dev/null | while IFS='|' read -r ts sev title msg; do
            case "$sev" in
                critical)
                    printf "  │  \033[1;31m!\033[0m %-35s│\n" "$title"
                    ;;
                warning)
                    printf "  │  \033[1;33m!\033[0m %-35s│\n" "$title"
                    ;;
                *)
                    printf "  │  \033[1;34mi\033[0m %-35s│\n" "$title"
                    ;;
            esac
        done

        echo "  ╰──────────────────────────────────────────╯"
        echo ""
    fi
fi
