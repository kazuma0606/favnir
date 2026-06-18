# v17.5.0 — REPL 品質向上 タスク

## ステータス: 完了

---

## タスク一覧

### T1: `ReplSession` に `history` 追加

- [x] `fav/src/driver.rs` の `struct ReplSession` に `history: Vec<String>` フィールドを追加
- [x] `ReplSession::new()` で `history: Vec::new()` を初期化
- [x] `add_history(&mut self, line: &str)` メソッドを追加
- [x] `print_history(&self)` メソッドを追加
  - 空の場合: `(no history)`
  - 各行を `1: ... ` 番号付きで表示

### T2: `BUILTIN_DOCS` テーブル追加

- [x] `cmd_repl` 付近に `const BUILTIN_DOCS: &[(&str, &str, &str)]` を追加
  - 少なくとも List / String / Json / Map / Result / IO の主要関数を収録（30件以上）
  - フォーマット: `(name, signature, one_line_description)`

### T3: `repl_doc_str` + `handle_doc_cmd` 追加

- [x] `pub fn repl_doc_str(target: &str) -> Option<String>` を実装
  - `BUILTIN_DOCS` を線形検索
  - マッチしたら `"{sig}\n  {desc}"` を返す
  - なければ `None`
- [x] `fn handle_doc_cmd(target: &str)` を実装
  - `repl_doc_str` を呼んで `println!` / `eprintln!`

### T4: `repl_complete_prefix` + `REPL_COMMANDS` テーブル

- [x] `const REPL_COMMANDS: &[&str]` を追加（全コマンド一覧）
- [x] `pub fn repl_complete_prefix(prefix: &str) -> Vec<String>` を実装
  - `":"` で始まる場合: `REPL_COMMANDS` から前方一致
  - それ以外: `BUILTIN_DOCS` の name から前方一致
- [x] 返り値はソート済みの `Vec<String>`

### T5: `handle_load_cmd` + `extract_top_level_names` 追加

- [x] `fn extract_top_level_names(src: &str) -> Vec<String>` を実装
  - `src.lines()` を走査し、`is_definition(line.trim())` の行から `extract_def_name` を適用
- [x] `pub fn handle_load_cmd(path: &str, session: &mut ReplSession)` を実装
  - `std::fs::read_to_string(path)` → エラーなら `eprintln!` して return
  - `check_source_str(&merged)` でエラーチェック（merged = session.definitions + src）
  - エラーなし → 定義名を抽出して `session` に追加、`"loaded: ..."` 表示
  - エラーあり → エラーメッセージを表示

### T6: `handle_paste_block` 追加

- [x] `pub fn handle_paste_block(src: &str, session: &mut ReplSession)` を実装
  - `is_definition(src.trim())` → `handle_definition`
  - それ以外 → `handle_expression`
- [x] `cmd_repl` のループに `:paste` ディスパッチを追加
  - `:paste` を受けたら `:end` が来るまで行を収集
  - 収集した行を `"\n".join()` して `handle_paste_block` に渡す

### T7: `handle_save_cmd` 追加

- [x] `fn handle_save_cmd(path: &str, session: &ReplSession)` を実装
  - `std::fs::write(path, &session.definitions)` → 成功メッセージ or エラーメッセージ

### T8: `cmd_repl` ループ更新

- [x] 既存の `":quit" | ":q" => break,` ブランチを維持しつつ以下を追加:
  - `":history"` → `session.print_history()`
  - `":paste"` → 複数行収集 + `handle_paste_block`
  - `line.starts_with(":doc ")` → `handle_doc_cmd(line[5..].trim())`
  - `line.starts_with(":load ")` → `handle_load_cmd(line[6..].trim(), &mut session)`
  - `line.starts_with(":save ")` → `handle_save_cmd(line[6..].trim(), &session)`
- [x] 各入力を `session.add_history(line)` で記録
- [x] バージョン文字列を `"Favnir v9.10.0"` → `env!("CARGO_PKG_VERSION")` に変更

### T9: `print_repl_help` 更新

- [x] 新コマンド（`:doc`, `:load`, `:save`, `:history`, `:paste`）を一覧に追加

### T10: テスト追加（`fav/src/driver.rs`）

- [x] `v175000_tests` モジュールを `driver.rs` に追加（5件）
- [x] `tempfile` が `[dev-dependencies]` にあることを確認（既存依存）
- [x] `cargo test v175000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（1636 tests pass）

### T11: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `17.5.0` に更新
- [x] `fav/Cargo.lock` を更新

### T12: ドキュメント

- [x] `site/content/docs/tools/repl.mdx` を新規作成
  - 全コマンドの説明
  - `:paste` モードの例
  - タブ補完の使い方

---

## 完了条件チェックリスト

- [x] `repl_doc_str("List.map")` が型シグネチャを返す
- [x] `repl_complete_prefix("List.")` が `List.map` 等を含む
- [x] `repl_complete_prefix(":d")` が `":doc "` を含む
- [x] 一時ファイルから `handle_load_cmd` で定義を取り込める
- [x] `handle_paste_block` で複数行定義がセッションに追加される
- [x] `cmd_repl` に `:history` / `:paste` / `:doc` / `:load` / `:save` が追加される
- [x] `cargo test v175000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（v9100_tests 3件も pass）

---

## 実装メモ

- `v174000_tests::version_is_17_4_0` は Cargo.toml バージョン更新により削除（v175000 の version_is_17_5_0 に置き換え）
- PDB サイズ制限エラー（LNK1318）が発生した場合は `target/debug/deps/fav-*.pdb` を削除して再ビルド
- commit: `792bd6a`
