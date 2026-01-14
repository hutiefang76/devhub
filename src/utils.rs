use crate::error::{DevHubError, Result};
use crate::types::{BenchmarkResult, Mirror};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::fs;

const REQUEST_TIMEOUT: u64 = 5;

/// 备份文件
pub async fn backup_file(path: &Path) -> Result<()> {
    if fs::try_exists(path).await.unwrap_or(false) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let backup_name = format!("{}.bak.{}", file_name, timestamp);
        let backup_path = path.with_file_name(backup_name);

        fs::copy(path, &backup_path).await?;
        println!("备份已创建: {:?}", backup_path);
    }
    Ok(())
}

/// 恢复到最近的备份
pub async fn restore_latest_backup(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let prefix = format!("{}.bak.", file_name);

    if !fs::try_exists(parent).await.unwrap_or(false) {
        return Err(DevHubError::Custom(format!("目录不存在: {:?}", parent)));
    }

    let mut entries = fs::read_dir(parent).await?;
    let mut backups = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) {
            backups.push(entry.path());
        }
    }

    if backups.is_empty() {
        return Err(DevHubError::Custom("未找到备份文件".to_string()));
    }

    backups.sort();
    let latest = backups.last().unwrap();

    println!("正在从备份恢复: {:?}", latest);
    fs::copy(latest, path).await?;
    println!("配置已恢复");

    Ok(())
}

/// 并发测试所有镜像源的延迟
pub async fn benchmark_mirrors(mirrors: Vec<Mirror>) -> Vec<BenchmarkResult> {
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .unwrap_or_default();

    let pb = ProgressBar::new(mirrors.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message("测速中...");

    let tasks = mirrors.into_iter().map(|m| {
        let client = client.clone();
        let pb = pb.clone();
        async move {
            let res = check_latency(&client, m).await;
            pb.inc(1);
            res
        }
    });

    let mut results = futures::future::join_all(tasks).await;
    pb.finish_with_message("测速完成");

    results.sort_by_key(|r| r.latency_ms);
    results
}

/// 单个源测速
async fn check_latency(client: &Client, mirror: Mirror) -> BenchmarkResult {
    let start = Instant::now();

    let url_to_test = mirror
        .url
        .trim_start_matches("sparse+")
        .trim_start_matches("git+")
        .split(',')
        .next()
        .unwrap_or(&mirror.url);

    let latency_ms = match client.head(url_to_test).send().await {
        Ok(resp) => {
            if resp.status().is_success() || resp.status().is_redirection() {
                start.elapsed().as_millis() as u64
            } else {
                u64::MAX
            }
        }
        Err(_) => u64::MAX,
    };

    BenchmarkResult { mirror, latency_ms }
}

/// 执行 shell 命令
pub async fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = tokio::process::Command::new(cmd)
        .args(args)
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(DevHubError::CommandFailed(stderr.to_string()))
    }
}

/// 检测命令是否存在
pub async fn command_exists(cmd: &str) -> bool {
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    run_command(which_cmd, &[cmd]).await.is_ok()
}

/// 获取命令路径
pub async fn get_command_path(cmd: &str) -> Option<String> {
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    run_command(which_cmd, &[cmd]).await.ok()
}
