use crate::{
    cli::input_path_from_args, clipboard::copy_file_to_clipboard, errors::AppResult,
    output::write_output_file, paths::validate_root_path,
};
use std::ffi::OsString;
pub(crate) fn run<I>(args: I) -> AppResult<()>
where
    I: IntoIterator<Item = OsString>,
{
    let root_path = input_path_from_args(args)?;
    validate_root_path(&root_path)?;
    println!("正在生成文档...");
    let output_path = write_output_file(&root_path)?;
    copy_file_to_clipboard(&output_path)?;
    println!("文档文件已复制到剪贴板: {}", output_path.display());
    Ok(())
}
#[cfg(test)]
mod tests;
