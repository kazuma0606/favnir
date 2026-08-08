# v69.9.0 タスクリスト

Status: COMPLETE
Version: 69.9.0
Note: コードフリーズ・最終 lint / チェック — driver.rs に 2 テスト追加
Base tests: 3553
Target tests: 3555（+2）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3553 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.8.0` であることを確認
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` に `"Intelligent ETL 1.0 宣言"` が含まれることを確認（テスト前提）
- [x] `site/content/playground/etl-samples.mdx` に `"schema Order"` と `"bind"` が含まれることを確認（テスト前提）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.8.0 行が「完了 ✓」であることを確認
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.9.0 行が「未着手」であることを確認
- [x] `driver.rs` に `v69800_tests` が存在することを確認（挿入先の確認）
- [x] `driver.rs` に `v69900_tests` が存在しないことを確認（重複防止）

---

## T1: `driver.rs` — テスト追加

テストモジュールは降順（最新が先頭）。v69900 を v69800_tests の直前に挿入する。

- [x] `v69800_tests` の直前に `v69900_tests` モジュールを追加（挿入後: v69900 → v69800 → v69700 → v69600 → ...）
  - [x] `code_freeze_v699_v70_roadmap_has_milestone_declaration` テストを追加
    - [x] `include_str!("../../versions/roadmap/roadmap-v69.1-v70.0.md")`
    - [x] `src.contains("Intelligent ETL 1.0 宣言")` アサート
    - [x] `src.contains("3559")` アサート
  - [x] `code_freeze_v699_playground_etl_samples_complete` テストを追加
    - [x] `include_str!("../../site/content/playground/etl-samples.mdx")`
    - [x] `src.contains("schema Order")` アサート
    - [x] `src.contains("bind")` アサート

---

## T2: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3555 tests passed, 0 failed** を確認

---

## T3: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.9.0 行を確定（3555、+2）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.9.0「状態」列を「完了 ✓」に変更
- [x] `versions/current.md` の「進行中バージョン」を `v69.8.0` から `v69.9.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。
