use crate::errors::AppResult;
use std::path::Path;
#[cfg(target_os = "windows")]
#[path = "clipboard/win32.rs"]
mod win32;
#[cfg(target_os = "windows")]
#[path = "clipboard/windows.rs"]
mod windows;
pub(crate) fn copy_file_to_clipboard(file_path: &Path) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        windows::copy_file_to_clipboard(file_path)?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = file_path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "当前平台暂不支持复制到剪贴板",
        )
        .into())
    }
}
