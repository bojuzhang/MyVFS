#!/usr/bin/env python3
"""Recover a binary PCAP file from packetdump serial hex output."""

import argparse
import re
import struct
import subprocess
import sys
from pathlib import Path

BEGIN = "PCAP_BEGIN"
END = "PCAP_END"
PCAP_LITTLE_MAGICS = {
    bytes.fromhex("d4c3b2a1"),
    bytes.fromhex("4d3cb2a1"),
}
PCAP_BIG_MAGICS = {
    bytes.fromhex("a1b2c3d4"),
    bytes.fromhex("a1b23c4d"),
}
PCAP_MAGICS = PCAP_LITTLE_MAGICS | PCAP_BIG_MAGICS


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


def parse_pcap(data: bytes) -> tuple[dict[str, int], list[dict[str, object]]]:
    if len(data) < 24:
        raise ValueError("PCAP data is shorter than a global header")

    magic = data[:4]
    if magic in PCAP_LITTLE_MAGICS:
        endian = "<"
    elif magic in PCAP_BIG_MAGICS:
        endian = ">"
    else:
        raise ValueError("PCAP magic is invalid; check packetdump or pcap encoder")

    version_major, version_minor, thiszone, sigfigs, snaplen, network = struct.unpack(
        endian + "HHiIII", data[4:24]
    )
    header = {
        "version_major": version_major,
        "version_minor": version_minor,
        "thiszone": thiszone,
        "sigfigs": sigfigs,
        "snaplen": snaplen,
        "network": network,
    }

    records: list[dict[str, object]] = []
    offset = 24
    while offset < len(data):
        if len(data) - offset < 16:
            raise ValueError("PCAP record header is truncated")
        ts_sec, ts_frac, incl_len, orig_len = struct.unpack(
            endian + "IIII", data[offset : offset + 16]
        )
        offset += 16
        if len(data) - offset < incl_len:
            raise ValueError("PCAP record payload is truncated")
        frame = data[offset : offset + incl_len]
        offset += incl_len
        records.append(
            {
                "ts_sec": ts_sec,
                "ts_frac": ts_frac,
                "incl_len": incl_len,
                "orig_len": orig_len,
                "frame": frame,
            }
        )
    return header, records


def mac(bytes_: bytes) -> str:
    if len(bytes_) != 6:
        return "??:??:??:??:??:??"
    return ":".join(f"{byte:02x}" for byte in bytes_)


def ethertype(frame: bytes) -> int:
    if len(frame) < 14:
        return 0
    return int.from_bytes(frame[12:14], "big")


def frame_payload(frame: bytes) -> bytes:
    if len(frame) <= 14:
        return b""
    return frame[14:].rstrip(b"\x00")


def ascii_preview(bytes_: bytes) -> str:
    return "".join(chr(byte) if 0x20 <= byte <= 0x7E else "." for byte in bytes_)


def print_summary(data: bytes) -> None:
    header, records = parse_pcap(data)
    print(
        "pcap file content: "
        f"version={header['version_major']}.{header['version_minor']} "
        f"snaplen={header['snaplen']} network={header['network']} "
        f"records={len(records)}"
    )
    for index, record in enumerate(records, start=1):
        frame = record["frame"]
        assert isinstance(frame, bytes)
        print(
            f"pcap record#{index}: "
            f"ts={record['ts_sec']}.{record['ts_frac']:06d} "
            f"incl_len={record['incl_len']} orig_len={record['orig_len']} "
            f"dst={mac(frame[:6])} src={mac(frame[6:12])} "
            f"ethertype=0x{ethertype(frame):04x} "
            f"payload=\"{ascii_preview(frame_payload(frame))}\""
        )


def check_expectations(
    data: bytes, expect_records: int | None, expect_payloads: list[str]
) -> None:
    _, records = parse_pcap(data)
    if expect_records is not None and len(records) != expect_records:
        raise ValueError(f"expected {expect_records} PCAP records, got {len(records)}")

    for payload in expect_payloads:
        expected = payload.encode()
        if not any(expected in record["frame"] for record in records):
            raise ValueError(f"expected payload not found in PCAP: {payload}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", nargs="?", help="demo log, or stdin when omitted")
    parser.add_argument("-o", "--output", default="cap.pcap", help="output PCAP path")
    parser.add_argument("--tcpdump", action="store_true", help="run tcpdump -r after writing")
    parser.add_argument("--summary", action="store_true", help="print decoded PCAP records")
    parser.add_argument("--expect-records", type=int, help="require this many records")
    parser.add_argument(
        "--expect-payload",
        action="append",
        default=[],
        help="require a payload substring; may be used multiple times",
    )
    args = parser.parse_args()

    try:
        data = decode_pcap(extract_hex(read_text(args.log)))
        check_expectations(data, args.expect_records, args.expect_payload)
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    output = Path(args.output)
    output.write_bytes(data)
    print(f"wrote {len(data)} bytes to {output}")
    if args.summary:
        print_summary(data)

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
