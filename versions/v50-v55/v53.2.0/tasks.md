# Tasks: v53.2.0 — bench × par 統合（par stage 個別計測）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3165 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `BenchStats` に `par_stages` フィールドが**存在しない**ことを確認:
  - [x] `rg -n "par_stages" fav/src/driver.rs` → 0 件
- [x] `collect_par_stage_names` が**存在しない**ことを確認:
  - [x] `rg -n "collect_par_stage_names" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53200_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53200_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53100_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53100_tests" fav/src/driver.rs` → 行番号を特定
- [x] `BenchStats` 構造体の現状フィールドを確認:
  - [x] `rg -n "pub struct BenchStats" fav/src/driver.rs` → 行番号を特定
- [x] `FlwStep::Par` と `FlwStep::ParDistributed` が AST に存在することを確認:
  - [x] `rg -n "Par(" fav/src/ast.rs` → `FlwStep::Par(Vec<String>)` を特定
- [x] `seq ... = par [...]` 構文が `FlwDef` にパースされることを確認:
  - [x] `rg -n "FlwDef\|parse_flw\|parse_seq" fav/src/frontend/parser.rs` → 行番号を特定
- [x] `Cargo.toml` の現在バージョンが `53.1.0` であることを確認

---

## T1 — `collect_par_stage_names` 追加

- [x] `driver.rs` bench セクション（`// ── bench ──` コメント付近）に関数追加:
  - [x] `pub fn collect_par_stage_names(program: &ast::Program) -> Vec<String>`
  - [x] `Item::FlwDef` を走査して `FlwStep::Par` / `FlwStep::ParDistributed` を収集
  - [x] 重複なし（`!names.contains(n)` でガード）
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `BenchStats` 更新

- [x] `BenchStats` 構造体に `pub par_stages: Vec<String>` フィールドを追加
- [x] `compute_bench_stats` の返却値に `par_stages: vec![]` を追加
- [x] `bench_stats_to_json` の `serde_json::json!` に `"par_stages": s.par_stages` を追加
- [x] **`v176000_tests::bench_json_output`（行 32061）** の BenchStats リテラルに `par_stages: vec![]` を追加:
  - [x] 追加しないと struct literal 網羅性エラーでコンパイル失敗する
- [x] `bench_stats_to_compare_json` は変更対象外（`metrics` マップ形式のため BenchStats フィールドを直接参照しない）— 誤って変更しないこと
- [x] `cargo build` → コンパイルエラーなし確認（`bench_json_output` 含む既存構築箇所が全て通るか確認）

---

## T3 — `cmd_bench` 更新

- [x] `cmd_bench` 内 `exec_bench_case_timed` 成功パスで `par_stages` を後付け:
  - [x] `let mut stats = compute_bench_stats(desc, timings);` に変更（`let` → `let mut`）
  - [x] `stats.par_stages = collect_par_stage_names(prog);` を追加
- [x] `cargo build` → コンパイルエラーなし確認

---

## T4 — `driver.rs` — `v53200_tests` 追加

- [x] `rg -n "v53100_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53100_tests` モジュールの直前に `v53200_tests` を追加:
  - [x] `bench_par_stage_individual` テスト:
    - [x] `Parser::parse_str` でパース（stage Enrich, Validate + `seq pipeline = par [Enrich, Validate]`）
    - [x] `collect_par_stage_names` が ["Enrich", "Validate"] を返すことを assert
  - [x] `bench_par_stage_total` テスト:
    - [x] `BenchStats { ..., par_stages: vec!["Enrich", "Validate"] }` を構築
    - [x] `bench_stats_to_json` 呼び出し → JSON に `"par_stages"` / `"Enrich"` / `"Validate"` が含まれることを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T5 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.1.0"` → `version = "53.2.0"` に変更
- [x] v53100_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3167 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T6 — 後処理

- [x] `CHANGELOG.md` に v53.2.0 エントリ追加
- [x] `versions/current.md` を v53.2.0（3167 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.2.0 実績欄を更新（未実施 → COMPLETE、テスト数 3167）:
  - [x] 推定値 3161 → 実績 3167 に修正（v53.1.0 コードレビューで +3、ベースが 3165）
  - [x] v53.3.0 の推定値を 3167 + 2 = 3169 に確認・修正
  - [x] v53.4.0〜v54.0.0 の推定値もロードマップ上の差分（+6）を反映するか、または冒頭に「v53.2.0 完了時点でベースが +6 ずれ」という注記を追加するか判断し記録
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
