#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
LOG="${1:-${LOG:-qemu.log}}"
PCAP="${PCAP:-cap.pcap}"

fail() {
    echo "error: $*" >&2
    exit 1
}

require_log() {
    local pattern="$1"
    local message="$2"
    if ! grep -Eq "$pattern" "$LOG"; then
        fail "$message"
    fi
}

[[ -f "$LOG" ]] || fail "log not found: $LOG"

require_log "OpenSBI|qemu|QEMU|virtio|Boot" "QEMU startup output was not found in $LOG"
require_log "packetfs mount success" "packetfs mount success output was not found"
require_log "/mnt/packetfs/packets open success" "packetdump did not report packets open success"
require_log "PCAP_BEGIN" "PCAP_BEGIN was not found"
require_log "PCAP_END" "PCAP_END was not found"

python3 "$SCRIPT_DIR/collect-pcap.py" "$LOG" -o "$PCAP"
[[ -s "$PCAP" ]] || fail "collected PCAP is missing or empty: $PCAP"

if ! command -v tcpdump >/dev/null 2>&1; then
    fail "tcpdump not found; install tcpdump and check PATH"
fi
if ! tcpdump -r "$PCAP" >/dev/null; then
    fail "tcpdump -r $PCAP failed"
fi

if ! awk '
    $1 ~ /^captured_packets=/ {
        split($1, parts, "=");
        if (parts[2] + 0 > 0) {
            found = 1;
        }
    }
    END { exit found ? 0 : 1 }
' "$LOG"; then
    fail "stats did not show captured_packets > 0"
fi

echo "packetfs demo checks passed"
