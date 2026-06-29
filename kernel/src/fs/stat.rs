#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType {
    Directory,
    Regular,
    Pipe,
    CharDevice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub inode_id: u64,
    pub file_type: FileType,
    pub mode: u16,
    pub size: usize,
    pub nlink: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Stat {
    pub inode_id: u64,
    pub mode: u16,
    pub size: usize,
    pub file_type: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub inode_id: u64,
    pub file_type: FileType,
}

impl FileType {
    pub fn as_u16(self) -> u16 {
        match self {
            Self::Directory => 1,
            Self::Regular => 2,
            Self::Pipe => 3,
            Self::CharDevice => 4,
        }
    }
}

impl From<Metadata> for Stat {
    fn from(metadata: Metadata) -> Self {
        Self {
            inode_id: metadata.inode_id,
            mode: metadata.mode,
            size: metadata.size,
            file_type: metadata.file_type.as_u16(),
        }
    }
}
