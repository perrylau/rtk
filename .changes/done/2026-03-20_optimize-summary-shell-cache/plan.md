---
change: optimize-summary-shell-cache
date: 2026-03-20
lane: standard
risk: low
target: Windows / cross-platform CLI runtime
harness: Rust unit tests with gnullvm toolchain
interface: summary heuristics; shell selection helper
status: done
---

## Intent
降低 RTK 在常用包装命令路径上的额外开销，优先优化 `summary` 的热点逻辑和 `shell_command()` 的重复 shell 探测。

## Scope
- In: `src/utils.rs` 的 shell 探测缓存；`src/summary.rs` 的重复 regex 编译与不必要字符串分配；必要测试更新
- Out: `main.rs` 的 passthrough 捕获路径、`toml_filter.rs` 管线重构、`discover` 会话扫描优化

## Tasks
| # | Task | Files | Acceptance | Verify | Dep | Done |
|---|------|-------|------------|--------|-----|------|
| 1.1 | 为 shell 选择增加缓存，避免每次命令都重复 `which` 探测 | `src/utils.rs` | `shell_command()` 在未显式设置环境变量时不再每次重复查找 `pwsh`/`powershell`；现有行为不回退 | 单测覆盖缓存分支或选择分支；全量 `utils::tests::` 通过 | - | [x] |
| 1.2 | 优化 `summary` 热点路径，去掉动态 regex 编译并减少重复分配 | `src/summary.rs` | `extract_number()` 不再每次动态编译 regex；摘要逻辑在现有输出语义下保持一致 | 相关 `summary`/全量测试通过；抽样运行 `rtk summary` 行为无明显回退 | 1.1 | [x] |

## Verification Notes
- Baseline: 性能审查确认热点位于 `src/utils.rs::shell_command()` 与 `src/summary.rs`；当前 `.changes/` 无活动变更
- 1.1: `shell_command()` 现在使用 `OnceLock<AutoShellAvailability>` 缓存自动发现的 `pwsh` / `powershell` 可用性，显式 `RTK_SHELL` / `SHELL` 仍保持实时读取
- 1.2: `summary` 去掉了整串 `to_lowercase()` 扫描和动态 `Regex::new(...)`；改为 ASCII case-insensitive 匹配与手写数字提取，并补充了 4 个单测覆盖
- Final smoke: 提权运行 `stable-x86_64-pc-windows-gnullvm + llvm-mingw` 的 `cargo test --quiet --target x86_64-pc-windows-gnullvm`，结果 `991 passed, 0 failed, 3 ignored`

## Change Notes
- MODIFIED: `src/utils.rs` 为自动 shell 探测增加一次性缓存，减少重复 PATH 查找
- MODIFIED: `src/summary.rs` 改用轻量级大小写不敏感匹配与手写数字提取，减少重复分配和动态 regex 编译
