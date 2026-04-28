use crate::{
    content::read_file_content,
    errors::AppResult,
    inventory::{FileEntry, ProjectInventory, TreeEntryKind, collect_project_inventory},
};
use std::{io, path::Path};
pub(crate) fn write_project_markdown<W>(root_path: &Path, writer: &mut W) -> AppResult<()>
where
    W: io::Write,
{
    let inventory = collect_project_inventory(root_path)?;
    write_directory_tree(&inventory, writer)?;
    write_file_contents(&inventory.content_files, writer)
}
fn write_directory_tree<W>(inventory: &ProjectInventory, writer: &mut W) -> AppResult<()>
where
    W: io::Write,
{
    writer.write_all("## 1. 目录结构\n\n".as_bytes())?;
    writeln!(writer, "{}/", inventory.root_name)?;
    for entry in &inventory.tree_entries {
        let indent = "    ".repeat(entry.depth);
        match entry.kind {
            TreeEntryKind::Directory => writeln!(writer, "{indent}{}/", entry.name)?,
            TreeEntryKind::File => writeln!(writer, "{indent}{}", entry.name)?,
        }
    }
    Ok(())
}
fn write_file_contents<W>(files: &[FileEntry], writer: &mut W) -> AppResult<()>
where
    W: io::Write,
{
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
#[cfg(test)]
mod tests;
