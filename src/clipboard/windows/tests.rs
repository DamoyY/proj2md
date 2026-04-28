use super::{DropFiles, build_file_drop_payload};
use crate::test_support::must;
use core::{mem::size_of, ptr};
use std::{os::windows::ffi::OsStrExt as _, path::Path};
#[test]
fn file_drop_payload_contains_header_and_double_null_path() {
    let path = Path::new(r"C:\tmp\项目.txt");
    let payload = must(build_file_drop_payload(path), "构建剪贴板负载失败");
    let header_size = size_of::<DropFiles>();
    let expected_path_units: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain([0_u16, 0_u16])
        .collect();
    let expected_size = header_size + expected_path_units.len() * size_of::<u16>();
    assert_eq!(payload.len(), expected_size);
    let header = unsafe { ptr::read_unaligned(payload.as_ptr().cast::<DropFiles>()) };
    let expected_header_size = must(u32::try_from(header_size), "头部大小必须可转换");
    assert_eq!(header.p_files, expected_header_size);
    assert_eq!(header.f_nc, 0_i32);
    assert_eq!(header.f_wide, 1_i32);
    let Some(path_bytes) = payload.get(header_size..) else {
        panic!("剪贴板负载缺少路径区域");
    };
    let expected_bytes_len = expected_path_units.len() * size_of::<u16>();
    let expected_bytes = unsafe {
        core::slice::from_raw_parts(
            expected_path_units.as_ptr().cast::<u8>(),
            expected_bytes_len,
        )
    };
    assert_eq!(path_bytes, expected_bytes);
}
