#!/usr/bin/env python3
"""Recover a binary PCAP file from packetdump serial hex output."""

import argparse
import re
import subprocess
import sys
from pathlib import Path

BEGIN = "PCAP_BEGIN"
END = "PCAP_END"
PCAP_MAGICS = {
    bytes.fromhex("d4c3b2a1"),
    bytes.fromhex("a1b2c3d4"),
    bytes.fromhex("4d3cb2a1"),
    bytes.fromhex("a1b23c4d"),
}


def read_text(path: str | None) -> str:
    if path is None or path == "-":
        return sys.stdin.read()
    return Path(path).read_text(errors="ignore")


def extract_hex(text: str) -> str:
    begin = text.find(BEGIN)
    if begin < 0:
        raise ValueError(f"{BEGIN} not found")
    body_start = begin + len(BEGIN)
    end = text.find(END, body_start)
    if end < 0:
        raise ValueError(f"{END} not found")
    body = text[body_start:end]
    return "".join(re.findall(r"[0-9a-fA-F]", body))


def decode_pcap(hex_text: str) -> bytes:
    if not hex_text:
        raise ValueError("no PCAP hex was collected")
    if len(hex_text) % 2 != 0:
        raise ValueError("PCAP hex length is odd")
    data = bytes.fromhex(hex_text)
    if len(data) < 24:
        raise ValueError("PCAP data is shorter than a global header")
    if data[:4] not in PCAP_MAGICS:
        raise ValueError("PCAP magic is invalid; check packetdump or pcap encoder")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", nargs="?", help="QEMU serial log, or stdin when omitted")
    parser.add_argument("-o", "--output", default="cap.pcap", help="output PCAP path")
    parser.add_argument("--tcpdump", action="store_true", help="run tcpdump -r after writing")
    args = parser.parse_args()

    try:
        data = decode_pcap(extract_hex(read_text(args.log)))
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    output = Path(args.output)
    output.write_bytes(data)
    print(f"wrote {len(data)} bytes to {output}")

    if args.tcpdump:
        try:
            subprocess.run(["tcpdump", "-r", str(output)], check=True)
        except FileNotFoundError:
            print("error: tcpdump not found", file=sys.stderr)
            return 1
        except subprocess.CalledProcessError as exc:
            print(f"error: tcpdump failed with status {exc.returncode}", file=sys.stderr)
            return exc.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
