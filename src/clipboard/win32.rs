use core::{ffi::c_void, ptr::null_mut};
use std::io;
const CF_HDROP: u32 = 15;
const GMEM_MOVEABLE: u32 = 0x0002;
type Bool = i32;
type Handle = *mut c_void;
pub(super) type Hglobal = *mut c_void;
type Hwnd = *mut c_void;
type Uint = u32;
#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(new_owner: Hwnd) -> Bool;
    fn EmptyClipboard() -> Bool;
    fn SetClipboardData(format: Uint, memory: Handle) -> Handle;
    fn CloseClipboard() -> Bool;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: Uint, bytes: usize) -> Hglobal;
    fn GlobalLock(memory: Hglobal) -> *mut c_void;
    fn GlobalUnlock(memory: Hglobal) -> Bool;
    fn GlobalFree(memory: Hglobal) -> Hglobal;
    fn GetLastError() -> u32;
}
pub(super) fn allocate_global_memory(size: usize) -> io::Result<Hglobal> {
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, size) };
    if memory.is_null() {
        Err(last_os_error("分配全局内存失败"))
    } else {
        Ok(memory)
    }
}
pub(super) fn lock_global_memory(memory: Hglobal) -> io::Result<*mut u8> {
    let pointer = unsafe { GlobalLock(memory) }.cast::<u8>();
    if pointer.is_null() {
        Err(last_os_error("锁定全局内存失败"))
    } else {
        Ok(pointer)
    }
}
pub(super) fn unlock_global_memory(memory: Hglobal) -> io::Result<()> {
    if unsafe { GlobalUnlock(memory) } == 0_i32 {
        let code = unsafe { GetLastError() };
        if code != 0 {
            return Err(win32_error("解锁全局内存失败", code));
        }
    }
    Ok(())
}
pub(super) fn free_global_memory(memory: Hglobal) -> io::Result<()> {
    let result = unsafe { GlobalFree(memory) };
    if result.is_null() {
        Ok(())
    } else {
        Err(last_os_error("释放全局内存失败"))
    }
}
pub(super) fn set_clipboard_file_drop(memory: Hglobal) -> io::Result<()> {
    if unsafe { OpenClipboard(null_mut()) } == 0_i32 {
        return Err(last_os_error("打开剪贴板失败"));
    }
    let operation_result = empty_clipboard_and_set_file(memory);
    let close_result = close_clipboard();
    merge_clipboard_results(operation_result, close_result)
}
fn empty_clipboard_and_set_file(memory: Hglobal) -> io::Result<()> {
    if unsafe { EmptyClipboard() } == 0_i32 {
        return Err(last_os_error("清空剪贴板失败"));
    }
    if unsafe { SetClipboardData(CF_HDROP, memory.cast()) }.is_null() {
        Err(last_os_error("写入剪贴板失败"))
    } else {
        Ok(())
    }
}
fn close_clipboard() -> io::Result<()> {
    if unsafe { CloseClipboard() } == 0_i32 {
        Err(last_os_error("关闭剪贴板失败"))
    } else {
        Ok(())
    }
}
fn merge_clipboard_results(
    operation_result: io::Result<()>,
    close_result: io::Result<()>,
) -> io::Result<()> {
    match (operation_result, close_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(operation_error), Ok(())) => Err(operation_error),
        (Ok(()), Err(close_error)) => Err(close_error),
        (Err(operation_error), Err(close_error)) => Err(io::Error::other(format!(
            "{operation_error}; {close_error}"
        ))),
    }
}
fn last_os_error(action: &str) -> io::Error {
    let code = unsafe { GetLastError() };
    if code == 0 {
        io::Error::other(action.to_owned())
    } else {
        win32_error(action, code)
    }
}
fn win32_error(action: &str, code: u32) -> io::Error {
    i32::try_from(code).map_or_else(
        |_| io::Error::other(format!("{action}: Win32 错误码 {code}")),
        |raw_code| {
            io::Error::other(format!(
                "{action}: {}",
                io::Error::from_raw_os_error(raw_code)
            ))
        },
    )
}
