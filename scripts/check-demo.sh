#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
LOG="${1:-${LOG:-target/packetfs-demo.log}}"
PCAP="${PCAP:-target/cap.pcap}"
SUMMARY="${SUMMARY:-target/cap.summary.txt}"

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

mkdir -p "$(dirname -- "$LOG")"
mkdir -p "$(dirname -- "$PCAP")"
mkdir -p "$(dirname -- "$SUMMARY")"
if [[ "${CHECK_DEMO_RUN:-1}" == "1" ]]; then
    (cd "$REPO_ROOT" && "$SCRIPT_DIR/run-qemu.sh") >"$LOG" 2>&1
fi

[[ -f "$LOG" ]] || fail "log not found: $LOG"

require_log "PACKETFS_DEMO_BEGIN" "demo begin marker was not found"
require_log "PACKETFS_DEMO_END" "demo end marker was not found"
require_log "VFS_STAT label=mountpoint-before path=/mnt/packetfs rc=0 .* type=directory" "mountpoint stat before mount was not shown"
require_log "VFS_MOUNT target=/mnt/packetfs fs=packetfs options=snaplen=2048,capacity=256 rc=0" "packetfs mount output was not found"
require_log "VFS_STAT label=mountpoint-after path=/mnt/packetfs rc=0 .* type=directory" "mountpoint stat after mount was not shown"
require_log "VFS_DIRENT path=/mnt/packetfs name=packets inode=[0-9]+ type=regular" "packets dirent was not shown"
require_log "VFS_DIRENT path=/mnt/packetfs name=stats inode=[0-9]+ type=regular" "stats dirent was not shown"
require_log "VFS_OPEN_WRITE path=/mnt/packetfs/packets flags=O_WRONLY rc=-13" "write-open denial was not shown"
require_log "VFS_OPEN path=/mnt/packetfs/packets flags=O_RDONLY rc=[0-9]+" "packetdump did not report packets open"
require_log "VFS_SECOND_READER path=/mnt/packetfs/packets flags=O_RDONLY rc=-16" "single-reader guard was not shown"
require_log "VFS_WRITE_ATTEMPT path=/mnt/packetfs/packets fd=[0-9]+ bytes=\"packetfs-write-attempt\" rc=-13" "write attempt denial was not shown"
require_log "READ_COMPLETE path=/mnt/packetfs/packets chunks=[0-9]+ bytes=[0-9]+ records=3" "packet stream did not report three records"
require_log "PCAP_RECORDS total=3" "decoded PCAP record count was not shown"
require_log "PCAP_BEGIN" "PCAP_BEGIN was not found"
require_log "PCAP_END" "PCAP_END was not found"
require_log "STATS_BEGIN path=/mnt/packetfs/stats" "stats begin marker was not found"
require_log "STATS_END path=/mnt/packetfs/stats" "stats end marker was not found"
require_log "VFS_UMOUNT target=/mnt/packetfs rc=0" "umount output was not found"
require_log "VFS_STAT label=mountpoint-after-umount path=/mnt/packetfs rc=0 .* type=directory" "mountpoint stat after umount was not shown"

for index in 1 2 3; do
    require_log "TX_FRAME index=${index} .* payload=\"packetfs-demo-frame-${index}" "tx frame ${index} payload was not shown"
    require_log "RX_SUBMIT index=${index} result=Queued" "rx submit ${index} did not queue"
    require_log "READ_CHUNK index=[0-9]+ phase=record_payload record=${index}" "record ${index} payload read chunk was not shown"
    require_log "PCAP_RECORD index=${index} .* payload=\"packetfs-demo-frame-${index}" "decoded PCAP record ${index} payload was not shown"
done

python3 "$SCRIPT_DIR/collect-pcap.py" "$LOG" -o "$PCAP" \
    --summary \
    --expect-records 3 \
    --expect-payload packetfs-demo-frame-1 \
    --expect-payload packetfs-demo-frame-2 \
    --expect-payload packetfs-demo-frame-3 \
    >"$SUMMARY"
cat "$SUMMARY"
[[ -s "$PCAP" ]] || fail "collected PCAP is missing or empty: $PCAP"

if ! grep -Eq "pcap file content: .* records=3" "$SUMMARY"; then
    fail "PCAP summary did not show three records"
fi
for index in 1 2 3; do
    if ! grep -Eq "pcap record#${index}: .*payload=\"packetfs-demo-frame-${index}" "$SUMMARY"; then
        fail "PCAP file summary did not show payload for record ${index}"
    fi
done

if command -v tcpdump >/dev/null 2>&1; then
    if ! tcpdump -r "$PCAP" >/dev/null; then
        fail "tcpdump -r $PCAP failed"
    fi
else
    echo "warning: tcpdump not found; skipped tcpdump validation" >&2
fi

require_stat_value() {
    local field="$1"
    local expected="$2"
    if ! awk -F= -v field="$field" -v expected="$expected" '
        $1 == field {
            found = ($2 + 0 == expected)
        }
        END { exit found ? 0 : 1 }
    ' "$LOG"; then
        fail "stats did not show ${field}=${expected}"
    fi
}

require_stat_value "captured_packets" 3
require_stat_value "captured_bytes" 180
require_stat_value "read_packets" 3
require_stat_value "read_bytes" 180
require_stat_value "queued_packets" 0

if ! awk -F= '
    $1 == "reader_active" {
        found = ($2 == "false")
    }
    END { exit found ? 0 : 1 }
' "$LOG"; then
    fail "stats did not show reader_active=false"
fi

echo "packetfs demo checks passed"
