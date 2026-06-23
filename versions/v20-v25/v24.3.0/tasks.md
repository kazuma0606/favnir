# v24.3.0 — 継続的パフォーマンス回帰検知タスク

## ステータス: COMPLETE（2026-06-23）

---

## タスク一覧

### T0: 事前確認 + v24.2.0.json 修正

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.2.0"` であること
- [x] `grep -n "mod v243000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "cmd_bench_compare" fav/src/driver.rs | head -3` — 0 件
- [x] **T0-1**: `benchmarks/v24.2.0.json` の `metrics` を数値のみに修正（`stage4_deferred: true`, `new_ignored_tests: 2`, `fixture_count: 5` を削除し `test_count` / `duration_ms` のみに）

---

### T1: `fav/src/driver.rs` — `cmd_bench_compare` 追加

- [x] **T1-1**: `fn extract_bench_metrics(json: &str) -> Vec<(String, f64)>` を追加
- [x] **T1-2**: `fn extract_bench_version(json: &str) -> String` を追加
- [x] **T1-3**: `pub fn cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` を追加
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: `fav/src/main.rs` — `"bench"` サブコマンド追加

- [x] `Some("spec")` アームの直後に `Some("bench")` アームを追加
  - `--baseline` / `--current` / `--threshold`（省略時 5.0）/ `--emit-md` 解析
  - `driver::cmd_bench_compare` 呼び出し
  - `ok=false` の場合 `process::exit(1)`
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T3: `fav/src/driver.rs` — v243000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_2_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T5-1 より前に必須）**: `v242000_tests::version_is_24_2_0` テスト関数を**削除**
- [x] **T3-2**: `v243000_tests` モジュールを `v242000_tests` の直後に追加（5 件）
  - `version_is_24_3_0`
  - `bench_compare_no_regression`
  - `bench_compare_regression_detected`
  - `bench_compare_emit_md_has_header`
  - `changelog_has_v24_3_0`
- [x] `cargo test v243000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1944 件合格）を確認

---

### T4: `.github/workflows/bench.yml` 更新

- [x] `--baseline benchmarks/v20.0.0.json` → `--baseline benchmarks/v24.2.0.json`
- [x] `--threshold 10` → `--threshold 5`
- [x] `--emit-md` の行末に `|| exit 1` 追加

---

### T5: Cargo.toml + CHANGELOG + benchmarks + mdx

- [x] `fav/Cargo.toml` の `version = "24.2.0"` → `"24.3.0"` に変更（T3-1 完了後）
- [x] `CHANGELOG.md` 先頭に v24.3.0 エントリを追加
- [x] `benchmarks/latest.json` を新規作成（テンプレート、`"timestamp": ""` フィールド必須 — compare.fav が参照）
- [x] `benchmarks/v24.3.0.json` を新規作成（test_count: 1944、`duration_ms` は実測値に置き換えること）
- [x] `site/content/docs/performance/benchmark-regression.mdx` を新規作成
- [x] `cargo test v243000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1944 件合格）

---

## テスト一覧（v243000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_3_0` | Cargo.toml に `version = "24.3.0"` | — |
| `bench_compare_no_regression` | duration_ms +0.6% < threshold 5.0% | `(true, "OK: ...")` |
| `bench_compare_regression_detected` | duration_ms +22% > threshold 5.0% | `(false, "REGRESSION: ...")` |
| `bench_compare_emit_md_has_header` | emit_md=true の出力に `# Benchmark` | — |
| `changelog_has_v24_3_0` | CHANGELOG.md に `[v24.3.0]` | — |

---

---

## コードレビュー対応（2026-06-23 — code-reviewer 指摘）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `extract_bench_metrics` の JSON 構造変化で誤パースの可能性 | spec/plan にフラット数値のみ制約を明記済み。現在の JSON 定義は全て安全。追加対応不要 |
| [MED] | `fav run` が `Result.err` 時に exit 1 を返すか不明 | `|| exit 1` を追加することで伝搬の有無に関わらずカバー済み。bench.yml にコメント追記 |
| [LOW] | `cmd_bench_compare` の baseline-only キー無視の設計意図が docstring 未記載 | `///` コメントに設計上の注意を追記 |
| [LOW] | `compare.fav` の `pct_change`（Float 返し）に `bind` を使用 | v20.1.0 からの既存コード — v24.3.0 スコープ外 |

---

## 完了条件チェックリスト

- [x] `benchmarks/v24.2.0.json` の metrics が数値のみに修正済み
- [x] `pub fn cmd_bench_compare` 実装済み
- [x] `fav bench` CLI サブコマンド追加済み（`--baseline` 必須、`--threshold` 省略時 5.0）
- [x] `v242000_tests::version_is_24_2_0` が削除済み（T5-1 より前）
- [x] `cargo test v243000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1944 件合格）
- [x] `.github/workflows/bench.yml` — baseline: v24.2.0.json、threshold: 5、`|| exit 1` 追加済み
- [x] `CHANGELOG.md` に v24.3.0 エントリ
- [x] `benchmarks/latest.json` 作成済み
- [x] `benchmarks/v24.3.0.json` 作成済み（test_count: 1944）
- [x] `site/content/docs/performance/benchmark-regression.mdx` 作成済み
