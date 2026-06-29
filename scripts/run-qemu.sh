#!/usr/bin/env bash
set -euo pipefail

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
