# DevHub Pro - 详细设计文档

> **面向开发者的完整实施指南**
>
> 本文档包含开发环境配置、Java→Rust快速入门、详细开发步骤、技术实现细节和注意事项。

---

## 目录

- **[第一章 Java开发者Rust快速入门](#第一章-java开发者rust快速入门)**
- **[第二章 开发环境配置](#第二章-开发环境配置)**
- **[第三章 详细开发步骤](#第三章-详细开发步骤)**
- **[第四章 核心模块实现](#第四章-核心模块实现)**
- **[第五章 GUI开发指南](#第五章-gui开发指南)**
- **[第六章 测试策略](#第六章-测试策略)**
- **[第七章 部署与发布](#第七章-部署与发布)**
- **[第八章 常见问题与注意事项](#第八章-常见问题与注意事项)**

---

## 第一章 Java开发者Rust快速入门

### 1.1 核心概念对比

#### 所有权系统（Ownership）- Java开发者必须理解

```rust
// Java: 垃圾回收自动管理内存
// Rust: 所有权系统在编译期保证内存安全

// 基本规则
// 1. 每个值有一个所有者（owner）
// 2. 同一时间只能有一个所有者
// 3. 所有者离开作用域，值被自动释放

fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // 移动（move），s1不再有效
    // println!("{}", s1);  // 编译错误！value borrowed here after move

    let s3 = s2.clone();  // 深拷贝，s2仍然有效
    println!("{}", s2);   // OK
    println!("{}", s3);   // OK
}
```

#### 借用与引用（Borrowing）

```rust
fn main() {
    let s = String::from("hello");

    // 不可变借用（多个可以共存）
    let len = calculate_length(&s);  // 借用，不获取所有权
    println!("Length of '{}' is {}.", s, len);

    // 可变借用（同一时间只能有一个）
    let mut s2 = String::from("hello");
    change(&mut s2);
}

fn calculate_length(s: &String) -> usize {  // 借用
    s.len()
}  // s离开作用域，但因为它没有所有权，所以不会释放

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

### 1.2 语法速查表

#### 变量与数据类型

| Java | Rust | 说明 |
|------|------|------|
| `int x = 5;` | `let x: i32 = 5;` | 整数需明确大小 |
| `Integer x = null;` | `let x: Option<i32> = None;` | 用Option替代null |
| `final String s = "hi";` | `let s = "hi";` | 默认不可变 |
| `String s = "hi";` | `let mut s = String::from("hi");` | 可变需mut |
| `List<String> list` | `Vec<String>` | 动态数组 |

#### 控制流

| Java | Rust |
|------|------|
| `if (x > 0) { ... }` | `if x > 0 { ... }` |
| `for (int i=0; i<10; i++)` | `for i in 0..10` |
| `for (String s : list)` | `for s in &list` |
| `switch(x) { case 1: ... }` | `match x { 1 => ... }` |

#### 函数与方法

```java
// Java
public class Calculator {
    private int value;

    public Calculator(int init) {
        this.value = init;
    }

    public int add(int x) {
        return this.value + x;
    }
}
```

```rust
// Rust
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new(init: i32) -> Self {
        Self { value: init }
    }

    pub fn add(&self, x: i32) -> i32 {
        self.value + x
    }
}
```

### 1.3 错误处理模式对比

#### Java的try-catch vs Rust的Result

```java
// Java
public String readFile(String path) throws IOException {
    try {
        return Files.readString(Paths.get(path));
    } catch (IOException e) {
        logger.error("Failed to read file", e);
        throw e;
    }
}

// 使用
try {
    String content = readFile("test.txt");
} catch (IOException e) {
    e.printStackTrace();
}
```

```rust
// Rust
use std::fs;
use anyhow::{Result, Context};

fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))
}

// 使用
match read_file("test.txt") {
    Ok(content) => println!("{}", content),
    Err(e) => eprintln!("Error: {}", e),
}

// 或使用?运算符快速传播错误
fn main() -> Result<()> {
    let content = read_file("test.txt")?;
    println!("{}", content);
    Ok(())
}
```

### 1.4 面向对象模式转换

#### 接口 → Trait

```java
// Java
interface Detector {
    boolean detect();
}

class PythonDetector implements Detector {
    public boolean detect() {
        return true;
    }
}
```

```rust
// Rust
trait Detector {
    fn detect(&self) -> bool;
}

struct PythonDetector;

impl Detector for PythonDetector {
    fn detect(&self) -> bool {
        true
    }
}
```

#### 抽象类 → Trait + 默认实现

```java
// Java
abstract class Tool {
    abstract String getName();
    String getVersion() { return "1.0.0"; }
}
```

```rust
// Rust
trait Tool {
    fn get_name(&self) -> String;
    fn get_version(&self) -> String {
        "1.0.0".to_string()  // 默认实现
    }
}
```

### 1.5 集合与迭代

```java
// Java
List<String> list = new ArrayList<>();
list.add("hello");
list.add("world");

for (String s : list) {
    System.out.println(s);
}

list.stream().filter(s -> s.length() > 3).count();
```

```rust
// Rust
let mut list = Vec::new();
list.push("hello".to_string());
list.push("world".to_string());

for s in &list {
    println!("{}", s);
}

list.iter().filter(|s| s.len() > 3).count();
```

### 1.6 异步编程

```java
// Java
CompletableFuture<String> future = CompletableFuture.supplyAsync(() -> {
    try {
        Thread.sleep(1000);
        return "Done";
    } catch (InterruptedException e) {
        return "Error";
    }
});

future.thenAccept(result -> System.out.println(result));
```

```rust
// Rust
use tokio::time::{sleep, Duration};

async fn fetch_data() -> String {
    sleep(Duration::from_secs(1)).await;
    "Done".to_string()
}

#[tokio::main]
async fn main() {
    let result = fetch_data().await;
    println!("{}", result);
}
```

---

## 第二章 开发环境配置

### 2.1 推荐IDE：RustRover

**为什么选择RustRover？**
- JetBrains出品，IDEA用户熟悉界面
- 智能代码补全和实时类型检查
- 内置Cargo集成（构建、测试、运行一键完成）
- 强大的调试器（支持条件断点、表达式求值）
- Git集成和代码审查工具
- 零学习成本，快捷键完全一致

### 2.2 安装步骤

#### 步骤1：安装Rust工具链

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows（下载并运行）
# https://rustup.rs/

# 配置环境变量（重启终端）
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 步骤2：安装RustRover

1. 下载：https://www.jetbrains.com/rust/
2. IDEA用户：在JetBrains Toolbox中直接安装
3. 首次启动：选择"Standard"安装，包含所有默认插件

#### 步骤3：配置RustRover

```
Settings/Preferences → Rust
├── Rustup toolchain: 自动检测（通常在 ~/.cargo/bin）
├── Cargo: 自动检测
└── Formatter: rustfmt（自动格式化工具）

Settings/Preferences → Editor → Code Style → Rust
├── 最大行宽: 100
└── 缩进: 4空格
```

### 2.3 IDEA用户快捷键对照

| 功能 | IDEA | RustRover |
|------|------|-----------|
| 运行 | `Ctrl+R` | `Ctrl+R` |
| 调试 | `Ctrl+D` | `Ctrl+D` |
| 格式化代码 | `Ctrl+Alt+L` | `Ctrl+Alt+L` |
| 重命名 | `Shift+F6` | `Shift+F6` |
| 查找用法 | `Alt+F7` | `Alt+F7` |
| 跳转实现 | `Ctrl+Alt+B` | `Ctrl+Alt+B` |
| 查看文档 | `Ctrl+Q` | `Ctrl+Q` |
| 结构视图 | `Ctrl+F12` | `Ctrl+F12` |
| 导航到类 | `Ctrl+N` | `Ctrl+N` |
| 导航到文件 | `Ctrl+Shift+N` | `Ctrl+Shift+N` |

### 2.4 Cargo常用命令

| Maven/Gradle | Cargo | 说明 |
|---------------|-------|------|
| `mvn clean install` | `cargo build` | 开发构建 |
| `mvn package` | `cargo build --release` | 发布构建（优化） |
| `mvn test` | `cargo test` | 运行测试 |
| `mvn clean` | `cargo clean` | 清理构建产物 |
| `mvn dependency:tree` | `cargo tree` | 查看依赖树 |
| `pom.xml / build.gradle` | `Cargo.toml` | 配置文件 |

```bash
# 项目初始化
cargo new devhub --bin        # 创建二进制项目
cargo new devhub-lib --lib    # 创建库项目

# 开发工作流
cargo build                   # 编译（debug模式）
cargo build --release         # 编译（release模式，优化）
cargo run                     # 编译并运行
cargo test                    # 运行所有测试
cargo test -- --show-output   # 显示测试输出
cargo doc                     # 生成文档
cargo doc --open              # 生成并打开文档

# 代码质量
cargo clippy                  # Linter检查
cargo fmt                     # 格式化代码
cargo audit                   # 安全审计

# 依赖管理
cargo update                  # 更新依赖
cargo clean                   # 清理构建缓存
```

### 2.5 项目结构

```
devhub/
├── Cargo.toml              # 项目配置文件（类似pom.xml）
├── Cargo.lock              # 依赖版本锁定文件（自动生成）
├── src/
│   ├── main.rs            # 二进制入口文件
│   ├── lib.rs             # 库入口文件
│   └── ...
├── tests/                 # 集成测试
├── benches/               # 性能测试
└── examples/              # 示例代码
```

---

## 第三章 详细开发步骤

### 3.1 阶段1：项目初始化（第1天）

#### 任务清单

- [ ] 创建Rust项目
- [ ] 配置Cargo.toml依赖
- [ ] 创建目录结构
- [ ] 配置git仓库
- [ ] 设置CI/CD（可选）

#### 详细步骤

```bash
# 1. 创建项目
cargo new devhub --bin
cd devhub

# 2. 编辑Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "devhub"
version = "0.1.0"
edition = "2021"
authors = ["DevHub Pro Team"]
description = "一站式开发环境管理平台"

[dependencies]
# GUI框架
egui = "0.29"
eframe = { version = "0.29", default-features = false, features = [
    "accesskit",
    "default_fonts",
    "glow",
] }

# 异步运行时
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# HTTP客户端
reqwest = { version = "0.12", features = ["json"] }

# 工具库
regex = "1"
dirs = "5"
anyhow = "1"
thiserror = "1"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 数据库
rusqlite = { version = "0.32", features = ["bundled"] }

[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"

[[bin]]
name = "devhub"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
EOF

# 3. 创建目录结构
mkdir -p src/{core,services,tools,ui,utils}
mkdir -p tests/{unit,integration}

# 4. 创建模块文件
touch src/core/mod.rs
touch src/services/mod.rs
touch src/tools/mod.rs
touch src/ui/mod.rs
touch src/utils/mod.rs

# 5. 初始化git
git init
cat > .gitignore << 'EOF'
/target
**/*.rs.bk
Cargo.lock
.DS_Store
EOF

git add .
git commit -m "feat: initial project structure"
```

#### 验收标准

- [ ] `cargo build` 编译通过
- [ ] `cargo test` 无测试失败
- [ ] `cargo clippy` 无警告
- [ ] 目录结构符合预期

### 3.2 阶段2：核心Trait定义（第1-2天）

#### 核心Trait列表

```rust
// src/core/mod.rs
pub mod detector;
pub mod mirror;
pub mod installer;
pub mod version;
pub mod environ;
pub mod command;

pub use detector::*;
pub use mirror::*;
pub use installer::*;
pub use version::*;
pub use environ::*;
pub use command::*;
```

#### 实现步骤

1. **定义ToolDetector trait**
2. **定义MirrorConfigurator trait**
3. **定义ToolInstaller trait**
4. **定义VersionManager trait**
5. **定义EnvConfigurator trait**
6. **定义CommandExecutor trait**

#### 验收标准

- [ ] 所有核心trait定义完成
- [ ] 每个trait有清晰的文档注释
- [ ] 编译通过，无错误

### 3.3 阶段3：服务层实现（第2-3天）

#### 服务列表

- ShellExecutor - Shell命令执行
- SpeedTestService - 网络测速
- ConfigManager - 配置管理
- DatabaseManager - 数据库操作

#### 实现示例

```rust
// src/services/shell.rs
use std::process::Command;
use crate::core::command::*;

pub struct ShellExecutor;

impl CommandExecutor for ShellExecutor {
    fn exec(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(cmd)
            .args(args)
            .output()?;

        Ok(CommandOutput {
            status: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
```

#### 验收标准

- [ ] 所有服务实现完成
- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过

### 3.4 阶段4：Python工具实现（第3-4天）

#### 实现清单

- [ ] Python解释器检测
- [ ] pip镜像源配置
- [ ] conda镜像源配置
- [ ] uv镜像源配置
- [ ] 常用包版本检测

#### 实现示例

```rust
// src/tools/python.rs
use crate::core::*;
use std::path::PathBuf;

pub struct PythonTool {
    detector: ShellDetector,
    pip_config: PipMirrorConfig,
}

impl PythonTool {
    pub fn new() -> Self {
        Self {
            detector: ShellDetector::new("python3", "--version"),
            pip_config: PipMirrorConfig::new(),
        }
    }
}
```

#### 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_detection() {
        let python = PythonTool::new();
        let result = python.detector.detect().unwrap();
        assert!(result.installed);
    }

    #[tokio::test]
    async fn test_pip_mirror_config() {
        let config = PipMirrorConfig::new();
        let mirrors = config.list_mirrors();
        assert!(!mirrors.is_empty());
    }
}
```

#### 验收标准

- [ ] Python工具功能完整
- [ ] 测试覆盖率 > 80%
- [ ] 手动测试通过

### 3.5 阶段5：GUI框架搭建（第4-5天）

#### GUI任务清单

- [ ] egui窗口框架
- [ ] DevHubApp状态管理
- [ ] Dashboard视图
- [ ] 工具卡片组件
- [ ] 基础交互

#### 实现示例

```rust
// src/ui/app.rs
use eframe::egui;

pub struct DevHubApp {
    tools: Vec<ToolState>,
    current_view: View,
}

impl eframe::App for DevHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_header(ctx);
        match &self.current_view {
            View::Dashboard => self.render_dashboard(ctx),
            View::ToolDetail(name) => self.render_tool_detail(ctx, name),
        }
    }
}
```

#### 验收标准

- [ ] GUI窗口正常启动
- [ ] 基础交互功能正常
- [ ] UI响应流畅（60fps）

### 3.6 阶段6：功能集成（第5-6天）

#### 集成任务

- [ ] 镜像源选择器组件
- [ ] 测速结果显示
- [ ] 应用镜像源功能
- [ ] 错误处理和提示
- [ ] UI优化

#### 验收标准

- [ ] 所有功能集成完成
- [ ] 用户体验友好
- [ ] 无critical bug

### 3.7 阶段7：测试与优化（第6-7天）

#### 测试任务

- [ ] 集成测试编写
- [ ] 性能测试
- [ ] Bug修复
- [ ] 用户文档编写

#### 性能目标

- 启动时间 < 1秒
- 测速响应 < 5秒
- UI刷新率 60fps

#### 验收标准

- [ ] 所有测试通过
- [ ] 性能达标
- [ ] 文档完整

---

## 第四章 核心模块实现

### 4.1 软件检测模块

#### 实现策略

```rust
// 检测优先级
1. which命令检查PATH
2. 常见安装路径检查
3. 版本管理器检查
4. 系统包管理器检查
```

#### 代码示例

```rust
use std::process::Command;

pub fn detect_tool(tool_name: &str) -> DetectionInfo {
    // 1. which命令
    if let Ok(output) = Command::new("which").arg(tool_name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DetectionInfo {
                installed: true,
                path: Some(PathBuf::from(path)),
                version: get_version(tool_name),
            };
        }
    }

    // 2. 未找到
    DetectionInfo {
        installed: false,
        path: None,
        version: None,
    }
}

fn get_version(tool_name: &str) -> Option<String> {
    let version_flags = vec!["--version", "-v", "version"];

    for flag in version_flags {
        if let Ok(output) = Command::new(tool_name).arg(flag).output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()?
                    .trim()
                    .to_string();
                return Some(version);
            }
        }
    }

    None
}
```

### 4.2 镜像源配置模块

#### pip配置示例

```rust
use std::path::PathBuf;

pub struct PipMirrorConfig {
    config_path: PathBuf,
}

impl PipMirrorConfig {
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pip");

        Self {
            config_path: config_dir.join("pip.conf"),
        }
    }

    pub fn apply(&self, mirror: &Mirror) -> Result<()> {
        // 创建配置目录
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 写入配置
        let content = format!(
            "[global]\nindex-url = {}\ntrusted-host = {}\n",
            mirror.url,
            extract_domain(&mirror.url)?
        );

        fs::write(&self.config_path, content)?;
        Ok(())
    }
}

fn extract_domain(url: &str) -> Result<String> {
    Ok(url.replace("https://", "")
        .replace("http://", "")
        .split('/')
        .next()
        .unwrap_or("mirrors.aliyun.com")
        .to_string())
}
```

### 4.3 网络测速模块

```rust
use reqwest::Client;
use std::time::Instant;

pub async fn test_mirror_speed(url: &str) -> u64 {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let start = Instant::now();

    match client.head(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            start.elapsed().as_millis() as u64
        }
        _ => u64::MAX,  // 超时或失败
    }
}

pub async fn benchmark_mirrors(mirrors: Vec<Mirror>) -> Vec<SpeedResult> {
    let tasks: Vec<_> = mirrors
        .into_iter()
        .map(|mirror| {
            async move {
                let latency = test_mirror_speed(&mirror.url).await;
                SpeedResult { mirror, latency_ms: latency }
            }
        })
        .collect();

    futures::future::join_all(tasks).await
}
```

### 4.4 环境变量管理

```rust
use std::collections::HashMap;

pub struct EnvManager {
    shell_config: PathBuf,
}

impl EnvManager {
    pub fn new() -> Result<Self> {
        let shell = std::env::var("SHELL").unwrap_or("/bin/zsh".to_string());
        let config_file = match shell.as_str() {
            "/bin/zsh" => "~/.zshrc",
            "/bin/bash" => "~/.bashrc",
            _ => "~/.profile",
        };

        Ok(Self {
            shell_config: PathBuf::from(config_file),
        })
    }

    pub fn set_env_var(&self, key: &str, value: &str) -> Result<()> {
        let mut content = fs::read_to_string(&self.shell_config)
            .unwrap_or_default();

        let export_line = format!("export {}=\"{}\"", key, value);

        // 检查是否已存在
        if content.contains(&format!("export {}=", key)) {
            // 替换
            let re = regex::Regex::new(&format!(
                r"^export {}=.*$",
                regex::escape(key)
            ))?;
            content = re.replace(&content, &export_line).to_string();
        } else {
            // 添加
            content.push_str(&format!("\n{}\n", export_line));
        }

        fs::write(&self.shell_config, content)?;
        Ok(())
    }
}
```

---

## 第五章 GUI开发指南

### 5.1 egui基础概念

#### 核心组件

```rust
// 1. 应用状态
pub struct DevHubApp {
    tools: Vec<ToolState>,
    current_view: View,
}

// 2. 视图枚举
pub enum View {
    Dashboard,
    ToolDetail(String),
    MirrorConfig,
}

// 3. 实现eframe::App trait
impl eframe::App for DevHubApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 每帧调用
    }
}
```

### 5.2 布局模式

#### 基本布局

```rust
use egui::{CentralPanel, TopBottomPanel, SidePanel};

impl DevHubApp {
    fn render_ui(&mut self, ctx: &egui::Context) {
        // 顶部面板（标题栏）
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("DevHub Pro");
        });

        // 左侧面板（导航）
        SidePanel::left("sidebar").show(ctx, |ui| {
            if ui.button("工具箱").clicked() {
                self.current_view = View::Dashboard;
            }
        });

        // 中央面板（主内容）
        CentralPanel::default().show(ctx, |ui| {
            match &self.current_view {
                View::Dashboard => self.render_dashboard(ui),
                _ => {}
            }
        });
    }
}
```

#### 组件示例

```rust
fn render_tool_card(&self, ui: &mut egui::Ui, tool: &ToolState) {
    egui::Frame::none()
        .fill(ui.visuals().panel_fill)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 状态图标
                if tool.detected {
                    ui.label(egui::RichText::new("✅").size(20.0));
                } else {
                    ui.label(egui::RichText::new("❌").size(20.0));
                }

                // 工具名称
                ui.label(egui::RichText::new(&tool.name).size(18.0));

                // 版本信息
                if let Some(ref version) = tool.version {
                    ui.label(format!("v{}", version));
                }

                ui.separator();

                // 操作按钮
                if ui.button("配置").clicked() {
                    // 打开配置页面
                }
            });
        });
}
```

### 5.3 异步操作处理

```rust
use std::sync::{Arc, Mutex};

pub struct DevHubApp {
    // 异步任务状态
    test_results: Arc<Mutex<Vec<SpeedResult>>>,
    testing: bool,
}

impl DevHubApp {
    fn start_speed_test(&mut self, ctx: egui::Context) {
        if self.testing {
            return;
        }

        self.testing = true;
        let results = self.test_results.clone();

        tokio::spawn(async move {
            let test_results = benchmark_mirrors(mirrors).await;

            *results.lock().unwrap() = test_results;
            ctx.request_repaint();  // 请求重绘
        });
    }
}
```

---

## 第六章 测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_domain_extraction() {
        assert_eq!(
            extract_domain("https://mirrors.aliyun.com/pypi/simple/"),
            "mirrors.aliyun.com"
        );
    }

    #[tokio::test]
    async fn test_pip_config_apply() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = PipMirrorConfig {
            config_path: temp_dir.path().join("pip.conf"),
        };

        let mirror = Mirror {
            name: "阿里云".to_string(),
            url: "https://mirrors.aliyun.com/pypi/simple/".to_string(),
        };

        assert!(config.apply(&mirror).is_ok());

        let content = fs::read_to_string(config.config_path).unwrap();
        assert!(content.contains("index-url"));
    }
}
```

### 6.2 集成测试

```rust
// tests/integration/python_tool_test.rs

use devhub::tools::python::PythonTool;

#[tokio::test]
async fn test_python_full_workflow() {
    let python = PythonTool::new();

    // 1. 检测
    let result = python.detector.detect().unwrap();
    assert!(result.installed);

    // 2. 镜像源配置
    let mirrors = python.pip_config.list_mirrors();
    assert!(!mirrors.is_empty());

    // 3. 应用镜像源
    python.pip_config.apply(&mirrors[0]).unwrap();
    let current = python.pip_config.get_current().unwrap();
    assert_eq!(current, Some(mirrors[0].url.clone()));

    // 4. 清理
    python.pip_config.restore_default().unwrap();
}
```

### 6.3 性能测试

```rust
// benches/speed_test.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use devhub::services::speed_test::test_mirror_speed;

fn bench_mirror_speed(c: &mut Criterion) {
    let mirrors = vec![
        "https://mirrors.aliyun.com",
        "https://mirrors.tuna.tsinghua.edu.cn",
    ];

    for mirror in mirrors {
        c.bench_with_input(
            BenchmarkId::new("mirror_speed", mirror),
            &mirror,
            |b, url| {
                b.iter(|| {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(test_mirror_speed(black_box(url)))
                });
            },
        );
    }
}

criterion_group!(benches, bench_mirror_speed);
criterion_main!(benches);
```

---

## 第七章 部署与发布

### 7.1 构建发布版本

```bash
# 编译优化版本
cargo build --release

# 输出路径
# target/release/devhub (macOS/Linux)
# target/release/devhub.exe (Windows)
```

### 7.2 打包为可执行文件

```bash
# macOS
codesign --force --deep --sign "Developer ID Application: Your Name" target/release/devhub

# 创建DMG安装包
hdiutil create -volname "DevHub Pro" -srcfolder target/release/ -ov -format UDZO devhub.dmg

# Windows（使用NSIS）
# 1. 下载NSIS: https://nsis.sourceforge.io/
# 2. 创建installer.nsi
# 3. makensis installer.nsi

# Linux
tar -czf devhub-linux-x86_64.tar.gz -C target/release devhub
```

### 7.3 Homebrew发布（macOS）

```ruby
# Formula/devhub.rb
class Devhub < Formula
  desc "一站式开发环境管理平台"
  homepage "https://github.com/yourusername/devhub"
  url "https://github.com/yourusername/devhub/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "your-sha256-hash"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/devhub", "--version"
  end
end
```

### 7.4 CI/CD配置

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest, windows-latest]
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: devhub-${{ matrix.target }}
          path: target/release/devhub*
```

---

## 第八章 常见问题与注意事项

### 8.1 Rust常见陷阱

#### 1. 所有权转移

```rust
// ❌ 错误
let v1 = vec![1, 2, 3];
let v2 = v1;
println!("{:?}", v1);  // 编译错误：value borrowed here after move

// ✅ 正确
let v1 = vec![1, 2, 3];
let v2 = v1.clone();
println!("{:?}", v1);  // OK
```

#### 2. 字符串类型

```rust
// &str - 字符串切片（引用）
let s1: &str = "hello";

// String - 堆分配字符串（拥有所有权）
let s2: String = String::from("hello");

// 转换
let s3: String = s1.to_string();
let s4: &str = &s2;
```

#### 3. 生命周期

```rust
// ❌ 错误
fn longest(s1: &str, s2: &str) -> &str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}

// ✅ 正确（明确生命周期）
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}
```

### 8.2 性能优化建议

#### 1. 减少克隆

```rust
// ❌ 频繁克隆
fn process(data: Vec<u8>) {
    let cloned = data.clone();
    // ...
}

// ✅ 使用引用
fn process(data: &Vec<u8>) {
    // ...
}
```

#### 2. 异步并发

```rust
// ❌ 串行执行
for mirror in mirrors {
    test_speed(&mirror).await;
}

// ✅ 并发执行
let tasks: Vec<_> = mirrors
    .into_iter()
    .map(|m| test_speed(&m))
    .collect();
futures::future::join_all(tasks).await;
```

#### 3. 使用Release模式

```bash
# 开发模式（无优化，编译快）
cargo run

# 发布模式（全优化，运行快）
cargo run --release
```

### 8.3 调试技巧

#### 1. 使用dbg!宏

```rust
let x = 5;
let y = dbg!(x * 2);  // 打印：[src/main.rs:2] x * 2 = 10
```

#### 2. 环境变量日志

```bash
# 设置日志级别
RUST_LOG=debug cargo run

# 只显示某个模块的日志
RUST_LOG=devhub::tools::python=trace cargo run
```

#### 3. 可视化调用图

```bash
# 安装cargo-flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin devhub
```

### 8.4 GUI开发注意事项

#### 1. 避免阻塞UI线程

```rust
// ❌ 阻塞UI
fn on_click(&mut self) {
    let result = blocking_operation();  // 阻塞
    self.result = result;
}

// ✅ 异步处理
fn on_click(&mut self, ctx: egui::Context) {
    tokio::spawn(async move {
        let result = blocking_operation().await;
        // 通过channel发送回UI
        ctx.request_repaint();
    });
}
```

#### 2. 保持UI流畅

```rust
// 使用ctx.request_repaint()请求重绘
// 而不是频繁更新状态

impl eframe::App for DevHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置连续重绘（用于动画）
        ctx.request_repaint();

        // 或设置定时重绘
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
```

### 8.5 跨平台兼容性

#### 平台检测

```rust
use std::env;

fn get_platform() -> &'static str {
    env::consts::OS  // "windows", "macos", "linux"
}

fn get_arch() -> &'static str {
    env::consts::ARCH  // "x86_64", "aarch64", etc.
}

// 使用
if cfg!(target_os = "macos") {
    // macOS特定代码
}
```

#### 路径处理

```rust
use std::path::PathBuf;

// 跨平台路径拼接
let path = PathBuf::from("folder")
    .join("subfolder")
    .join("file.txt");

// 获取配置目录
let config_dir = dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."));
```

---

## 附录A：Rust学习资源

- [The Rust Book](https://doc.rust-lang.org/book/) - 官方教程
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - 示例驱动学习
- [egui文档](https://docs.rs/egui/) - GUI框架文档
- [Async Rust Book](https://rust-lang.github.io/async-book/) - 异步编程

---

## 附录B：常用Cargo插件

```bash
# 安装有用的工具
cargo install cargo-edit      # cargo add命令
cargo install cargo-watch     # 自动重新编译
cargo install cargo-audit     # 安全审计
cargo install cargo-outdated  # 检查更新
cargo install cargo-tree      # 依赖树可视化

# 使用示例
cargo add serde --features derive  # 添加依赖
cargo watch -x check               # 监视文件变化
cargo audit                       # 安全检查
```

---

**文档版本：** v1.0
**最后更新：** 2025-01-12
**作者：** DevHub Pro Team
