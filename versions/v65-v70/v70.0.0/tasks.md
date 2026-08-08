# v70.0.0 タスクリスト

Status: COMPLETE
Version: 70.0.0
Note: Intelligent ETL 1.0 宣言 ★クリーンアップ — Cargo.toml バージョン更新 + MILESTONE/README/CHANGELOG 更新 + driver.rs に 4 テスト
Base tests: 3555
Target tests: 3559（+4）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3555 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（これから 70.0.0 に変更する）
- [x] `versions/current.md` の「進行中バージョン」が `v69.9.0` であることを確認
- [x] `MILESTONE.md` 先頭が v69.0.0 エントリであることを確認（v70.0.0 を先頭に挿入する前提）
- [x] `CHANGELOG.md` 先頭が `[v69.0.0]` エントリであることを確認
- [x] `driver.rs` に `v69900_tests` が存在することを確認（挿入先の確認）
- [x] `driver.rs` に `v70000_tests` が存在しないことを確認（重複防止）
- [x] `fav/tmp/hello.fav` が存在することを確認（cargo clean 後の復元基準確認）

---

## T1: `fav/Cargo.toml` — バージョン更新

- [x] version を `"69.0.0"` から `"70.0.0"` に変更

---

## T2: `MILESTONE.md` — 先頭にエントリ追加

- [x] v70.0.0（2026-08-08）— Intelligent ETL 1.0 のエントリを先頭に追加
  - [x] 宣言文（「型チェックが、LLM の出力を安全にする。...」）を含む
  - [x] `"Intelligent ETL"` キーワードを含む
  - [x] v65.1〜v69.9 達成内容のサマリーを含む

---

## T3: `README.md` — v70.0.0 宣言追記

- [x] v70.0.0 宣言セクションを追加
  - [x] `"Intelligent ETL"` または `"v70.0"` を含む

---

## T4: `CHANGELOG.md` — 先頭にエントリ追加

- [x] `[v70.0.0]` エントリを先頭に追加
  - [x] `"v70.0.0"` キーワードを含む
  - [x] 4 件のテスト名を含む
  - [x] `version "69.0.0" → "70.0.0"` の変更記録を含む
  - [x] ★クリーンアップ NOTE を含む

---

## T5: `driver.rs` — テスト追加

テストモジュールは降順（最新が先頭）。v70000 を v69900_tests の直前に挿入する。

- [x] `v69900_tests` の直前に `v70000_tests` モジュールを追加（挿入後: v70000 → v69900 → v69800 → ...）
  - [x] `cargo_toml_version_is_70_0_0` テストを追加
    - [x] `include_str!("../Cargo.toml")`
    - [x] `src.contains("version = \"70.0.0\"")` アサート
  - [x] `changelog_has_v70_0_0` テストを追加
    - [x] `include_str!("../../CHANGELOG.md")`
    - [x] `src.contains("v70.0.0")` アサート
  - [x] `milestone_has_intelligent_etl` テストを追加
    - [x] `include_str!("../../MILESTONE.md")`
    - [x] `src.contains("Intelligent ETL")` アサート
  - [x] `readme_mentions_intelligent_etl` テストを追加
    - [x] `include_str!("../../README.md")`
    - [x] `src.contains("Intelligent ETL") || src.contains("v70.0")` アサート

---

## T6: ビルド・テスト確認（cargo clean 前）

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3559 tests passed, 0 failed** を確認

---

## T7: ★クリーンアップ

- [x] `cargo clean` 実行
- [x] `fav/tmp/hello.fav` を復元（以下の内容で）:
  ```
  fn add(a: Int, b: Int) -> Int { a + b }
  fn main() -> Bool { add(1, 2) == 3 }
  ```
- [x] `cargo test --bin fav -- --test-threads=8` で **3559 tests passed, 0 failed** を確認（クリーンビルド）

---

## T8: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v70.0.0 行を確定（3559、+4）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v70.0.0「状態」列を「完了 ✓」に変更
- [x] `versions/current.md` の「最新安定版」を v70.0.0 に更新
- [x] `versions/current.md` の「進行中バージョン」を次バージョンに更新（v71.x.x スプリント開始まで空欄可）
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

## コードレビュー指摘と対応（code-reviewer）

- [MED] 旧バージョンのアサートメッセージ 9 箇所に `"69.0.0"` が残存 → `"70.0.0"` に一斉更新（条件式は問題なし、メッセージ精度を修正）
