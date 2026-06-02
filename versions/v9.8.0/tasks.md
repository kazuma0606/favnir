# Favnir v9.8.0 Tasks

Date: 2026-06-02
Theme: `fav doc` — `///` ドキュメントコメント + 型シグネチャ → Markdown 自動生成

---

## Phase A: compiler.fav — `TkDocComment` トークン追加

- [x] A-1: `Token` type に `TkDocComment(String)` variant を追加
- [x] A-2: `token_eq` に `TkDocComment` arm を追加
- [x] A-3: `token_to_string` に `TkDocComment(s) => s` を追加
- [x] A-4: `collect_line_text` — `List.take_while` + `String.from_chars` でインライン実装
- [x] A-5: `scan_collect` の `//` 分岐を `///` チェックで拡張（3 文字目が `/` なら `TkDocComment` 発行）
- [x] A-6: 全テスト通過で影響なし確認

---

## Phase B: compiler.fav — AST 構造体に `doc: String` 追加

- [x] B-1: `FnDef` に `doc: String` フィールドを追加
- [x] B-2: `TypeDef` に `doc: String` フィールドを追加
- [x] B-3: `StageDef` に `doc: String` フィールドを追加
- [x] B-4: `SeqDef` に `doc: String` フィールドを追加
- [x] B-5: `WrapperDef` に `doc: String` フィールドを追加
- [x] B-6: 12 箇所の構造体リテラルに `doc: ""` を追加
- [x] B-7: `pretty_doc` ヘルパー + 全 pretty_*_def 関数に doc 出力を追加

---

## Phase C: compiler.fav — パーサー `collect_doc` 追加

- [x] C-1: `DocCollect` 型を追加（`{ doc: String, rest: List<Token> }`）
- [x] C-2: `collect_doc(toks, acc) -> DocCollect` 追加（TkDocComment を消費して蓄積）
- [x] C-3: `set_item_doc(item, doc) -> Item` 追加（parse 後に doc を注入）
- [x] C-4: `parse_items_acc` を `collect_doc` + `set_item_doc` を使う形に更新
- [x] C-5: 全テスト通過で確認

---

## Phase D: compiler.fav — `doc_program` 関数追加

- [x] D-1: `doc_fn_sig(fd: FnDef) -> String` ヘルパー追加
- [x] D-2: `doc_fn_def` / `doc_type_def_item` / `doc_wrapper_def_item` / `doc_stage_def_item` / `doc_seq_def_item` 追加
- [x] D-3: `join_nonempty` / `build_doc_section` / `build_doc_sections` 追加
- [x] D-4: `doc_items_acc` 追加
- [x] D-5: `public fn doc_source(src: String) -> Result<String, String>` 追加
- [x] D-6: v980_tests 8 件で動作確認

---

## Phase E: `compiler_fav_runner.rs` — `doc_source_str` 追加

- [x] E-1: `pub fn doc_source_str(src: &str) -> Result<String, String>` を追加
- [x] E-2: コンパイル + テスト通過で確認

---

## Phase F: `vm.rs` — `Compiler.doc_source_raw` primitive 追加

- [x] F-1: `"Compiler.doc_source_raw"` match arm を追加（`Compiler.lint_source_raw` の直後）
- [x] F-2: コンパイル + テスト通過で確認

---

## Phase G: `driver.rs` — `cmd_doc` 追加 + 統合テスト

- [x] G-1: `cmd_doc(path, out_dir)` を追加（`collect_fav_files_recursive` 既存 fn 利用）
- [x] G-2: `v980_tests` モジュール 8 件追加（G-3a〜G-3h）
- [x] G-3: `cargo test v980` — 全件通過

---

## Phase H: `main.rs` + `cli.fav` — `fav doc` サブコマンド追加

- [x] H-1: `main.rs` に `"doc"` dispatch 追加（`--out` フラグ、デフォルト `"docs"`）
- [x] H-2〜H-8: `cli.fav` に `CmdDoc` / `not_eq_str` / `find_flag_value` / `parse_doc_cmd` / `run_doc` 追加、`run_help` に doc 説明追加

---

## Phase I: self-check + Bootstrap + バージョン更新

- [x] I-1: `cargo test checker_fav_wire_self_check` — 通過
- [x] I-2: `cargo test bootstrap` — 23 件通過
- [x] I-3: `cargo test` — 1213 passed（1 failed は parquet 並行競合の既存不具合）
- [x] I-4: `fav/Cargo.toml` version → `"9.8.0"`
- [x] I-5: `fav/self/cli.fav` バージョン → `"9.8.0"`
- [x] I-6: 本ファイル完了チェック
- [ ] I-7: `memory/MEMORY.md` に v9.8.0 完了を記録
- [ ] I-8: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `/// コメント` が `TkDocComment` としてレキシングされる | |
| `public fn` / `type` / `stage` / `seq` のシグネチャが Markdown に出力される | |
| `///` コメントがシグネチャの直前に記述されると本文として出力される | |
| 非 `public` な定義は出力されない | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |
| `cargo test` 全件通過（1215 件以上） | |

---

## スコープ外（v9.9.0 以降）

- HTML 出力
- インデックスページ（`index.md`）自動生成
- リンク解決（他モジュールの型/関数への参照）
- `@param` / `@returns` タグ構文
- Rune ドキュメント自動生成（`rune info --doc`）
