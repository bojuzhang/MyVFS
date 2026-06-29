#!/usr/bin/env python3
"""Send one Ethernet frame through an existing TAP interface."""

import argparse
import errno
import fcntl
import os
import struct
import sys

TUN_DEVICE = "/dev/net/tun"
TUNSETIFF = 0x400454CA
IFF_TAP = 0x0002
IFF_NO_PI = 0x1000
MIN_ETHERNET_FRAME = 60


def parse_mac(text: str) -> bytes:
    parts = text.split(":")
    if len(parts) != 6:
        raise argparse.ArgumentTypeError(f"invalid MAC address: {text}")
    try:
        raw = bytes(int(part, 16) for part in parts)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"invalid MAC address: {text}") from exc
    if len(raw) != 6:
        raise argparse.ArgumentTypeError(f"invalid MAC address: {text}")
    return raw


def parse_ethertype(text: str) -> int:
    try:
        value = int(text, 0)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"invalid ethertype: {text}") from exc
    if not 0 <= value <= 0xFFFF:
        raise argparse.ArgumentTypeError("ethertype must fit in 16 bits")
    return value


def open_tap(name: str) -> int:
    if len(name.encode()) >= 16:
        raise ValueError("TAP interface name must be shorter than 16 bytes")
    fd = os.open(TUN_DEVICE, os.O_RDWR)
    try:
        ifreq = struct.pack("16sH", name.encode(), IFF_TAP | IFF_NO_PI)
        fcntl.ioctl(fd, TUNSETIFF, ifreq)
    except Exception:
        os.close(fd)
        raise
    return fd


def build_frame(args: argparse.Namespace) -> bytes:
    payload = args.payload.encode()
    if args.payload_hex is not None:
        payload = bytes.fromhex(args.payload_hex)
    frame = args.dst + args.src + struct.pack("!H", args.ethertype) + payload
    if len(frame) < MIN_ETHERNET_FRAME:
        frame += bytes(MIN_ETHERNET_FRAME - len(frame))
    return frame


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tap", default="tap0", help="TAP interface name")
    parser.add_argument("--dst", type=parse_mac, default=parse_mac("ff:ff:ff:ff:ff:ff"))
    parser.add_argument("--src", type=parse_mac, default=parse_mac("02:00:00:00:00:01"))
    parser.add_argument("--ethertype", type=parse_ethertype, default=0x88B5)
    parser.add_argument("--payload", default="packetfs-demo-frame")
    parser.add_argument("--payload-hex", help="payload bytes as hexadecimal")
    args = parser.parse_args()

    try:
        frame = build_frame(args)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    try:
        fd = open_tap(args.tap)
    except FileNotFoundError:
        print(f"error: {TUN_DEVICE} not found; run on a host with TAP support", file=sys.stderr)
        return 1
    except PermissionError:
        print("error: permission denied opening TAP; run setup-tap.sh or use sudo", file=sys.stderr)
        return 1
    except OSError as exc:
        if exc.errno in (errno.ENODEV, errno.EINVAL):
            print(f"error: TAP {args.tap} is not available; run scripts/setup-tap.sh", file=sys.stderr)
        else:
            print(f"error: failed to open TAP {args.tap}: {exc}", file=sys.stderr)
        return 1

    try:
        written = os.write(fd, frame)
    finally:
        os.close(fd)
    print(f"sent {written} bytes on {args.tap}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
