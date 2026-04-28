use super::input_path_from_args;
use crate::test_support::{must, must_err};
use std::{ffi::OsString, io, path::PathBuf};
#[test]
fn explicit_argument_becomes_input_path() {
    let path = must(
        input_path_from_args([OsString::from("proj2md"), OsString::from("sample-project")]),
        "解析显式路径参数失败",
    );
    assert_eq!(path, PathBuf::from("sample-project"));
}
#[test]
fn missing_argument_uses_current_directory() {
    let path = must(
        input_path_from_args([OsString::from("proj2md")]),
        "解析缺省路径参数失败",
    );
    assert!(path.is_dir());
}
#[test]
fn extra_arguments_are_rejected() {
    let err = must_err(
        input_path_from_args([
            OsString::from("proj2md"),
            OsString::from("one"),
            OsString::from("two"),
        ]),
        "多个路径参数必须报错",
    );
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
