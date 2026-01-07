use core::fmt::Write as _;
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead as _, Write as _},
    path::Path,
};
use encoding_rs::UTF_16LE;
use ignore::WalkBuilder;
const OUTPUT_FILENAME: &str = "project.md";
fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(|ext| ext.to_str())
}
fn read_file_content(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        return Some(content);
    }
    if let Ok(bytes) = fs::read(path) {
        let (result, _, had_errors) = UTF_16LE.decode(&bytes);
        if !had_errors {
            return Some(result.into_owned());
        }
    }
    None
}
fn generate_directory_tree(root_path: &Path) -> Result<String, core::fmt::Error> {
    let mut tree_str = String::from("## 1. 目录结构\n\n");
    let root_name = root_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root");
    writeln!(tree_str, "{root_name}/")?;
    for entry in WalkBuilder::new(root_path).build().filter_map(Result::ok) {
        let path = entry.path();
        let rel_path = path.strip_prefix(root_path).unwrap_or(path);
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let depth = rel_path.components().count();
        let indent = "    ".repeat(depth);
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            let dir_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            writeln!(tree_str, "{indent}{dir_name}/")?;
        } else {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name != OUTPUT_FILENAME {
                writeln!(tree_str, "{indent}{file_name}")?;
            }
        }
    }
    Ok(tree_str)
}
fn generate_file_contents(root_path: &Path) -> Result<String, core::fmt::Error> {
    let mut content_str = String::from("\n## 2. 文件内容\n\n");
    for entry in WalkBuilder::new(root_path).build().filter_map(Result::ok) {
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            continue;
        }
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name == OUTPUT_FILENAME {
            continue;
        }
        let rel_path = path
            .strip_prefix(root_path)
            .unwrap_or(path)
            .to_string_lossy();
        let ext = get_extension(file_name).unwrap_or("");
        writeln!(content_str, "### {rel_path}")?;
        writeln!(content_str, "```{ext}")?;
        if let Some(file_content) = read_file_content(path) {
            content_str.push_str(&file_content);
            if !file_content.ends_with('\n') {
                content_str.push('\n');
            }
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
    print!("请输入项目路径: ");
    io::stdout().flush()?;
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}
fn run() -> Result<(), Box<dyn core::error::Error>> {
    let path_str = get_input_path()?;
    let root_path = Path::new(&path_str);
    if !root_path.exists() {
        eprintln!("错误: 路径不存在: {path_str}");
        return Ok(());
    }
    if !root_path.is_dir() {
        eprintln!("错误: 路径不是目录: {path_str}");
        return Ok(());
    }
    let gitignore_path = root_path.join(".gitignore");
    if !gitignore_path.exists() {
        eprintln!("错误: .gitignore 文件不存在");
        return Ok(());
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
