use std::io::{Read, Write};

use super::error::{FsError, FsResult};
use super::stat::{FileType, Metadata};
use super::vfs::{File, SeekFrom, UserBuffer};

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, mut buf: UserBuffer<'_>) -> FsResult<usize> {
        let mut total = 0;
        let mut stdin = std::io::stdin();
        for slice in &mut buf.buffers {
            if slice.is_empty() {
                continue;
            }
            let read = stdin.read(slice).map_err(|_| FsError::Eio)?;
            total += read;
            if read < slice.len() {
                break;
            }
        }
        Ok(total)
    }

    fn write(&self, _buf: UserBuffer<'_>) -> FsResult<usize> {
        Err(FsError::Eacces)
    }

    fn stat(&self) -> FsResult<Metadata> {
        Ok(Metadata {
            inode_id: 0,
            file_type: FileType::CharDevice,
            mode: 0o444,
            size: 0,
            nlink: 1,
        })
    }

    fn close(&self) -> FsResult<()> {
        Ok(())
    }

    fn seek(&self, _pos: SeekFrom) -> FsResult<usize> {
        Err(FsError::Espipe)
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, _buf: UserBuffer<'_>) -> FsResult<usize> {
        Err(FsError::Eacces)
    }

    fn write(&self, buf: UserBuffer<'_>) -> FsResult<usize> {
        let mut stdout = std::io::stdout();
        let data = buf.to_vec();
        stdout.write_all(&data).map_err(|_| FsError::Eio)?;
        stdout.flush().map_err(|_| FsError::Eio)?;
        Ok(data.len())
    }

    fn stat(&self) -> FsResult<Metadata> {
        Ok(Metadata {
            inode_id: 1,
            file_type: FileType::CharDevice,
            mode: 0o222,
            size: 0,
            nlink: 1,
        })
    }

    fn close(&self) -> FsResult<()> {
        Ok(())
    }

    fn seek(&self, _pos: SeekFrom) -> FsResult<usize> {
        Err(FsError::Espipe)
    }
}
