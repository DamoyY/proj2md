use super::win32;
use core::{mem::size_of, ptr};
use std::{io, os::windows::ffi::OsStrExt as _, path::Path};
type Bool = i32;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DropFiles {
    p_files: u32,
    pt: Point,
    f_nc: Bool,
    f_wide: Bool,
}
pub(super) fn copy_file_to_clipboard(file_path: &Path) -> io::Result<()> {
    let memory = build_file_drop_handle(file_path)?;
    if let Err(err) = win32::set_clipboard_file_drop(memory) {
        return Err(free_memory_after_error(memory, err));
    }
    Ok(())
}
fn build_file_drop_handle(file_path: &Path) -> io::Result<win32::Hglobal> {
    let payload = build_file_drop_payload(file_path)?;
    let memory = win32::allocate_global_memory(payload.len())?;
    let buffer = match win32::lock_global_memory(memory) {
        Ok(pointer) => pointer,
        Err(err) => return Err(free_memory_after_error(memory, err)),
    };
    unsafe {
        ptr::copy_nonoverlapping(payload.as_ptr(), buffer, payload.len());
    }
    if let Err(err) = win32::unlock_global_memory(memory) {
        return Err(free_memory_after_error(memory, err));
    }
    Ok(memory)
}
fn build_file_drop_payload(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut wide_path: Vec<u16> = file_path.as_os_str().encode_wide().collect();
    wide_path.push(0);
    wide_path.push(0);
    let header_size = size_of::<DropFiles>();
    let header_offset = u32::try_from(header_size).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("DROPFILES 头部大小超出 Win32 范围: {err}"),
        )
    })?;
    let path_bytes = wide_path
        .len()
        .checked_mul(size_of::<u16>())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "计算路径缓冲区大小时发生溢出")
        })?;
    let total_size = header_size.checked_add(path_bytes).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "计算剪贴板数据大小时发生溢出")
    })?;
    let mut payload = vec![0_u8; total_size];
    let header = DropFiles {
        p_files: header_offset,
        pt: Point { x: 0, y: 0 },
        f_nc: 0_i32,
        f_wide: 1_i32,
    };
    unsafe {
        ptr::copy_nonoverlapping(
            ptr::from_ref(&header).cast::<u8>(),
            payload.as_mut_ptr(),
            header_size,
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(
            wide_path.as_ptr().cast::<u8>(),
            payload.as_mut_ptr().wrapping_add(header_size),
            path_bytes,
        );
    }
    Ok(payload)
}
fn free_memory_after_error(memory: win32::Hglobal, err: io::Error) -> io::Error {
    if let Err(free_err) = win32::free_global_memory(memory) {
        io::Error::other(format!("{err}; 释放全局内存失败: {free_err}"))
    } else {
        err
    }
}
#[cfg(test)]
#[path = "windows/tests.rs"]
mod tests;
