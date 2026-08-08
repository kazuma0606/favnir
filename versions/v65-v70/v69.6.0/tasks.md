# v69.6.0 タスクリスト

Status: COMPLETE
Version: 69.6.0
Note: Playground の UI 改善・サンプル追加 — ai-examples.mdx 拡充 + etl-samples.mdx 新規作成 + driver.rs に 2 テスト
Base tests: 3547
Target tests: 3549（+2）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3547 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.5.0` であることを確認
- [x] `site/content/playground/ai-examples.mdx` に "Autodiff Demo" / "GradientStep" が含まれないことを確認（重複防止）
- [x] `site/content/playground/etl-samples.mdx` が存在しないことを確認
- [x] `driver.rs` に `v69600_tests` が存在しないことを確認

---

## T1: `ai-examples.mdx` — Autodiff Demo 追加

- [x] `site/content/playground/ai-examples.mdx` の `## コスト見積もり` 見出し直前に「5. Autodiff Demo（WASM）」セクションを追加
  - [x] `GradientStep` ステージのサンプルコードを含む（`bind grad <- Rune.autodiff.gradient(...)` 構文）
  - [x] `ComputeJacobian` ステージのサンプルコードを含む（`Rune.autodiff.jacobian`）
  - [x] WASM 実動作の旨を明記

---

## T2: `etl-samples.mdx` — 新規作成

- [x] `site/content/playground/etl-samples.mdx` を新規作成
  - [x] `schema Order` 定義を含む
  - [x] CSV フィルタリングサンプル（`List.filter`）を含む
  - [x] 集計パイプラインサンプル（`bind amounts <- List.map(...)`）を含む
  - [x] Full ETL Pipeline サンプル（LoadOrders → Transform → ETLPipeline）を含む
  - [x] `bind x <- expr` 構文の説明を含む（pure 関数への bind も有効と明記）
  - [x] "ETL" キーワードが含まれること
  - [x] "bind" キーワードが含まれること

---

## T3: `driver.rs` — テスト追加

テストモジュールは降順（最新が先頭）で並んでいる。v69600 を v69500_tests の直前に挿入する。

- [x] `v69500_tests` の直前に `v69600_tests` モジュールを追加（挿入後: v69600 → v69500 → v69200 → ...）
  - [x] `playground_has_autodiff_sample` テストを追加
    - [x] `include_str!("../../site/content/playground/ai-examples.mdx")`
    - [x] `src.contains("Autodiff Demo")` アサート
    - [x] `src.contains("GradientStep")` アサート（既存テーブル行との区別を保証）
  - [x] `playground_etl_samples_page_exists` テストを追加
    - [x] `include_str!("../../site/content/playground/etl-samples.mdx")`
    - [x] `src.contains("ETL")` アサート
    - [x] `src.contains("bind")` アサート
    - [x] `src.contains("schema Order")` アサート（空ファイルとの区別を保証）

---

## T4: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3549 tests passed, 0 failed** を確認

---

## T5: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.6.0 行を確定（3549、+2）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.6.0「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を `v69.5.0` から `v69.6.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

---

## 設計上の意図的省略

- Playground UI（JavaScript / React）の実際の変更: 将来フェーズ（本 sub-version は MDX コンテンツのみ）
- WASM ビルドや実際のブラウザ動作確認: 将来フェーズ
