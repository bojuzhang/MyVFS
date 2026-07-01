use crate::fs::{self as vfs, DynFile, FileHandle, FileType, FsError, OpenFlags, Stat};
use crate::syscall::mm::{
    translated_byte_buffer, translated_const_byte_buffer, translated_optional_str, translated_str,
    user_buffer_from_bytes, write_user_value,
};
use crate::task::{current_task, current_user_token};

pub fn sys_mount(fs_name: *const u8, target: *const u8, options: *const u8) -> isize {
    let token = current_user_token();
    let result = (|| {
        let fs_name = translated_str(token, fs_name)?;
        let target = translated_str(token, target)?;
        let options = translated_optional_str(token, options)?;
        vfs::mount_fs(&fs_name, &target, &options)
    })();
    result.map(|_| 0).unwrap_or_else(|err| err.as_isize())
}

pub fn sys_umount(target: *const u8) -> isize {
    let token = current_user_token();
    let result = (|| {
        let target = translated_str(token, target)?;
        vfs::umount_fs(&target)
    })();
    result.map(|_| 0).unwrap_or_else(|err| err.as_isize())
}

pub fn sys_mkdir(path: *const u8) -> isize {
    let token = current_user_token();
    let result = (|| {
        let path = translated_str(token, path)?;
        vfs::mkdir_path(&path)
    })();
    result.map(|_| 0).unwrap_or_else(|err| err.as_isize())
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let token = current_user_token();
    let result = (|| {
        let path = translated_str(token, path)?;
        let flags = OpenFlags::from_bits(flags)?;
        let file = vfs::open_path(&path, flags)?;
        let task = current_task().ok_or(FsError::Eio)?;
        let mut fd_table = task.fd_table.lock().map_err(|_| FsError::Eio)?;
        fd_table.alloc(FileHandle {
            file,
            flags,
            offset: 0,
            debug_path: path,
        })
    })();
    result
        .map(|fd| fd as isize)
        .unwrap_or_else(|err| err.as_isize())
}

pub fn sys_close(fd: usize) -> isize {
    let result = (|| {
        let task = current_task().ok_or(FsError::Eio)?;
        let mut fd_table = task.fd_table.lock().map_err(|_| FsError::Eio)?;
        fd_table.close(fd)
    })();
    result.map(|_| 0).unwrap_or_else(|err| err.as_isize())
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let result = (|| {
        let (file, flags) = fd_file_and_flags(fd)?;
        if !flags.readable() || !file.readable() {
            return Err(FsError::Eacces);
        }
        let user_buf = translated_byte_buffer(token, buf, len)?;
        let read = file.read(user_buf)?;
        advance_fd_offset(fd, read)?;
        Ok(read)
    })();
    result
        .map(|n| n as isize)
        .unwrap_or_else(|err| err.as_isize())
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let result = (|| {
        let (file, flags) = fd_file_and_flags(fd)?;
        if !flags.writable() || !file.writable() {
            return Err(FsError::Eacces);
        }
        let mut bytes = translated_const_byte_buffer(token, buf, len)?;
        let user_buf = user_buffer_from_bytes(&mut bytes);
        let written = file.write(user_buf)?;
        advance_fd_offset(fd, written)?;
        Ok(written)
    })();
    result
        .map(|n| n as isize)
        .unwrap_or_else(|err| err.as_isize())
}

pub fn sys_stat(path: *const u8, stat_ptr: *mut Stat) -> isize {
    let token = current_user_token();
    let result = (|| {
        if stat_ptr.is_null() {
            return Err(FsError::Einval);
        }
        let path = translated_str(token, path)?;
        let stat = Stat::from(vfs::stat_path(&path)?);
        write_user_value(token, stat_ptr, stat)
    })();
    result.map(|_| 0).unwrap_or_else(|err| err.as_isize())
}

pub fn sys_getdents(fd: usize, buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let result = (|| {
        let (file, flags, debug_path, offset) = fd_dir_snapshot(fd)?;
        if !flags.readable() || !file.readable() {
            return Err(FsError::Eacces);
        }
        if file.stat()?.file_type != FileType::Directory {
            return Err(FsError::Enotdir);
        }

        let entries = vfs::read_dir_path(&debug_path)?;
        let encoded = encode_dir_entries(&entries);
        let copied = if offset >= encoded.len() {
            0
        } else {
            let mut user_buf = translated_byte_buffer(token, buf, len)?;
            user_buf.write_from_slice(&encoded[offset..])
        };
        advance_fd_offset(fd, copied)?;
        Ok(copied)
    })();
    result
        .map(|n| n as isize)
        .unwrap_or_else(|err| err.as_isize())
}

fn fd_file_and_flags(fd: usize) -> vfs::FsResult<(DynFile, OpenFlags)> {
    let task = current_task().ok_or(FsError::Eio)?;
    let fd_table = task.fd_table.lock().map_err(|_| FsError::Eio)?;
    let handle = fd_table.get(fd)?;
    Ok((handle.file.clone(), handle.flags))
}

fn fd_dir_snapshot(fd: usize) -> vfs::FsResult<(DynFile, OpenFlags, String, usize)> {
    let task = current_task().ok_or(FsError::Eio)?;
    let fd_table = task.fd_table.lock().map_err(|_| FsError::Eio)?;
    let handle = fd_table.get(fd)?;
    Ok((
        handle.file.clone(),
        handle.flags,
        handle.debug_path.clone(),
        handle.offset,
    ))
}

fn advance_fd_offset(fd: usize, copied: usize) -> vfs::FsResult<()> {
    let task = current_task().ok_or(FsError::Eio)?;
    let mut fd_table = task.fd_table.lock().map_err(|_| FsError::Eio)?;
    let handle = fd_table.get_mut(fd)?;
    handle.advance_offset(copied)?;
    Ok(())
}

fn encode_dir_entries(entries: &[vfs::DirEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        let name = entry.name.as_bytes();
        let Ok(name_len) = u16::try_from(name.len()) else {
            continue;
        };
        out.extend_from_slice(&entry.inode_id.to_le_bytes());
        out.extend_from_slice(&entry.file_type.as_u16().to_le_bytes());
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(name);
    }
    out
}
