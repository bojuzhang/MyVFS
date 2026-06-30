#[macro_use]
extern crate user_lib;

use core::fmt;
use core::str;
use std::process;
use user_lib::{
    close, getdents, mount, open, read, stat, umount, write, Stat, O_RDONLY, O_WRONLY, STDOUT,
};

const PACKETFS_TARGET: &str = "/mnt/packetfs";
const PACKETS_PATH: &str = "/mnt/packetfs/packets";
const STATS_PATH: &str = "/mnt/packetfs/stats";
const MOUNT_OPTIONS: &str = "snaplen=2048,capacity=256";
const READ_BUF_SIZE: usize = 32;
const STATS_BUF_SIZE: usize = 2048;
const DIRENT_BUF_SIZE: usize = 512;
const PCAP_HEX_BYTES_PER_LINE: usize = 32;
const PCAP_GLOBAL_HEADER_LEN: usize = 24;
const PCAP_RECORD_HEADER_LEN: usize = 16;
const TARGET_RECORDS: usize = 3;
const MAX_EXPORT_BYTES: usize = 64 * 1024;
const PCAP_MAGIC_LE: [u8; 4] = [0xd4, 0xc3, 0xb2, 0xa1];
const EXPECTED_EACCES: isize = -13;
const EXPECTED_EBUSY: isize = -16;

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    println!("PACKETFS_DEMO_BEGIN");

    if print_stat("mountpoint-before", PACKETFS_TARGET) < 0 {
        return 1;
    }

    let rc = mount("packetfs", PACKETFS_TARGET, MOUNT_OPTIONS);
    println!(
        "VFS_MOUNT target={} fs=packetfs options={} rc={}",
        PACKETFS_TARGET, MOUNT_OPTIONS, rc
    );
    if rc < 0 {
        eprintln!("packetfs mount failed: {}", rc);
        return 1;
    }

    if print_stat("mountpoint-after", PACKETFS_TARGET) < 0
        || dump_dir(PACKETFS_TARGET) < 0
        || print_stat("packets-file", PACKETS_PATH) < 0
        || print_stat("stats-file", STATS_PATH) < 0
        || show_write_open_denied() < 0
    {
        return 1;
    }

    inject_demo_rx_frames();

    let fd = open(PACKETS_PATH, O_RDONLY);
    println!("VFS_OPEN path={} flags=O_RDONLY rc={}", PACKETS_PATH, fd);
    if fd < 0 {
        eprintln!("{} open failed: {}", PACKETS_PATH, fd);
        return 1;
    }

    if show_single_reader_guard() < 0 || show_write_attempt_denied(fd as usize) < 0 {
        let _ = close(fd as usize);
        return 1;
    }

    let mut pcap_bytes = Vec::new();
    if read_packet_stream(fd as usize, &mut pcap_bytes) < 0 {
        let _ = close(fd as usize);
        return 1;
    }

    let rc = close(fd as usize);
    println!("VFS_CLOSE path={} fd={} rc={}", PACKETS_PATH, fd, rc);
    if rc < 0 {
        eprintln!("{} close failed: {}", PACKETS_PATH, rc);
        return 1;
    }

    if print_pcap_records(&pcap_bytes) < 0 {
        return 1;
    }

    println!("PCAP_BEGIN");
    dump_hex(&pcap_bytes);
    println!("PCAP_END");

    if dump_stats() < 0 {
        return 1;
    }

    let rc = umount(PACKETFS_TARGET);
    println!("VFS_UMOUNT target={} rc={}", PACKETFS_TARGET, rc);
    if rc < 0 {
        eprintln!("{} umount failed: {}", PACKETFS_TARGET, rc);
        return 1;
    }

    if print_stat("mountpoint-after-umount", PACKETFS_TARGET) < 0 {
        return 1;
    }

    println!("PACKETFS_DEMO_END");
    0
}

fn print_stat(label: &str, path: &str) -> isize {
    let mut metadata = Stat::default();
    let rc = stat(path, &mut metadata);
    if rc < 0 {
        println!("VFS_STAT label={} path={} rc={}", label, path, rc);
        eprintln!("{} stat failed: {}", path, rc);
        return -1;
    }

    println!(
        "VFS_STAT label={} path={} rc={} inode={} type={} mode=0o{:o} size={}",
        label,
        path,
        rc,
        metadata.inode_id,
        file_type_name(metadata.file_type),
        metadata.mode,
        metadata.size
    );
    0
}

fn dump_dir(path: &str) -> isize {
    let fd = open(path, O_RDONLY);
    println!("VFS_OPENDIR path={} rc={}", path, fd);
    if fd < 0 {
        eprintln!("{} opendir failed: {}", path, fd);
        return -1;
    }

    let mut buf = [0u8; DIRENT_BUF_SIZE];
    let n = getdents(fd as usize, &mut buf);
    println!("VFS_GETDENTS path={} fd={} bytes={}", path, fd, n);
    if n < 0 {
        eprintln!("{} getdents failed: {}", path, n);
        let _ = close(fd as usize);
        return -1;
    }

    if parse_dirents(path, &buf[..n as usize]) < 0 {
        let _ = close(fd as usize);
        return -1;
    }

    let eof = getdents(fd as usize, &mut buf);
    println!("VFS_GETDENTS_EOF path={} fd={} bytes={}", path, fd, eof);
    if eof < 0 {
        eprintln!("{} getdents eof check failed: {}", path, eof);
        let _ = close(fd as usize);
        return -1;
    }

    let rc = close(fd as usize);
    println!("VFS_CLOSEDIR path={} fd={} rc={}", path, fd, rc);
    if rc < 0 {
        eprintln!("{} closedir failed: {}", path, rc);
        return -1;
    }
    0
}

fn parse_dirents(path: &str, bytes: &[u8]) -> isize {
    let mut offset = 0usize;
    while offset < bytes.len() {
        if bytes.len() - offset < 12 {
            eprintln!("{} getdents entry is truncated", path);
            return -1;
        }

        let inode_id = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let file_type = u16::from_le_bytes([bytes[offset + 8], bytes[offset + 9]]);
        let name_len = u16::from_le_bytes([bytes[offset + 10], bytes[offset + 11]]) as usize;
        offset += 12;

        if bytes.len() - offset < name_len {
            eprintln!("{} getdents name is truncated", path);
            return -1;
        }

        let name = match str::from_utf8(&bytes[offset..offset + name_len]) {
            Ok(name) => name,
            Err(_) => {
                eprintln!("{} getdents name is not utf8", path);
                return -1;
            }
        };
        println!(
            "VFS_DIRENT path={} name={} inode={} type={}",
            path,
            name,
            inode_id,
            file_type_name(file_type)
        );
        offset += name_len;
    }
    0
}

fn show_write_open_denied() -> isize {
    let fd = open(PACKETS_PATH, O_WRONLY);
    println!(
        "VFS_OPEN_WRITE path={} flags=O_WRONLY rc={} expected={}",
        PACKETS_PATH, fd, EXPECTED_EACCES
    );
    if fd >= 0 {
        let _ = close(fd as usize);
        eprintln!("{} unexpectedly opened writable", PACKETS_PATH);
        return -1;
    }
    if fd != EXPECTED_EACCES {
        eprintln!(
            "{} writable open returned unexpected rc {}",
            PACKETS_PATH, fd
        );
        return -1;
    }
    0
}

fn show_single_reader_guard() -> isize {
    let second_fd = open(PACKETS_PATH, O_RDONLY);
    println!(
        "VFS_SECOND_READER path={} flags=O_RDONLY rc={} expected={}",
        PACKETS_PATH, second_fd, EXPECTED_EBUSY
    );
    if second_fd >= 0 {
        let _ = close(second_fd as usize);
        eprintln!("{} unexpectedly allowed a second reader", PACKETS_PATH);
        return -1;
    }
    if second_fd != EXPECTED_EBUSY {
        eprintln!(
            "{} second reader returned unexpected rc {}",
            PACKETS_PATH, second_fd
        );
        return -1;
    }
    0
}

fn show_write_attempt_denied(fd: usize) -> isize {
    let attempted = b"packetfs-write-attempt";
    let rc = write(fd, attempted);
    println!(
        "VFS_WRITE_ATTEMPT path={} fd={} bytes=\"{}\" rc={} expected={}",
        PACKETS_PATH,
        fd,
        AsciiPreview(attempted),
        rc,
        EXPECTED_EACCES
    );
    if rc != EXPECTED_EACCES {
        eprintln!(
            "{} write attempt returned unexpected rc {}",
            PACKETS_PATH, rc
        );
        return -1;
    }
    0
}

fn read_packet_stream(fd: usize, pcap_bytes: &mut Vec<u8>) -> isize {
    println!(
        "READ_BEGIN path={} fd={} target_records={}",
        PACKETS_PATH, fd, TARGET_RECORDS
    );

    let mut exported = 0usize;
    let mut read_index = 1usize;
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
            return -1;
        }

        let remaining = pcap.current_remaining();
        if remaining == 0 {
            eprintln!("packetdump internal PCAP parser state is empty before target records");
            return -1;
        }
        if exported + remaining > MAX_EXPORT_BYTES && !pcap.at_record_boundary() {
            eprintln!("packetdump next PCAP record segment exceeds max export bytes");
            return -1;
        }

        let phase = pcap.current_phase();
        let read_len = min(READ_BUF_SIZE, remaining);
        let n = read(fd, &mut buf[..read_len]);
        if n < 0 {
            eprintln!("{} read failed: {}", PACKETS_PATH, n);
            return -1;
        }
        if n == 0 {
            eprintln!(
                "{} returned EOF before a complete PCAP record was exported",
                PACKETS_PATH
            );
            return -1;
        }

        let n = n as usize;
        let chunk = &buf[..n];
        print_read_chunk(read_index, phase, read_len, chunk);
        exported += n;
        pcap_bytes.extend_from_slice(chunk);
        if let Err(message) = pcap.consume(chunk) {
            eprintln!("packetdump PCAP parse failed: {}", message);
            return -1;
        }
        read_index += 1;
    }

    println!(
        "READ_COMPLETE path={} chunks={} bytes={} records={}",
        PACKETS_PATH,
        read_index - 1,
        exported,
        pcap.records_complete()
    );
    0
}

fn print_read_chunk(index: usize, phase: PcapPhase, requested: usize, bytes: &[u8]) {
    match phase {
        PcapPhase::GlobalHeader => println!(
            "READ_CHUNK index={} phase=global_header requested={} got={} hex={} ascii=\"{}\"",
            index,
            requested,
            bytes.len(),
            HexBytes(bytes),
            AsciiPreview(bytes)
        ),
        PcapPhase::RecordHeader(record) => println!(
            "READ_CHUNK index={} phase=record_header record={} requested={} got={} hex={} ascii=\"{}\"",
            index,
            record,
            requested,
            bytes.len(),
            HexBytes(bytes),
            AsciiPreview(bytes)
        ),
        PcapPhase::RecordPayload(record) => println!(
            "READ_CHUNK index={} phase=record_payload record={} requested={} got={} hex={} ascii=\"{}\"",
            index,
            record,
            requested,
            bytes.len(),
            HexBytes(bytes),
            AsciiPreview(bytes)
        ),
    }
}

fn print_pcap_records(bytes: &[u8]) -> isize {
    if bytes.len() < PCAP_GLOBAL_HEADER_LEN {
        eprintln!("PCAP output is shorter than the global header");
        return -1;
    }
    if bytes[..4] != PCAP_MAGIC_LE {
        eprintln!("PCAP output has invalid little-endian magic");
        return -1;
    }

    let mut offset = PCAP_GLOBAL_HEADER_LEN;
    let mut index = 1usize;
    while offset < bytes.len() {
        if bytes.len() - offset < PCAP_RECORD_HEADER_LEN {
            eprintln!("PCAP record header is truncated");
            return -1;
        }

        let ts_sec = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let ts_usec = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let incl_len = u32::from_le_bytes([
            bytes[offset + 8],
            bytes[offset + 9],
            bytes[offset + 10],
            bytes[offset + 11],
        ]) as usize;
        let orig_len = u32::from_le_bytes([
            bytes[offset + 12],
            bytes[offset + 13],
            bytes[offset + 14],
            bytes[offset + 15],
        ]) as usize;
        offset += PCAP_RECORD_HEADER_LEN;

        if bytes.len() - offset < incl_len {
            eprintln!("PCAP record payload is truncated");
            return -1;
        }

        let frame = &bytes[offset..offset + incl_len];
        println!(
            "PCAP_RECORD index={} ts={}.{:06} incl_len={} orig_len={} dst={} src={} ethertype=0x{:04x} payload=\"{}\"",
            index,
            ts_sec,
            ts_usec,
            incl_len,
            orig_len,
            MacAddr(frame.get(0..6).unwrap_or(&[])),
            MacAddr(frame.get(6..12).unwrap_or(&[])),
            ethertype(frame),
            AsciiPreview(frame_payload(frame))
        );
        offset += incl_len;
        index += 1;
    }

    println!("PCAP_RECORDS total={}", index - 1);
    0
}

fn dump_stats() -> isize {
    let fd = open(STATS_PATH, O_RDONLY);
    println!("VFS_OPEN path={} flags=O_RDONLY rc={}", STATS_PATH, fd);
    if fd < 0 {
        eprintln!("{} open failed: {}", STATS_PATH, fd);
        return -1;
    }

    let mut buf = [0u8; STATS_BUF_SIZE];
    let mut used = 0usize;
    loop {
        if used == buf.len() {
            eprintln!("{} output exceeds {} bytes", STATS_PATH, STATS_BUF_SIZE);
            let _ = close(fd as usize);
            return -1;
        }
        let n = read(fd as usize, &mut buf[used..]);
        if n < 0 {
            eprintln!("{} read failed: {}", STATS_PATH, n);
            let _ = close(fd as usize);
            return -1;
        }
        if n == 0 {
            break;
        }
        used += n as usize;
    }

    println!("STATS_BEGIN path={} bytes={}", STATS_PATH, used);
    let _ = write(STDOUT, &buf[..used]);
    if used == 0 || buf[used - 1] != b'\n' {
        println!();
    }
    println!("STATS_END path={}", STATS_PATH);

    let rc = close(fd as usize);
    println!("VFS_CLOSE path={} fd={} rc={}", STATS_PATH, fd, rc);
    if rc < 0 {
        eprintln!("{} close failed: {}", STATS_PATH, rc);
        return -1;
    }
    0
}

#[cfg(not(target_arch = "riscv64"))]
fn inject_demo_rx_frames() {
    for index in 1..=TARGET_RECORDS {
        let frame = demo_frame(index);
        print_tx_frame(index, &frame);
        let result = kernel::net::inject_rx_frame_for_test(&frame);
        println!("RX_SUBMIT index={} result={:?}", index, result);
    }
}

#[cfg(target_arch = "riscv64")]
fn inject_demo_rx_frames() {}

#[cfg(not(target_arch = "riscv64"))]
fn demo_frame(index: usize) -> [u8; 60] {
    let mut frame = [0u8; 60];
    frame[..6].copy_from_slice(&[0xff; 6]);
    frame[6..12].copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, index as u8]);
    frame[12..14].copy_from_slice(&0x88b5u16.to_be_bytes());
    let payload: &[u8] = match index {
        1 => b"packetfs-demo-frame-1 mount-tree",
        2 => b"packetfs-demo-frame-2 read-loop",
        _ => b"packetfs-demo-frame-3 pcap-file",
    };
    frame[14..14 + payload.len()].copy_from_slice(payload);
    frame
}

fn print_tx_frame(index: usize, frame: &[u8]) {
    println!(
        "TX_FRAME index={} len={} dst={} src={} ethertype=0x{:04x} payload=\"{}\" hex={}",
        index,
        frame.len(),
        MacAddr(frame.get(0..6).unwrap_or(&[])),
        MacAddr(frame.get(6..12).unwrap_or(&[])),
        ethertype(frame),
        AsciiPreview(frame_payload(frame)),
        HexBytes(frame)
    );
}

#[derive(Clone, Copy)]
enum PcapPhase {
    GlobalHeader,
    RecordHeader(usize),
    RecordPayload(usize),
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

    fn current_phase(&self) -> PcapPhase {
        if self.global_len < PCAP_GLOBAL_HEADER_LEN {
            PcapPhase::GlobalHeader
        } else if self.record_header_len < PCAP_RECORD_HEADER_LEN {
            PcapPhase::RecordHeader(self.records_complete + 1)
        } else {
            PcapPhase::RecordPayload(self.records_complete + 1)
        }
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
            if self.global_len == PCAP_GLOBAL_HEADER_LEN && self.global_header[..4] != PCAP_MAGIC_LE
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

struct HexBytes<'a>(&'a [u8]);

impl fmt::Display for HexBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

struct AsciiPreview<'a>(&'a [u8]);

impl fmt::Display for AsciiPreview<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            let ch = if (0x20..=0x7e).contains(&byte) {
                byte as char
            } else {
                '.'
            };
            write!(f, "{}", ch)?;
        }
        Ok(())
    }
}

struct MacAddr<'a>(&'a [u8]);

impl fmt::Display for MacAddr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() != 6 {
            return f.write_str("??:??:??:??:??:??");
        }
        for (idx, byte) in self.0.iter().enumerate() {
            if idx > 0 {
                f.write_str(":")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

fn file_type_name(file_type: u16) -> &'static str {
    match file_type {
        1 => "directory",
        2 => "regular",
        3 => "pipe",
        4 => "char",
        _ => "unknown",
    }
}

fn ethertype(frame: &[u8]) -> u16 {
    if frame.len() < 14 {
        return 0;
    }
    u16::from_be_bytes([frame[12], frame[13]])
}

fn frame_payload(frame: &[u8]) -> &[u8] {
    if frame.len() <= 14 {
        return &[];
    }
    let payload = &frame[14..];
    let end = payload
        .iter()
        .rposition(|byte| *byte != 0)
        .map(|index| index + 1)
        .unwrap_or(0);
    &payload[..end]
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
    let mut line = [0u8; PCAP_HEX_BYTES_PER_LINE * 2];
    for chunk in bytes.chunks(PCAP_HEX_BYTES_PER_LINE) {
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
