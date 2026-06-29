#!/usr/bin/env bash
set -euo pipefail

TAP="${TAP:-${1:-tap0}}"
TAP_USER="${TAP_USER:-${USER:-}}"

if [[ -z "$TAP_USER" ]]; then
    TAP_USER="$(id -un)"
fi

if [[ ! -e /dev/net/tun ]]; then
    echo "error: /dev/net/tun does not exist; this host does not expose TAP support" >&2
    exit 1
fi

if ! command -v ip >/dev/null 2>&1; then
    echo "error: ip command not found; install iproute2" >&2
    exit 1
fi

if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
    SUDO=()
else
    SUDO=(sudo)
fi

if ip link show "$TAP" >/dev/null 2>&1; then
    echo "reusing existing TAP device: $TAP"
else
    if ! "${SUDO[@]}" ip tuntap add "$TAP" mode tap user "$TAP_USER"; then
        echo "error: failed to create $TAP; sudo or CAP_NET_ADMIN is required" >&2
        exit 1
    fi
fi

if ! "${SUDO[@]}" ip link set "$TAP" up; then
    echo "error: failed to bring $TAP up; sudo or CAP_NET_ADMIN is required" >&2
    exit 1
fi

ip link show "$TAP"
