use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use ignore::WalkBuilder;
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, BufWriter, Write as _},
    path::{Path, PathBuf},
};
const OUTPUT_FILENAME: &str = "project.md";
const EXTRA_EXCLUDED_FILES: [&str; 2] = ["LICENSE", "README.md"];
#[derive(Debug)]
struct FileEntry {
    absolute_path: PathBuf,
    relative_path: String,
    code_block_language: String,
}
#[cfg(target_os = "windows")]
mod windows_clipboard {
    use core::{
        ffi::c_void,
        mem::size_of,
        ptr::{self, null_mut},
    };
    use std::{io, os::windows::ffi::OsStrExt as _, path::Path};
    const CF_HDROP: u32 = 15;
    const GMEM_MOVEABLE: u32 = 0x0002;
    type Bool = i32;
    type Handle = *mut c_void;
    type Hglobal = *mut c_void;
    type Hwnd = *mut c_void;
    type Uint = u32;
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
    fn last_os_error(action: &str) -> io::Error {
        // SAFETY: GetLastError 不接收参数，也不会解引用任何 Rust 指针。
        let code = unsafe { GetLastError() };
        if code == 0 {
            return io::Error::other(action.to_owned());
        }
        win32_error(action, code)
    }
    fn allocate_global_memory(size: usize) -> io::Result<Hglobal> {
        // SAFETY: GlobalAlloc 只读取传入的大小和值，不会触碰 Rust 管理的内存。
        let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, size) };
        if memory.is_null() {
            return Err(last_os_error("分配全局内存失败"));
        }
        Ok(memory)
    }
    fn free_global_memory(memory: Hglobal) -> io::Result<()> {
        // SAFETY: 句柄由 GlobalAlloc 返回，调用方保证这里传入的仍是待释放的全局内存句柄。
        let result = unsafe { GlobalFree(memory) };
        if result.is_null() {
            return Ok(());
        }
        Err(last_os_error("释放全局内存失败"))
    }
    fn lock_global_memory(memory: Hglobal) -> io::Result<*mut u8> {
        // SAFETY: 句柄由 GlobalAlloc 返回，GlobalLock 只返回对应内存块的基地址。
        let pointer = unsafe { GlobalLock(memory) }.cast::<u8>();
        if pointer.is_null() {
            return Err(last_os_error("锁定全局内存失败"));
        }
        Ok(pointer)
    }
    fn unlock_global_memory(memory: Hglobal) -> io::Result<()> {
        // SAFETY: 句柄已成功传给 GlobalLock，这里按 Win32 约定对同一块内存解锁。
        if unsafe { GlobalUnlock(memory) } == 0_i32 {
            // SAFETY: GetLastError 不接收参数，也不会解引用任何 Rust 指针。
            let code = unsafe { GetLastError() };
            if code != 0 {
                return Err(win32_error("解锁全局内存失败", code));
            }
        }
        Ok(())
    }
    fn build_file_drop_handle(file_path: &Path) -> io::Result<Hglobal> {
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
        let memory = allocate_global_memory(total_size)?;
        let buffer = match lock_global_memory(memory) {
            Ok(pointer) => pointer,
            Err(err) => {
                if let Err(free_err) = free_global_memory(memory) {
                    return Err(io::Error::other(format!(
                        "{err}; 释放全局内存失败: {free_err}"
                    )));
                }
                return Err(err);
            }
        };
        let header = DropFiles {
            p_files: header_offset,
            pt: Point { x: 0, y: 0 },
            f_nc: 0_i32,
            f_wide: 1_i32,
        };
        // SAFETY: buffer 指向一块至少 total_size 字节的可写全局内存，这里只复制固定大小的头部字节。
        unsafe {
            ptr::copy_nonoverlapping(ptr::from_ref(&header).cast::<u8>(), buffer, header_size);
        }
        let path_buffer = buffer.wrapping_add(header_size);
        // SAFETY: path_buffer 指向头部之后的连续可写空间，长度至少为 path_bytes 字节。
        unsafe {
            ptr::copy_nonoverlapping(wide_path.as_ptr().cast::<u8>(), path_buffer, path_bytes);
        }
        if let Err(err) = unlock_global_memory(memory) {
            if let Err(free_err) = free_global_memory(memory) {
                return Err(io::Error::other(format!(
                    "{err}; 释放全局内存失败: {free_err}"
                )));
            }
            return Err(err);
        }
        Ok(memory)
    }
    fn set_clipboard_file_drop(memory: Hglobal) -> io::Result<()> {
        // SAFETY: 传入空窗口句柄表示当前任务线程访问系统剪贴板，不涉及 Rust 引用失效。
        if unsafe { OpenClipboard(null_mut()) } == 0_i32 {
            return Err(last_os_error("打开剪贴板失败"));
        }
        let operation_result = {
            // SAFETY: 剪贴板已成功打开，按 Win32 约定可以直接清空其内容。
            let empty_result = unsafe { EmptyClipboard() };
            if empty_result == 0_i32 {
                Err(last_os_error("清空剪贴板失败"))
            } else {
                // SAFETY: memory 指向一块 GMEM_MOVEABLE 全局内存，格式与 CF_HDROP 要求一致。
                let set_result = unsafe { SetClipboardData(CF_HDROP, memory.cast()) };
                if set_result.is_null() {
                    Err(last_os_error("写入剪贴板失败"))
                } else {
                    Ok(())
                }
            }
        };
        let close_result = {
            // SAFETY: 只有当前函数打开了剪贴板，因此这里必须在返回前关闭。
            let close_status = unsafe { CloseClipboard() };
            if close_status == 0_i32 {
                Err(last_os_error("关闭剪贴板失败"))
            } else {
                Ok(())
            }
        };
        match (operation_result, close_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(operation_error), Ok(())) => Err(operation_error),
            (Ok(()), Err(close_error)) => Err(close_error),
            (Err(operation_error), Err(close_error)) => Err(io::Error::other(format!(
                "{operation_error}; {close_error}"
            ))),
        }
    }
    pub(crate) fn copy_file_to_clipboard(file_path: &Path) -> io::Result<()> {
        let memory = build_file_drop_handle(file_path)?;
        if let Err(err) = set_clipboard_file_drop(memory) {
            if let Err(free_err) = free_global_memory(memory) {
                return Err(io::Error::other(format!(
                    "{err}; 释放全局内存失败: {free_err}"
                )));
            }
            return Err(err);
        }
        Ok(())
    }
}
fn build_walk(root_path: &Path) -> ignore::Walk {
    let mut builder = WalkBuilder::new(root_path);
    builder.require_git(false);
    builder.build()
}
fn os_str_to_utf8<'os>(
    os_value: Option<&'os OsStr>,
    display_path: &Path,
    subject: &str,
) -> Result<&'os str, io::Error> {
    let resolved_value = os_value.ok_or_else(|| {
        io::Error::other(format!("无法获取{subject}: {}", display_path.display()))
    })?;
    resolved_value.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{subject}包含无效 UTF-8: {}", display_path.display()),
        )
    })
}
fn path_to_utf8<'path>(path: &'path Path, subject: &str) -> Result<&'path str, io::Error> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{subject}包含无效 UTF-8: {}", path.display()),
        )
    })
}
fn root_name(root_path: &Path) -> Result<&str, io::Error> {
    root_path.file_name().map_or_else(
        || path_to_utf8(root_path, "根路径"),
        |name| os_str_to_utf8(Some(name), root_path, "根目录名"),
    )
}
fn is_excluded_file(file_name: &str) -> bool {
    file_name == OUTPUT_FILENAME || EXTRA_EXCLUDED_FILES.contains(&file_name)
}
fn is_binary(bytes: &[u8]) -> Result<bool, io::Error> {
    if bytes.is_empty() {
        return Ok(false);
    }
    let mut total = 0_usize;
    let mut control = 0_usize;
    for &byte in bytes.iter().take(8192) {
        total = total
            .checked_add(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "统计字节数时发生溢出"))?;
        if byte == 0 {
            return Ok(true);
        }
        if byte < 0x09 || (byte > 0x0D && byte < 0x20) {
            control = control.checked_add(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "统计控制字符时发生溢出")
            })?;
        }
    }
    let control_scaled = control
        .checked_mul(100)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "计算控制字符比例时发生溢出"))?;
    let total_scaled = total
        .checked_mul(30)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "计算字节总数比例时发生溢出"))?;
    Ok(control_scaled > total_scaled)
}
fn read_file_content(path: &Path) -> Result<String, Box<dyn core::error::Error>> {
    let bytes = fs::read(path)
        .map_err(|err| io::Error::new(err.kind(), format!("读取文件失败: {}", path.display())))?;
    if is_binary(&bytes)? {
        return Ok("(二进制文件)".to_owned());
    }
    if let Some((encoding, bom_len)) = Encoding::for_bom(&bytes) {
        let bytes_no_bom = bytes.get(bom_len..).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BOM 长度异常: {}", path.display()),
            )
        })?;
        let (text, _, had_errors) = encoding.decode(bytes_no_bom);
        if !had_errors {
            return Ok(text.into_owned());
        }
    }
    if let Ok(text) = core::str::from_utf8(&bytes) {
        return Ok(text.to_owned());
    }
    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let (text, _, had_errors) = encoding.decode(&bytes);
    if !had_errors {
        return Ok(text.into_owned());
    }
    Ok("(解码失败)".to_owned())
}
fn collect_files_and_write_directory_tree<W: io::Write>(
    root_path: &Path,
    writer: &mut W,
) -> Result<Vec<FileEntry>, Box<dyn core::error::Error>> {
    writer.write_all("## 1. 目录结构\n\n".as_bytes())?;
    writeln!(writer, "{}/", root_name(root_path)?)?;
    let mut files = Vec::new();
    for entry_result in build_walk(root_path) {
        let entry = entry_result.map_err(|err| io::Error::other(format!("遍历目录失败: {err}")))?;
        let path = entry.path();
        let rel_path = path.strip_prefix(root_path).map_err(|err| {
            io::Error::other(format!("无法计算相对路径: {}: {err}", path.display()))
        })?;
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let depth = rel_path.components().count();
        let indent = "    ".repeat(depth);
        let file_type = entry
            .file_type()
            .ok_or_else(|| io::Error::other(format!("无法获取文件类型: {}", path.display())))?;
        if file_type.is_dir() {
            let dir_name = os_str_to_utf8(path.file_name(), path, "目录名")?;
            writeln!(writer, "{indent}{dir_name}/")?;
            continue;
        }
        let file_name = os_str_to_utf8(path.file_name(), path, "文件名")?;
        if is_excluded_file(file_name) {
            continue;
        }
        writeln!(writer, "{indent}{file_name}")?;
        let relative_path = path_to_utf8(rel_path, "相对路径")?.to_owned();
        let code_block_language = match path.extension() {
            Some(extension) => os_str_to_utf8(Some(extension), path, "文件扩展名")?.to_owned(),
            None => String::new(),
        };
        files.push(FileEntry {
            absolute_path: path.to_path_buf(),
            relative_path,
            code_block_language,
        });
    }
    Ok(files)
}
fn write_file_contents<W: io::Write>(
    files: &[FileEntry],
    writer: &mut W,
) -> Result<(), Box<dyn core::error::Error>> {
    writer.write_all("\n## 2. 文件内容\n\n".as_bytes())?;
    for file in files {
        writeln!(writer, "### {}", file.relative_path)?;
        writeln!(writer, "```{}", file.code_block_language)?;
        let file_content = read_file_content(&file.absolute_path)?;
        writer.write_all(file_content.as_bytes())?;
        if !file_content.ends_with('\n') {
            writer.write_all(b"\n")?;
        }
        writer.write_all(b"```\n\n")?;
    }
    Ok(())
}
fn get_input_path() -> Result<String, io::Error> {
    env::args().nth(1).map_or_else(
        || {
            let cwd = env::current_dir()?;
            Ok(cwd.to_string_lossy().to_string())
        },
        Ok,
    )
}
fn create_output_writer() -> Result<(PathBuf, BufWriter<fs::File>), Box<dyn core::error::Error>> {
    let temp_dir = env::temp_dir().join("proj2md");
    fs::create_dir_all(&temp_dir).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("创建临时目录失败: {}: {err}", temp_dir.display()),
        )
    })?;
    let output_path = temp_dir.join(OUTPUT_FILENAME);
    let file = fs::File::create(&output_path).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("创建输出文件失败: {}: {err}", output_path.display()),
        )
    })?;
    Ok((output_path, BufWriter::new(file)))
}
fn write_output_file(root_path: &Path) -> Result<PathBuf, Box<dyn core::error::Error>> {
    let (output_path, mut writer) = create_output_writer()?;
    let files = collect_files_and_write_directory_tree(root_path, &mut writer)?;
    write_file_contents(&files, &mut writer)?;
    writer.flush()?;
    Ok(output_path)
}
fn copy_file_to_clipboard(file_path: &Path) -> Result<(), Box<dyn core::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        windows_clipboard::copy_file_to_clipboard(file_path)?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = file_path;
        Err(io::Error::new(io::ErrorKind::Unsupported, "当前平台暂不支持复制到剪贴板").into())
    }
}
fn run() -> Result<(), Box<dyn core::error::Error>> {
    let path_str = get_input_path()?;
    let root_path = Path::new(&path_str);
    if !root_path.exists() {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, format!("路径不存在: {path_str}")).into(),
        );
    }
    if !root_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("路径不是目录: {path_str}"),
        )
        .into());
    }
    println!("正在生成文档...");
    let output_path = write_output_file(root_path)?;
    copy_file_to_clipboard(&output_path)?;
    println!("文档文件已复制到剪贴板: {}", output_path.display());
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("错误: {err}");
    }
}
