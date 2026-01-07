# proj2md

`proj2md` 是一个基于 Rust 编写的命令行工具，用于将整个代码项目的目录结构和文件内容转换成一个单一的 Markdown 文件。

该工具的主要用途是将代码库转换为大语言模型易于理解和分析的格式，方便进行代码审查、重构建议或项目理解。

## 特性

- **目录树生成**：自动生成项目文件结构树。
- **内容聚合**：将所有文件的内容读取并写入同一个 Markdown 文件。
- **智能忽略**：基于 `ignore` crate，自动识别并遵守 `.gitignore` 规则，排除无需处理的文件（如 `target/`, `node_modules/` 等）。
- **语法高亮**：根据文件扩展名自动为 Markdown 代码块添加语言标识。
- **编码支持**：支持 UTF-8 及 UTF-16LE 编码文件读取。

## 使用方法

### 运行要求
目标目录下**必须**包含 `.gitignore` 文件，否则工具会报错并停止运行。这是为了防止意外扫描并输出像 `node_modules` 或 `target` 这样巨大的目录。

### 方式一：命令行参数
直接在命令后跟上项目路径：

```bash
# 开发环境运行
cargo run --release -- /path/to/your/project

# 或者直接运行二进制文件
./target/release/proj2md /path/to/your/project
```

### 方式二：交互式输入
如果不带参数运行，程序会提示输入路径：

```bash
cargo run --release
# 输出: 请输入项目路径: 
```

## 输出格式

运行成功后，工具会在目标项目的根目录下生成一个名为 `project.md` 的文件。

**project.md 示例：**

```markdown
## 1. 目录结构

root/
    src/
        main.rs
    Cargo.toml

## 2. 文件内容

### Cargo.toml
(toml代码块内容...)

### src/main.rs
(rust代码块内容...)
```

## 注意事项

1. **安全性**：生成的 Markdown 文件包含项目所有（未忽略的）源代码，请勿将包含敏感信息（如 API Key、密码）的生成文件随意分享。
2. **大文件**：虽然工具可以处理大项目，但生成的 Markdown 文件可能非常大，请确保接收方（如 LLM 的上下文窗口）能够处理该大小的文本。
3. **忽略规则**：工具严格遵守目标目录下的 `.gitignore` 文件。如果你想包含通常被忽略的文件，请修改 `.gitignore`。

## 许可证

[MIT License](LICENSE)