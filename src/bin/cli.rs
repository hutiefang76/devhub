use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use devhub::{benchmark_mirrors, get_manager, Mirror, SUPPORTED_TOOLS};

#[derive(Parser)]
#[command(name = "devhub")]
#[command(about = "开发环境管理工具 - 镜像源配置", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示当前配置状态
    Status {
        /// 工具名称 (pip, npm, cargo 等)，不指定则显示全部
        name: Option<String>,
    },
    /// 测试镜像源速度
    Test {
        /// 工具名称
        name: String,
    },
    /// 应用镜像源
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { name } => handle_status(name).await?,
        Commands::Test { name } => handle_test(&name).await?,
        Commands::Use { name, source, fastest } => handle_use(&name, source, fastest).await?,
        Commands::Restore { name } => handle_restore(&name).await?,
        Commands::List => handle_list(),
    }

    Ok(())
}

fn handle_list() {
    println!("支持的工具列表:\n");
    println!("{:<10} {}", "工具", "说明");
    println!("{}", "-".repeat(50));

    let descriptions = [
        ("pip", "Python 包管理器"),
        ("uv", "Python 包管理器 (Rust 实现)"),
        ("conda", "Python 环境管理器"),
        ("npm", "Node.js 包管理器"),
        ("yarn", "Node.js 包管理器"),
        ("pnpm", "Node.js 包管理器"),
        ("cargo", "Rust 包管理器"),
        ("go", "Go 模块代理"),
        ("maven", "Java 构建工具"),
        ("gradle", "Java 构建工具"),
        ("docker", "容器镜像仓库"),
        ("brew", "macOS 包管理器"),
        ("apt", "Debian/Ubuntu 包管理器"),
        ("git", "Git 仓库镜像"),
    ];

    for (name, desc) in descriptions {
        println!("{:<10} {}", name, desc);
    }
}

async fn handle_status(name: Option<String>) -> Result<()> {
    let tools: Vec<String> = match name {
        Some(n) => vec![n],
        None => SUPPORTED_TOOLS.iter().map(|&s| s.to_string()).collect(),
    };

    println!("{}", "-".repeat(80));
    println!("{:<10} {:<50} {}", "工具", "当前源 URL", "状态");
    println!("{}", "-".repeat(80));

    for tool_name in tools {
        let manager = match get_manager(&tool_name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let current_url = manager.current_url().await.unwrap_or(None);
        let candidates = manager.list_candidates();

        let (url_display, status_display) = match current_url {
            Some(url) => {
                let known_name = candidates
                    .iter()
                    .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "自定义".to_string());

                (url, format!("[{}]", known_name))
            }
            None => ("默认".to_string(), "[官方/默认]".to_string()),
        };

        let mut url_short = url_display.clone();
        if url_short.len() > 48 {
            url_short = format!("{}...", &url_short[..45]);
        }

        println!("{:<10} {:<50} {}", manager.name(), url_short, status_display);
    }
    println!("{}", "-".repeat(80));

    Ok(())
}

async fn handle_test(name: &str) -> Result<()> {
    let manager = get_manager(name)?;
    let mut candidates = manager.list_candidates();

    let mut current_url_opt = manager.current_url().await.ok().flatten();

    if current_url_opt.is_none() {
        if let Some(official) = candidates.iter().find(|m| m.name.eq_ignore_ascii_case("Official")) {
            current_url_opt = Some(official.url.clone());
        }
    }

    if let Some(ref current_url) = current_url_opt {
        let is_known = candidates.iter().any(|m| {
            m.url == *current_url || m.url.trim_end_matches('/') == current_url.trim_end_matches('/')
        });

        if !is_known {
            candidates.push(Mirror::new("当前", current_url));
        }
    }

    let results = benchmark_mirrors(candidates).await;

    println!();
    println!("{:<4} {:<10} {:<12} URL", "排名", "延迟", "名称");
    println!("{}", "-".repeat(70));

    for (i, res) in results.iter().enumerate() {
        let latency_str = if res.latency_ms == u64::MAX {
            "超时".to_string()
        } else {
            format!("{}ms", res.latency_ms)
        };

        println!(
            "{:<4} {:<10} {:<12} {}",
            i + 1,
            latency_str,
            res.mirror.name,
            res.mirror.url
        );
    }

    if let Some(best) = results.first() {
        if best.latency_ms < u64::MAX {
            println!("{}", "-".repeat(70));
            println!("推荐: '{}' 是最快的镜像源", best.mirror.name);
            println!("执行 'devhub use {} {}' 应用此镜像", name, best.mirror.name);
        }
    }

    Ok(())
}

async fn handle_use(name: &str, source_name: Option<String>, fastest: bool) -> Result<()> {
    let manager = get_manager(name)?;

    if manager.requires_sudo() {
        eprintln!("注意: 修改 {} 配置通常需要 sudo 权限", name);
    }

    let target_mirror = if fastest {
        println!("正在寻找最快的镜像源...");
        let results = benchmark_mirrors(manager.list_candidates()).await;

        let valid_results: Vec<_> = results.into_iter().filter(|r| r.latency_ms < u64::MAX).collect();

        if valid_results.is_empty() {
            bail!("所有镜像源均超时，请检查网络连接");
        }

        let best = &valid_results[0];
        println!("最快镜像源: {} ({}ms)", best.mirror.name, best.latency_ms);
        best.mirror.clone()
    } else {
        let target_name = source_name.unwrap();
        let candidates = manager.list_candidates();

        match candidates.into_iter().find(|m| m.name.eq_ignore_ascii_case(&target_name)) {
            Some(m) => m,
            None => bail!("未找到镜像源 '{}'，执行 'devhub test {}' 查看可用列表", target_name, name),
        }
    };

    println!("正在应用 {} 镜像...", target_mirror.name);
    manager.set_source(&target_mirror).await?;
    println!("成功! {} 现在使用 {} 镜像", name, target_mirror.name);

    Ok(())
}

async fn handle_restore(name: &str) -> Result<()> {
    let manager = get_manager(name)?;

    if manager.requires_sudo() {
        eprintln!("注意: 恢复 {} 配置通常需要 sudo 权限", name);
    }

    println!("正在恢复 {} 配置...", name);
    manager.restore().await?;
    println!("成功! {} 配置已恢复", name);

    Ok(())
}
