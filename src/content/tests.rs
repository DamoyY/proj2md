use super::{BINARY_MARKER, is_binary, read_file_content};
use crate::test_support::{TestDir, must, must_err};
#[test]
fn empty_file_is_not_binary() {
    assert!(!must(is_binary(&[]), "检测空文件失败"));
}
#[test]
fn nul_byte_marks_binary_content() {
    assert!(must(is_binary(b"abc\0def"), "检测 NUL 字节失败"));
}
#[test]
fn dense_control_bytes_mark_binary_content() {
    assert!(must(is_binary(&[1, 2, 3, b'a', b'b']), "检测控制字符失败"));
}
#[test]
fn ordinary_text_is_not_binary() {
    assert!(!must(
        is_binary("hello\n世界".as_bytes()),
        "检测普通文本失败"
    ));
}
#[test]
fn read_file_content_reads_utf8() {
    let dir = must(TestDir::new("utf8"), "创建测试目录失败");
    let file = must(
        dir.write_str("note.txt", "hello\n世界"),
        "写入 UTF-8 文件失败",
    );
    let content = must(read_file_content(&file), "读取 UTF-8 文件失败");
    assert_eq!(content, "hello\n世界");
}
#[test]
fn read_file_content_removes_utf8_bom() {
    let dir = must(TestDir::new("utf8-bom"), "创建测试目录失败");
    let file = must(
        dir.write_bytes("note.txt", &[0xEF, 0xBB, 0xBF, b'o', b'k']),
        "写入 UTF-8 BOM 文件失败",
    );
    let content = must(read_file_content(&file), "读取 UTF-8 BOM 文件失败");
    assert_eq!(content, "ok");
}
#[test]
fn read_file_content_decodes_utf16le_bom() {
    let dir = must(TestDir::new("utf16le"), "创建测试目录失败");
    let file = must(
        dir.write_bytes("note.txt", &[0xFF, 0xFE, b'H', 0, b'i', 0]),
        "写入 UTF-16LE 文件失败",
    );
    let content = must(read_file_content(&file), "读取 UTF-16LE 文件失败");
    assert_eq!(content, "Hi");
}
#[test]
fn read_file_content_marks_binary_files() {
    let dir = must(TestDir::new("binary"), "创建测试目录失败");
    let file = must(
        dir.write_bytes("image.bin", &[0, 1, 2, 3]),
        "写入二进制文件失败",
    );
    let content = must(read_file_content(&file), "读取二进制文件失败");
    assert_eq!(content, BINARY_MARKER);
}
#[test]
fn read_file_content_reports_missing_file() {
    let dir = must(TestDir::new("missing-file"), "创建测试目录失败");
    let err = must_err(
        read_file_content(&dir.path().join("missing.txt")),
        "缺失文件必须报错",
    );
    assert!(err.to_string().contains("读取文件失败"));
}
