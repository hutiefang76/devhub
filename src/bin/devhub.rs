use anyhow::Result;
use clap::{Parser, Subcommand};
use devhub::sources::{get_manager, SUPPORTED_TOOLS};
use devhub::utils::benchmark_mirrors;

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
            ("超时".to_string(), "❌")
        } else {
            (format!("{}ms", r.latency_ms), "✅")
        };
        println!("{:<25} {:<15} {}", r.mirror.name, latency, status);
    }

    // 显示最快的镜像
    if let Some(fastest) = results.iter().filter(|r| r.latency_ms < u64::MAX).min_by_key(|r| r.latency_ms) {
        println!("\n🚀 最快镜像: {} ({}ms)", fastest.mirror.name, fastest.latency_ms);
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
    println!("✅ 已切换到: {} ({})", mirror.name, mirror.url);

    Ok(())
}

async fn handle_restore(name: &str) -> Result<()> {
    let manager = get_manager(name)?;
    println!("正在恢复 {} 的默认配置...", name);
    manager.restore().await?;
    println!("✅ 已恢复默认配置");
    Ok(())
}

fn handle_list() -> Result<()> {
    println!("\n支持的工具:\n");
    println!("  Python:     pip, uv, conda");
    println!("  JavaScript: npm, yarn, pnpm");
    println!("  Rust:       cargo");
    println!("  Java:       maven, gradle");
    println!("  Go:         go");
    println!("  Docker:     docker");
    println!("  系统:       brew (macOS/Linux), choco (Windows), apt (Linux), git");
    println!();
    Ok(())
}
