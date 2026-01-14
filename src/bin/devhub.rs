use anyhow::Result;
use clap::{Parser, Subcommand};
use devhub::sources::{get_manager, SUPPORTED_TOOLS};
use devhub::utils::benchmark_mirrors;
use std::process::Command;

#[derive(Parser)]
#[command(name = "devhub")]
#[command(version = "0.2.0")]
#[command(about = "DevHub Pro - 开发环境镜像源管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查看当前配置状态
    Status {
        /// 工具名称 (pip, npm, cargo等)，省略则显示全部
        name: Option<String>,
    },
    /// 测速所有镜像源
    Test {
        /// 工具名称
        name: String,
    },
    /// 切换镜像源
    Use {
        /// 工具名称
        name: String,

        /// 镜像源名称 (如 Aliyun, Tuna)
        #[arg(required_unless_present = "fastest")]
        source: Option<String>,

        /// 自动选择最快的镜像源
        #[arg(long, short)]
        fastest: bool,
    },
    /// 恢复默认配置
    Restore {
        /// 工具名称
        name: String,
    },
    /// 列出支持的工具
    List,
    /// 显示系统信息和已安装工具
    Info,
    /// 检查工具版本更新
    Check {
        /// 工具名称，省略则检查全部
        name: Option<String>,
    },
    /// 检测安装冲突
    Conflicts,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { name } => handle_status(name).await?,
        Commands::Test { name } => handle_test(&name).await?,
        Commands::Use { name, source, fastest } => handle_use(&name, source, fastest).await?,
        Commands::Restore { name } => handle_restore(&name).await?,
        Commands::List => handle_list()?,
        Commands::Info => handle_info()?,
        Commands::Check { name } => handle_check(name)?,
        Commands::Conflicts => handle_conflicts()?,
    }

    Ok(())
}

async fn handle_status(name: Option<String>) -> Result<()> {
    let tools: Vec<String> = match name {
        Some(n) => vec![n],
        None => SUPPORTED_TOOLS.iter().map(|&s| s.to_string()).collect(),
    };

    println!("\n{:<12} {:<20} {}", "工具", "当前镜像源", "URL");
    println!("{}", "-".repeat(70));

    for tool in tools {
        match get_manager(&tool) {
            Ok(manager) => {
                let current_url = manager.current_url().await.unwrap_or(None);
                let candidates = manager.list_candidates();

                let (name, url) = match &current_url {
                    Some(url) => {
                        let name = candidates
                            .iter()
                            .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| "自定义".to_string());
                        (name, url.clone())
                    }
                    None => ("官方默认".to_string(), "-".to_string()),
                };

                println!("{:<12} {:<20} {}", tool, name, url);
            }
            Err(_) => {
                println!("{:<12} {:<20} {}", tool, "不支持", "-");
            }
        }
    }

    println!();
    Ok(())
}

async fn handle_test(name: &str) -> Result<()> {
    let manager = get_manager(name)?;
    let mirrors = manager.list_candidates();

    println!("\n正在测速 {} 的镜像源...\n", name);

    let results = benchmark_mirrors(mirrors).await;

    println!("{:<25} {:<15} {}", "镜像源", "延迟", "状态");
    println!("{}", "-".repeat(50));

    for r in &results {
        let (latency, status) = if r.latency_ms == u64::MAX {
            ("超时".to_string(), "X")
        } else {
            (format!("{}ms", r.latency_ms), "OK")
        };
        println!("{:<25} {:<15} {}", r.mirror.name, latency, status);
    }

    // 显示最快的镜像
    if let Some(fastest) = results.iter().filter(|r| r.latency_ms < u64::MAX).min_by_key(|r| r.latency_ms) {
        println!("\n[FASTEST] {} ({}ms)", fastest.mirror.name, fastest.latency_ms);
    }

    println!();
    Ok(())
}

async fn handle_use(name: &str, source: Option<String>, fastest: bool) -> Result<()> {
    let manager = get_manager(name)?;
    let mirrors = manager.list_candidates();

    let mirror = if fastest {
        println!("正在测速选择最快镜像...");
        let results = benchmark_mirrors(mirrors).await;
        results
            .into_iter()
            .filter(|r| r.latency_ms < u64::MAX)
            .min_by_key(|r| r.latency_ms)
            .map(|r| r.mirror)
            .ok_or_else(|| anyhow::anyhow!("所有镜像源均超时"))?
    } else {
        let source_name = source.ok_or_else(|| anyhow::anyhow!("请指定镜像源名称或使用 --fastest"))?;
        mirrors
            .into_iter()
            .find(|m| m.name.to_lowercase().contains(&source_name.to_lowercase()))
            .ok_or_else(|| anyhow::anyhow!("未找到镜像源: {}", source_name))?
    };

    println!("正在切换 {} 到 {}...", name, mirror.name);
    manager.set_source(&mirror).await?;
    println!("[OK] 已切换到: {} ({})", mirror.name, mirror.url);

    Ok(())
}

async fn handle_restore(name: &str) -> Result<()> {
    let manager = get_manager(name)?;
    println!("正在恢复 {} 的默认配置...", name);
    manager.restore().await?;
    println!("[OK] 已恢复默认配置");
    Ok(())
}

fn handle_list() -> Result<()> {
    let os = get_os_name();

    println!("\nDevHub Pro - 支持的工具\n");
    println!("当前系统: {}\n", os);

    println!("  Python:     pip, uv, conda");
    println!("  JavaScript: npm, yarn, pnpm");
    println!("  Rust:       cargo");
    println!("  Java:       maven, gradle");
    println!("  Go:         go");
    println!("  Docker:     docker");

    match os.as_str() {
        "macOS" => println!("  系统:       brew, git"),
        "Linux" => println!("  系统:       brew, apt, git"),
        "Windows" => println!("  系统:       choco, git"),
        _ => println!("  系统:       git"),
    }

    println!("\n使用示例:");
    println!("  devhub status           # 查看所有工具状态");
    println!("  devhub test pip         # 测速 pip 镜像源");
    println!("  devhub use pip aliyun   # 切换 pip 到阿里云镜像");
    println!("  devhub use pip -f       # 自动选择最快镜像");
    println!("  devhub restore pip      # 恢复默认配置");
    println!("  devhub info             # 显示系统信息");
    println!("  devhub check            # 检查版本更新");
    println!("  devhub conflicts        # 检测安装冲突");
    println!();
    Ok(())
}

fn handle_info() -> Result<()> {
    let os = get_os_name();
    let arch = get_arch();

    println!("\n系统信息:");
    println!("  操作系统: {}", os);
    println!("  架构: {}", arch);
    println!();

    println!("已安装工具:");
    println!("{:<12} {:<15} {}", "工具", "版本", "路径");
    println!("{}", "-".repeat(60));

    for tool in SUPPORTED_TOOLS {
        if let Some((version, path)) = get_tool_version(tool) {
            println!("{:<12} {:<15} {}", tool, version, path);
        }
    }

    println!();
    Ok(())
}

fn handle_check(name: Option<String>) -> Result<()> {
    let tools: Vec<&str> = match &name {
        Some(n) => vec![n.as_str()],
        None => SUPPORTED_TOOLS.to_vec(),
    };

    // 预设的最新版本信息
    let latest_versions: Vec<(&str, &str, &str)> = vec![
        ("pip", "24.3.1", "https://pip.pypa.io/"),
        ("uv", "0.5.0", "https://github.com/astral-sh/uv/releases"),
        ("npm", "10.9.0", "https://nodejs.org/"),
        ("yarn", "4.5.3", "https://yarnpkg.com/"),
        ("pnpm", "9.15.0", "https://pnpm.io/"),
        ("go", "1.23.4", "https://go.dev/dl/"),
        ("docker", "27.4.0", "https://docs.docker.com/engine/install/"),
        ("git", "2.47.1", "https://git-scm.com/"),
        ("maven", "3.9.9", "https://maven.apache.org/"),
        ("gradle", "8.12", "https://gradle.org/"),
    ];

    println!("\n版本更新检查:\n");
    println!("{:<12} {:<15} {:<15} {}", "工具", "当前版本", "最新版本", "状态");
    println!("{}", "-".repeat(60));

    for tool in tools {
        if let Some((current, _)) = get_tool_version(tool) {
            if let Some((_, latest, _)) = latest_versions.iter().find(|(t, _, _)| *t == tool) {
                let has_update = compare_versions(&current, latest);
                let status = if has_update { "[UPDATE]" } else { "[OK]" };
                println!("{:<12} {:<15} {:<15} {}", tool, current, latest, status);
            }
        }
    }

    println!();
    Ok(())
}

fn handle_conflicts() -> Result<()> {
    println!("\n安装冲突检测:\n");

    let os = get_os_name();
    let managers = match os.as_str() {
        "macOS" => vec!["brew"],
        "Linux" => vec!["apt", "brew"],
        "Windows" => vec!["choco"],
        _ => vec![],
    };

    let version_managers = vec!["pyenv", "nvm", "sdkman", "rustup"];

    println!("{:<12} {:<15} {}", "工具", "安装来源", "路径");
    println!("{}", "-".repeat(60));

    for tool in SUPPORTED_TOOLS {
        let mut sources: Vec<(String, String)> = Vec::new();

        // 检查包管理器
        for manager in &managers {
            if check_package_manager(tool, manager) {
                if let Some((_, path)) = get_tool_version(tool) {
                    sources.push((manager.to_string(), path));
                }
            }
        }

        // 检查版本管理器
        for vm in &version_managers {
            if check_version_manager(tool, vm) {
                sources.push((vm.to_string(), format!("via {}", vm)));
            }
        }

        if sources.len() > 1 {
            println!("{:<12} [CONFLICT]", tool);
            for (src, path) in &sources {
                println!("             - {}: {}", src, path);
            }
        } else if sources.len() == 1 {
            let (src, path) = &sources[0];
            println!("{:<12} {:<15} {}", tool, src, path);
        }
    }

    println!();
    Ok(())
}

// 辅助函数

fn get_os_name() -> String {
    if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x86_64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64 (ARM64)".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_tool_version(tool: &str) -> Option<(String, String)> {
    let (cmd, args) = match tool {
        "pip" => ("pip3", vec!["--version"]),
        "uv" => ("uv", vec!["--version"]),
        "conda" => ("conda", vec!["--version"]),
        "npm" => ("npm", vec!["--version"]),
        "yarn" => ("yarn", vec!["--version"]),
        "pnpm" => ("pnpm", vec!["--version"]),
        "cargo" => ("cargo", vec!["--version"]),
        "go" => ("go", vec!["version"]),
        "maven" => ("mvn", vec!["--version"]),
        "gradle" => ("gradle", vec!["--version"]),
        "docker" => ("docker", vec!["--version"]),
        "brew" => ("brew", vec!["--version"]),
        "choco" => ("choco", vec!["--version"]),
        "apt" => ("apt", vec!["--version"]),
        "git" => ("git", vec!["--version"]),
        _ => return None,
    };

    let output = Command::new(cmd).args(&args).output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = extract_version(&stdout);

        // 获取路径
        let which_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
        let path = Command::new(which_cmd)
            .arg(cmd)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).lines().next()?.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "-".to_string());

        Some((version, path))
    } else {
        None
    }
}

fn extract_version(output: &str) -> String {
    let first_line = output.lines().next().unwrap_or(output);
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    for part in &parts {
        if part.chars().any(|c| c.is_numeric()) && part.contains('.') {
            let version = part.trim_start_matches('v').trim_start_matches("go");
            return version.to_string();
        }
    }

    first_line.to_string()
}

fn compare_versions(current: &str, latest: &str) -> bool {
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
            return true;
        } else if c > l {
            return false;
        }
    }

    curr_parts.len() < latest_parts.len()
}

fn check_package_manager(tool: &str, manager: &str) -> bool {
    let pkg_name = match (tool, manager) {
        ("pip", "brew") => "python",
        ("pip", "apt") => "python3-pip",
        ("pip", "choco") => "python",
        ("npm", "brew") => "node",
        ("npm", "apt") => "nodejs",
        ("npm", "choco") => "nodejs",
        _ => tool,
    };

    let cmd = match manager {
        "brew" => format!("brew list {} 2>/dev/null", pkg_name),
        "apt" => format!("dpkg -s {} 2>/dev/null", pkg_name),
        "choco" => format!("choco list --local-only {} 2>nul", pkg_name),
        _ => return false,
    };

    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let args: Vec<&str> = if cfg!(target_os = "windows") {
        vec!["/c", &cmd]
    } else {
        vec!["-c", &cmd]
    };

    Command::new(shell)
        .args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_version_manager(tool: &str, manager: &str) -> bool {
    let relevant = match (tool, manager) {
        ("pip" | "uv" | "conda", "pyenv") => true,
        ("npm" | "yarn" | "pnpm", "nvm") => true,
        ("maven" | "gradle", "sdkman") => true,
        ("cargo", "rustup") => true,
        _ => return false,
    };

    if !relevant {
        return false;
    }

    let cmd = match manager {
        "pyenv" => "pyenv version 2>/dev/null",
        "nvm" => "bash -c 'source ~/.nvm/nvm.sh 2>/dev/null && nvm current'",
        "sdkman" => "bash -c 'source ~/.sdkman/bin/sdkman-init.sh 2>/dev/null && sdk current java'",
        "rustup" => "rustup show active-toolchain 2>/dev/null",
        _ => return false,
    };

    Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
