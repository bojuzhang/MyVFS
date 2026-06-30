#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
MODE="${PACKETFS_DEMO_MODE:-host}"

if [[ "${1:-}" == "--qemu" ]]; then
    MODE="qemu"
    shift
fi

if [[ "$MODE" == "host" ]]; then
    cd "$REPO_ROOT"
    exec cargo run -q -p user --bin "${DEMO_BIN:-packetdump}" -- "$@"
fi

if [[ "$MODE" != "qemu" ]]; then
    echo "error: unknown PACKETFS_DEMO_MODE: $MODE" >&2
    exit 1
fi

QEMU_BIN="${QEMU_BIN:-qemu-system-riscv64}"
KERNEL="${KERNEL:-target/riscv64gc-unknown-none-elf/release/kernel}"
TAP="${TAP:-tap0}"

if ! command -v "$QEMU_BIN" >/dev/null 2>&1; then
    echo "error: $QEMU_BIN not found; install QEMU RISC-V support" >&2
    exit 1
fi

if [[ ! -f "$KERNEL" ]]; then
    echo "error: kernel image not found: $KERNEL" >&2
    echo "hint: build the kernel before running this script" >&2
    exit 1
fi

exec "$QEMU_BIN" \
    -machine virt \
    -nographic \
    -kernel "$KERNEL" \
    -device virtio-net-device,netdev=net0 \
    -netdev "tap,id=net0,ifname=${TAP},script=no,downscript=no" \
    "$@"
