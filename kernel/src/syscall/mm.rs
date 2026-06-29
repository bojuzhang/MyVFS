pub use crate::fs::UserBuffer;

use crate::fs::{FsError, FsResult};

const MAX_USER_STR: usize = 4096;

pub fn translated_str(_token: usize, ptr: *const u8) -> FsResult<String> {
    if ptr.is_null() {
        return Err(FsError::Einval);
    }

    let mut bytes = Vec::new();
    for index in 0..MAX_USER_STR {
        let byte = unsafe { *ptr.add(index) };
        if byte == 0 {
            return String::from_utf8(bytes).map_err(|_| FsError::Einval);
        }
        bytes.push(byte);
    }
    Err(FsError::Einval)
}

pub fn translated_optional_str(token: usize, ptr: *const u8) -> FsResult<String> {
    if ptr.is_null() {
        Ok(String::new())
    } else {
        translated_str(token, ptr)
    }
}

pub fn translated_byte_buffer<'a>(
    _token: usize,
    ptr: *mut u8,
    len: usize,
) -> FsResult<UserBuffer<'a>> {
    if len > 0 && ptr.is_null() {
        return Err(FsError::Einval);
    }
    let slice = if len == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    };
    Ok(UserBuffer::new(slice))
}

pub fn translated_const_byte_buffer(
    _token: usize,
    ptr: *const u8,
    len: usize,
) -> FsResult<Vec<u8>> {
    if len > 0 && ptr.is_null() {
        return Err(FsError::Einval);
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len).to_vec() })
}

pub fn user_buffer_from_bytes(bytes: &mut [u8]) -> UserBuffer<'_> {
    UserBuffer::new(bytes)
}

pub fn write_user_value<T: Copy>(_token: usize, ptr: *mut T, value: T) -> FsResult<()> {
    if ptr.is_null() {
        return Err(FsError::Einval);
    }
    unsafe {
        ptr.write(value);
    }
    Ok(())
}
