# v72.4.0 タスクリスト — REPL 2.0

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.3.0` であることを確認
- [x] `cargo test` が 3620 tests pass（0 failures）であることを確認（v72.3.0 完了後の実測値。code-reviewer が +2 追加したためロードマップ記載の 3618 ではなく 3620 が正しいベース）
- [x] `driver.rs` に `v723000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v724000_tests` が未存在であることを確認
- [x] `driver.rs` に `repl_tab_complete` が未存在であることを確認
- [x] `driver.rs` に `pub fn needs_continuation`（すでに pub）が存在することを確認 — pub 変更不要と判明
- [x] `driver.rs` 内の `"72.3.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく（28 件）

---

## T1: `driver.rs` — `repl_tab_complete` 追加 + `needs_continuation` pub 化

- [x] `needs_continuation` がすでに `pub fn` であることを確認した（変更不要）
- [x] `repl_tab_complete(prefix: &str, scope: &[&str]) -> Vec<String>` を `pub fn` で追加した
  - `prefix` に前方一致する要素を `scope` から返す
  - 大文字小文字区別あり
- [x] `cargo build` でエラーがないことを確認

---

## T2: `ReplSession.timing_enabled` + `:timing` ハンドラ追加

- [x] `ReplSession` 構造体に `timing_enabled: bool` フィールドを追加した
- [x] `ReplSession::new()` に `timing_enabled: false` を追加した
- [x] `cmd_repl` の `match line` ブロックに `:timing on` / `:timing off` ハンドラを追加した
- [x] `handle_expression` に `std::time::Instant` タイミング計測を追加した
  - `session.timing_enabled` が true のとき `(Xms)` を println! する
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v724000_tests` 追加（`driver.rs`）

> **前提**: T1 完了済みであること（`repl_tab_complete` / `needs_continuation` が `pub fn` で存在すること）。

- [x] `v723000_tests` モジュールの直後に `v724000_tests` モジュールを追加した
- [x] `use super::{needs_continuation, repl_tab_complete}` を追加した
- [x] `repl2_tab_completion` テストを実装した
  - `repl_tab_complete("Li", &["List", "Csv", "linq"])` が `["List"]` を返すことを assert
- [x] `repl2_multiline_input` テストを実装した
  - `needs_continuation("fn main() {")` が `true` を返すことを assert
  - `needs_continuation("let x = 1")` が `false` を返すことを assert
- [x] `cargo build` でエラーがないことを確認

---

## T4: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x] `fav/Cargo.toml` の `version = "72.3.0"` → `version = "72.4.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.3.0\"` 文字列を `version = \"72.4.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.3.0"` を `"72.4.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.3.0"` を `"72.4.0"` に replace_all した
- [x] T0 で記録した 28 件 → 置換後 72.4.0 が 29 件（セクションヘッダー含む）、72.3.0 残 2 件はセクションヘッダーコメントのみ

---

## T5: 部分テスト確認

- [x] `cargo test repl2` で 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3622 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v72.4.0]` エントリを先頭に追加した

---

## T8: `versions/current.md` 更新

- [x] 「進行中バージョン」を `v72.4.0`（REPL 2.0）に更新した
- [x] 「次に切る版」を `v72.5.0` に更新した

---

## T9: 最終確認（T7・T8 完了後のドキュメント更新後リグレッション確認）

- [x] `cargo test repl2` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3622 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.4.0` であることを確認
- [x] `repl_tab_complete("Li", &["List", "Csv"])` が `["List"]` を返すことを確認（テストで担保）
- [x] `needs_continuation("fn main() {")` が `true` を返すことを確認（テストで担保）
- [x] `versions/current.md` の「進行中バージョン」が `v72.4.0`、「次に切る版」が `v72.5.0` であることを確認

---

## スコープ外（明示的除外）

- `rustyline` クレートによる readline 編集・TAB キー統合（v72.5.0 以降）
- `:import rune` REPL コマンド（現状 REPL はインポート非対応）
- REPL の WASM 対応（web UI 統合は別タスク）
- `site/content/docs/cli/repl.mdx` 更新（v72.5.0 以降）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `Instant::now()` がコンパイル後に置かれており実行時間のみ計測 | `build_eval_source` 直後（コンパイル前）に移動し合計レイテンシを計測 |
| [MED] | `reset()` が `timing_enabled` をリセットしない | `self.timing_enabled = false;` を `reset()` に追加 |
| [MED] | 空プレフィックス・空スコープ・マッチなしのエッジケースが未テスト | 3 件テスト追加（`empty_prefix`/`no_match`/`empty_scope`） |
| [MED] | `repl_tab_complete` が v72.3.0 セクションより前に配置（バージョン昇順逆転） | v72.3.0 `cmd_ai_generate` の直後・テストモジュールの直前に移動 |
| [LOW] | `"let x = 1"` が Favnir に存在しない構文 | `"bind x <- 1"` に修正 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `repl2_tab_completion` が pass
- [x] `repl2_multiline_input` が pass
- [x] `repl2_tab_completion_empty_prefix_returns_all` が pass（コードレビュー対応で追加）
- [x] `repl2_tab_completion_no_match_returns_empty` が pass（コードレビュー対応で追加）
- [x] `repl2_tab_completion_empty_scope_returns_empty` が pass（コードレビュー対応で追加）
- [x] テスト総数: 3625（+5）
