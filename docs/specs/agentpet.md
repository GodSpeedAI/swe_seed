# 🛠️ Cross-Platform Refactoring Specification for agentpet

---

> **Key Takeaway:**
> This specification provides a complete, actionable plan to refactor the [agentpet](https://github.com/ntd4996/agentpet) project for robust cross-platform support (Windows, Linux, macOS). It details every required file change, new file, and architectural adaptation—especially the migration to a `justfile`-driven workflow and the extension of the Tauri+Rust codebase to Linux. All instructions are precise and copy-paste-ready for direct implementation.

---

## Table of Contents

1. [Architectural Audit & Core Decisions](#1-architectural-audit--core-decisions)
2. [File-by-File Change List](#2-file-by-file-change-list)
3. [Complete justfile Specification](#3-complete-justfile-specification)
4. [Cross-Platform IPC Strategy](#4-cross-platform-ipc-strategy)
5. [Dependency & Tooling Prerequisites](#5-dependency--tooling-prerequisites)
6. [README.md Update Specification](#6-readmemd-update-specification)
7. [CI/CD Workflow Files](#7-cicd-workflow-files)
8. [Settlement Criteria](#8-settlement-criteria)

---

## 1. Architectural Audit & Core Decisions

### Current State

| Platform | Implementation | Build System | Automation | IPC | Platform Issues |
|----------|----------------|--------------|------------|-----|----------------|
| macOS    | Swift + SwiftUI, Unix-socket daemon | Xcode/SwiftPM | Bash script (`build-app.sh`) | Unix socket (`/tmp/agentpet.sock`) | Not portable, bash-only scripts, POSIX permissions |
| Windows  | Tauri + Rust   | Cargo/Tauri  | None       | Windows-specific code | Uses `APPDATA`, `cmd /C`, no Linux support |
| Linux    | None           | N/A          | N/A        | N/A | No implementation |

### Core Decisions

- **Swift+SwiftUI is macOS-only.**
  No practical path to port to Linux or Windows.
- **Tauri+Rust is cross-platform.**
  Extend the Windows Tauri app to support Linux (and optionally macOS in the future).
- **All automation must move to a cross-platform justfile.**
- **IPC must be abstracted:**
  - Unix socket on macOS/Linux
  - Named pipe on Windows

---

## 2. File-by-File Change List

### 2.1. `scripts/build-app.sh`

**Action:** DELETE
**Why:** Bash-only, not portable. All build automation moves to justfile.

---

### 2.2. `justfile` (NEW, at project root)

**Action:** CREATE
**Why:** Primary cross-platform automation entrypoint. SeeSection 3](#3-complete-justfile-specification) for full content.

---

### 2.3. `windows/` → `desktop/`

**Action:** RENAME directory
**Why:** Tauri+Rust codebase will now target Windows and Linux (and optionally macOS).
**How:**

```shell
git mv windows desktop
```

---

### 2.4. `desktop/src/main.rs`

**Action:** MODIFY
**What to change:**

- Remove Windows-only path logic (`APPDATA`, `cmd /C`)
- Use `dirs` crate for config/data directories
- Refactor IPC logic to use cross-platform abstraction (seection 4](#4-cross-platform-ipc-strategy))
- Use `std::path::PathBuf` for all paths

---

### 2.5. `desktop/src/ipc.rs` (NEW or MODIFY)

**Action:** CREATE or EXTEND
**What to implement:**

- Cross-platform IPC:
  - Unix domain socket on macOS/Linux
  - Named pipe on Windows
- Use `tokio` async runtime for all platforms
- Use `uds_windows` crate for named pipes on Windows

---

### 2.6. `desktop/Cargo.toml`

**Action:** MODIFY
**What to change:**

- Add `dirs` crate for platform-agnostic directories
- Add `uds_windows` crate under `[target.'cfg(windows)'.dependencies]`
- Add `nix = { version = "0.29", features = ["user"] }` under
  `[target.'cfg(unix)'.dependencies]` for the Unix UID fallback used by IPC
- Ensure all dependencies are cross-platform

---

### 2.7. `desktop/tauri.conf.json`

**Action:** MODIFY
**What to change:**

- Update bundle targets:
  - Windows: `msi`, `nsis`
  - Linux: `deb`, `appimage`
- Ensure icons, resources, and metadata are correct for all platforms

---

### 2.8. macOS Swift source (`Sources/AgentPetDaemon/main.swift` and related)

**Action:** MODIFY
**What to change:**

- Guard Unix socket and POSIX permission code with `#if os(macOS)`
- Prevent accidental Linux Swift builds from using macOS-specific code

---

### 2.9. `.github/workflows/build-macos.yml` (NEW)

**Action:** CREATE
**What to implement:**

- Matrix build for macOS using Xcode 16/Swift 6
- Runs `just build-macos`

---

### 2.10. `.github/workflows/build-desktop.yml` (NEW)

**Action:** CREATE
**What to implement:**

- Matrix build for Windows and Linux using Tauri
- Runs `just build-windows` and `just build-linux`

---

### 2.11. `README.md`

**Action:** UPDATE
**What to change:**

- Add Linux prerequisites and build instructions
- Add Windows cleanup commands (PowerShell)
- Replace `./scripts/build-app.sh` references with `just build` for the current platform,
  or with `just build-macos`, `just build-linux`, or `just build-windows` in
  platform-specific sections
- Add unified "Quick Start" using just commands

---

## 3. Complete justfile Specification

> **Place this file at the project root as `justfile`.**

```makefile
# justfile for agentpet: cross-platform automation

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-cu"]

# OS-conditional variables
os_name := os()
os_family := os_family()
desktop_dir := justfile_directory() / "desktop"

# Build recipes
[macos]
build-macos:
    xcodebuild -scheme AgentPet -configuration Release -derivedDataPath build

[windows]
[working-directory('desktop')]
build-windows:
    cargo tauri build --release

[linux]
[working-directory('desktop')]
build-linux:
    cargo tauri build --release

# Dispatcher
build:
    just build-{{os_name}}

# Dev recipes
[macos]
dev-macos:
    open build/AgentPet.app

[windows]
[working-directory('desktop')]
dev-desktop:
    cargo tauri dev

[linux]
[working-directory('desktop')]
dev-desktop:
    cargo tauri dev

# Clean
clean:
    just clean-{{os_name}}

[windows]
clean-windows:
    Remove-Item -Recurse -Force "$env:APPDATA\agentpet" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$env:USERPROFILE\.agentpet" -ErrorAction SilentlyContinue

[macos]
clean-macos:
    rm -rf ~/.agentpet
    rm -f ~/Library/Preferences/com.agentpet.app.plist

[linux]
clean-linux:
    rm -rf ~/.agentpet

# Install dependencies
install-deps:
    just install-deps-{{os_name}}

[windows]
install-deps-windows:
    winget install --id Rustlang.Rustup -e --source winget
    cargo install tauri-cli
    winget install --id Just.Just -e --source winget
    # Visual Studio Build Tools and WebView2 must be installed manually

[macos]
install-deps-macos:
    brew install just
    xcode-select --install
    # Xcode 16 and Swift 6 must be installed from the App Store
    cargo install tauri-cli

[linux]
install-deps-linux:
    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
    cargo install tauri-cli
    cargo install just

# Uninstall / cleanup
uninstall:
    just uninstall-{{os_name}}

[windows]
uninstall-windows:
    Remove-Item -Recurse -Force "$env:APPDATA\agentpet" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$env:USERPROFILE\.agentpet" -ErrorAction SilentlyContinue

[macos]
uninstall-macos:
    rm -rf ~/.agentpet
    rm -f ~/Library/Preferences/com.agentpet.app.plist

[linux]
uninstall-linux:
    rm -rf ~/.agentpet

# Help
help:
    @echo "AgentPet cross-platform automation"
    @echo "Available commands:"
    @echo "  just build           # Build for current platform"
    @echo "  just build-macos     # Build macOS app"
    @echo "  just build-windows   # Build Windows app"
    @echo "  just build-linux     # Build Linux app"
    @echo "  just dev-macos       # Run macOS app"
    @echo "  just dev-desktop     # Run Tauri dev server (Windows/Linux)"
    @echo "  just clean           # Remove user data"
    @echo "  just install-deps    # Install all prerequisites"
    @echo "  just uninstall       # Remove all app data"
```

---

## 4. Cross-Platform IPC Strategy

### IPC Abstraction

- **macOS:**
  Use Unix domain socket at `$TMPDIR/agentpet.sock` (fallback to `/tmp/agentpet.sock` if `$TMPDIR` is unset).
- **Linux:**
  Use `$XDG_RUNTIME_DIR/agentpet.sock` if set, else `/tmp/agentpet-$UID.sock`.
- **Windows:**
  Use named pipe: `\\.\pipe\agentpet` (via `uds_windows` crate or `tokio` named pipe support).

### Implementation Details

- **In `desktop/src/ipc.rs`:**
  - Export a function `get_ipc_endpoint()` returning the correct socket/pipe path for the current platform.
  - Use `cfg!(windows)` and `cfg!(unix)` for compile-time branching.
  - Use `dirs` crate to resolve runtime directories.
  - All IPC server/client code must use this abstraction.

**Example Rust snippet:**

```rust
// desktop/src/ipc.rs
pub fn get_ipc_endpoint() -> String {
    #[cfg(target_os = "windows")]
    {
        r"\\.\pipe\agentpet".to_string()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            format!("{}/agentpet.sock", xdg_runtime_dir)
        } else {
            format!("/tmp/agentpet-{}.sock", nix::unistd::Uid::current())
        }
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("TMPDIR")
            .map(|tmp| format!("{}/agentpet.sock", tmp.trim_end_matches('/')))
            .unwrap_or_else(|_| "/tmp/agentpet.sock".to_string())
    }
}
```

---

## 5. Dependency & Tooling Prerequisites

| Platform | Prerequisites |
|----------|---------------|
| **All**  | `just` command runner (`cargo install just`, `brew install just`, `winget install just`, or `scoop install just`) |
| **macOS** | Xcode 16, Swift 6, `xcode-select --install`, `brew install just`, `cargo install tauri-cli` |
| **Windows** | Rust stable toolchain, `cargo install tauri-cli`, Visual Studio C++ Build Tools, WebView2, `winget install just` |
| **Linux** | Rust stable toolchain, `cargo install tauri-cli`, `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`, `cargo install just` |

---

## 6. README.md Update Specification

### Add/Replace the following sections

#### Quick Start

```markdown
## Quick Start

1. **Install prerequisites:**

   - macOS: `brew install just`, install Xcode 16 and Swift 6 from the App Store
   - Windows: `winget install just`, install Rust, Visual Studio C++ Build Tools, WebView2
   - Linux: `cargo install just`, install Rust, and run:
     ```
     sudo apt-get update
     sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
     ```

2. **Install project dependencies:**
   ```

   just install-deps

   ```

3. **Build the app:**
   - macOS: `just build-macos`
   - Windows: `just build-windows`
   - Linux: `just build-linux`

4. **Run the app:**
   - macOS: `just dev-macos`
   - Windows/Linux: `just dev-desktop`
```

#### Linux Build Instructions

```markdown
## Linux Build Instructions

1. Install system dependencies:
   ```

   sudo apt-get update
   sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

   ```

2. Install Rust and Tauri CLI:
   ```

   curl --proto '=https' --tlsv1.2 -sSf <https://sh.rustup.rs> | sh
   cargo install tauri-cli

   ```

3. Build:
   ```

   just build-linux

   ```
```

#### Windows Cleanup Commands

```markdown
## Windows Cleanup

To remove all AgentPet data on Windows, run in PowerShell:
```

Remove-Item -Recurse -Force "$env:APPDATA\agentpet"
Remove-Item -Recurse -Force "$env:USERPROFILE\.agentpet" -ErrorAction SilentlyContinue

```
```

#### Build command migration

Use `just build-macos`, `just build-linux`, or `just build-windows` only where the
documentation is explicitly platform-specific.

---

## 7. CI/CD Workflow Files

### `.github/workflows/build-macos.yml`

```yaml
name: Build macOS

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-macos:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - name: Install Just
        run: brew install just
      - name: Install Xcode 16
        run: sudo xcode-select -s /Applications/Xcode_16.0.app
      - name: Build macOS app
        run: just build-macos
```

---

### `.github/workflows/build-desktop.yml`

```yaml
name: Build Desktop (Windows & Linux)

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Just
        run: choco install just
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Install Tauri CLI
        run: cargo install tauri-cli
      - name: Build Windows app
        run: just build-windows

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Just
        run: cargo install just
      - name: Install system dependencies
        run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Install Tauri CLI
        run: cargo install tauri-cli
      - name: Build Linux app
        run: just build-linux
```

---

## 8. Settlement Criteria

- All build, dev, clean, install, and uninstall commands work on Windows, Linux, and macOS via `just`.
- Tauri+Rust app builds and runs on Windows and Linux (and optionally macOS).
- Swift+SwiftUI app builds and runs on macOS.
- IPC works on all platforms (Unix socket on macOS/Linux, named pipe on Windows).
- CI/CD pipelines build and test on all three platforms.
- README is accurate and up-to-date.
- No platform-specific scripts remain outside the justfile.

---
