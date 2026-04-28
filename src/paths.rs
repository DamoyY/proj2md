use std::{ffi::OsStr, io, path::Path};
pub(crate) fn validate_root_path(root_path: &Path) -> io::Result<()> {
    if !root_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("路径不存在: {}", root_path.display()),
        ));
    }
    if !root_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("路径不是目录: {}", root_path.display()),
        ));
    }
    Ok(())
}
pub(crate) fn os_str_to_utf8<'os>(
    os_value: Option<&'os OsStr>,
    display_path: &Path,
    subject: &str,
) -> io::Result<&'os str> {
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
pub(crate) fn path_to_utf8<'path>(path: &'path Path, subject: &str) -> io::Result<&'path str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{subject}包含无效 UTF-8: {}", path.display()),
        )
    })
}
pub(crate) fn root_name(root_path: &Path) -> io::Result<String> {
    root_path.file_name().map_or_else(
        || path_to_utf8(root_path, "根路径").map(ToOwned::to_owned),
        |name| os_str_to_utf8(Some(name), root_path, "根目录名").map(ToOwned::to_owned),
    )
}
pub(crate) fn relative_path_text(relative_path: &Path) -> io::Result<String> {
    path_to_utf8(relative_path, "相对路径").map(ToOwned::to_owned)
}
pub(crate) fn code_block_language(path: &Path) -> io::Result<String> {
    path.extension().map_or_else(
        || Ok(String::new()),
        |extension| os_str_to_utf8(Some(extension), path, "文件扩展名").map(ToOwned::to_owned),
    )
}
#[cfg(test)]
mod tests;
