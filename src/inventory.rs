use crate::{
    config::{EXTRA_EXCLUDED_FILES, OUTPUT_FILENAME},
    errors::AppResult,
    paths,
};
use ignore::{Walk, WalkBuilder};
use std::{
    io,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileEntry {
    pub(crate) absolute_path: PathBuf,
    pub(crate) relative_path: String,
    pub(crate) code_block_language: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectInventory {
    pub(crate) root_name: String,
    pub(crate) tree_entries: Vec<TreeEntry>,
    pub(crate) content_files: Vec<FileEntry>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TreeEntry {
    pub(crate) depth: usize,
    pub(crate) name: String,
    pub(crate) kind: TreeEntryKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TreeEntryKind {
    Directory,
    File,
}
pub(crate) fn collect_project_inventory(root_path: &Path) -> AppResult<ProjectInventory> {
    let mut tree_entries = Vec::new();
    let mut content_files = Vec::new();
    for entry_result in build_walk(root_path) {
        let entry = entry_result.map_err(|err| io::Error::other(format!("遍历目录失败: {err}")))?;
        let path = entry.path();
        let relative_path = path.strip_prefix(root_path).map_err(|err| {
            io::Error::other(format!("无法计算相对路径: {}: {err}", path.display()))
        })?;
        if relative_path.as_os_str().is_empty() {
            continue;
        }
        let file_type = entry
            .file_type()
            .ok_or_else(|| io::Error::other(format!("无法获取文件类型: {}", path.display())))?;
        if file_type.is_dir() {
            push_directory(path, relative_path, &mut tree_entries)?;
        } else {
            push_file(path, relative_path, &mut tree_entries, &mut content_files)?;
        }
    }
    Ok(ProjectInventory {
        root_name: paths::root_name(root_path)?,
        tree_entries,
        content_files,
    })
}
fn build_walk(root_path: &Path) -> Walk {
    let mut builder = WalkBuilder::new(root_path);
    builder.require_git(false);
    builder.build()
}
fn push_directory(
    path: &Path,
    relative_path: &Path,
    tree_entries: &mut Vec<TreeEntry>,
) -> AppResult<()> {
    tree_entries.push(TreeEntry {
        depth: relative_path.components().count(),
        name: paths::os_str_to_utf8(path.file_name(), path, "目录名")?.to_owned(),
        kind: TreeEntryKind::Directory,
    });
    Ok(())
}
fn push_file(
    path: &Path,
    relative_path: &Path,
    tree_entries: &mut Vec<TreeEntry>,
    content_files: &mut Vec<FileEntry>,
) -> AppResult<()> {
    let file_name = paths::os_str_to_utf8(path.file_name(), path, "文件名")?;
    if is_excluded_file(file_name) {
        return Ok(());
    }
    tree_entries.push(TreeEntry {
        depth: relative_path.components().count(),
        name: file_name.to_owned(),
        kind: TreeEntryKind::File,
    });
    content_files.push(FileEntry {
        absolute_path: path.to_path_buf(),
        relative_path: paths::relative_path_text(relative_path)?,
        code_block_language: paths::code_block_language(path)?,
    });
    Ok(())
}
fn is_excluded_file(file_name: &str) -> bool {
    file_name == OUTPUT_FILENAME || EXTRA_EXCLUDED_FILES.contains(&file_name)
}
#[cfg(test)]
mod tests;
