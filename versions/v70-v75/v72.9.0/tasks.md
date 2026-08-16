# v72.9.0 タスクリスト — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.8.0` であることを確認
- [x] `cargo test` が 3640 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v728000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v729000_tests` が未存在であることを確認
- [x] v72.1〜v72.8 の各テストモジュール（`v721000_tests`〜`v728000_tests`）が driver.rs に存在することを grep で確認
- [x] `driver.rs` 内の `"72.8.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: `v729000_tests` モジュール追加

- [x] `v728000_tests` モジュールの直後に `v729000_tests` モジュールを追加した
- [x] `use super::*` を追加した
- [x] `dev_exp2_all_stable` テストを実装した
  - v72.1〜v72.8 の代表テスト名 9 件（`vscode_extension_package_json_valid` / `ai_explain_e0374_returns_hint` / `ai_generate_returns_valid_fav_code` / `repl2_tab_completion` / `playground2_template_gallery_has_5_entries` / `init_template_ai_etl_valid` / `watch2_session_field_defaults` / `learn_chapter1_exists` / `learn_chapter5_exists`）が `include_str!("driver.rs")` に含まれることを assert
- [x] `vscode_repl2_playground2_e2e` テストを実装した
  - `vscode_extension_lsp_integration` / `repl2_multiline_input` / `playground2_share_url_format` の 3 シンボルが driver.rs に含まれることを assert（VS Code 拡張・REPL 2.0・Playground 2.0 に絞る）
- [x] `cargo build` でエラーがないことを確認

---

## T2: バージョン更新

- [x] `fav/Cargo.toml` の `version = "72.8.0"` → `version = "72.9.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.8.0\"` 文字列を `version = \"72.9.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.8.0"` を `"72.9.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.8.0"` を `"72.9.0"` に replace_all した
- [x] 残存 72.8.0 はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `grep "72.8.0" driver.rs` で意図的保持分以外がゼロ件であることを確認

---

## T3: 部分テスト確認

- [x] `cargo test v729000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3642 tests pass（0 failures）であることを確認

---

## T5: ドキュメント更新

- [x] `## [v72.9.0]` エントリを `CHANGELOG.md` 先頭に追加した
  - Added: `dev_exp2_all_stable` / `vscode_repl2_playground2_e2e`
  - Tests: 2 件、合計テスト数 3642（+2）
- [x] `versions/roadmap/roadmap-v72.1-v73.0.md` の v72.9.0 完了条件テスト数が実測値と一致していることを確認した

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v72.9.0)` に更新した
- [x] 「進行中バージョン」を `v72.9.0`（安定化）に更新した
- [x] 「次に切る版」を `v73.0.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v729000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3642 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.9.0` であることを確認
- [x] `CHANGELOG.md` に `[v72.9.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v72.9.0` であることを確認
- [x] `versions/current.md` の「次に切る版」が `v73.0.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [LOW] | `use std::io::BufRead;` が関数先頭ではなく途中に配置、`Write` と非対称 | 関数先頭に `use std::io::{BufRead, Write};` としてまとめ、`std::io::Write::flush(...)` を `stdout().flush()` に簡略化 |
| [LOW] | `dev_exp2_all_stable` は `include_str!` 自己参照のためテスト削除を検知できないトートロジー | コードフリーズ記録用途として許容範囲。修正なし（設計上の制限として認識） |

---

## スコープ外（明示的除外）

- 新機能の追加（安定化専用スプリントのため）
- バグ修正以外のコード変更
- サイト側ドキュメント更新（v73.0.0 以降）
