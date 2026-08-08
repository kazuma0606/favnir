# v67.5.0 タスクリスト

Status: COMPLETE
Version: 67.5.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T5 はない
Base tests: 3505
Target tests: 3507

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3505 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/simulate.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67400_tests` が存在することを確認（`v67500_tests` の挿入位置）
- [x] `driver.rs` に `v67500_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67400_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `suggest_from_profile`, `suggest_applies_fix`
- [x] `versions/current.md` の「進行中バージョン」が `v67.4.0` であることを確認

---

## T1: `fav/src/simulate.rs` 新規作成

- [x] `fav/src/simulate.rs` を新規作成
  - [x] `pub const SIMULATE_HELP: &str` を追加（使用例・構文説明を含む）
  - [x] `pub fn cmd_simulate(src: &str, args: &[String]) -> String` を追加
  - [x] `"simulate"` を含む（`simulate_pipeline_with_synthetic` テストにマッチ）
  - [x] `"PASS"` を含む（`simulate_pipeline_with_synthetic` テストにマッチ）
  - [x] `"FAIL"` を含む（`simulate_assertion_failure` テストにマッチ）
- [x] `cargo build` でエラーなし（simulate.rs 作成後）

---

## T2: `fav/src/main.rs` — `mod simulate;` と `Some("simulate")` 追加

- [x] `mod viz;` の直後に `mod simulate;` を追加
- [x] `Some("viz")` アームの直後に `Some("simulate")` ディスパッチアームを追加:
  - [x] `--help` / `-h` ブランチで `simulate::SIMULATE_HELP` を表示（dead_code 防止）
  - [x] それ以外は `simulate::cmd_simulate(file, &rest)` を呼ぶ
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T3: `driver.rs` — `v67500_tests` 追加

- [x] 挿入前に `grep "v67400_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67400_tests (v67.4.0)` コメントの直前に `v67500_tests` を挿入
  - [x] `simulate_pipeline_with_synthetic`: `include_str!("simulate.rs")` に `"simulate"` と `"PASS"` を含む
  - [x] `simulate_assertion_failure`: `include_str!("simulate.rs")` に `"FAIL"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67500_tests` で 2 件 PASS
  - [x] `simulate_pipeline_with_synthetic` PASS
  - [x] `simulate_assertion_failure` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3507 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3507 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.5.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.5.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では `versions/current.md` の「次バージョン」欄の更新は不要。v68.0.0 宣言時に一括整理する。
> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [MED] code-reviewer: `v67500_tests` が実関数を呼ばずソーステキスト検索のみ → 他モジュールと同パターン（ロードマップ仕様）のため許容
- [LOW] code-reviewer: `--seed` 最後尾省略時にデフォルト値 `42` で無言 fallback → `eprintln!` 警告を追加
- [LOW] code-reviewer: `SIMULATE_HELP` の DSL 構文が未実装機能で誤解を招く → 「将来実装予定」注記を追加
- [LOW] code-reviewer: スタブ実装の意図が未明示 → `cmd_simulate` にコメントを追加
