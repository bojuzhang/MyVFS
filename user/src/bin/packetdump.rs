#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::str;
use user_lib::{close, mount, open, read, O_RDONLY};

const PACKETFS_TARGET: &str = "/mnt/packetfs";
const PACKETS_PATH: &str = "/mnt/packetfs/packets";
const MOUNT_OPTIONS: &str = "snaplen=2048,capacity=256";
const READ_BUF_SIZE: usize = 512;
const HEX_BYTES_PER_LINE: usize = 32;
const PCAP_GLOBAL_HEADER_LEN: usize = 24;
const PCAP_RECORD_HEADER_LEN: usize = 16;
const TARGET_RECORDS: usize = 1;
const MAX_EXPORT_BYTES: usize = 64 * 1024;
const PCAP_MAGIC_LE: [u8; 4] = [0xd4, 0xc3, 0xb2, 0xa1];

#[no_mangle]
fn main() -> i32 {
    let rc = mount("packetfs", PACKETFS_TARGET, MOUNT_OPTIONS);
    if rc < 0 {
        eprintln!("packetfs mount failed: {}", rc);
        return 1;
    }
    println!("packetfs mount success: {}", PACKETFS_TARGET);

    let fd = open(PACKETS_PATH, O_RDONLY);
    if fd < 0 {
        eprintln!("{} open failed: {}", PACKETS_PATH, fd);
        return 1;
    }
    println!("{} open success: fd={}", PACKETS_PATH, fd);

    println!("PCAP_BEGIN");
    let mut exported = 0usize;
    let mut pcap = PcapParser::new();
    let mut buf = [0u8; READ_BUF_SIZE];
    loop {
        if pcap.records_complete() >= TARGET_RECORDS {
            break;
        }
        if exported >= MAX_EXPORT_BYTES {
            if pcap.at_record_boundary() {
                break;
            }
            eprintln!("packetdump reached max export bytes before a PCAP record boundary");
            let _ = close(fd as usize);
            return 1;
        }

        let remaining = pcap.current_remaining();
        if remaining == 0 {
            eprintln!("packetdump internal PCAP parser state is empty before target records");
            let _ = close(fd as usize);
            return 1;
        }
        if exported + remaining > MAX_EXPORT_BYTES && !pcap.at_record_boundary() {
            eprintln!("packetdump next PCAP record segment exceeds max export bytes");
            let _ = close(fd as usize);
            return 1;
        }

        let read_len = min(READ_BUF_SIZE, remaining);
        let n = read(fd as usize, &mut buf[..read_len]);
        if n < 0 {
            eprintln!("{} read failed: {}", PACKETS_PATH, n);
            let _ = close(fd as usize);
            return 1;
        }
        if n == 0 {
            eprintln!("{} returned EOF before a complete PCAP record was exported", PACKETS_PATH);
            let _ = close(fd as usize);
            return 1;
        }
        let n = n as usize;
        dump_hex(&buf[..n]);
        exported += n;
        if let Err(message) = pcap.consume(&buf[..n]) {
            eprintln!("packetdump PCAP parse failed: {}", message);
            let _ = close(fd as usize);
            return 1;
        }
    }
    println!("PCAP_END");

    let rc = close(fd as usize);
    if rc < 0 {
        eprintln!("{} close failed: {}", PACKETS_PATH, rc);
        return 1;
    }
    0
}

struct PcapParser {
    global_header: [u8; PCAP_GLOBAL_HEADER_LEN],
    global_len: usize,
    record_header: [u8; PCAP_RECORD_HEADER_LEN],
    record_header_len: usize,
    payload_remaining: usize,
    records_complete: usize,
}

impl PcapParser {
    const fn new() -> Self {
        Self {
            global_header: [0; PCAP_GLOBAL_HEADER_LEN],
            global_len: 0,
            record_header: [0; PCAP_RECORD_HEADER_LEN],
            record_header_len: 0,
            payload_remaining: 0,
            records_complete: 0,
        }
    }

    fn records_complete(&self) -> usize {
        self.records_complete
    }

    fn at_record_boundary(&self) -> bool {
        self.global_len == PCAP_GLOBAL_HEADER_LEN
            && self.record_header_len == 0
            && self.payload_remaining == 0
    }

    fn current_remaining(&self) -> usize {
        if self.global_len < PCAP_GLOBAL_HEADER_LEN {
            PCAP_GLOBAL_HEADER_LEN - self.global_len
        } else if self.record_header_len < PCAP_RECORD_HEADER_LEN {
            PCAP_RECORD_HEADER_LEN - self.record_header_len
        } else {
            self.payload_remaining
        }
    }

    fn consume(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if self.global_len < PCAP_GLOBAL_HEADER_LEN {
            let remaining = PCAP_GLOBAL_HEADER_LEN - self.global_len;
            if bytes.len() > remaining {
                return Err("read crossed the global header boundary");
            }
            let end = self.global_len + bytes.len();
            self.global_header[self.global_len..end].copy_from_slice(bytes);
            self.global_len = end;
            if self.global_len == PCAP_GLOBAL_HEADER_LEN
                && self.global_header[..4] != PCAP_MAGIC_LE
            {
                return Err("invalid little-endian PCAP magic");
            }
            return Ok(());
        }

        if self.record_header_len < PCAP_RECORD_HEADER_LEN {
            let remaining = PCAP_RECORD_HEADER_LEN - self.record_header_len;
            if bytes.len() > remaining {
                return Err("read crossed a record header boundary");
            }
            let end = self.record_header_len + bytes.len();
            self.record_header[self.record_header_len..end].copy_from_slice(bytes);
            self.record_header_len = end;
            if self.record_header_len == PCAP_RECORD_HEADER_LEN {
                self.payload_remaining = u32::from_le_bytes([
                    self.record_header[8],
                    self.record_header[9],
                    self.record_header[10],
                    self.record_header[11],
                ]) as usize;
                if self.payload_remaining > MAX_EXPORT_BYTES {
                    return Err("record incl_len exceeds max export bytes");
                }
                if self.payload_remaining == 0 {
                    self.finish_record();
                }
            }
            return Ok(());
        }

        if bytes.len() > self.payload_remaining {
            return Err("read crossed a record payload boundary");
        }
        self.payload_remaining -= bytes.len();
        if self.payload_remaining == 0 {
            self.finish_record();
        }
        Ok(())
    }

    fn finish_record(&mut self) {
        self.records_complete += 1;
        self.record_header = [0; PCAP_RECORD_HEADER_LEN];
        self.record_header_len = 0;
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

fn dump_hex(bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut line = [0u8; HEX_BYTES_PER_LINE * 2];
    for chunk in bytes.chunks(HEX_BYTES_PER_LINE) {
        for (idx, byte) in chunk.iter().enumerate() {
            line[idx * 2] = HEX[(byte >> 4) as usize];
            line[idx * 2 + 1] = HEX[(byte & 0x0f) as usize];
        }
        let len = chunk.len() * 2;
        if let Ok(s) = str::from_utf8(&line[..len]) {
            println!("{}", s);
        }
    }
}
