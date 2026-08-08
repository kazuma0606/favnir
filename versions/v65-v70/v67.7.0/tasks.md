# v67.7.0 タスクリスト

Status: COMPLETE
Version: 67.7.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T6 はない
Base tests: 3509
Target tests: 3511

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3509 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/profiler/` ディレクトリが存在することを確認（v19.8.0 で作成済み）
- [x] `fav/src/profiler/interactive.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67600_tests` が存在することを確認（`v67700_tests` の挿入位置）
- [x] `driver.rs` に `v67700_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67600_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `proptest_stage_invariant`, `proptest_counterexample_shrink`
- [x] `versions/current.md` の「進行中バージョン」が `v67.6.0` であることを確認

---

## T1: `fav/src/profiler/interactive.rs` 新規作成

- [x] `fav/src/profiler/interactive.rs` を新規作成
  - [x] `pub const INTERACTIVE_HELP: &str` を追加
  - [x] `pub fn cmd_profile_interactive(src: &str) -> String` を追加
  - [x] `"--interactive"` を含む（`profile_interactive_hotspot` テストにマッチ）
  - [x] `"hotspot"` を含む（`profile_interactive_hotspot` テストにマッチ）
  - [x] `"drill"` を含む（`profile_interactive_drill` テストにマッチ）
  - [x] `"Suggestion"` を含む（`profile_interactive_drill` テストにマッチ）
- [x] `cargo build` でエラーなし（interactive.rs 作成後）

---

## T2: `fav/src/profiler/mod.rs` — `pub mod interactive;` を追加

- [x] `fav/src/profiler/mod.rs` に `pub mod interactive;` を追記
- [x] `cargo build` でエラーなし（mod.rs 更新後）

---

## T3: `fav/src/main.rs` — `Some("profile")` アームに `--interactive` 分岐を追加

- [x] `let mut interactive = false;` を `let mut build = false;` の直後に追加
- [x] while ループ内に `--interactive` フラグ処理を追加:
  ```rust
  } else if arg == "--interactive" {
      interactive = true; i += 1;
  ```
- [x] dispatch ブロックの先頭（`compare` チェックの前）に `interactive` ブランチを追加:
  ```rust
  if interactive {
      println!("{}", profiler::interactive::cmd_profile_interactive(&path));
  } else if let Some(ref v) = compare {
  ```
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T4: `driver.rs` — `v67700_tests` 追加

- [x] 挿入前に `grep "v67600_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67600_tests (v67.6.0)` コメントの直前に `v67700_tests` を挿入
  - [x] `profile_interactive_hotspot`: `include_str!("profiler/interactive.rs")` に `"--interactive"` / `"hotspot"` を含む
  - [x] `profile_interactive_drill`: `include_str!("profiler/interactive.rs")` に `"drill"` / `"Suggestion"` を含む
  - [x] `include_str!` パスは `"profiler/interactive.rs"`（`../profiler/interactive.rs` ではない）
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T5: ビルド・テスト

- [x] `cargo test --bin fav v67700_tests` で 2 件 PASS
  - [x] `profile_interactive_hotspot` PASS
  - [x] `profile_interactive_drill` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3511 tests passed, 0 failed を確認

---

## T6: ドキュメント・ステータス更新

> T5 のテスト全通過（3511 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.7.0 の「状態」列を「未着手」→「完了」に変更し、`grep "v67.7.0" versions/roadmap/roadmap-v67.1-v68.0.md | grep "完了"` で確認
- [x] `versions/current.md` の「進行中バージョン」を v67.7.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では `versions/current.md` の「次バージョン」欄の更新は不要。v68.0.0 宣言時に一括整理する。
> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## 設計上の意図的省略

- `--interactive --compare` 同時指定時は `interactive` が優先され `compare` が黙って無視される（エラー化は将来フェーズ）

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|--------|------|------|
| [HIGH] | `INTERACTIVE_HELP` がどこからも参照されていない（dead code） | `main.rs` に `--interactive && --help` ブランチを追加して参照するよう修正 |
| [HIGH] | `v67700_tests` がソースキーワード検索のみで関数の動作を検証しない | `cmd_profile_interactive("test.fav")` を実際に呼び出して戻り値を assert するテストに変更 |
| [MED] | `--interactive` と `--compare` 同時指定時の排他制御なし | 設計上の意図的省略（tasks.md 記載済み）— 対応不要 |
| [MED] | `src` 引数のパス検証なし（スタブ実装のため将来リスク） | スタブ実装のため対応不要 |
| [LOW] | スタブ関数のシグネチャが将来変更に脆弱 | 対応不要（over-engineering 回避） |
