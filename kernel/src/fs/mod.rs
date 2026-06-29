pub mod error;
pub mod fd;
pub mod mount;
pub mod packetfs;
pub mod path;
pub mod ramfs;
pub mod stat;
pub mod stdio;
pub mod vfs;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub use error::{FsError, FsResult};
pub use fd::{FdTable, FileHandle, OpenFlags};
pub use stat::{DirEntry, FileType, Metadata, Stat};
pub use vfs::{DynFile, DynInode, File, FileSystem, Inode, SeekFrom, UserBuffer};

use crate::sync::Mutex;

use mount::MountTable;
use path::PathResolver;
use ramfs::RamFs;
use vfs::DynFileSystem;

static ROOT_FS: OnceLock<Arc<RamFs>> = OnceLock::new();
static MOUNT_TABLE: OnceLock<Arc<MountTable>> = OnceLock::new();
static FS_REGISTRY: OnceLock<Mutex<HashMap<&'static str, DynFileSystem>>> = OnceLock::new();

pub fn init() -> FsResult<()> {
    if ROOT_FS.get().is_some() || MOUNT_TABLE.get().is_some() {
        return Err(FsError::Ebusy);
    }

    let root_fs = Arc::new(RamFs::new());
    let mnt = root_fs.create_dir(&root_fs.root, "mnt")?;
    root_fs.create_dir(&mnt, "packetfs")?;

    let mount_table = Arc::new(MountTable::new(root_fs.root_inode()));
    ROOT_FS.set(root_fs).map_err(|_| FsError::Ebusy)?;
    MOUNT_TABLE.set(mount_table).map_err(|_| FsError::Ebusy)?;

    let packetfs = packetfs::make_packetfs(packetfs::PacketFsConfig::default())?;
    register_filesystem(packetfs)?;
    Ok(())
}

pub fn register_filesystem(fs: DynFileSystem) -> FsResult<()> {
    let mut registry = registry().lock().map_err(|_| FsError::Eio)?;
    if registry.contains_key(fs.name()) {
        return Err(FsError::Ebusy);
    }
    registry.insert(fs.name(), fs);
    Ok(())
}

pub fn open_path(path: &str, flags: OpenFlags) -> FsResult<DynFile> {
    let flags = flags.validate()?;
    let inode = resolver()?.resolve(path)?;
    let metadata = inode.metadata()?;
    if flags.contains(OpenFlags::DIRECTORY) && metadata.file_type != FileType::Directory {
        return Err(FsError::Enotdir);
    }
    inode.open(flags)
}

pub fn mount_fs(fs_name: &str, target: &str, options: &str) -> FsResult<()> {
    let fs = {
        let registry = registry().lock().map_err(|_| FsError::Eio)?;
        registry.get(fs_name).cloned().ok_or(FsError::Enodev)?
    };
    mount_table()?.mount(fs, target, options)
}

pub fn umount_fs(target: &str) -> FsResult<()> {
    mount_table()?.umount(target)
}

pub fn stat_path(path: &str) -> FsResult<Metadata> {
    resolver()?.resolve(path)?.metadata()
}

pub fn read_dir_path(path: &str) -> FsResult<Vec<DirEntry>> {
    resolver()?.resolve(path)?.readdir()
}

pub fn stdin() -> DynFile {
    Arc::new(stdio::Stdin)
}

pub fn stdout() -> DynFile {
    Arc::new(stdio::Stdout)
}

pub fn root_inode() -> FsResult<DynInode> {
    Ok(root_fs()?.root_inode())
}

pub fn mount_table() -> FsResult<Arc<MountTable>> {
    MOUNT_TABLE.get().cloned().ok_or(FsError::Eio)
}

fn root_fs() -> FsResult<Arc<RamFs>> {
    ROOT_FS.get().cloned().ok_or(FsError::Eio)
}

fn resolver() -> FsResult<PathResolver> {
    let root = root_inode()?;
    Ok(PathResolver {
        root: root.clone(),
        cwd: root,
        mount_table: mount_table()?,
    })
}

fn registry() -> &'static Mutex<HashMap<&'static str, DynFileSystem>> {
    FS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}
