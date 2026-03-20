---
change: package-windows-and-proxy-perf
date: 2026-03-20
lane: standard
risk: medium
target: Windows packaging + cross-platform CLI runtime
harness: Rust unit tests with gnullvm toolchain; manual packaging smoke
interface: Windows release packaging contract; proxy passthrough streaming path
status: done
---

## Intent
补上 Windows 可复用的打包脚本，确保 gnullvm 产物和所需 DLL 一起分发，同时继续推进上一轮性能优化，把 proxy passthrough 的热点路径再降一层开销。

## Scope
- In: 当前未提交的 `src/utils.rs` / `src/summary.rs` 性能优化；Windows 打包脚本；`src/main.rs` proxy passthrough 流式转发优化；必要文档更新
- Out: `toml_filter.rs` 重构、`discover` 会话扫描优化、重新设计 Windows 发布目标三元组

## Tasks
| # | Task | Files | Acceptance | Verify | Dep | Done |
|---|------|-------|------------|--------|-----|------|
| 1.1 | 正式纳入并保留当前 `summary/utils` 性能优化 | `src/utils.rs`, `src/summary.rs`, `.changes/done/2026-03-20_optimize-summary-shell-cache/plan.md` | 当前未提交性能优化保持成立并纳入本次变更记录；行为不回退 | 相关单测 + 全量 gnullvm 测试通过 | - | [x] |
| 1.2 | 新增 Windows gnullvm 打包脚本，自动复制 exe 与运行时 DLL | `scripts/package-windows-gnullvm.ps1`, `CONTRIBUTING.md` | 脚本可在已验证的 gnullvm/llvm-mingw 环境中生成可运行的发布目录；默认至少包含 `rtk.exe` 和运行时 DLL | 脚本 `-WhatIf`/参数检查；实际运行一次并验证产物存在 | 1.1 | [x] |
| 1.3 | 优化 `main.rs` passthrough 的流式转发热点 | `src/main.rs` | stdout/stderr 锁不再每个 chunk 重复获取；避免重复格式化命令字符串；现有 proxy 行为不回退 | 全量 gnullvm 测试通过；必要时补单测或 smoke | 1.2 | [x] |

## Verification Notes
- Baseline: 工作区已有未提交的 `src/utils.rs` / `src/summary.rs` 性能优化；之前已验证 `gnullvm + llvm-mingw` 可跑通全量测试；Windows 全局 `rtk.exe` 曾因缺少 `libunwind.dll` 导致无输出
- 1.1: 保留上一轮已实现的 `summary/utils` 性能优化，相关改动继续存在于工作区；`cargo test --quiet --target x86_64-pc-windows-gnullvm` 通过
- 1.2: `scripts/package-windows-gnullvm.ps1` 已可生成 `dist\windows-gnullvm-test4`，打印 `Runtime DLLs: libunwind.dll`；打包出的 `rtk.exe` 通过绝对路径 `--version` 启动验证
- 1.3: `main.rs` proxy passthrough 现在在 stdout/stderr 线程中只获取一次输出锁，并复用预先格式化的命令字符串用于 tracking，避免重复 `join` / `format!`
- Final smoke: 在已验证环境下运行 `cargo test --quiet --target x86_64-pc-windows-gnullvm`，结果 `991 passed, 0 failed, 3 ignored`

## Change Notes
- MODIFIED: `src/utils.rs` 保留 shell 自动探测缓存优化
- MODIFIED: `src/summary.rs` 保留轻量级大小写不敏感匹配与手写数字提取优化
- MODIFIED: `src/main.rs` 优化 proxy passthrough 的流式输出锁使用与 tracking 字符串构造
- MODIFIED: `CONTRIBUTING.md` 记录 Windows 打包脚本用法
- ADDED: `scripts/package-windows-gnullvm.ps1`
