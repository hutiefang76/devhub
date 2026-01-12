# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DevHub Pro** is a development environment management platform for Chinese developers. It provides:
- Software detection and installation
- Mirror source configuration for package managers (pip, npm, maven, go modules, cargo, docker, etc.)
- Version management integration (pyenv, nvm, SDKMAN, etc.)
- Environment variable management
- Cross-platform GUI using Rust + egui

**Tech Stack:** Rust + egui + tokio + SQLite

## Documentation Structure

This project is currently in planning phase with three key documents:

- **01_REQUIREMENTS.md** - Complete feature list, GUI mockups, mirror sources for all tools
- **02_ARCHITECTURE.md** - Layered architecture, core trait definitions, database schema
- **03_DETAILED_DESIGN.md** - Java→Rust guide, development setup, implementation steps

**Read these first** before making any architectural decisions.

## Common Commands

### Build & Run
```bash
cargo build                    # Debug build
cargo build --release          # Optimized release build
cargo run                      # Build and run
cargo test                     # Run all tests
cargo test -- --show-output    # Run tests with output
```

### Code Quality
```bash
cargo clippy                   # Linter (run before commits)
cargo fmt                      # Format code
cargo doc --open               # Generate and open docs
```

### Development Workflow
```bash
cargo add <crate>              # Add dependency (requires cargo-edit)
cargo tree                     # View dependency tree
cargo clean                    # Clean build artifacts
cargo watch -x check           # Auto-recompile on file changes
```

### Testing
```bash
cargo test                     # All tests
cargo test <module>            # Tests in specific module
cargo test --test <name>       # Integration test
cargo nextest run              # Parallel test runner (if installed)
```

## Architecture Overview

### Layered Architecture (SOLID Principles)

```
┌──────────────────────────────────────────────┐
│            Presentation Layer                │
│                 (egui)                       │  ← User interaction
├──────────────────────────────────────────────┤
│          Application Layer                   │  ← State management
│         (DevHubApp, ViewState)               │
├──────────────────────────────────────────────┤
│            Service Layer                     │  ← Business logic
│       (Detector, MirrorConfigurator)         │
├──────────────────────────────────────────────┤
│          Infrastructure Layer                │  ← Shell commands, HTTP
│      (ShellExecutor, SpeedTestService)       │
└──────────────────────────────────────────────┘
```

### Core Traits (Dependency Inversion)

The architecture relies on **trait-based abstractions** defined in `src/core/`:

- **ToolDetector** - Detects if software is installed (`fn detect() -> Result<DetectionInfo>`)
- **MirrorConfigurator** - Configures mirror sources (`fn apply(&Mirror) -> Result<()>`)
- **ToolInstaller** - Software installation (`fn install(version) -> Result<()>`)
- **VersionManager** - Version switching (`fn set_global_version(&str) -> Result<()>`)
- **EnvConfigurator** - Environment variable management
- **CommandExecutor** - Shell command execution

**Key principle:** High-level modules depend on traits, not concrete implementations.

### Directory Structure (Planned)

```
src/
├── core/              # Core trait definitions (abstractions)
│   ├── detector.rs    # ToolDetector trait
│   ├── mirror.rs      # MirrorConfigurator trait
│   ├── installer.rs   # ToolInstaller trait
│   ├── version.rs     # VersionManager trait
│   └── command.rs     # CommandExecutor trait
│
├── services/          # Infrastructure services
│   ├── shell.rs       # Shell command execution
│   └── speed_test.rs  # Network speed testing
│
├── tools/             # Tool implementations (per programming language)
│   ├── python.rs      # Python tool set (pip, conda, uv, pyenv)
│   ├── node.rs        # Node.js tool set (npm, yarn, pnpm, nvm)
│   ├── java.rs        # Java tool set (maven, gradle, SDKMAN)
│   ├── go.rs          # Go tool set (go modules, gvm)
│   ├── rust.rs        # Rust tool set (cargo, rustup)
│   ├── docker.rs      # Docker tool set
│   └── system.rs      # System tools (brew, apt, chocolatey)
│
├── ui/                # GUI layer (egui)
│   ├── app.rs         # DevHubApp state management
│   └── views/         # View components
│
└── utils/             # Utility functions
```

## Implementation Guidelines

### Error Handling

Use `anyhow::Result` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Result, Context};

fn read_config(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
}
```

### Async/Await Patterns

- Use `tokio` for async runtime
- Use `async-trait` for trait methods
- Avoid blocking operations in async context
- Use `?` operator for error propagation

```rust
use async_trait::async_trait;

#[async_trait]
impl ToolDetector for ShellDetector {
    async fn detect(&self) -> Result<DetectionInfo> {
        // Implementation
    }
}
```

### Mirror Source Configuration

When implementing mirror configurators:

1. **Backup first:** Always create backup before modifying config files
2. **Validate:** Test mirror availability before applying
3. **Restore:** Provide `restore_default()` method
4. **Platform-aware:** Detect OS and use appropriate config paths

Example paths:
- macOS: `~/Library/Application Support/` or `~/.config/`
- Linux: `~/.config/` or `/etc/`
- Windows: `%APPDATA%` or registry

### GUI Development (egui)

- **Don't block UI thread:** Use `tokio::spawn` for long-running tasks
- **Request repaint:** Call `ctx.request_repaint()` after async updates
- **State management:** Keep all UI state in `DevHubApp` struct
- **Components:** Create reusable widgets in `ui/widgets/`

```rust
// Pattern for async operations in GUI
fn on_click(&mut self, ctx: egui::Context) {
    tokio::spawn(async move {
        let result = long_running_operation().await;
        ctx.request_repaint();  // Trigger UI update
    });
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
    }

    #[tokio::test]
    async fn test_async_detection() {
        let detector = ShellDetector::new("python3", "--version");
        let result = detector.detect().await.unwrap();
        assert!(result.installed);
    }
}
```

### Integration Tests
Place in `tests/integration/` directory. Test full workflows:
```bash
cargo test --test integration
```

## Target Audience

**Important:** The primary users are **Chinese developers**. All mirror sources are optimized for China's network environment (Aliyun, Tsinghua, Tencent Cloud, etc.).

## Platform Considerations

- **macOS:** Use Homebrew for system packages, config in `~/Library/Application Support/`
- **Linux:** Detect distro (Ubuntu/Debian use apt, Fedora/CentOS use dnf/yum)
- **Windows:** Support WSL detection, Chocolatey for packages

Use conditional compilation:
```rust
#[cfg(target_os = "macos")]
{ /* macOS-specific code */ }

#[cfg(target_os = "linux")]
{ /* Linux-specific code */ }

#[cfg(target_os = "windows")]
{ /* Windows-specific code */ }
```

## Dependencies

Key crates (from `Cargo.toml`):
- **egui/eframe** - GUI framework
- **tokio** - Async runtime
- **reqwest** - HTTP client for speed testing
- **rusqlite** - SQLite database
- **regex** - Config parsing
- **dirs** - Cross-platform path resolution
- **anyhow/thiserror** - Error handling
- **serde** - Serialization

## Performance Goals

- **Startup time:** < 1 second
- **Speed test:** < 5 seconds for all mirrors
- **UI refresh:** 60fps
- **Binary size:** Minimize with `strip = true` in release profile

## When Adding New Tool Support

1. Add mirror sources to `01_REQUIREMENTS.md`
2. Create implementation in `src/tools/<language>.rs`
3. Implement required traits for the tool
4. Add unit tests
5. Update GUI to display new tool
6. Document config file locations
