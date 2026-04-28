use super::write_output_file_in;
use crate::{
    config::OUTPUT_FILENAME,
    test_support::{TestDir, must},
};
use std::fs;
#[test]
fn write_output_file_in_creates_markdown_in_requested_directory() {
    let project = must(TestDir::new("output-project"), "创建项目测试目录失败");
    let output_dir = must(TestDir::new("output-target"), "创建输出测试目录失败");
    must(
        project.write_str("src/main.rs", "fn main() {}\n"),
        "写入 main.rs 失败",
    );
    let output_path = must(
        write_output_file_in(project.path(), output_dir.path()),
        "写入输出文件失败",
    );
    let document = must(fs::read_to_string(&output_path), "读取输出文件失败");
    assert_eq!(output_path, output_dir.path().join(OUTPUT_FILENAME));
    assert!(document.contains("## 1. 目录结构"));
    assert!(document.contains("## 2. 文件内容"));
    assert!(document.contains("fn main() {}"));
}
