use super::{TreeEntryKind, collect_project_inventory, is_excluded_file};
use crate::test_support::{TestDir, must};
use std::path::PathBuf;
#[test]
fn generated_and_repository_metadata_files_are_excluded() {
    assert!(is_excluded_file("project.md"));
    assert!(is_excluded_file("README.md"));
    assert!(is_excluded_file("LICENSE"));
    assert!(!is_excluded_file("Cargo.toml"));
}
#[test]
fn visible_files_are_collected_with_tree_metadata() {
    let dir = must(TestDir::new("inventory-visible"), "创建测试目录失败");
    must(
        dir.write_str("src/main.rs", "fn main() {}\n"),
        "写入 main.rs 失败",
    );
    must(
        dir.write_str("Cargo.toml", "[package]\n"),
        "写入 Cargo.toml 失败",
    );
    must(dir.create_dir("empty"), "创建空目录失败");
    let inventory = must(collect_project_inventory(dir.path()), "收集项目清单失败");
    let main_path = relative_path(["src", "main.rs"]);
    assert!(
        inventory
            .root_name
            .starts_with("proj2md-test-inventory-visible-")
    );
    assert!(has_tree_entry(
        &inventory.tree_entries,
        TreeEntryKind::Directory,
        "src"
    ));
    assert!(has_tree_entry(
        &inventory.tree_entries,
        TreeEntryKind::Directory,
        "empty"
    ));
    assert!(has_tree_entry(
        &inventory.tree_entries,
        TreeEntryKind::File,
        "main.rs"
    ));
    assert!(
        inventory
            .content_files
            .iter()
            .any(|file| { file.relative_path == main_path && file.code_block_language == "rs" })
    );
}
#[test]
fn excluded_files_are_not_listed_or_collected() {
    let dir = must(TestDir::new("inventory-excluded"), "创建测试目录失败");
    must(dir.write_str("README.md", "readme"), "写入 README 失败");
    must(dir.write_str("LICENSE", "license"), "写入 LICENSE 失败");
    must(dir.write_str("project.md", "old"), "写入旧输出文件失败");
    let inventory = must(collect_project_inventory(dir.path()), "收集项目清单失败");
    assert!(inventory.tree_entries.is_empty());
    assert!(inventory.content_files.is_empty());
}
#[test]
fn gitignore_rules_are_honored() {
    let dir = must(TestDir::new("inventory-ignore"), "创建测试目录失败");
    must(
        dir.write_str(".gitignore", "ignored.txt\nignored_dir/\n"),
        "写入 gitignore 失败",
    );
    must(dir.write_str("ignored.txt", "ignored"), "写入忽略文件失败");
    must(
        dir.write_str("ignored_dir/file.txt", "ignored"),
        "写入忽略目录文件失败",
    );
    must(dir.write_str("visible.txt", "visible"), "写入可见文件失败");
    let inventory = must(collect_project_inventory(dir.path()), "收集项目清单失败");
    assert!(
        inventory
            .content_files
            .iter()
            .any(|file| file.relative_path == "visible.txt")
    );
    assert!(
        !inventory
            .content_files
            .iter()
            .any(|file| file.relative_path == "ignored.txt")
    );
    assert!(!has_tree_entry(
        &inventory.tree_entries,
        TreeEntryKind::Directory,
        "ignored_dir"
    ));
}
fn has_tree_entry(entries: &[super::TreeEntry], kind: TreeEntryKind, name: &str) -> bool {
    entries
        .iter()
        .any(|entry| entry.kind == kind && entry.name == name)
}
fn relative_path<const N: usize>(parts: [&str; N]) -> String {
    let path: PathBuf = parts.into_iter().collect();
    match path.into_os_string().into_string() {
        Ok(value) => value,
        Err(value) => panic!("测试路径必须是 UTF-8: {value:?}"),
    }
}
