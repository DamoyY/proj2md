use crate::{config::OUTPUT_FILENAME, errors::AppResult, markdown::write_project_markdown};
use std::{
    env, fs,
    io::{self, BufWriter, Write as _},
    path::{Path, PathBuf},
};
pub(crate) fn write_output_file(root_path: &Path) -> AppResult<PathBuf> {
    let output_dir = env::temp_dir().join("proj2md");
    write_output_file_in(root_path, &output_dir)
}
fn write_output_file_in(root_path: &Path, output_dir: &Path) -> AppResult<PathBuf> {
    let (output_path, mut writer) = create_output_writer(output_dir)?;
    write_project_markdown(root_path, &mut writer)?;
    writer.flush()?;
    Ok(output_path)
}
fn create_output_writer(output_dir: &Path) -> AppResult<(PathBuf, BufWriter<fs::File>)> {
    fs::create_dir_all(output_dir).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("创建临时目录失败: {}: {err}", output_dir.display()),
        )
    })?;
    let output_path = output_dir.join(OUTPUT_FILENAME);
    let file = fs::File::create(&output_path).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("创建输出文件失败: {}: {err}", output_path.display()),
        )
    })?;
    Ok((output_path, BufWriter::new(file)))
}
#[cfg(test)]
mod tests;
