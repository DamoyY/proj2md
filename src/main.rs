use core::fmt::Write as _;
use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::{self, BufRead as _, Write as _},
    path::Path,
};

use dialoguer::MultiSelect;
use encoding_rs::UTF_16LE;
use walkdir::WalkDir;

const OUTPUT_FILENAME: &str = "project_documentation.md";
const IGNORE_DIRS: &[&str] = &["__pycache__", ".vs", "target", ".claude"];

fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(|ext| ext.to_str())
}

fn scan_extensions(root_path: &Path, ignore_dirs: &HashSet<&str>) -> Vec<String> {
    let mut extensions: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|dir_entry| {
            !dir_entry
                .file_name()
                .to_str()
                .is_some_and(|name| ignore_dirs.contains(name))
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            if file_name == OUTPUT_FILENAME {
                continue;
            }
            if let Some(ext) = get_extension(file_name) {
                extensions.insert(format!(".{ext}"));
            }
        }
    }

    let mut ext_vec: Vec<String> = extensions.into_iter().collect();
    ext_vec.sort();
    ext_vec
}

fn select_extensions(extensions: &[String]) -> Result<HashSet<String>, io::Error> {
    if extensions.is_empty() {
        return Ok(HashSet::new());
    }

    let selections = MultiSelect::new()
        .with_prompt("使用 ↑↓ 移动，空格键选择/取消，回车确认")
        .items(extensions)
        .interact()?;

    let selected: HashSet<String> = selections
        .into_iter()
        .filter_map(|idx| extensions.get(idx).cloned())
        .collect();

    Ok(selected)
}

fn should_include_file(filename: &str, whitelist: &HashSet<String>) -> bool {
    get_extension(filename).is_some_and(|ext| whitelist.contains(&format!(".{ext}")))
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

fn generate_directory_tree(
    root_path: &Path,
    ignore_dirs: &HashSet<&str>,
    whitelist: &HashSet<String>,
) -> Result<String, core::fmt::Error> {
    let mut tree_str = String::from("## 1. 目录结构\n\n");
    let root_name = root_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root");

    writeln!(tree_str, "{root_name}/")?;

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|dir_entry| {
            !dir_entry
                .file_name()
                .to_str()
                .is_some_and(|name| ignore_dirs.contains(name))
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(root_path).unwrap_or(path);

        if rel_path.as_os_str().is_empty() {
            continue;
        }

        let depth = rel_path.components().count();
        let indent = "    ".repeat(depth);

        if entry.file_type().is_dir() {
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
            if should_include_file(file_name, whitelist) {
                writeln!(tree_str, "{indent}{file_name}")?;
            }
        }
    }

    Ok(tree_str)
}

fn generate_file_contents(
    root_path: &Path,
    ignore_dirs: &HashSet<&str>,
    whitelist: &HashSet<String>,
) -> Result<String, core::fmt::Error> {
    let mut content_str = String::from("\n## 2. 文件内容\n\n");

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|dir_entry| {
            !dir_entry
                .file_name()
                .to_str()
                .is_some_and(|name| ignore_dirs.contains(name))
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if !should_include_file(file_name, whitelist) {
            continue;
        }

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

    let ignore_dirs: HashSet<&str> = IGNORE_DIRS.iter().copied().collect();

    println!("正在扫描目录: {path_str} ...");

    let extensions = scan_extensions(root_path, &ignore_dirs);

    if extensions.is_empty() {
        eprintln!("错误: 目录中没有找到任何文件");
        return Ok(());
    }

    println!("找到以下文件类型，请选择要包含的类型：");

    let whitelist = select_extensions(&extensions)?;

    if whitelist.is_empty() {
        eprintln!("错误: 未选择任何文件类型");
        return Ok(());
    }

    println!("正在生成文档...");

    let tree_part = generate_directory_tree(root_path, &ignore_dirs, &whitelist)?;
    let content_part = generate_file_contents(root_path, &ignore_dirs, &whitelist)?;

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
