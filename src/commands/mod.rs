use crate::sources::{get_manager, SUPPORTED_TOOLS};
use crate::types::Mirror;
use crate::utils::benchmark_mirrors;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,           // "macos", "linux", "windows"
    pub os_version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_path: Option<String>,
    pub config_path: Option<String>,
    pub supported_on_current_os: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub current_url: Option<String>,
    pub current_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub name: String,
    pub url: String,
    pub latency_ms: u64,
    pub is_timeout: bool,
}

#[tauri::command]
pub fn list_supported_tools() -> Vec<String> {
    SUPPORTED_TOOLS.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
pub async fn get_tool_status(name: String) -> Result<ToolStatus, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let current_url = manager.current_url().await.map_err(|e| e.to_string())?;
    let candidates = manager.list_candidates();

    let current_name = current_url.as_ref().and_then(|url| {
        candidates.iter()
            .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
            .map(|m| m.name.clone())
    });

    Ok(ToolStatus {
        name: manager.name().to_string(),
        current_url,
        current_name,
    })
}

#[tauri::command]
pub async fn get_all_status() -> Vec<ToolStatus> {
    let mut results = Vec::new();
    for tool in SUPPORTED_TOOLS {
        if let Ok(status) = get_tool_status(tool.to_string()).await {
            results.push(status);
        }
    }
    results
}

#[tauri::command]
pub fn list_mirrors(name: String) -> Result<Vec<Mirror>, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    Ok(manager.list_candidates())
}

#[tauri::command]
pub async fn test_mirrors(name: String) -> Result<Vec<SpeedTestResult>, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let mirrors = manager.list_candidates();
    let results = benchmark_mirrors(mirrors).await;

    Ok(results.into_iter().map(|r| SpeedTestResult {
        name: r.mirror.name,
        url: r.mirror.url,
        latency_ms: if r.latency_ms == u64::MAX { 0 } else { r.latency_ms },
        is_timeout: r.latency_ms == u64::MAX,
    }).collect())
}

#[tauri::command]
pub async fn test_single_mirror(url: String) -> Result<u64, String> {
    let mirror = Mirror::new("test", &url);
    let results = benchmark_mirrors(vec![mirror]).await;
    if let Some(r) = results.first() {
        if r.latency_ms == u64::MAX {
            Ok(9999)
        } else {
            Ok(r.latency_ms)
        }
    } else {
        Ok(9999)
    }
}

#[tauri::command]
pub async fn apply_mirror(name: String, mirror: Mirror) -> Result<(), String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    manager.set_source(&mirror).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_default(name: String) -> Result<(), String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    manager.restore().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_fastest_mirror(name: String) -> Result<Mirror, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let mirrors = manager.list_candidates();
    let results = benchmark_mirrors(mirrors).await;

    let fastest = results.into_iter()
        .filter(|r| r.latency_ms < u64::MAX)
        .min_by_key(|r| r.latency_ms)
        .ok_or_else(|| "所有镜像源均超时".to_string())?;

    manager.set_source(&fastest.mirror).await.map_err(|e| e.to_string())?;
    Ok(fastest.mirror)
}

#[tauri::command]
pub async fn sync_java_mirrors(mirror_name: String) -> Result<(), String> {
    let maven_manager = get_manager("maven").map_err(|e| e.to_string())?;
    let gradle_manager = get_manager("gradle").map_err(|e| e.to_string())?;

    let maven_mirrors = maven_manager.list_candidates();
    let gradle_mirrors = gradle_manager.list_candidates();

    let maven_mirror = maven_mirrors.iter()
        .find(|m| m.name.eq_ignore_ascii_case(&mirror_name))
        .ok_or_else(|| format!("Maven 镜像源 {} 不存在", mirror_name))?;

    let gradle_mirror = gradle_mirrors.iter()
        .find(|m| m.name.eq_ignore_ascii_case(&mirror_name))
        .ok_or_else(|| format!("Gradle 镜像源 {} 不存在", mirror_name))?;

    maven_manager.set_source(maven_mirror).await.map_err(|e| e.to_string())?;
    gradle_manager.set_source(gradle_mirror).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    SystemInfo {
        os: os.to_string(),
        os_version: std::env::consts::OS.to_string(),
        arch: arch.to_string(),
    }
}

// 工具在各平台的支持情况
fn is_tool_supported_on_os(tool: &str, os: &str) -> bool {
    match tool {
        // 系统包管理器 - 平台特定
        "brew" => os == "macos" || os == "linux",
        "apt" => os == "linux",
        "choco" => os == "windows",
        // 其他工具 - 全平台
        _ => true,
    }
}

// 获取工具版本命令
fn get_version_command(tool: &str) -> Option<(&str, &[&str])> {
    match tool {
        "pip" => Some(("pip3", &["--version"])),
        "uv" => Some(("uv", &["--version"])),
        "conda" => Some(("conda", &["--version"])),
        "npm" => Some(("npm", &["--version"])),
        "yarn" => Some(("yarn", &["--version"])),
        "pnpm" => Some(("pnpm", &["--version"])),
        "cargo" => Some(("cargo", &["--version"])),
        "go" => Some(("go", &["version"])),
        "maven" => Some(("mvn", &["--version"])),
        "gradle" => Some(("gradle", &["--version"])),
        "docker" => Some(("docker", &["--version"])),
        "brew" => Some(("brew", &["--version"])),
        "choco" => Some(("choco", &["--version"])),
        "apt" => Some(("apt", &["--version"])),
        "git" => Some(("git", &["--version"])),
        _ => None,
    }
}

// 获取工具配置文件路径
fn get_config_path(tool: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let path = match tool {
        "pip" => home.join(".config/pip/pip.conf"),
        "uv" => home.join(".config/uv/uv.toml"),
        "conda" => home.join(".condarc"),
        "npm" | "pnpm" => home.join(".npmrc"),
        "yarn" => home.join(".yarnrc"),
        "cargo" => home.join(".cargo/config.toml"),
        "go" => return Some("go env GOPROXY".to_string()),
        "maven" => home.join(".m2/settings.xml"),
        "gradle" => home.join(".gradle/init.gradle"),
        "docker" => {
            if cfg!(target_os = "macos") {
                home.join("Library/Group Containers/group.com.docker/settings.json")
            } else if cfg!(target_os = "linux") {
                std::path::PathBuf::from("/etc/docker/daemon.json")
            } else {
                home.join(".docker/daemon.json")
            }
        },
        "git" => home.join(".gitconfig"),
        _ => return None,
    };
    Some(path.to_string_lossy().to_string())
}

// 获取工具安装路径
fn get_install_path(tool: &str) -> Option<String> {
    let cmd = match tool {
        "pip" => "which pip3",
        "uv" => "which uv",
        "conda" => "which conda",
        "npm" => "which npm",
        "yarn" => "which yarn",
        "pnpm" => "which pnpm",
        "cargo" => "which cargo",
        "go" => "which go",
        "maven" => "which mvn",
        "gradle" => "which gradle",
        "docker" => "which docker",
        "brew" => "which brew",
        "apt" => "which apt",
        "git" => "which git",
        _ => return None,
    };

    #[cfg(target_os = "windows")]
    let cmd = cmd.replace("which", "where");

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

#[tauri::command]
pub fn get_tool_info(name: String) -> ToolInfo {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let supported = is_tool_supported_on_os(&name, os);

    // 如果不支持当前系统，直接返回
    if !supported {
        return ToolInfo {
            name: name.clone(),
            installed: false,
            version: None,
            install_path: None,
            config_path: get_config_path(&name),
            supported_on_current_os: false,
        };
    }

    // 检测版本
    let version = if let Some((cmd, args)) = get_version_command(&name) {
        Command::new(cmd)
            .args(args)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    // 提取版本号
                    Some(extract_version(&v))
                } else {
                    None
                }
            })
    } else {
        None
    };

    let installed = version.is_some();
    let install_path = if installed { get_install_path(&name) } else { None };
    let config_path = get_config_path(&name);

    ToolInfo {
        name,
        installed,
        version,
        install_path,
        config_path,
        supported_on_current_os: supported,
    }
}

// 提取版本号
fn extract_version(output: &str) -> String {
    // 尝试从输出中提取版本号
    let first_line = output.lines().next().unwrap_or(output);

    // 常见格式: "pip 23.0.1", "npm 10.2.0", "go version go1.21.0"
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    for part in &parts {
        // 检查是否像版本号 (包含数字和点)
        if part.chars().any(|c| c.is_numeric()) && part.contains('.') {
            // 移除前缀 v 或 go
            let version = part.trim_start_matches('v').trim_start_matches("go");
            return version.to_string();
        }
    }

    first_line.to_string()
}

#[tauri::command]
pub fn get_all_tools_info() -> Vec<ToolInfo> {
    SUPPORTED_TOOLS.iter().map(|t| get_tool_info(t.to_string())).collect()
}

// ============================================
// 版本管理功能
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    pub path: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManagerInfo {
    pub manager_name: String,      // "pyenv", "jenv", "nvm", "rustup"
    pub installed: bool,
    pub current_version: Option<String>,
    pub versions: Vec<InstalledVersion>,
    pub env_var_name: Option<String>,   // "JAVA_HOME", "PYENV_ROOT"
    pub env_var_value: Option<String>,
    pub is_consistent: bool,    // 环境变量与版本管理器是否一致
    pub inconsistency_message: Option<String>,
}

// 获取工具对应的版本管理器
fn get_version_manager_for_tool(tool: &str) -> Option<&'static str> {
    match tool {
        "pip" | "uv" | "conda" => Some("pyenv"),
        "maven" | "gradle" => Some("jenv"),
        "npm" | "yarn" | "pnpm" => Some("nvm"),
        "cargo" => Some("rustup"),
        "go" => Some("goenv"),
        _ => None,
    }
}

// 检测版本管理器是否安装
fn is_version_manager_installed(manager: &str) -> bool {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", manager))
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

// 获取pyenv版本列表
fn get_pyenv_versions() -> Vec<InstalledVersion> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pyenv versions --bare")
        .output();

    let mut versions = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // 获取当前版本
            let current = Command::new("sh")
                .arg("-c")
                .arg("pyenv version-name")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            for line in stdout.lines() {
                let version = line.trim().to_string();
                if !version.is_empty() {
                    let is_current = current.as_ref().map(|c| c == &version).unwrap_or(false);
                    versions.push(InstalledVersion {
                        version: version.clone(),
                        path: None,
                        is_current,
                    });
                }
            }
        }
    }

    versions
}

// 获取jenv版本列表
fn get_jenv_versions() -> Vec<InstalledVersion> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("jenv versions --bare 2>/dev/null")
        .output();

    let mut versions = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // 获取当前版本
            let current = Command::new("sh")
                .arg("-c")
                .arg("jenv version-name 2>/dev/null")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            for line in stdout.lines() {
                let version = line.trim().to_string();
                if !version.is_empty() && !version.starts_with("system") {
                    let is_current = current.as_ref().map(|c| c == &version).unwrap_or(false);

                    // 获取JDK路径
                    let path = Command::new("sh")
                        .arg("-c")
                        .arg(format!("jenv prefix {} 2>/dev/null", version))
                        .output()
                        .ok()
                        .and_then(|o| {
                            if o.status.success() {
                                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                            } else {
                                None
                            }
                        });

                    versions.push(InstalledVersion {
                        version,
                        path,
                        is_current,
                    });
                }
            }
        }
    }

    versions
}

// 获取nvm版本列表
fn get_nvm_versions() -> Vec<InstalledVersion> {
    // nvm 是 shell 函数，需要通过 bash 加载
    let output = Command::new("bash")
        .arg("-c")
        .arg("source ~/.nvm/nvm.sh 2>/dev/null && nvm list --no-colors 2>/dev/null")
        .output();

    let mut versions = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("->") && line.contains("N/A") {
                    continue;
                }

                let is_current = line.starts_with("->");
                let version = line
                    .trim_start_matches("->")
                    .trim_start_matches("*")
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();

                if !version.is_empty() && version.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                    versions.push(InstalledVersion {
                        version,
                        path: None,
                        is_current,
                    });
                }
            }
        }
    }

    versions
}

// 获取rustup版本列表
fn get_rustup_versions() -> Vec<InstalledVersion> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("rustup show")
        .output();

    let mut versions = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut in_toolchains = false;

            for line in stdout.lines() {
                if line.contains("installed toolchains") {
                    in_toolchains = true;
                    continue;
                }
                if line.contains("active toolchain") {
                    break;
                }

                if in_toolchains && !line.is_empty() {
                    let is_current = line.contains("(default)");
                    let version = line
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !version.is_empty() && !version.starts_with('-') {
                        versions.push(InstalledVersion {
                            version,
                            path: None,
                            is_current,
                        });
                    }
                }
            }
        }
    }

    versions
}

#[tauri::command]
pub fn get_version_manager_info(tool: String) -> Option<VersionManagerInfo> {
    let manager_name = get_version_manager_for_tool(&tool)?;
    let installed = is_version_manager_installed(manager_name);

    if !installed {
        return Some(VersionManagerInfo {
            manager_name: manager_name.to_string(),
            installed: false,
            current_version: None,
            versions: vec![],
            env_var_name: None,
            env_var_value: None,
            is_consistent: true,
            inconsistency_message: None,
        });
    }

    let versions = match manager_name {
        "pyenv" => get_pyenv_versions(),
        "jenv" => get_jenv_versions(),
        "nvm" => get_nvm_versions(),
        "rustup" => get_rustup_versions(),
        _ => vec![],
    };

    let current_version = versions.iter()
        .find(|v| v.is_current)
        .map(|v| v.version.clone());

    // 检查环境变量一致性
    let (env_var_name, env_var_value, is_consistent, inconsistency_message) =
        check_version_consistency(manager_name, &current_version);

    Some(VersionManagerInfo {
        manager_name: manager_name.to_string(),
        installed,
        current_version,
        versions,
        env_var_name,
        env_var_value,
        is_consistent,
        inconsistency_message,
    })
}

// 检查版本一致性
fn check_version_consistency(
    manager: &str,
    current_version: &Option<String>,
) -> (Option<String>, Option<String>, bool, Option<String>) {
    match manager {
        "jenv" => {
            let java_home = std::env::var("JAVA_HOME").ok();

            if let (Some(jenv_ver), Some(home)) = (current_version, &java_home) {
                // 检查 JAVA_HOME 是否包含 jenv 设置的版本
                let is_consistent = home.contains(jenv_ver) ||
                    home.contains(&jenv_ver.replace(".", ""));

                let message = if !is_consistent {
                    Some(format!(
                        "JAVA_HOME ({}) 与 jenv 当前版本 ({}) 不一致",
                        home, jenv_ver
                    ))
                } else {
                    None
                };

                (Some("JAVA_HOME".to_string()), java_home, is_consistent, message)
            } else {
                (Some("JAVA_HOME".to_string()), java_home, true, None)
            }
        }
        "pyenv" => {
            let pyenv_root = std::env::var("PYENV_ROOT").ok();
            // pyenv 一般通过 shims 工作，不需要严格检查
            (Some("PYENV_ROOT".to_string()), pyenv_root, true, None)
        }
        _ => (None, None, true, None),
    }
}

#[tauri::command]
pub fn switch_version(tool: String, version: String) -> Result<String, String> {
    let manager_name = get_version_manager_for_tool(&tool)
        .ok_or_else(|| format!("工具 {} 没有对应的版本管理器", tool))?;

    let cmd = match manager_name {
        "pyenv" => format!("pyenv global {}", version),
        "jenv" => format!("jenv global {}", version),
        "nvm" => format!("source ~/.nvm/nvm.sh && nvm alias default {}", version),
        "rustup" => format!("rustup default {}", version),
        _ => return Err(format!("不支持的版本管理器: {}", manager_name)),
    };

    let shell = if manager_name == "nvm" { "bash" } else { "sh" };

    let output = Command::new(shell)
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        Ok(format!("已切换到版本 {}", version))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("切换失败: {}", stderr))
    }
}

// 获取工具的安装命令
fn get_install_command(tool: &str, os: &str) -> Option<String> {
    match (tool, os) {
        // Python 工具
        ("pip", "macos") => Some("brew install python".to_string()),
        ("pip", "linux") => Some("sudo apt install python3-pip -y".to_string()),
        ("pip", "windows") => Some("choco install python -y".to_string()),
        ("uv", _) => Some("curl -LsSf https://astral.sh/uv/install.sh | sh".to_string()),
        ("conda", "macos") => Some("brew install miniconda".to_string()),
        ("conda", "windows") => Some("choco install miniconda3 -y".to_string()),

        // JavaScript 工具
        ("npm", "macos") => Some("brew install node".to_string()),
        ("npm", "linux") => Some("sudo apt install nodejs npm -y".to_string()),
        ("npm", "windows") => Some("choco install nodejs -y".to_string()),
        ("yarn", _) => Some("npm install -g yarn".to_string()),
        ("pnpm", _) => Some("npm install -g pnpm".to_string()),

        // Rust
        ("cargo", _) => Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y".to_string()),

        // Java
        ("maven", "macos") => Some("brew install maven".to_string()),
        ("maven", "linux") => Some("sudo apt install maven -y".to_string()),
        ("maven", "windows") => Some("choco install maven -y".to_string()),
        ("gradle", "macos") => Some("brew install gradle".to_string()),
        ("gradle", "linux") => Some("sudo apt install gradle -y".to_string()),
        ("gradle", "windows") => Some("choco install gradle -y".to_string()),

        // Go
        ("go", "macos") => Some("brew install go".to_string()),
        ("go", "linux") => Some("sudo apt install golang -y".to_string()),
        ("go", "windows") => Some("choco install golang -y".to_string()),

        // Docker
        ("docker", "macos") => Some("brew install --cask docker".to_string()),
        ("docker", "linux") => Some("curl -fsSL https://get.docker.com | sh".to_string()),
        ("docker", "windows") => Some("choco install docker-desktop -y".to_string()),

        // 系统工具
        ("brew", "macos") | ("brew", "linux") => Some(
            "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string()
        ),
        ("choco", "windows") => Some(
            "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))".to_string()
        ),
        ("git", "macos") => Some("brew install git".to_string()),
        ("git", "linux") => Some("sudo apt install git -y".to_string()),
        ("git", "windows") => Some("choco install git -y".to_string()),

        _ => None,
    }
}

// 国内镜像安装命令（备用）
fn get_cn_install_command(tool: &str) -> Option<String> {
    match tool {
        // Homebrew 国内安装
        "brew" => Some(
            "/bin/bash -c \"$(curl -fsSL https://gitee.com/cunkai/HomebrewCN/raw/master/Homebrew.sh)\"".to_string()
        ),
        // Rust 国内安装 (rsproxy)
        "cargo" => Some(
            "curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh -s -- -y".to_string()
        ),
        // uv 国内安装 (使用 pip 安装)
        "uv" => Some(
            "pip install uv -i https://mirrors.aliyun.com/pypi/simple/".to_string()
        ),
        // Docker 国内安装
        "docker" => Some(
            "curl -fsSL https://get.docker.com | sh -s -- --mirror Aliyun".to_string()
        ),
        // Node.js 国内安装 (使用 nvm 国内镜像)
        "npm" => Some(
            "curl -o- https://gitee.com/mirrors/nvm/raw/master/install.sh | bash".to_string()
        ),
        // Go 国内安装 (使用 goproxy)
        "go" => Some(
            "brew install go && go env -w GOPROXY=https://goproxy.cn,direct".to_string()
        ),
        _ => None,
    }
}

#[tauri::command]
pub fn install_tool(name: String) -> Result<String, String> {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let cmd = get_install_command(&name, os)
        .ok_or_else(|| format!("不支持安装 {} 在 {} 系统", name, os))?;

    // 先尝试正常安装
    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        return Ok(format!("{} 安装成功", name));
    }

    // 如果失败，尝试国内镜像
    if let Some(cn_cmd) = get_cn_install_command(&name) {
        let cn_output = Command::new("sh")
            .arg("-c")
            .arg(&cn_cmd)
            .output()
            .map_err(|e| format!("执行国内镜像命令失败: {}", e))?;

        if cn_output.status.success() {
            return Ok(format!("{} 安装成功（使用国内镜像）", name));
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("安装失败: {}", stderr))
}

// ============================================
// 版本更新检测功能
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionUpdateInfo {
    pub tool: String,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub update_url: Option<String>,
}

// 获取工具的最新版本（从预设的版本信息或简单的版本比较）
fn get_latest_version_info(tool: &str) -> Option<(&'static str, &'static str)> {
    // 返回 (latest_version, download_url)
    // 这些是常见工具的最新稳定版本（定期更新）
    match tool {
        "pip" => Some(("24.3.1", "https://pip.pypa.io/")),
        "uv" => Some(("0.5.0", "https://github.com/astral-sh/uv/releases")),
        "npm" => Some(("10.9.0", "https://nodejs.org/")),
        "yarn" => Some(("4.5.3", "https://yarnpkg.com/")),
        "pnpm" => Some(("9.15.0", "https://pnpm.io/")),
        "go" => Some(("1.23.4", "https://go.dev/dl/")),
        "docker" => Some(("27.4.0", "https://docs.docker.com/engine/install/")),
        "git" => Some(("2.47.1", "https://git-scm.com/")),
        "maven" => Some(("3.9.9", "https://maven.apache.org/")),
        "gradle" => Some(("8.12", "https://gradle.org/")),
        _ => None,
    }
}

// 比较版本号
fn compare_versions(current: &str, latest: &str) -> bool {
    // 简单比较：提取数字部分进行比较
    let parse_version = |v: &str| -> Vec<u32> {
        v.split(|c: char| !c.is_numeric())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let curr_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for (c, l) in curr_parts.iter().zip(latest_parts.iter()) {
        if c < l {
            return true; // 有更新
        } else if c > l {
            return false;
        }
    }

    // 如果当前版本数字部分少于最新版本，也认为有更新
    curr_parts.len() < latest_parts.len()
}

#[tauri::command]
pub fn check_version_update(tool: String) -> Option<VersionUpdateInfo> {
    let tool_info = get_tool_info(tool.clone());

    if !tool_info.installed {
        return None;
    }

    let current_version = tool_info.version.clone();
    let latest_info = get_latest_version_info(&tool);

    let (latest_version, has_update, update_url) = match (current_version.as_ref(), latest_info) {
        (Some(current), Some((latest, url))) => {
            let has_update = compare_versions(current, latest);
            (Some(latest.to_string()), has_update, Some(url.to_string()))
        }
        _ => (None, false, None),
    };

    Some(VersionUpdateInfo {
        tool,
        current_version,
        latest_version,
        has_update,
        update_url,
    })
}

#[tauri::command]
pub fn check_all_updates() -> Vec<VersionUpdateInfo> {
    SUPPORTED_TOOLS
        .iter()
        .filter_map(|t| check_version_update(t.to_string()))
        .filter(|v| v.has_update)
        .collect()
}

// ============================================
// 冲突检测功能 (brew/choco/apt/版本管理器)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSource {
    pub manager: String,  // "brew", "choco", "apt", "pyenv", "nvm", "sdkman", "rustup", "manual"
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub tool: String,
    pub has_conflict: bool,
    pub sources: Vec<InstallSource>,
    pub warning_message: Option<String>,
}

// 检测工具是否通过特定包管理器安装
fn check_install_source(tool: &str, manager: &str) -> Option<InstallSource> {
    let check_cmd = match manager {
        "brew" => format!("brew list {} 2>/dev/null && brew --prefix {}", tool, tool),
        "apt" => format!("dpkg -s {} 2>/dev/null && which {}", tool, tool),
        "choco" => format!("choco list --local-only {} 2>nul", tool),
        "pyenv" => "pyenv root 2>/dev/null".to_string(),
        "nvm" => "bash -c 'source ~/.nvm/nvm.sh 2>/dev/null && nvm which current'".to_string(),
        "sdkman" => "bash -c 'source ~/.sdkman/bin/sdkman-init.sh 2>/dev/null && sdk current java'".to_string(),
        "rustup" => "rustup which rustc 2>/dev/null".to_string(),
        "conda" => "conda info --base 2>/dev/null".to_string(),
        _ => return None,
    };

    let shell = if manager == "choco" { "cmd" } else { "sh" };
    let args: Vec<&str> = if manager == "choco" { vec!["/c", &check_cmd] } else { vec!["-c", &check_cmd] };

    let output = Command::new(shell)
        .args(&args)
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .last()
            .unwrap_or("")
            .trim()
            .to_string();

        if !path.is_empty() {
            Some(InstallSource {
                manager: manager.to_string(),
                path,
            })
        } else {
            None
        }
    } else {
        None
    }
}

// 获取工具在各包管理器中的名称
fn get_package_name<'a>(tool: &'a str, manager: &str) -> &'a str {
    match (tool, manager) {
        ("pip", "brew") => "python",
        ("pip", "apt") => "python3-pip",
        ("pip", "choco") => "python",
        ("npm", "brew") => "node",
        ("npm", "apt") => "nodejs",
        ("npm", "choco") => "nodejs",
        ("cargo", "brew") => "rust",
        ("go", "apt") => "golang",
        ("maven", "apt") => "maven",
        ("gradle", "apt") => "gradle",
        _ => tool,
    }
}

// 获取工具可能的安装源（版本管理器等）
fn get_potential_sources(tool: &str) -> Vec<&'static str> {
    match tool {
        "pip" | "uv" => vec!["pyenv", "conda", "brew", "apt", "choco"],
        "conda" => vec!["brew", "choco"],
        "npm" | "yarn" | "pnpm" => vec!["nvm", "brew", "apt", "choco"],
        "maven" | "gradle" => vec!["sdkman", "brew", "apt", "choco"],
        "cargo" => vec!["rustup", "brew", "apt"],
        "go" => vec!["brew", "apt", "choco"],
        "docker" => vec!["brew", "apt", "choco"],
        "git" => vec!["brew", "apt", "choco"],
        "brew" => vec![],  // brew 本身不存在冲突
        "choco" => vec![], // choco 本身不存在冲突
        "apt" => vec![],   // apt 本身不存在冲突
        _ => vec![],
    }
}

#[tauri::command]
pub fn check_tool_conflict(tool: String) -> ConflictInfo {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    // 根据工具获取可能的安装源
    let potential_sources = get_potential_sources(&tool);

    // 过滤当前系统支持的包管理器
    let valid_sources: Vec<&str> = potential_sources.into_iter().filter(|s| {
        match (*s, os) {
            ("brew", "windows") => false,
            ("apt", "macos") | ("apt", "windows") => false,
            ("choco", "macos") | ("choco", "linux") => false,
            _ => true,
        }
    }).collect();

    let mut sources = Vec::new();

    // 检查各包管理器/版本管理器
    for source in &valid_sources {
        let pkg_name = get_package_name(&tool, source);

        // 特殊处理版本管理器
        match *source {
            "pyenv" | "nvm" | "sdkman" | "rustup" | "conda" => {
                if let Some(install_source) = check_install_source(&tool, source) {
                    sources.push(install_source);
                }
            }
            _ => {
                if let Some(install_source) = check_install_source(pkg_name, source) {
                    sources.push(install_source);
                }
            }
        }
    }

    // 检查手动安装（通过 which/where）
    if let Some(path) = get_install_path(&tool) {
        // 检查路径是否已经在某个包管理器中
        let is_managed = sources.iter().any(|s| {
            path.contains(&s.path) || s.path.contains(&path) ||
            path.contains(&s.manager) || s.path.is_empty()
        });

        if !is_managed && !path.is_empty() {
            sources.push(InstallSource {
                manager: "system".to_string(),
                path,
            });
        }
    }

    let has_conflict = sources.len() > 1;
    let warning_message = if has_conflict {
        let manager_names: Vec<&str> = sources.iter().map(|s| s.manager.as_str()).collect();
        Some(format!(
            "{} 检测到多个安装来源: {}。可能导致版本冲突或 PATH 优先级问题。",
            tool,
            manager_names.join(", ")
        ))
    } else {
        None
    };

    ConflictInfo {
        tool,
        has_conflict,
        sources,
        warning_message,
    }
}

#[tauri::command]
pub fn check_all_conflicts() -> Vec<ConflictInfo> {
    SUPPORTED_TOOLS
        .iter()
        .map(|t| check_tool_conflict(t.to_string()))
        .filter(|c| c.has_conflict)
        .collect()
}

// ============================================
// 卸载冲突源功能
// ============================================

#[tauri::command]
pub async fn uninstall_from_source(tool: String, source: String) -> Result<String, String> {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let pkg_name = get_package_name(&tool, &source);

    let uninstall_cmd = match (source.as_str(), os) {
        ("brew", _) => format!("brew uninstall {}", pkg_name),
        ("apt", "linux") => format!("sudo apt remove {} -y", pkg_name),
        ("choco", "windows") => format!("choco uninstall {} -y", pkg_name),
        ("pyenv", _) => {
            // pyenv 不能直接卸载，提示用户
            return Ok("pyenv 管理的 Python 版本请使用 'pyenv uninstall <version>' 命令手动卸载".to_string());
        }
        ("nvm", _) => {
            return Ok("nvm 管理的 Node.js 版本请使用 'nvm uninstall <version>' 命令手动卸载".to_string());
        }
        ("sdkman", _) => {
            return Ok("SDKMAN 管理的 Java 版本请使用 'sdk uninstall java <version>' 命令手动卸载".to_string());
        }
        ("rustup", _) => {
            return Ok("rustup 管理的工具链请使用 'rustup toolchain remove <toolchain>' 命令手动卸载".to_string());
        }
        ("conda", _) => {
            return Ok("conda 环境请使用 'conda remove <package>' 命令手动卸载".to_string());
        }
        ("system", _) | ("manual", _) => {
            return Err("系统安装的工具无法自动卸载，请手动删除".to_string());
        }
        _ => return Err(format!("不支持从 {} 卸载", source)),
    };

    let shell = if os == "windows" { "cmd" } else { "sh" };
    let args: Vec<&str> = if os == "windows" {
        vec!["/c", &uninstall_cmd]
    } else {
        vec!["-c", &uninstall_cmd]
    };

    let output = Command::new(shell)
        .args(&args)
        .output()
        .map_err(|e| format!("执行卸载命令失败: {}", e))?;

    if output.status.success() {
        Ok(format!("已从 {} 卸载 {}", source, tool))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("卸载失败: {}", stderr))
    }
}

// ============================================
// 异步安装工具（修复卡顿问题）
// ============================================

#[tauri::command]
pub async fn install_tool_async(name: String) -> Result<String, String> {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let cmd = get_install_command(&name, os)
        .ok_or_else(|| format!("不支持安装 {} 在 {} 系统", name, os))?;

    // 使用 tokio 的异步命令执行
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .await
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        return Ok(format!("{} 安装成功", name));
    }

    // 如果失败，尝试国内镜像
    if let Some(cn_cmd) = get_cn_install_command(&name) {
        let cn_output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cn_cmd)
            .output()
            .await
            .map_err(|e| format!("执行国内镜像命令失败: {}", e))?;

        if cn_output.status.success() {
            return Ok(format!("{} 安装成功（使用国内镜像）", name));
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("安装失败: {}", stderr))
}

#[tauri::command]
pub async fn sync_java_home(target_version: Option<String>) -> Result<String, String> {
    // 获取目标版本（如果没有指定，使用当前 jenv 版本）
    let version = if let Some(v) = target_version {
        v
    } else {
        let output = Command::new("sh")
            .arg("-c")
            .arg("jenv version-name 2>/dev/null")
            .output()
            .map_err(|e| format!("获取 jenv 版本失败: {}", e))?;

        if !output.status.success() {
            return Err("jenv 未安装或未配置".to_string());
        }
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    // 获取 jenv 对应版本的路径
    let prefix_output = Command::new("sh")
        .arg("-c")
        .arg(format!("jenv prefix {} 2>/dev/null", version))
        .output()
        .map_err(|e| format!("获取 Java 路径失败: {}", e))?;

    if !prefix_output.status.success() {
        return Err(format!("无法获取版本 {} 的路径", version));
    }

    let java_path = String::from_utf8_lossy(&prefix_output.stdout).trim().to_string();

    // 确定 shell 配置文件
    let home = dirs::home_dir().ok_or("无法获取 home 目录")?;
    let shell = std::env::var("SHELL").unwrap_or_default();

    let config_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    // 读取当前配置文件内容
    let content = std::fs::read_to_string(&config_file)
        .unwrap_or_default();

    // 检查是否已有 JAVA_HOME 设置
    let java_home_line = format!("export JAVA_HOME=\"{}\"", java_path);
    let new_content = if content.contains("export JAVA_HOME=") {
        // 替换现有的 JAVA_HOME
        let re = regex::Regex::new(r#"export JAVA_HOME=.*"#).unwrap();
        re.replace(&content, java_home_line.as_str()).to_string()
    } else {
        // 追加 JAVA_HOME
        format!("{}\n\n# JAVA_HOME (managed by DevHub)\n{}\n", content.trim_end(), java_home_line)
    };

    // 备份并写入新配置
    let backup_path = config_file.with_extension("bak");
    std::fs::copy(&config_file, &backup_path)
        .map_err(|e| format!("备份配置文件失败: {}", e))?;

    std::fs::write(&config_file, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 启用 jenv export 插件
    let _ = Command::new("sh")
        .arg("-c")
        .arg("jenv enable-plugin export 2>/dev/null")
        .output();

    // 更新当前环境变量
    std::env::set_var("JAVA_HOME", &java_path);

    Ok(format!(
        "已将 JAVA_HOME 更新为: {}\n配置文件: {}\n请重新打开终端或执行 source {} 使配置生效",
        java_path,
        config_file.display(),
        config_file.display()
    ))
}
