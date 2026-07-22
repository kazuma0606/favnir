# Tasks: v54.3.0 — パフォーマンスリグレッションスイート CI 統合

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3189 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54300_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v54300_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v54200_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v54200_tests" fav/src/driver.rs` → 行番号を特定
- [x] `Cargo.toml` の現在バージョンが `54.2.0` であることを確認
- [x] `benchmarks/baseline.json` が未存在であることを確認
- [x] bench.yml に `--fail-on-regression` が未存在であることを確認
- [x] `fav bench --all` が未実装であることを確認（main.rs に `"--all"` アームなし）

---

## T1 — `benchmarks/baseline.json` 新規作成

- [x] `benchmarks/baseline.json` を新規作成:
  - [x] `"version"` フィールドを含む
  - [x] `"metrics"` フィールドを含む（checker_ms / compiler_ms / total_pipeline_ms）
  - [x] `"regression": false` フィールドを含む
  - [x] notes に「初回 CI 実行後に fav bench --json 実測値で更新すること（CONTRIBUTING.md 参照）」を明記

---

## T2 — `.github/workflows/bench.yml` 拡張

- [x] `continue-on-error: true` をジョブ全体（job レベル）から削除し `Run benchmarks` ステップ限定に移動:
  - [x] job レベルの `continue-on-error: true` を削除
  - [x] `Run benchmarks` ステップに `continue-on-error: true` を追加
- [x] `v54.3.0` 用の 2 ステップを末尾に追加:
  - [x] `Run perf regression unit tests`: `cargo test bench_ -- --nocapture`（working-directory: fav）
  - [x] `Regression check against baseline`: `$FAV bench --all --compare benchmarks/baseline.json --fail-on-regression || exit 1`
- [x] ファイル末尾に改行を付与

---

## T3 — `main.rs` — `fav bench --all` フラグ追加

- [x] bench コマンドの match アームに追加（`"--fail-on-regression"` の直後）:
  - [x] `"--all" => { i += 1; }` — file 省略と等価の no-op フラグ
- [x] `cargo build` → コンパイルエラーなし確認

---

## T4 — `driver.rs` — `v54300_tests` 追加

- [x] `v54200_tests` の直前に `v54300_tests` を追加（2 テスト）:
  - [x] `ci_perf_regression_suite`:
    - [x] `bench_yml.contains("--fail-on-regression")`
    - [x] `bench_yml.contains("baseline.json")`
    - [x] `bench_yml.contains("cargo test bench_")`
    - [x] `bench_yml.contains("--all")`
  - [x] `ci_perf_baseline_recorded`:
    - [x] `baseline.contains("\"version\"")`
    - [x] `baseline.contains("\"metrics\"")`
    - [x] `baseline.contains("\"regression\"")`

---

## T5 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.2.0"` → `version = "54.3.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3191 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T6 — 後処理

- [x] `CHANGELOG.md`: v54.3.0 エントリ追加（v54.2.0 の直上）
- [x] `versions/current.md` を v54.3.0（3191 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.3.0 実績欄を更新（COMPLETE・3191 tests・2026-07-22）

---

## T7 — コードレビュー対応

- [x] [HIGH] `fav bench --all` フラグ未実装 → main.rs に `"--all" => { i += 1; }` 追加
- [x] [MED] `continue-on-error: true` がジョブ全体に適用されリグレッション検出が無効 → ステップ限定に移動
- [x] [MED] baseline.json プレースホルダー値の扱いが不明 → notes に更新手順を明記
- [x] [LOW] bench.yml 末尾改行なし → 末尾改行追加
- [x] [LOW] テストに `--all` アサーションがない → `bench_yml.contains("--all")` を追加

---

## T8 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T8 全 `[x]`）
