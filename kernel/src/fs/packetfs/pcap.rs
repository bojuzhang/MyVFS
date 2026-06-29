use super::ring::PacketRecord;

pub const PCAP_MAGIC: u32 = 0xa1b2c3d4;
pub const PCAP_VERSION_MAJOR: u16 = 2;
pub const PCAP_VERSION_MINOR: u16 = 4;
pub const LINKTYPE_ETHERNET: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcapGlobalHeader {
    pub magic_number: u32,
    pub version_major: u16,
    pub version_minor: u16,
    pub thiszone: i32,
    pub sigfigs: u32,
    pub snaplen: u32,
    pub network: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcapRecordHeader {
    pub ts_sec: u32,
    pub ts_usec: u32,
    pub incl_len: u32,
    pub orig_len: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PcapStreamState {
    pub global_header_done: bool,
}

pub fn encode_global_header(snaplen: usize) -> [u8; 24] {
    let header = PcapGlobalHeader {
        magic_number: PCAP_MAGIC,
        version_major: PCAP_VERSION_MAJOR,
        version_minor: PCAP_VERSION_MINOR,
        thiszone: 0,
        sigfigs: 0,
        snaplen: snaplen as u32,
        network: LINKTYPE_ETHERNET,
    };

    let mut out = [0u8; 24];
    out[0..4].copy_from_slice(&header.magic_number.to_le_bytes());
    out[4..6].copy_from_slice(&header.version_major.to_le_bytes());
    out[6..8].copy_from_slice(&header.version_minor.to_le_bytes());
    out[8..12].copy_from_slice(&header.thiszone.to_le_bytes());
    out[12..16].copy_from_slice(&header.sigfigs.to_le_bytes());
    out[16..20].copy_from_slice(&header.snaplen.to_le_bytes());
    out[20..24].copy_from_slice(&header.network.to_le_bytes());
    out
}

pub fn encode_record_header(record: &PacketRecord) -> [u8; 16] {
    let header = PcapRecordHeader {
        ts_sec: (record.timestamp_us / 1_000_000) as u32,
        ts_usec: (record.timestamp_us % 1_000_000) as u32,
        incl_len: record.cap_len as u32,
        orig_len: record.wire_len as u32,
    };

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&header.ts_sec.to_le_bytes());
    out[4..8].copy_from_slice(&header.ts_usec.to_le_bytes());
    out[8..12].copy_from_slice(&header.incl_len.to_le_bytes());
    out[12..16].copy_from_slice(&header.orig_len.to_le_bytes());
    out
}

pub fn encode_record(record: &PacketRecord) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + record.cap_len);
    out.extend_from_slice(&encode_record_header(record));
    out.extend_from_slice(&record.data);
    out
}
