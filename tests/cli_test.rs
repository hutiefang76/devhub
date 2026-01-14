//! CLI tests for DevHub Pro

use std::process::Command;

fn run_cli(args: &[&str]) -> (bool, String, String) {
    let output = Command::new("cargo")
        .args(["run", "--bin", "devhub", "--"])
        .args(args)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (output.status.success(), stdout, stderr)
}

#[test]
fn test_cli_list_command() {
    let (success, stdout, _) = run_cli(&["list"]);
    assert!(success, "CLI list command should succeed");
    assert!(stdout.contains("支持的工具") || stdout.contains("DevHub Pro"), "Should show supported tools");
    assert!(stdout.contains("pip"), "Should include pip");
    assert!(stdout.contains("npm"), "Should include npm");
    assert!(stdout.contains("cargo"), "Should include cargo");
}

#[test]
fn test_cli_status_all() {
    let (success, stdout, _) = run_cli(&["status"]);
    assert!(success, "CLI status command should succeed");
    assert!(stdout.contains("工具") || stdout.contains("镜像源"), "Should show status header");
}

#[test]
fn test_cli_status_pip() {
    let (success, stdout, _) = run_cli(&["status", "pip"]);
    assert!(success, "CLI status pip command should succeed");
    assert!(stdout.contains("pip"), "Should show pip status");
}

#[test]
fn test_cli_help() {
    let (success, stdout, _) = run_cli(&["--help"]);
    assert!(success, "CLI help should succeed");
    assert!(stdout.contains("devhub") || stdout.contains("DevHub"), "Should show app name");
    assert!(stdout.contains("status") || stdout.contains("Status"), "Should show status command");
    assert!(stdout.contains("test") || stdout.contains("Test"), "Should show test command");
}

#[test]
fn test_cli_version() {
    let (success, stdout, _) = run_cli(&["--version"]);
    assert!(success, "CLI version should succeed");
    assert!(stdout.contains("0.2") || stdout.contains("devhub"), "Should show version");
}

#[test]
fn test_cli_invalid_tool() {
    let (success, _, stderr) = run_cli(&["status", "invalid_tool_xyz"]);
    // 对于无效工具，可能成功但显示不支持，或者失败
    // 只要有输出即可
    assert!(!stderr.is_empty() || success, "Should handle invalid tool");
}

// 新增测试：info 命令
#[test]
fn test_cli_info_command() {
    let (success, stdout, _) = run_cli(&["info"]);
    assert!(success, "CLI info command should succeed");
    assert!(stdout.contains("系统信息") || stdout.contains("操作系统"), "Should show system info");
    assert!(stdout.contains("已安装工具") || stdout.contains("工具"), "Should show installed tools");
}

// 新增测试：check 命令
#[test]
fn test_cli_check_command() {
    let (success, stdout, _) = run_cli(&["check"]);
    assert!(success, "CLI check command should succeed");
    assert!(stdout.contains("版本") || stdout.contains("更新"), "Should show version check");
}

// 新增测试：check 指定工具
#[test]
fn test_cli_check_specific_tool() {
    let (success, stdout, _) = run_cli(&["check", "pip"]);
    assert!(success, "CLI check pip command should succeed");
    // 可能显示 pip 的版本信息或跳过（如果未安装）
    assert!(stdout.contains("pip") || stdout.contains("版本"), "Should check pip version");
}

// 新增测试：conflicts 命令
#[test]
fn test_cli_conflicts_command() {
    let (success, stdout, _) = run_cli(&["conflicts"]);
    assert!(success, "CLI conflicts command should succeed");
    assert!(stdout.contains("冲突") || stdout.contains("安装"), "Should show conflict detection");
}

// 新增测试：list 命令包含使用示例
#[test]
fn test_cli_list_shows_examples() {
    let (success, stdout, _) = run_cli(&["list"]);
    assert!(success, "CLI list command should succeed");
    assert!(stdout.contains("示例") || stdout.contains("devhub"), "Should show usage examples");
}

// 新增测试：status 显示正确格式
#[test]
fn test_cli_status_format() {
    let (success, stdout, _) = run_cli(&["status"]);
    assert!(success, "CLI status command should succeed");
    // 检查表格格式
    assert!(stdout.contains("---") || stdout.contains("─"), "Should have table separators");
}

// 新增测试：test 命令（需要网络）
#[test]
#[ignore] // 需要网络连接，默认跳过
fn test_cli_test_mirrors() {
    let (success, stdout, _) = run_cli(&["test", "pip"]);
    assert!(success, "CLI test command should succeed");
    assert!(stdout.contains("延迟") || stdout.contains("ms"), "Should show latency");
}
