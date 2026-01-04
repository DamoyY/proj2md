use encoding_rs::UTF_16LE;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

const WHITELIST: &[&str] = &[".py", ".json", ".sln", ".h", ".cpp", ".rs", ".toml", ".yaml"];
const OUTPUT_FILENAME: &str = "project_documentation.md";
const IGNORE_DIRS: &[&str] = &["__pycache__", ".vs", "target", ".claude"];

fn should_include_file(filename: &str) -> bool {
    WHITELIST.iter().any(|ext| filename.ends_with(ext))
}

fn get_extension_for_highlight(filename: &str) -> &str {
    if let Some(pos) = filename.rfind('.') {
        &filename[pos + 1..]
    } else {
        ""
    }
}

fn read_file_content(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        return Some(content);
    }

    if let Ok(mut file) = File::open(path) {
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_ok() {
            let (result, _, had_errors) = UTF_16LE.decode(&bytes);
            if !had_errors {
                return Some(result.into_owned());
            }
        }
    }

    None
}

fn generate_directory_tree(root_path: &Path, ignore_dirs: &HashSet<&str>) -> String {
    let mut tree_str = String::from("## 1. 目录结构\n\n");
    let root_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root");

    tree_str.push_str(&format!("{}/\n", root_name));

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| {
            !e.file_name()
                .to_str()
                .map(|s| ignore_dirs.contains(s))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(root_path).unwrap_or(path);

        if rel_path.as_os_str().is_empty() {
            continue;
        }

        let depth = rel_path.components().count();
        let indent = "    ".repeat(depth);

        if entry.file_type().is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            tree_str.push_str(&format!("{}{}/\n", indent, dir_name));
        } else if entry.file_type().is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if should_include_file(file_name) {
                tree_str.push_str(&format!("{}{}\n", indent, file_name));
            }
        }
    }

    tree_str
}

fn generate_file_contents(root_path: &Path, ignore_dirs: &HashSet<&str>) -> String {
    let mut content_str = String::from("\n## 2. 文件内容\n\n");

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| {
            !e.file_name()
                .to_str()
                .map(|s| ignore_dirs.contains(s))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !should_include_file(file_name) {
            continue;
        }

        if file_name == OUTPUT_FILENAME {
            continue;
        }

        let rel_path = path
            .strip_prefix(root_path)
            .unwrap_or(path)
            .to_string_lossy();

        let ext = get_extension_for_highlight(file_name);

        content_str.push_str(&format!("### {}\n", rel_path));
        content_str.push_str(&format!("```{}\n", ext));

        if let Some(file_content) = read_file_content(path) {
            content_str.push_str(&file_content);
            if !file_content.ends_with('\n') {
                content_str.push('\n');
            }
        }

        content_str.push_str("```\n\n");
    }

    content_str
}

fn get_input_path() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        return args[1].clone();
    }

    print!("请输入项目路径: ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).expect("无法读取输入");

    line.trim().to_string()
}

fn main() {
    let path_str = get_input_path();
    let root_path = Path::new(&path_str);

    if !root_path.exists() {
        eprintln!("错误: 路径不存在: {}", path_str);
        return;
    }

    if !root_path.is_dir() {
        eprintln!("错误: 路径不是目录: {}", path_str);
        return;
    }

    let ignore_dirs: HashSet<&str> = IGNORE_DIRS.iter().copied().collect();

    println!("正在扫描目录: {} ...", path_str);

    let tree_part = generate_directory_tree(root_path, &ignore_dirs);
    let content_part = generate_file_contents(root_path, &ignore_dirs);

    let final_content = format!("{}{}", tree_part, content_part);
    let output_path = root_path.join(OUTPUT_FILENAME);

    let mut file = File::create(&output_path).expect("无法创建输出文件");
    file.write_all(final_content.as_bytes())
        .expect("无法写入文件");

    println!("文档已生成: {}", output_path.display());
}
