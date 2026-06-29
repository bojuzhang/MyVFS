#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, mount, open, read, write, O_RDONLY, STDOUT};

const PACKETFS_TARGET: &str = "/mnt/packetfs";
const STATS_PATH: &str = "/mnt/packetfs/stats";
const MOUNT_OPTIONS: &str = "snaplen=2048,capacity=256";
const STATS_BUF_SIZE: usize = 2048;

const REQUIRED_FIELDS: [&[u8]; 9] = [
    b"captured_packets",
    b"captured_bytes",
    b"read_packets",
    b"read_bytes",
    b"queued_packets",
    b"dropped_full",
    b"dropped_inactive",
    b"truncated_packets",
    b"reader_active",
];

#[no_mangle]
fn main() -> i32 {
    let mount_rc = mount("packetfs", PACKETFS_TARGET, MOUNT_OPTIONS);
    if mount_rc == 0 {
        println!("packetfs mount success: {}", PACKETFS_TARGET);
    }

    let fd = open(STATS_PATH, O_RDONLY);
    if fd < 0 {
        eprintln!("{} open failed: {}", STATS_PATH, fd);
        if mount_rc < 0 {
            eprintln!("packetfs mount returned: {}", mount_rc);
        }
        return 1;
    }

    let mut buf = [0u8; STATS_BUF_SIZE];
    let mut used = 0usize;
    loop {
        if used == buf.len() {
            eprintln!("{} output exceeds {} bytes", STATS_PATH, STATS_BUF_SIZE);
            let _ = close(fd as usize);
            return 1;
        }
        let n = read(fd as usize, &mut buf[used..]);
        if n < 0 {
            eprintln!("{} read failed: {}", STATS_PATH, n);
            let _ = close(fd as usize);
            return 1;
        }
        if n == 0 {
            break;
        }
        used += n as usize;
    }

    let _ = write(STDOUT, &buf[..used]);
    let rc = close(fd as usize);
    if rc < 0 {
        eprintln!("{} close failed: {}", STATS_PATH, rc);
        return 1;
    }

    for field in REQUIRED_FIELDS {
        if !contains(&buf[..used], field) {
            eprintln!("packetstat missing field: {}", FieldName(field));
            return 1;
        }
    }
    0
}

struct FieldName(&'static [u8]);

impl core::fmt::Display for FieldName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in self.0 {
            f.write_str(match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => {
                    core::str::from_utf8(core::slice::from_ref(byte)).unwrap_or("?")
                }
                _ => "?",
            })?;
        }
        Ok(())
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.windows(needle.len()).any(|window| window == needle)
}
