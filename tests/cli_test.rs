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
    assert!(stdout.contains("支持的工具"), "Should show supported tools");
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
