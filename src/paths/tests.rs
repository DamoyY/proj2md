use super::{code_block_language, root_name, validate_root_path};
use crate::test_support::{TestDir, must, must_err};
use std::{io, path::Path};
#[test]
fn validate_root_path_accepts_directory() {
    let dir = must(TestDir::new("valid-root"), "创建测试目录失败");
    must(validate_root_path(dir.path()), "校验目录失败");
}
#[test]
fn validate_root_path_rejects_missing_path() {
    let dir = must(TestDir::new("missing-root"), "创建测试目录失败");
    let err = must_err(
        validate_root_path(&dir.path().join("missing")),
        "缺失路径必须报错",
    );
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}
#[test]
fn validate_root_path_rejects_file() {
    let dir = must(TestDir::new("file-root"), "创建测试目录失败");
    let file = must(dir.write_str("file.txt", "content"), "写入测试文件失败");
    let err = must_err(validate_root_path(&file), "文件路径必须报错");
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
#[test]
fn root_name_uses_last_path_component() {
    let dir = must(TestDir::new("root-name"), "创建测试目录失败");
    let name = must(root_name(dir.path()), "读取根目录名称失败");
    assert!(name.starts_with("proj2md-test-root-name-"));
}
#[test]
fn code_block_language_uses_file_extension() {
    let language = must(
        code_block_language(Path::new("src/main.rs")),
        "读取文件扩展名失败",
    );
    assert_eq!(language, "rs");
}
#[test]
fn code_block_language_is_empty_without_extension() {
    let language = must(
        code_block_language(Path::new("LICENSE")),
        "读取无扩展名文件失败",
    );
    assert_eq!(language, "");
}
