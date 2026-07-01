use crate::sync::Mutex;

use super::error::{FsError, FsResult};
use super::stat::FileType;
use super::vfs::{DynFileSystem, DynInode};

pub struct MountEntry {
    pub target_path: String,
    pub mountpoint_inode: DynInode,
    pub root_inode: DynInode,
    pub fs: DynFileSystem,
}

pub struct MountTable {
    pub root: DynInode,
    pub entries: Mutex<Vec<MountEntry>>,
}

impl MountTable {
    pub fn new(root: DynInode) -> Self {
        Self {
            root,
            entries: Mutex::new(Vec::new()),
        }
    }

    pub fn mount(&self, fs: DynFileSystem, target: &str, options: &str) -> FsResult<()> {
        let target_path = normalize_target(target)?;

        if target_path == "/" {
            return Err(FsError::Ebusy);
        }

        let mut entries = self.entries.lock().map_err(|_| FsError::Eio)?;
        reject_duplicate_or_recursive_mount(&entries, &target_path)?;

        let mountpoint = self.resolve_mountpoint(&target_path, &entries)?;
        if mountpoint.metadata()?.file_type != FileType::Directory {
            return Err(FsError::Enotdir);
        }
        if entries
            .iter()
            .any(|entry| same_inode(&entry.mountpoint_inode, &mountpoint))
        {
            return Err(FsError::Ebusy);
        }

        let root_inode = fs.mount(options)?;
        entries.push(MountEntry {
            target_path,
            mountpoint_inode: mountpoint,
            root_inode,
            fs,
        });
        Ok(())
    }

    pub fn umount(&self, target: &str) -> FsResult<()> {
        let target = normalize_target(target)?;
        let mut entries = self.entries.lock().map_err(|_| FsError::Eio)?;
        let index = entries
            .iter()
            .position(|entry| entry.target_path == target)
            .ok_or(FsError::Enodev)?;
        entries[index].fs.umount()?;
        entries.remove(index);
        Ok(())
    }

    pub fn follow_mount(&self, inode: &DynInode) -> DynInode {
        let Ok(entries) = self.entries.lock() else {
            return inode.clone();
        };
        entries
            .iter()
            .find(|entry| same_inode(&entry.mountpoint_inode, inode))
            .map(|entry| entry.root_inode.clone())
            .unwrap_or_else(|| inode.clone())
    }

    pub fn is_mountpoint(&self, inode: &DynInode) -> bool {
        let Ok(entries) = self.entries.lock() else {
            return false;
        };
        entries
            .iter()
            .any(|entry| same_inode(&entry.mountpoint_inode, inode))
    }

    fn resolve_mountpoint(&self, target: &str, entries: &[MountEntry]) -> FsResult<DynInode> {
        let components = parse_components(target)?;
        let mut current = self.root.clone();
        for (index, component) in components.iter().enumerate() {
            current = current.lookup(&component)?;
            if index + 1 != components.len() {
                current = follow_mount_in_entries(entries, &current);
            }
        }
        Ok(current)
    }
}

fn same_inode(left: &DynInode, right: &DynInode) -> bool {
    std::sync::Arc::ptr_eq(left, right)
}

fn follow_mount_in_entries(entries: &[MountEntry], inode: &DynInode) -> DynInode {
    entries
        .iter()
        .find(|entry| same_inode(&entry.mountpoint_inode, inode))
        .map(|entry| entry.root_inode.clone())
        .unwrap_or_else(|| inode.clone())
}

fn reject_duplicate_or_recursive_mount(entries: &[MountEntry], target: &str) -> FsResult<()> {
    if entries.iter().any(|entry| {
        entry.target_path == target
            || is_path_below(target, &entry.target_path)
            || is_path_below(&entry.target_path, target)
    }) {
        return Err(FsError::Ebusy);
    }
    Ok(())
}

fn is_path_below(path: &str, ancestor: &str) -> bool {
    ancestor != "/"
        && path.len() > ancestor.len()
        && path.starts_with(ancestor)
        && path.as_bytes().get(ancestor.len()) == Some(&b'/')
}

fn normalize_target(target: &str) -> FsResult<String> {
    if target.is_empty() || !target.starts_with('/') {
        return Err(FsError::Einval);
    }
    let components = parse_components(target)?;
    if components.is_empty() {
        return Ok("/".to_string());
    }
    Ok(format!("/{}", components.join("/")))
}

fn parse_components(path: &str) -> FsResult<Vec<String>> {
    if path.is_empty() {
        return Err(FsError::Einval);
    }

    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            name => components.push(name.to_string()),
        }
    }
    Ok(components)
}
