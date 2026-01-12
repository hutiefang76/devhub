# DevHub Pro - 技术架构文档

> **遵循SOLID原则的架构设计**

---

## ⚠️ 架构变更说明

### 原计划 vs 实际实现

**原计划**: 使用egui构建纯Rust GUI应用
**实际采用**: Tauri (Rust后端 + React前端)

### 变更原因

经过技术评估,最终选择Tauri方案,主要基于以下考量:

| 考量维度 | egui方案 | Tauri方案 | 选择依据 |
|---------|---------|----------|---------|
| **UI能力** | Immediate Mode GUI,实现复杂UI困难 | HTML/CSS/React,Web级UI能力 | ✅ 需要现代化界面("优雅"要求) |
| **开发效率** | 手写GUI代码,开发慢 | React组件复用,开发快 | ✅ 10倍效率提升 |
| **样式定制** | 需手写样式代码,困难 | CSS/Tailwind,简单 | ✅ 符合"简单"要求 |
| **二进制大小** | 1-3MB | 4.6MB | ⚠️ 略大但仍符合"轻巧"要求 |
| **跨平台支持** | 支持 | 支持(WebView更成熟) | ✅ 完全满足 |
| **中文支持** | 需手动配置字体 | Web字体成熟,完美支持 | ✅ 开箱即用 |

### 核心决策

用户要求**"优雅、轻巧、简单、跨平台"**:
- ✅ **优雅**: Tauri + React远超egui的UI能力
- ✅ **轻巧**: 4.6MB vs 预期1-3MB,差异可接受
- ✅ **简单**: 前端用React比手写egui代码简单10倍
- ✅ **跨平台**: 都支持,Tauri的WebView更成熟

**结论**: Tauri在保证后三者的同时,在"优雅"上有显著优势,这是正确的技术选择。

---

## 🏗️ 整体架构 (Tauri实现)

### 分层架构

```
┌──────────────────────────────────────────────┐
│       Frontend (React + TypeScript)          │  ← 用户界面
│           (HTML/CSS/React)                   │
├──────────────────────────────────────────────┤
│          IPC Layer (Tauri Commands)          │  ← 接口层
│            (#[tauri::command])               │
├──────────────────────────────────────────────┤
│     Tools Layer (Python/Node/Java)           │  ← 工具实现
│   (PythonDetector, PipMirror, etc.)          │
├──────────────────────────────────────────────┤
│        Core Traits (抽象层)                   │  ← 业务抽象
│   (ToolDetector, MirrorConfigurator)         │
├──────────────────────────────────────────────┤
│          Services (基础服务)                  │  ← 底层服务
│      (ShellExecutor, SpeedTestService)       │
└──────────────────────────────────────────────┘
```

### Tauri架构特点

1. **前后端分离**: Rust处理业务逻辑,React处理UI渲染
2. **类型安全IPC**: Tauri自动序列化/反序列化,编译期保证类型正确
3. **异步支持**: IPC命令支持async fn,无需手动管理线程
4. **Web技术**: 利用成熟的前端生态(React/Vite/TypeScript)

---

## 📐 核心Trait设计（依赖倒置原则）

### 1. ToolDetector - 软件检测器

**职责：** 单一 - 检测软件是否安装

```rust
// src/core/detector.rs

use async_trait::async_trait;
use std::path::PathBuf;

/// 检测结果
#[derive(Debug, Clone)]
pub struct DetectionInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
}

/// 软件检测器Trait
#[async_trait]
pub trait ToolDetector: Send + Sync {
    /// 检测软件状态
    fn detect(&self) -> Result<DetectionInfo>;
}
```

**实现示例：**

```rust
// src/tools/detectors/shell_detector.rs

use crate::core::detector::*;
use crate::services::shell::ShellExecutor;

/// Shell命令检测器 - 通用实现
pub struct ShellDetector {
    tool_name: String,
    version_flag: String,
}

impl ShellDetector {
    pub fn new(tool_name: &str, version_flag: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            version_flag: version_flag.to_string(),
        }
    }
}

impl ToolDetector for ShellDetector {
    fn detect(&self) -> Result<DetectionInfo> {
        let executor = ShellExecutor;

        // 1. 检测路径
        let path = executor
            .exec("which", &[&self.tool_name])
            .ok()
            .filter(|o| o.status)
            .map(|o| PathBuf::from(o.stdout.trim()));

        // 2. 检测版本
        let version = if path.is_some() {
            executor
                .get_version(&self.tool_name, &self.version_flag)
                .ok()
                .flatten()
        } else {
            None
        };

        Ok(DetectionInfo {
            installed: path.is_some(),
            version,
            path,
        })
    }
}
```

---

### 2. MirrorConfigurator - 镜像源配置器

**职责：** 单一 - 配置镜像源

```rust
// src/core/mirror.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 镜像源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    pub name: String,
    pub url: String,
}

/// 测速结果
#[derive(Debug, Clone)]
pub struct SpeedResult {
    pub mirror: Mirror,
    pub latency_ms: u64,
}

/// 镜像源配置器Trait
#[async_trait]
pub trait MirrorConfigurator: Send + Sync {
    /// 获取当前镜像源
    fn get_current(&self) -> Result<Option<String>>;

    /// 列出可用镜像源
    fn list_mirrors(&self) -> Vec<Mirror>;

    /// 应用镜像源
    fn apply(&self, mirror: &Mirror) -> Result<()>;

    /// 测试镜像源速度
    async fn test_speed(&self) -> Result<Vec<SpeedResult>>;
}
```

**实现示例：**

```rust
// src/tools/mirrors/pip_mirror.rs

use crate::core::mirror::*;
use crate::services::speed_test::SpeedTestService;
use std::path::PathBuf;

/// pip镜像源配置器
pub struct PipMirrorConfig {
    config_path: PathBuf,
}

impl PipMirrorConfig {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pip");

        Self {
            config_path: config_dir.join("pip.conf"),
        }
    }

    /// 提取域名
    fn extract_domain(url: &str) -> String {
        url.replace("https://", "")
            .replace("http://", "")
            .split('/')
            .next()
            .unwrap_or("mirrors.aliyun.com")
            .to_string()
    }
}

impl MirrorConfigurator for PipMirrorConfig {
    fn get_current(&self) -> Result<Option<String>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        let re = regex::Regex::new(r"index-url\s*=\s*(.+)")?;

        Ok(re.captures(&content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string()))
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        vec![
            Mirror {
                name: "阿里云".to_string(),
                url: "https://mirrors.aliyun.com/pypi/simple/".to_string(),
            },
            Mirror {
                name: "清华".to_string(),
                url: "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(),
            },
            Mirror {
                name: "腾讯云".to_string(),
                url: "https://mirrors.cloud.tencent.com/pypi/simple".to_string(),
            },
            Mirror {
                name: "豆瓣".to_string(),
                url: "https://pypi.doubanio.com/simple".to_string(),
            },
        ]
    }

    fn apply(&self, mirror: &Mirror) -> Result<()> {
        // 创建配置目录
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 写入配置（KISS原则：简单直接）
        let content = format!(
            "[global]\nindex-url = {}\ntrusted-host = {}\n",
            mirror.url,
            Self::extract_domain(&mirror.url)
        );

        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    async fn test_speed(&self) -> Result<Vec<SpeedResult>> {
        let speed_test = SpeedTestService::new();
        let mirrors = self.list_mirrors();

        let mut results = Vec::new();
        for mirror in mirrors {
            let latency = speed_test.test_mirror(&mirror.url).await;
            results.push(SpeedResult {
                mirror,
                latency_ms: latency,
            });
        }

        Ok(results)
    }
}
```

---

### 3. CommandExecutor - 命令执行器

**职责：** 单一 - 执行Shell命令

```rust
// src/core/command.rs

use std::process::Output;

/// 命令输出
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: bool,
    pub stdout: String,
    pub stderr: String,
}

/// 命令执行器Trait
pub trait CommandExecutor: Send + Sync {
    /// 执行命令
    fn exec(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput>;

    /// 获取版本号
    fn get_version(&self, cmd: &str, flag: &str) -> Result<Option<String>>;
}
```

**实现示例：**

```rust
// src/services/shell.rs

use crate::core::command::*;
use std::process::Command;

/// Shell命令执行器
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

    fn get_version(&self, cmd: &str, flag: &str) -> Result<Option<String>> {
        match self.exec(cmd, &[flag]) {
            Ok(output) if output.status => {
                let version = output
                    .stdout
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();

                Ok(Some(version))
            }
            _ => Ok(None),
        }
    }
}
```

---

## 🎨 Tauri IPC架构

### Tauri Command定义

```rust
// src/commands/mod.rs

use crate::core::detector::*;
use crate::core::mirror::*;
use crate::tools::python::*;

/// 检测Python环境
///
/// # 前端调用
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri'
///
/// const info = await invoke<DetectionInfo>('detect_python')
/// console.log('Python已安装:', info.installed)
/// ```
#[tauri::command]
pub fn detect_python() -> Result<DetectionInfo, String> {
    let detector = PythonDetector::new();
    detector.detect().map_err(|e| e.to_string())
}

/// 获取当前pip镜像源
#[tauri::command]
pub fn get_current_pip_mirror() -> Result<Option<String>, String> {
    let config = PipMirror::new();
    config.get_current().map_err(|e| e.to_string())
}

/// 列出pip镜像源
#[tauri::command]
pub fn list_pip_mirrors() -> Vec<Mirror> {
    let config = PipMirror::new();
    config.list_mirrors()
}

/// 应用pip镜像源
#[tauri::command]
pub fn apply_pip_mirror(mirror: Mirror) -> Result<(), String> {
    let config = PipMirror::new();
    config.apply(&mirror).map_err(|e| e.to_string())
}

/// 批量测试镜像源速度(异步)
#[tauri::command]
pub async fn test_mirrors_speed(urls: Vec<String>) -> Vec<u64> {
    let service = SpeedTestService::new();
    service.test_mirrors(urls).await
}
```

### React前端集成

```typescript
// frontend/src/App.tsx

import { invoke } from '@tauri-apps/api/tauri'

interface DetectionInfo {
  installed: boolean
  version?: string
  path?: string
}

interface Mirror {
  name: string
  url: string
}

function App() {
  const [pythonInfo, setPythonInfo] = useState<DetectionInfo | null>(null)
  const [mirrors, setMirrors] = useState<Mirror[]>([])
  const [speeds, setSpeeds] = useState<number[]>([])

  // 检测Python
  const detectPython = async () => {
    const info = await invoke<DetectionInfo>('detect_python')
    setPythonInfo(info)
  }

  // 列出镜像源
  const loadMirrors = async () => {
    const list = await invoke<Mirror[]>('list_pip_mirrors')
    setMirrors(list)
  }

  // 测试速度
  const testSpeed = async () => {
    const urls = mirrors.map(m => m.url)
    const results = await invoke<number[]>('test_mirrors_speed', { urls })
    setSpeeds(results)
  }

  // 应用镜像源
  const applyMirror = async (mirror: Mirror) => {
    await invoke('apply_pip_mirror', { mirror })
  }

  return (
    <div>
      {/* UI组件 */}
    </div>
  )
}
```

---

## 📊 数据流设计 (Tauri实现)

### 检测流程

```
用户点击"检测"按钮 (React)
    ↓
调用 invoke('detect_python')
    ↓
Tauri IPC → Rust后端
    ↓
detect_python() command
    ↓
创建 PythonDetector
    ↓
detector.detect()
    ↓
ShellExecutor.exec("which", "python3")
    ↓
解析输出 → DetectionInfo
    ↓
序列化为JSON
    ↓
IPC → React前端
    ↓
setState更新UI
```

### 配置流程

```
用户选择镜像源 (React)
    ↓
调用 invoke('apply_pip_mirror', { mirror })
    ↓
Tauri IPC → Rust后端
    ↓
apply_pip_mirror(mirror) command
    ↓
创建 PipMirror
    ↓
configurator.apply(&mirror)
    ↓
写入 ~/.pip/pip.conf
    ↓
返回 Result<()>
    ↓
IPC → React前端
    ↓
显示成功提示
```

### 异步测速流程

```
用户点击"⚡ 测速" (React)
    ↓
调用 invoke('test_mirrors_speed', { urls })
    ↓
Tauri IPC → Rust后端
    ↓
async test_mirrors_speed(urls)
    ↓
创建 SpeedTestService
    ↓
Tokio并发执行HTTP HEAD请求
    ↓
收集延迟结果 → Vec<u64>
    ↓
序列化为JSON
    ↓
IPC → React前端
    ↓
setState更新延迟显示
```

---

## 🎯 设计模式应用

### 1. 策略模式（Strategy）

```rust
// 不同的检测策略
pub trait ToolDetector {
    fn detect(&self) -> Result<DetectionInfo>;
}

pub struct ShellDetector;
pub struct PathDetector;
pub struct RegistryDetector;  // Windows
```

### 2. 工厂模式（Factory）

```rust
pub trait ToolFactory {
    fn create_detector(&self) -> Box<dyn ToolDetector>;
    fn create_mirror_config(&self) -> Box<dyn MirrorConfigurator>;
}

pub struct PythonToolFactory;
impl ToolFactory for PythonToolFactory {
    fn create_detector(&self) -> Box<dyn ToolDetector> {
        Box::new(ShellDetector::new("python3", "--version"))
    }

    fn create_mirror_config(&self) -> Box<dyn MirrorConfigurator> {
        Box::new(PipMirrorConfig::new())
    }
}
```

### 3. 模板方法模式（Template Method）

```rust
pub struct ToolTemplate<D, M> {
    detector: D,
    mirror_config: M,
}

impl<D, M> ToolTemplate<D, M>
where
    D: ToolDetector,
    M: MirrorConfigurator,
{
    pub async fn detect_and_configure(&self) -> Result<()> {
        // 1. 检测
        let info = self.detector.detect()?;

        // 2. 配置
        if info.installed {
            let mirrors = self.mirror_config.list_mirrors();
            // ...
        }

        Ok(())
    }
}
```

---

## 📦 模块依赖关系 (Tauri实现)

```
┌─────────────────┐
│  frontend/      │  ← React用户界面
│  (React/TS)     │
└────────┬────────┘
         │ IPC (Tauri)
         ↓
┌─────────────────┐
│  commands/      │  ← Tauri命令层
│  (IPC接口)      │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  tools/         │  ← 工具实现层
│  (Python/Node)  │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  core/          │  ← 核心trait抽象
│  (Trait定义)    │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│ services/       │  ← 基础服务
│ (Shell/HTTP)    │
└─────────────────┘
```

**关键特点:**
- ✅ 前端通过Tauri IPC与后端通信(无直接依赖)
- ✅ Commands层是前后端边界
- ✅ Tools层实现具体工具逻辑
- ✅ Core定义抽象接口(依赖倒置)
- ✅ Services提供基础能力

---

## 🔄 Tauri开发工作流

### 开发模式
```bash
# 终端1: 启动前端开发服务器
cd frontend && npm run dev

# 终端2: 启动Tauri应用
cargo run
```

### 生产构建
```bash
# 1. 构建前端
cd frontend && npm run build

# 2. 构建Rust后端
cd .. && cargo build --release

# 输出: target/release/devhub (4.6MB)
```

### 添加新IPC接口
1. 在`src/commands/mod.rs`添加`#[tauri::command]`函数
2. 在`src/main.rs`的`tauri::Builder`中注册命令
3. 在前端使用`invoke('command_name')`调用

---

## 🎨 SOLID原则应用总结

### S - 单一职责原则

每个trait只负责一件事：
- `ToolDetector` - 只负责检测
- `MirrorConfigurator` - 只负责配置镜像源
- `CommandExecutor` - 只负责执行命令

### O - 开闭原则

通过trait扩展，无需修改现有代码：
```rust
// 添加新工具，只需实现trait
impl ToolDetector for RustDetector { }
```

### L - 里氏替换原则

子类型可以替换父类型：
```rust
let detector: Box<dyn ToolDetector> = Box::new(ShellDetector::new("python3", "--version"));
```

### I - 接口隔离原则

接口小而专一：
```rust
// 好的设计
trait ToolDetector {
    fn detect(&self) -> Result<DetectionInfo>;
}

// 避免胖接口
trait ToolManager {  // ❌ 不好
    fn detect(&self) -> Result<()>;
    fn install(&self) -> Result<()>;
    fn configure(&self) -> Result<()>;
    // ... 太多职责
}
```

### D - 依赖倒置原则

依赖抽象而非具体实现：
```rust
// 好 - 依赖trait
pub struct ToolManager {
    detector: Box<dyn ToolDetector>,  // 抽象
}

// 不好 - 依赖具体实现
pub struct ToolManager {
    detector: ShellDetector,  // 具体
}
```

---

## 📄 文档信息

**文档版本:** v2.0 (Tauri实现)
**最后更新:** 2026-01-12
**作者:** Frank Hu <hutiefang@gmail.com>
**架构决策:** 采用Tauri替代egui,优先考虑UI优雅性
**技术栈:** Rust + Tauri 1.8 + React 18 + TypeScript
