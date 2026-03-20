---
change: optimize-windows-exec-discovery
date: 2026-03-20
lane: standard
risk: medium
target: Windows CLI
harness: Rust unit tests / static review in current environment
interface: Windows shell execution contract; Codex/Copilot Windows usage guidance
status: done
---

## Intent
针对主要使用 Codex 和 Copilot 的 Windows 场景，修复 RTK 对临时命令字符串强制走 `cmd /C` 的兼容性问题，并补齐可控的 shell 选择说明。

## Scope
- In: Windows shell 执行抽象、`runner` / `summary` 接入、相关单测补充、Windows/Codex/Copilot 使用说明更新
- Out: Claude `discover` 项目编码修复、Windows hook 原生安装支持、全局路径规范化重构、其余 CLI 子命令行为调整

## Tasks
| # | Task | Files | Acceptance | Verify | Dep | Done |
|---|------|-------|------------|--------|-----|------|
| 1.1 | 引入统一的 shell 执行抽象并接入 `runner` / `summary` | `src/utils.rs`, `src/runner.rs`, `src/summary.rs` | Windows 下不再硬编码 `cmd /C`；优先尊重显式 shell 环境，默认回退策略集中在一处 | 代码检查；新增/更新单测覆盖 shell 选择逻辑；若环境允许运行 `cargo test` 则执行相关测试 | - | [x] |
| 1.2 | 更新 Windows 使用说明，明确 `RTK_SHELL` 与默认回退顺序 | `docs/TROUBLESHOOTING.md` | Windows 用户能看到 shell 选择行为、Codex/Copilot 下的推荐覆盖方式和排障步骤 | 文档检查；示例与实现逻辑一致 | 1.1 | [x] |

## Verification Notes
- Baseline: 已完成 Windows 相关代码路径审查；用户补充主要使用 Codex/Copilot，故将 Claude `discover` 优化移出本次范围；后续确认本机 `cargo` 位于 `D:\Rust`，可调用稳定版 toolchain
- 1.1: 已将 `runner` / `summary` 的 shell 启动收口到 `src/utils.rs::shell_command()`；新增 5 个 shell 选择单测覆盖 `RTK_SHELL`、`SHELL`、Windows 默认回退和 Unix 默认回退；`git diff --check` 通过；排查后确认本机可用测试链为 `stable-x86_64-pc-windows-gnullvm + D:\llvm-mingw\llvm-mingw-20260311-ucrt-x86_64\bin`
- 1.2: 已在 `docs/TROUBLESHOOTING.md` 增加 Windows/Codex/Copilot 说明，文档中的回退顺序与 `shell_command()` 注释和实现一致：`RTK_SHELL` → `SHELL` → `pwsh` → `powershell` → `COMSPEC` → `cmd`
- Final smoke: 在设置 `RUSTC`/`RUSTDOC` 指向 gnullvm toolchain、`CC=x86_64-w64-mingw32-clang.exe`、`AR=llvm-ar.exe`、`CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER=x86_64-w64-mingw32-clang.exe` 后，`cargo test --quiet utils::tests:: --target x86_64-pc-windows-gnullvm` 通过（43/43）；`cargo test --quiet --target x86_64-pc-windows-gnullvm` 通过（987 passed, 3 ignored）；已完成 diff 静态检查、实现与文档一致性检查、工作区变更复核

## Change Notes
- MODIFIED: `src/utils.rs` 新增统一 shell 选择抽象和单测
- MODIFIED: `src/utils.rs` 的两条执行环境相关测试改为真正跨平台
- MODIFIED: `src/runner.rs` 与 `src/summary.rs` 改为复用统一 shell 抽象
- MODIFIED: `docs/TROUBLESHOOTING.md` 增加 Windows 下 Codex/Copilot 的 shell 选择与覆盖说明
