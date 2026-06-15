# v17.5.0 — REPL 品質向上 タスク

## ステータス: 未着手

---

## タスク一覧

### T1: `ReplSession` に `history` 追加

- [ ] `fav/src/driver.rs` の `struct ReplSession` に `history: Vec<String>` フィールドを追加
- [ ] `ReplSession::new()` で `history: Vec::new()` を初期化
- [ ] `add_history(&mut self, line: &str)` メソッドを追加
- [ ] `print_history(&self)` メソッドを追加
  - 空の場合: `(no history)`
  - 各行を `1: ... ` 番号付きで表示

### T2: `BUILTIN_DOCS` テーブル追加

- [ ] `cmd_repl` 付近に `const BUILTIN_DOCS: &[(&str, &str, &str)]` を追加
  - 少なくとも List / String / Json / Map / Result / IO の主要関数を収録（30件以上）
  - フォーマット: `(name, signature, one_line_description)`

### T3: `repl_doc_str` + `handle_doc_cmd` 追加

- [ ] `pub fn repl_doc_str(target: &str) -> Option<String>` を実装
  - `BUILTIN_DOCS` を線形検索
  - マッチしたら `"{sig}\n  {desc}"` を返す
  - なければ `None`
- [ ] `fn handle_doc_cmd(target: &str)` を実装
  - `repl_doc_str` を呼んで `println!` / `eprintln!`

### T4: `repl_complete_prefix` + `REPL_COMMANDS` テーブル

- [ ] `const REPL_COMMANDS: &[&str]` を追加（全コマンド一覧）
- [ ] `pub fn repl_complete_prefix(prefix: &str) -> Vec<String>` を実装
  - `":"` で始まる場合: `REPL_COMMANDS` から前方一致
  - それ以外: `BUILTIN_DOCS` の name から前方一致
- [ ] 返り値はソート済みの `Vec<String>`

### T5: `handle_load_cmd` + `extract_top_level_names` 追加

- [ ] `fn extract_top_level_names(src: &str) -> Vec<String>` を実装
  - `src.lines()` を走査し、`is_definition(line.trim())` の行から `extract_def_name` を適用
- [ ] `pub fn handle_load_cmd(path: &str, session: &mut ReplSession)` を実装
  - `std::fs::read_to_string(path)` → エラーなら `eprintln!` して return
  - `check_source_str(&merged)` でエラーチェック（merged = session.definitions + src）
  - エラーなし → 定義名を抽出して `session` に追加、`"loaded: ..."` 表示
  - エラーあり → エラーメッセージを表示

### T6: `handle_paste_block` 追加

- [ ] `pub fn handle_paste_block(src: &str, session: &mut ReplSession)` を実装
  - `is_definition(src.trim())` → `handle_definition`
  - それ以外 → `handle_expression`
- [ ] `cmd_repl` のループに `:paste` ディスパッチを追加
  - `:paste` を受けたら `:end` が来るまで行を収集
  - 収集した行を `"\n".join()` して `handle_paste_block` に渡す

### T7: `handle_save_cmd` 追加

- [ ] `fn handle_save_cmd(path: &str, session: &ReplSession)` を実装
  - `std::fs::write(path, &session.definitions)` → 成功メッセージ or エラーメッセージ

### T8: `cmd_repl` ループ更新

- [ ] 既存の `":quit" | ":q" => break,` ブランチを維持しつつ以下を追加:
  - `":history"` → `session.print_history()`
  - `":paste"` → 複数行収集 + `handle_paste_block`
  - `line.starts_with(":doc ")` → `handle_doc_cmd(line[5..].trim())`
  - `line.starts_with(":load ")` → `handle_load_cmd(line[6..].trim(), &mut session)`
  - `line.starts_with(":save ")` → `handle_save_cmd(line[6..].trim(), &session)`
- [ ] 各入力を `session.add_history(line)` で記録
- [ ] バージョン文字列を `"Favnir v9.10.0"` → `env!("CARGO_PKG_VERSION")` に変更

### T9: `print_repl_help` 更新

- [ ] 新コマンド（`:doc`, `:load`, `:save`, `:history`, `:paste`）を一覧に追加

### T10: テスト追加（`fav/src/driver.rs`）

- [ ] `v175000_tests` モジュールを `driver.rs` に追加

```rust
#[cfg(test)]
mod v175000_tests {
    use super::{repl_doc_str, repl_complete_prefix, handle_load_cmd, handle_paste_block, ReplSession};

    #[test]
    fn version_is_17_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.5.0\""), "Cargo.toml should have version 17.5.0");
    }

    #[test]
    fn repl_doc_command() {
        // repl_doc_str("List.map") が Some を返しシグネチャを含む
        let doc = repl_doc_str("List.map");
        assert!(doc.is_some(), "List.map should have documentation");
        let doc = doc.unwrap();
        assert!(doc.contains("List<"), "doc should mention List type");
        assert!(doc.contains("List.map"), "doc should contain function name");
    }

    #[test]
    fn repl_type_command_basic() {
        // repl_complete_prefix で :type を補完できる
        let completions = repl_complete_prefix(":ty");
        assert!(completions.iter().any(|s| s.starts_with(":type")),
            "':ty' should complete to ':type'");
    }

    #[test]
    fn repl_load_file() {
        // 一時ファイルを作成し handle_load_cmd でセッションに取り込める
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("helpers.fav");
        std::fs::write(&path, "fn double(x: Int) -> Int { x * 2 }\n")
            .expect("write");
        let mut session = ReplSession::new();
        handle_load_cmd(path.to_str().unwrap(), &mut session);
        assert!(session.def_names.contains(&"double".to_string()),
            "double should be loaded into session");
    }

    #[test]
    fn repl_paste_mode() {
        // handle_paste_block で複数行定義をセッションに追加できる
        let mut session = ReplSession::new();
        handle_paste_block("fn triple(x: Int) -> Int { x * 3 }", &mut session);
        assert!(session.def_names.contains(&"triple".to_string()),
            "triple should be in session after paste");
    }
}
```

- [ ] `tempfile` が `[dev-dependencies]` にあるか確認（既存依存として使われているはず）
- [ ] `cargo test v175000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし（既存 `v9100_tests` も pass であること）

### T11: バージョン更新

- [ ] `fav/Cargo.toml` のバージョンを `17.5.0` に更新
- [ ] `fav/Cargo.lock` を `cargo build` で更新

### T12: ドキュメント

- [ ] `site/content/docs/tools/repl.mdx` を更新（または新規作成）
  - 全コマンドの説明
  - `:paste` モードの例
  - タブ補完の使い方

---

## 完了条件チェックリスト

- [ ] `repl_doc_str("List.map")` が型シグネチャを返す
- [ ] `repl_complete_prefix("List.")` が `List.map` 等を含む
- [ ] `repl_complete_prefix(":d")` が `":doc "` を含む
- [ ] 一時ファイルから `handle_load_cmd` で定義を取り込める
- [ ] `handle_paste_block` で複数行定義がセッションに追加される
- [ ] `cmd_repl` に `:history` / `:paste` / `:doc` / `:load` / `:save` が追加される
- [ ] `cargo test v175000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし（v9100_tests 3件も pass）

---

## 優先度

T1（history）→ T2（BUILTIN_DOCS）→ T3（doc）→ T4（complete）→ T5（load）→ T6（paste）→ T7（save）→ T8（cmd_repl 更新）→ T9（help 更新）→ T10（テスト）→ T11（version）→ T12（doc）

T10 のテストは T3/T4/T5/T6 が完了してから。
T8/T9 は最後でよい（テストは関数レベルで確認できるため）。

---

## 補足: `tempfile` 依存の確認

```toml
# Cargo.toml に既にあるか確認
[dev-dependencies]
tempfile = "3"
```

なければ追加が必要。
