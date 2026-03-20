---
change: optimize-toml-filter-fastpath
date: 2026-03-20
lane: standard
risk: low
target: cross-platform CLI runtime
harness: Rust unit tests with gnullvm toolchain
interface: TOML filter pipeline semantics
status: done
---

## Intent
为 `toml_filter` 增加低风险 fast path，在不需要预处理时让 `match_output` 直接对原始输出短路，减少不必要的拆行和字符串拼接。

## Scope
- In: `src/toml_filter.rs` 的 `apply_filter()` 快路径优化；相关单测补充
- Out: `toml_filter` 全管线重写、`replace` 语义调整、`discover` 和 `main` 其它热点

## Tasks
| # | Task | Files | Acceptance | Verify | Dep | Done |
|---|------|-------|------------|--------|-----|------|
| 1.1 | 为 `match_output` 增加 raw-stdout fast path | `src/toml_filter.rs` | 当 filter 不启用 `strip_ansi` / `replace` 时，`match_output` 可直接对原始 stdout 判断并短路返回；现有语义不回退 | 单测覆盖 fast path 等价行为；全量 gnullvm 测试通过 | - | [x] |
| 1.2 | 记录验证证据并归档 | `.changes/2026-03-20_optimize-toml-filter-fastpath/plan.md` | 计划记录包含基线、验证和变更说明 | 全量测试与必要 smoke 结果写入计划 | 1.1 | [x] |

## Verification Notes
- Baseline: `apply_filter()` 当前总是先 `stdout.lines().map(String::from).collect()`，即使某些 `match_output` 规则可以直接在原始输出上短路
- 1.1: `apply_filter()` 现在在 `strip_ansi=false` 且 `replace=[]` 且存在 `match_output` 时，先直接对原始 `stdout` 做匹配和 `unless` 判断，命中后立即返回，避免先拆行为 `Vec<String>` 再 `join("\n")`
- 1.2: 相关单测 `toml_filter::tests::` 通过（`51 passed`）；全量 `cargo test --quiet --target x86_64-pc-windows-gnullvm` 通过（`993 passed, 0 failed, 3 ignored`）
- Final smoke: `git diff --check` 通过

## Change Notes
- MODIFIED: `src/toml_filter.rs` 为 `match_output` 增加 raw-stdout fast path，并补充等价行为测试
