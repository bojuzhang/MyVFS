use std::sync::Arc;

use super::error::{FsError, FsResult};
use super::mount::MountTable;
use super::vfs::DynInode;

pub struct Path {
    pub raw: String,
    pub components: Vec<String>,
    pub is_absolute: bool,
}

pub struct PathResolver {
    pub root: DynInode,
    pub cwd: DynInode,
    pub mount_table: Arc<MountTable>,
}

impl Path {
    pub fn parse(raw: &str) -> FsResult<Self> {
        if raw.is_empty() {
            return Err(FsError::Einval);
        }

        let is_absolute = raw.starts_with('/');
        let mut components = Vec::new();
        for component in raw.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    if is_absolute {
                        components.pop();
                    } else if components.last().is_some_and(|last| last != "..") {
                        components.pop();
                    } else {
                        components.push(component.to_string());
                    }
                }
                name => components.push(name.to_string()),
            }
        }

        Ok(Self {
            raw: raw.to_string(),
            components,
            is_absolute,
        })
    }
}

impl PathResolver {
    pub fn resolve(&self, path: &str) -> FsResult<DynInode> {
        let path = Path::parse(path)?;
        let mut current = if path.is_absolute {
            self.root.clone()
        } else {
            self.cwd.clone()
        };
        current = self.mount_table.follow_mount(&current);
        let mut ancestors = Vec::new();

        for component in path.components {
            if component == ".." {
                if let Some(parent) = ancestors.pop() {
                    current = parent;
                } else if !same_inode(&current, &self.root) {
                    current = current.lookup("..")?;
                    current = self.mount_table.follow_mount(&current);
                }
                continue;
            }

            ancestors.push(current.clone());
            current = current.lookup(&component)?;
            current = self.mount_table.follow_mount(&current);
        }

        Ok(current)
    }

    pub fn resolve_parent(&self, path: &str) -> FsResult<(DynInode, String)> {
        let path = Path::parse(path)?;
        let name = path.components.last().cloned().ok_or(FsError::Einval)?;
        let parent_path = if path.components.len() == 1 {
            if path.is_absolute {
                "/".to_string()
            } else {
                ".".to_string()
            }
        } else {
            let prefix = path.components[..path.components.len() - 1].join("/");
            if path.is_absolute {
                format!("/{prefix}")
            } else {
                prefix
            }
        };
        Ok((self.resolve(&parent_path)?, name))
    }
}

fn same_inode(left: &DynInode, right: &DynInode) -> bool {
    Arc::ptr_eq(left, right)
}
