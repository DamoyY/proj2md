use core::fmt::Write as _;
use std::{
    env,
    fs::{self, File},
    io::{self, Write as _},
    path::Path,
};

use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use ignore::WalkBuilder;
const OUTPUT_FILENAME: &str = "project.md";
const EXTRA_EXCLUDED_FILES: [&str; 2] = ["LICENSE", "README.md"];
fn is_binary(bytes: &[u8]) -> Result<bool, io::Error> {
    if bytes.is_empty() {
        return Ok(false);
    }
    let mut total = 0_usize;
    let mut control = 0_usize;
    for &byte in bytes.iter().take(8192) {
        total = total
            .checked_add(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "统计字节数时发生溢出"))?;
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
    let total_scaled = total
        .checked_mul(30)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "计算字节总数比例时发生溢出"))?;
    Ok(control_scaled > total_scaled)
}
fn read_file_content(path: &Path) -> Result<String, Box<dyn core::error::Error>> {
    let bytes = fs::read(path)
        .map_err(|err| io::Error::new(err.kind(), format!("读取文件失败: {}", path.display())))?;
    if is_binary(&bytes)? {
        return Ok("(二进制文件)".to_owned());
    }
    if let Some((encoding, bom_len)) = Encoding::for_bom(&bytes) {
        let bytes_no_bom = bytes.get(bom_len..).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BOM 长度异常: {}", path.display()),
            )
        })?;
        let (text, _, had_errors) = encoding.decode(bytes_no_bom);
        if !had_errors {
            return Ok(text.into_owned());
        }
    }
    if let Ok(text) = core::str::from_utf8(&bytes) {
        return Ok(text.to_owned());
    }
    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let (text, _, had_errors) = encoding.decode(&bytes);
    if !had_errors {
        return Ok(text.into_owned());
    }
    Ok("(解码失败)".to_owned())
}
fn generate_directory_tree(root_path: &Path) -> Result<String, Box<dyn core::error::Error>> {
    let mut tree_str = String::from("## 1. 目录结构\n\n");
    let root_name = match root_path.file_name() {
        Some(name) => name.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("根目录名包含无效 UTF-8: {}", root_path.display()),
            )
        })?,
        None => root_path.as_os_str().to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("根路径包含无效 UTF-8: {}", root_path.display()),
            )
        })?,
    };
    writeln!(tree_str, "{root_name}/")?;
    for entry_result in WalkBuilder::new(root_path).build() {
        let entry = entry_result.map_err(|err| io::Error::other(format!("遍历目录失败: {err}")))?;
        let path = entry.path();
        let rel_path = path.strip_prefix(root_path).map_err(|err| {
            io::Error::other(format!("无法计算相对路径: {}: {err}", path.display()))
        })?;
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let depth = rel_path.components().count();
        let indent = "    ".repeat(depth);
        let file_type = entry
            .file_type()
            .ok_or_else(|| io::Error::other(format!("无法获取文件类型: {}", path.display())))?;
        if file_type.is_dir() {
            let dir_name = path
                .file_name()
                .ok_or_else(|| io::Error::other(format!("无法获取目录名: {}", path.display())))?
                .to_str()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("目录名包含无效 UTF-8: {}", path.display()),
                    )
                })?;
            writeln!(tree_str, "{indent}{dir_name}/")?;
        } else {
            let file_name = path
                .file_name()
                .ok_or_else(|| io::Error::other(format!("无法获取文件名: {}", path.display())))?
                .to_str()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("文件名包含无效 UTF-8: {}", path.display()),
                    )
                })?;
            if file_name != OUTPUT_FILENAME && !EXTRA_EXCLUDED_FILES.contains(&file_name) {
                writeln!(tree_str, "{indent}{file_name}")?;
            }
        }
    }
    Ok(tree_str)
}
fn generate_file_contents(root_path: &Path) -> Result<String, Box<dyn core::error::Error>> {
    let mut content_str = String::from("\n## 2. 文件内容\n\n");
    for entry_result in WalkBuilder::new(root_path).build() {
        let entry = entry_result.map_err(|err| io::Error::other(format!("遍历目录失败: {err}")))?;
        let file_type = entry.file_type().ok_or_else(|| {
            io::Error::other(format!("无法获取文件类型: {}", entry.path().display()))
        })?;
        if file_type.is_dir() {
            continue;
        }
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| io::Error::other(format!("无法获取文件名: {}", path.display())))?
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("文件名包含无效 UTF-8: {}", path.display()),
                )
            })?;
        if file_name == OUTPUT_FILENAME || EXTRA_EXCLUDED_FILES.contains(&file_name) {
            continue;
        }
        let rel_path = path
            .strip_prefix(root_path)
            .map_err(|err| {
                io::Error::other(format!("无法计算相对路径: {}: {err}", path.display()))
            })?
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("相对路径包含无效 UTF-8: {}", path.display()),
                )
            })?;
        let ext = match Path::new(file_name).extension() {
            Some(ext) => ext.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("文件扩展名包含无效 UTF-8: {}", path.display()),
                )
            })?,
            None => "",
        };
        writeln!(content_str, "### {rel_path}")?;
        writeln!(content_str, "```{ext}")?;
        let file_content = read_file_content(path)?;
        content_str.push_str(&file_content);
        if !file_content.ends_with('\n') {
            content_str.push('\n');
        }
        content_str.push_str("```\n\n");
    }
    Ok(content_str)
}
fn get_input_path() -> Result<String, io::Error> {
    let args: Vec<String> = env::args().collect();
    if let Some(path) = args.get(1) {
        return Ok(path.clone());
    }
    let cwd = env::current_dir()?;
    Ok(cwd.to_string_lossy().to_string())
}
fn run() -> Result<(), Box<dyn core::error::Error>> {
    let path_str = get_input_path()?;
    let root_path = Path::new(&path_str);
    if !root_path.exists() {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, format!("路径不存在: {path_str}")).into(),
        );
    }
    if !root_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("路径不是目录: {path_str}"),
        )
        .into());
    }
    let gitignore_path = root_path.join(".gitignore");
    if !gitignore_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, ".gitignore 文件不存在").into());
    }
    println!("正在生成文档...");
    let tree_part = generate_directory_tree(root_path)?;
    let content_part = generate_file_contents(root_path)?;
    let final_content = format!("{tree_part}{content_part}");
    let output_path = root_path.join(OUTPUT_FILENAME);
    let mut file = File::create(&output_path)?;
    file.write_all(final_content.as_bytes())?;
    println!("文档已生成: {}", output_path.display());
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("错误: {err}");
    }
}
