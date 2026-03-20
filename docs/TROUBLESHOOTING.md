# RTK Troubleshooting Guide

## Problem: "rtk gain" command not found

### Symptom
```bash
$ rtk --version
rtk 1.0.0  # (or similar)

$ rtk gain
rtk: 'gain' is not a rtk command. See 'rtk --help'.
```

### Root Cause
You installed the **wrong rtk package**. You have **Rust Type Kit** (reachingforthejack/rtk) instead of **Rust Token Killer** (rtk-ai/rtk).

### Solution

**1. Uninstall the wrong package:**
```bash
cargo uninstall rtk
```

**2. Install the correct one (Token Killer):**

#### Quick Install (Linux/macOS)
```bash
curl -fsSL https://github.com/rtk-ai/rtk/blob/master/install.sh | sh
```

#### Alternative: Manual Installation
```bash
cargo install --git https://github.com/rtk-ai/rtk
```

**3. Verify installation:**
```bash
rtk --version
rtk gain  # MUST show token savings stats, not error
```

If `rtk gain` now works, installation is correct.

---

## Problem: Confusion Between Two "rtk" Projects

### The Two Projects

| Project | Repository | Purpose | Key Command |
|---------|-----------|---------|-------------|
| **Rust Token Killer** ✅ | rtk-ai/rtk | LLM token optimizer for Claude Code | `rtk gain` |
| **Rust Type Kit** ❌ | reachingforthejack/rtk | Rust codebase query and type generator | `rtk query` |

### How to Identify Which One You Have

```bash
# Check if "gain" command exists
rtk gain

# Token Killer → Shows token savings stats
# Type Kit → Error: "gain is not a rtk command"
```

---

## Problem: cargo install rtk installs wrong package

### Why This Happens
If **Rust Type Kit** is published to crates.io under the name `rtk`, running `cargo install rtk` will install the wrong package.

### Solution
**NEVER use** `cargo install rtk` without verifying.

**Always use explicit repository URLs:**

```bash
# CORRECT - Token Killer
cargo install --git https://github.com/rtk-ai/rtk

# OR install from fork
git clone https://github.com/rtk-ai/rtk.git
cd rtk && git checkout feat/all-features
cargo install --path . --force
```

**After any installation, ALWAYS verify:**
```bash
rtk gain  # Must work if you want Token Killer
```

---

## Problem: RTK not working in Claude Code

### Symptom
Claude Code doesn't seem to be using rtk, outputs are verbose.

### Checklist

**1. Verify rtk is installed and correct:**
```bash
rtk --version
rtk gain  # Must show stats
```

**2. Initialize rtk for Claude Code:**
```bash
# Global (all projects)
rtk init --global

# Per-project
cd /your/project
rtk init
```

**3. Verify CLAUDE.md file exists:**
```bash
# Check global
cat ~/.claude/CLAUDE.md | grep rtk

# Check project
cat ./CLAUDE.md | grep rtk
```

**4. Install auto-rewrite hook (recommended for automatic RTK usage):**

**Option A: Automatic (recommended)**
```bash
rtk init -g
# → Installs hook + RTK.md automatically
# → Follow printed instructions to add hook to ~/.claude/settings.json
# → Restart Claude Code

# Verify installation
rtk init --show  # Should show "✅ Hook: executable, with guards"
```

**Option B: Manual (fallback)**
```bash
# Copy hook to Claude Code hooks directory
mkdir -p ~/.claude/hooks
cp .claude/hooks/rtk-rewrite.sh ~/.claude/hooks/
chmod +x ~/.claude/hooks/rtk-rewrite.sh
```

Then add to `~/.claude/settings.json` (replace `~` with full path):
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/yourname/.claude/hooks/rtk-rewrite.sh"
          }
        ]
      }
    ]
  }
}
```

**Note**: Use absolute path in `settings.json`, not `~/.claude/...`

---

## Problem: RTK not working in OpenCode

### Symptom
OpenCode runs commands without rtk, outputs are verbose.

### Checklist

**1. Verify rtk is installed and correct:**
```bash
rtk --version
rtk gain  # Must show stats
```

**2. Install the OpenCode plugin (global only):**
```bash
rtk init -g --opencode
```

**3. Verify plugin file exists:**
```bash
ls -la ~/.config/opencode/plugins/rtk.ts
```

**4. Restart OpenCode**
OpenCode must be restarted to load the plugin.

**5. Verify status:**
```bash
rtk init --show  # Should show "OpenCode: plugin installed"
```

---

## Problem: RTK commands fail on Windows ("program not found" or "No such file")

### Symptom
```
rtk vitest --run
# Error: program not found
# Or: The system cannot find the file specified

rtk lint .
# Error: No such file or directory
```

### Root Cause
On Windows, Node.js tools (vitest, eslint, tsc, etc.) are installed as `.CMD` or `.BAT` wrapper scripts, not as native `.exe` binaries. Rust's `std::process::Command::new("vitest")` does not honor the Windows `PATHEXT` environment variable, so it cannot find `vitest.CMD` even when it's on PATH.

### Solution
Update to rtk v0.23.1+ which resolves this via the `which` crate for proper PATH+PATHEXT resolution. All 16+ command modules now use `resolved_command()` instead of `Command::new()`.

```bash
cargo install --git https://github.com/rtk-ai/rtk
rtk --version  # Should be 0.23.1+
```

### Affected Commands
All commands that spawn external tools: `rtk vitest`, `rtk lint`, `rtk tsc`, `rtk pnpm`, `rtk playwright`, `rtk prisma`, `rtk next`, `rtk prettier`, `rtk ruff`, `rtk pytest`, `rtk pip`, `rtk mypy`, `rtk golangci-lint`, and others.

### Related: `rtk err`, `rtk test`, or `rtk summary` use the wrong shell on Windows

If you use RTK from Codex, Copilot, or another Windows-first agent, the wrapper commands
`rtk err <cmd>`, `rtk test <cmd>`, and `rtk summary <cmd>` now select a shell using this order:

1. `RTK_SHELL`
2. `SHELL`
3. `pwsh`
4. `powershell`
5. `COMSPEC`
6. `cmd`

This matters because PowerShell commands like `Get-ChildItem` or `$env:FOO` will fail if the
wrapper ends up running through `cmd.exe`.

**Recommended override for Codex / Copilot on Windows:**

```powershell
$env:RTK_SHELL = "pwsh"
rtk summary "Get-ChildItem Env: | Select-Object -First 5"
```

If you need an explicit path:

```powershell
$env:RTK_SHELL = "C:\Program Files\PowerShell\7\pwsh.exe"
```

If you prefer Git Bash semantics:

```powershell
$env:RTK_SHELL = "C:\Program Files\Git\bin\bash.exe"
rtk err "printf 'hello\n'"
```

### Validated Windows Rust Test Environment

If repository tests fail on Windows due to linker or toolchain drift, use this known-good setup:

```powershell
$toolchain = 'D:\Rust\.rustup\toolchains\stable-x86_64-pc-windows-gnullvm'
$llvm = 'D:\llvm-mingw\llvm-mingw-20260311-ucrt-x86_64\bin'

$env:PATH = "$llvm;D:\Rust\.cargo\bin;" + $env:PATH
$env:RUSTC = Join-Path $toolchain 'bin\rustc.exe'
$env:RUSTDOC = Join-Path $toolchain 'bin\rustdoc.exe'
$env:CC = Join-Path $llvm 'x86_64-w64-mingw32-clang.exe'
$env:AR = Join-Path $llvm 'llvm-ar.exe'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER = Join-Path $llvm 'x86_64-w64-mingw32-clang.exe'

& (Join-Path $toolchain 'bin\cargo.exe') test --quiet --target x86_64-pc-windows-gnullvm
```

This was verified successfully for this repository.

Avoid these failing combinations:

- `stable-x86_64-pc-windows-gnu` with `E:\Link7\anyui\mingw\bin` early in `PATH`
- `cargo.exe` / `rustc.exe` rustup shims without a configured default toolchain

---

## Problem: "command not found: rtk" after installation

### Symptom
```bash
$ cargo install --path . --force
   Compiling rtk v0.7.1
    Finished release [optimized] target(s)
  Installing ~/.cargo/bin/rtk

$ rtk --version
zsh: command not found: rtk
```

### Root Cause
`~/.cargo/bin` is not in your PATH.

### Solution

**1. Check if cargo bin is in PATH:**
```bash
echo $PATH | grep -o '[^:]*\.cargo[^:]*'
```

**2. If not found, add to PATH:**

For **bash** (`~/.bashrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For **zsh** (`~/.zshrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For **fish** (`~/.config/fish/config.fish`):
```fish
set -gx PATH $HOME/.cargo/bin $PATH
```

**3. Reload shell config:**
```bash
source ~/.bashrc  # or ~/.zshrc or restart terminal
```

**4. Verify:**
```bash
which rtk
rtk --version
rtk gain
```

---

## Problem: Compilation errors during installation

### Symptom
```bash
$ cargo install --path .
error: failed to compile rtk v0.7.1
```

### Solutions

**1. Update Rust toolchain:**
```bash
rustup update stable
rustup default stable
```

**2. Clean and rebuild:**
```bash
cargo clean
cargo build --release
cargo install --path . --force
```

**3. Check Rust version (minimum required):**
```bash
rustc --version  # Should be 1.70+ for most features
```

**4. If still fails, report issue:**
- GitHub: https://github.com/rtk-ai/rtk/issues

---

## Need More Help?

**Report issues:**
- Fork-specific: https://github.com/rtk-ai/rtk/issues
- Upstream: https://github.com/rtk-ai/rtk/issues

**Run the diagnostic script:**
```bash
# From the rtk repository root
bash scripts/check-installation.sh
```

This script will check:
- ✅ RTK installed and in PATH
- ✅ Correct version (Token Killer, not Type Kit)
- ✅ Available features (pnpm, vitest, next, etc.)
- ✅ Claude Code integration (CLAUDE.md files)
- ✅ Auto-rewrite hook status

The script provides specific fix commands for any issues found.
