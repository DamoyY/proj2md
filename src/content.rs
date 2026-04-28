use crate::{
    config::{BINARY_CONTROL_PERCENT, BINARY_SCAN_LIMIT},
    errors::AppResult,
};
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::Encoding;
use std::{fs, io, path::Path};
pub(crate) const BINARY_MARKER: &str = "(二进制文件)";
pub(crate) const DECODE_FAILURE_MARKER: &str = "(解码失败)";
pub(crate) fn is_binary(bytes: &[u8]) -> io::Result<bool> {
    if bytes.is_empty() {
        return Ok(false);
    }
    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    let mut control = 0_usize;
    for &byte in bytes.iter().take(sample_len) {
        if byte == 0 {
            return Ok(true);
        }
        if byte < 0x09 || (byte > 0x0D && byte < 0x20) {
            control = control.checked_add(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "统计控制字符时发生溢出")
            })?;
        }
    }
    let control_scaled = control
        .checked_mul(100)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "计算控制字符比例时发生溢出"))?;
    let total_scaled = sample_len
        .checked_mul(BINARY_CONTROL_PERCENT)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "计算字节总数比例时发生溢出"))?;
    Ok(control_scaled > total_scaled)
}
pub(crate) fn read_file_content(path: &Path) -> AppResult<String> {
    let bytes = fs::read(path)
        .map_err(|err| io::Error::new(err.kind(), format!("读取文件失败: {}", path.display())))?;
    if let Some(text) = decode_with_bom(path, &bytes)? {
        return Ok(text);
    }
    if is_binary(&bytes)? {
        return Ok(BINARY_MARKER.to_owned());
    }
    if let Ok(text) = core::str::from_utf8(&bytes) {
        return Ok(text.to_owned());
    }
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, Utf8Detection::Allow);
    let (text, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        Ok(DECODE_FAILURE_MARKER.to_owned())
    } else {
        Ok(text.into_owned())
    }
}
fn decode_with_bom(path: &Path, bytes: &[u8]) -> io::Result<Option<String>> {
    let Some((encoding, bom_len)) = Encoding::for_bom(bytes) else {
        return Ok(None);
    };
    let bytes_no_bom = bytes.get(bom_len..).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BOM 长度异常: {}", path.display()),
        )
    })?;
    let (text, _, had_errors) = encoding.decode(bytes_no_bom);
    if had_errors {
        Ok(None)
    } else {
        Ok(Some(text.into_owned()))
    }
}
#[cfg(test)]
mod tests;
