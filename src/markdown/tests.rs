use super::{write_directory_tree, write_file_contents, write_project_markdown};
use crate::{
    inventory::{FileEntry, ProjectInventory, TreeEntry, TreeEntryKind},
    test_support::{TestDir, must},
};
#[test]
fn write_directory_tree_renders_depth_and_kind() {
    let inventory = ProjectInventory {
        root_name: "root".to_owned(),
        tree_entries: vec![
            TreeEntry {
                depth: 1,
                name: "src".to_owned(),
                kind: TreeEntryKind::Directory,
            },
            TreeEntry {
                depth: 2,
                name: "main.rs".to_owned(),
                kind: TreeEntryKind::File,
            },
        ],
        content_files: Vec::new(),
    };
    let mut output = Vec::new();
    must(
        write_directory_tree(&inventory, &mut output),
        "写入目录树失败",
    );
    let document = must(String::from_utf8(output), "目录树输出必须是 UTF-8");
    assert_eq!(
        document,
        "## 1. 目录结构\n\nroot/\n    src/\n        main.rs\n"
    );
}
#[test]
fn write_file_contents_adds_trailing_newline_before_fence() {
    let dir = must(TestDir::new("markdown-content"), "创建测试目录失败");
    let file = must(dir.write_str("note.txt", "hello"), "写入内容文件失败");
    let entries = [FileEntry {
        absolute_path: file,
        relative_path: "note.txt".to_owned(),
        code_block_language: "txt".to_owned(),
    }];
    let mut output = Vec::new();
    must(
        write_file_contents(&entries, &mut output),
        "写入文件内容失败",
    );
    let document = must(String::from_utf8(output), "文件内容输出必须是 UTF-8");
    assert_eq!(
        document,
        "\n## 2. 文件内容\n\n### note.txt\n```txt\nhello\n```\n\n"
    );
}
#[test]
fn write_project_markdown_combines_tree_and_content() {
    let dir = must(TestDir::new("markdown-project"), "创建测试目录失败");
    must(
        dir.write_str("src/main.rs", "fn main() {}\n"),
        "写入 main.rs 失败",
    );
    let mut output = Vec::new();
    must(
        write_project_markdown(dir.path(), &mut output),
        "写入项目文档失败",
    );
    let document = must(String::from_utf8(output), "项目文档必须是 UTF-8");
    assert!(document.contains("## 1. 目录结构"));
    assert!(document.contains("## 2. 文件内容"));
    assert!(document.contains("main.rs"));
    assert!(document.contains("fn main() {}"));
}
