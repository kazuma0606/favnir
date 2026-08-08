# v69.8.0 タスクリスト

Status: COMPLETE
Version: 69.8.0
Note: パフォーマンス回帰テスト — v69-baseline.md 新規作成 + benchmarks.mdx 更新 + driver.rs に 2 テスト
Base tests: 3551
Target tests: 3553（+2）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3551 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.7.0` であることを確認
- [x] `benchmarks/compare/` ディレクトリが存在することを確認（新規ファイルの作成先確認）
- [x] `benchmarks/compare/v69-baseline.md` が存在しないことを確認
- [x] `site/content/docs/runtime/benchmarks.mdx` に `"Intelligent ETL"` が含まれないことを確認（重複防止）
- [x] `driver.rs` に `v69700_tests` が存在することを確認（挿入先の確認）
- [x] `driver.rs` に `v69800_tests` が存在しないことを確認（重複防止）

---

## T1: `benchmarks/compare/v69-baseline.md` — 新規作成

- [x] `benchmarks/compare/v69-baseline.md` を新規作成
  - [x] タイトル `# Performance Baseline: v65.0 → v69.x` を含む
  - [x] 比較環境セクションを含む
  - [x] コンパイル時間比較テーブル（v65.0 vs v69.x）を含む
  - [x] VM 実行時間（bench-results.json 実測値: mean_ms=0.191、p99_ms=0.200）を含む
  - [x] AOT 実行時間（bench-results.json 実測値: mean_ms=0.532、p99_ms=0.576）を含む
  - [x] AI ステージスループット参考値を含む
  - [x] `"v65.0"` キーワードを含む
  - [x] `"v69"` キーワードを含む

---

## T2: `benchmarks.mdx` — Intelligent ETL セクション追加

- [x] `site/content/docs/runtime/benchmarks.mdx` の `For tuning...` 行の直後に `## Intelligent ETL パフォーマンス` セクションを追加
  - [x] AI ETL パイプライン（LoadArticles / EmbedAndSummarize / StoreToVectorDB / SemanticSearch）のスループット参考値テーブルを含む
  - [x] `"Intelligent ETL"` キーワードを含む

---

## T3: `driver.rs` — テスト追加

テストモジュールは降順（最新が先頭）。v69800 を v69700_tests の直前に挿入する。

- [x] `v69700_tests` の直前に `v69800_tests` モジュールを追加（挿入後: v69800 → v69700 → v69600 → ...）
  - [x] `benchmark_compare_has_v69_baseline` テストを追加
    - [x] `include_str!("../../benchmarks/compare/v69-baseline.md")`
    - [x] `src.contains("v65.0")` アサート
    - [x] `src.contains("v69")` アサート
  - [x] `site_benchmarks_covers_intelligent_etl` テストを追加
    - [x] `include_str!("../../site/content/docs/runtime/benchmarks.mdx")`
    - [x] `src.contains("Intelligent ETL")` アサート

---

## T4: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3553 tests passed, 0 failed** を確認

---

## T5: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.8.0 行を確定（3553、+2）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.8.0「状態」列を「完了 ✓」に変更
- [x] `versions/current.md` の「進行中バージョン」を `v69.7.0` から `v69.8.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

---

## コードレビュー指摘と対応（code-reviewer）

- [MED] v693/v694 のテストギャップが未説明 → driver.rs に NOTE コメント追加（v69.3/v69.4 は意図的にテストなし）
- [LOW-1] benchmarks.mdx の相対リンクが壊れている → プレーンテキスト参照に変更（`` `benchmarks/compare/v69-baseline.md` ``）
- [LOW-2] bench-results.json の `speedup` フィールド命名 → 既存ファイルのスコープ外のため対応しない（意図的）

## 設計上の意図的省略

- 実際のベンチマーク実行（CI で自動計測）: 将来フェーズ
- AOT コンパイラの最適化: 将来フェーズ
