use super::run;
use crate::test_support::{TestDir, must, must_err};
use std::ffi::OsString;
#[test]
fn run_rejects_missing_root_before_generating_output() {
    let dir = must(TestDir::new("app-missing"), "创建测试目录失败");
    let missing = dir.path().join("missing");
    let err = must_err(
        run([OsString::from("proj2md"), missing.into_os_string()]),
        "缺失根路径必须报错",
    );
    assert!(err.to_string().contains("路径不存在"));
}
