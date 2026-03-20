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
- In: 当前已提交的 `src/utils.rs` / `src/summary.rs` 性能优化延续验证；Windows 打包脚本；`src/main.rs` proxy passthrough 流式转发与 tracking 路径优化；必要文档更新
- Out: `toml_filter.rs` 重构、`discover` 会话扫描优化、重新设计 Windows 发布目标三元组

## Tasks
| # | Task | Files | Acceptance | Verify | Dep | Done |
|---|------|-------|------------|--------|-----|------|
| 1.1 | 正式纳入并保留当前 `summary/utils` 性能优化 | `src/utils.rs`, `src/summary.rs`, `.changes/done/2026-03-20_optimize-summary-shell-cache/plan.md` | 当前已提交性能优化保持成立；行为不回退 | 相关单测 + 全量 gnullvm 测试通过 | - | [x] |
| 1.2 | 新增 Windows gnullvm 打包脚本，自动复制 exe 与运行时 DLL | `scripts/package-windows-gnullvm.ps1`, `CONTRIBUTING.md` | 脚本可在已验证的 gnullvm/llvm-mingw 环境中生成可运行的发布目录；默认至少包含 `rtk.exe` 和运行时 DLL | 脚本实际运行一次并验证产物存在 | 1.1 | [x] |
| 1.3 | 优化 `main.rs` passthrough 的流式转发热点 | `src/main.rs` | stdout/stderr 锁不再每个 chunk 重复获取；避免重复格式化命令字符串；现有 proxy 行为不回退 | 全量 gnullvm 测试通过 | 1.2 | [x] |
| 1.4 | 将 `proxy` 切换为真正的 passthrough tracking，避免为 tracking 保留整块输出 | `src/main.rs`, `README.md`, `docs/FEATURES.md` | `proxy` 仍然实时转发 stdout/stderr，但 tracking 改为 timing-only；不再为 token 估算收集完整输出；文档与行为一致 | 全量 gnullvm 测试通过；`rtk proxy` 手动 smoke 通过；必要文档已更新 | 1.3 | [x] |

## Verification Notes
- Baseline: 现有 `proxy` 路径仍会完整保留 stdout/stderr 到内存后调用 `timer.track(...)`；仓库中其它 passthrough 命令已使用 `track_passthrough()`
- 1.1: 已完成并在上一轮提交中验证
- 1.2: 已完成并在上一轮提交中验证
- 1.3: 已完成并在上一轮提交中验证
- 1.4: `proxy` 已改为流式直写 stdout/stderr，不再保留完整输出到内存；tracking 切换为 `track_passthrough()`；README 与 FEATURES 文案已同步更新
- Final smoke: `cargo test --quiet --target x86_64-pc-windows-gnullvm` 通过（`991 passed, 0 failed, 3 ignored`）；`rtk proxy pwsh -NoProfile -Command "1..20"` 手动 smoke 正常输出前几行

## Change Notes
- MODIFIED: `src/main.rs` 的 `proxy` 路径改为 timing-only passthrough tracking，移除完整输出保留
- MODIFIED: `README.md` 与 `docs/FEATURES.md` 更新 `rtk proxy` 文案，明确是 timing-only tracking
