# Favnir v9.12.0 Tasks

Date: 2026-06-02
Theme: ユーザー定義インターフェース self-hosted 対応 + LSP 定義ジャンプ改善

---

## Phase A: compiler.fav — lexer/parser/codegen 拡張

- [x] A-1: `Token` 型に `TkInterface` / `TkImpl` variant を追加
- [x] A-2: `keyword_token` 関数に `interface` / `impl` の分岐を追加
- [x] A-3: `InterfaceMethodDecl` / `InterfaceDef` / `ImplDeclDef` 型定義を追加
- [x] A-4: `Item` 型に `IInterface(InterfaceDef)` / `IImpl(ImplDeclDef)` variant を追加
- [x] A-5: `parse_interface_item` 関数を追加（`interface Name { method: type ... }` をパース）
- [x] A-6: `parse_impl_item` 関数を追加（`impl Iface for Type { fn ... }` をパース）
- [x] A-7: `parse_top_item` に `TkInterface` / `TkImpl` の分岐を追加
- [x] A-8: `compile_item` に `IInterface` / `IImpl` の分岐を追加
- [x] A-9: `compile_impl_decl` 関数を追加（各 FnDef を `TypeName_method` として compile_fn_def 経由で出力）
- [x] A-10: `cargo build` 通過確認

---

## Phase B: checker.fav — interface/impl 型チェック

- [x] B-1: `Token` 型に `TkInterface` / `TkImpl` variant を追加（compiler.fav と同様）
- [x] B-2: `keyword_token` 関数に `interface` / `impl` の分岐を追加
- [x] B-3: `InterfaceDef` / `ImplDeclDef` 型定義を追加
- [x] B-4: `Item` 型に `IInterface` / `IImpl` variant を追加
- [x] B-5: `parse_interface_item` / `parse_impl_item` / `parse_top_item` 更新
- [x] B-6: `ImplEntry` 型定義を追加（`{ iface: String, type_: String }`）
- [x] B-7: `collect_interface_schemes(prog, env)` を追加（IInterface を env に登録）
- [x] B-8: `collect_impl_decls(prog)` を追加（IImpl から ImplEntry リストを生成）
- [x] B-9: `check_type_with_impl(td, impl_list, iface_env)` を追加（E0014 / E0015 検出）
- [x] B-10: `check_impl_decl(id, env)` を追加（impl 内各 fn を check_fn_def で検証）
- [x] B-11: `check(prog)` を更新（collect_interface_schemes / collect_impl_decls 呼び出し追加、check_items に IInterface/IImpl 分岐追加）
- [x] B-12: E0014 エラー文言: `"Validatable is not implemented for Order"` + hint
- [x] B-13: E0015 エラー文言: `"undefined interface Validatable"`
- [x] B-14: `cargo build` 通過確認

---

## Phase C: LSP 定義ジャンプ改善

- [x] C-1: `LspServer` 構造体に `workspace_root: Option<String>` フィールドを追加
- [x] C-2: `initialize` ハンドラで `rootUri` を取り出して `self.workspace_root` に保存
- [x] C-3: `fav/src/lsp/definition.rs` に `handle_rune_definition(src, offset, workspace_root)` を追加
  - カーソル位置から `<ns>.<fn>` パターンを検出
  - `KNOWN_RUNES` に含まれる ns か確認
  - `<workspace_root>/rune_modules/<ns>/<ns>.fav` を読み込み `fn <fn>` 行を探す
  - Location を返す
- [x] C-4: `textDocument/definition` ハンドラを拡張（`handle_definition` が None → `handle_rune_definition` を試みる）
- [x] C-5: seq 内 stage 名ジャンプの調査
  - `checker.rs` の `def_at` に stage 名 usage span が記録されているか確認
  - 未記録なら `check_seq_def` に usage → def マッピングを追加
- [x] C-6: `cargo build` 通過確認

---

## Phase D: テスト + self-check + バージョン更新 + commit

- [x] D-1: `v9120_tests` モジュールを追加（6 件）
  - [x] D-1a: `interface_keyword_does_not_cause_parse_error` — `fav run` で interface キーワードが通る
  - [x] D-1b: `impl_keyword_does_not_cause_parse_error` — `fav run` で impl キーワードが通る
  - [x] D-1c: `missing_impl_e0014_detected` — with UserDefinedIface + impl 欠落 → E0014
  - [x] D-1d: `undefined_interface_e0015_detected` — 未定義インターフェース参照 → E0015
  - [x] D-1e: `rune_definition_jump_http_get` — http.get の定義ジャンプが rune_modules/http/http.fav へ
  - [x] D-1f: `rune_definition_returns_none_for_unknown_namespace` — 未知 ns → None
- [x] D-2: `cargo test v9120` — 6 件通過
- [x] D-3: `cargo test checker_fav_wire_self_check` — 通過
- [x] D-4: `cargo test bootstrap` — 通過
- [x] D-5: `cargo test` — 全件通過
- [x] D-6: `fav/Cargo.toml` version → `"9.12.0"`
- [x] D-7: `fav/self/cli.fav` の `run_version` → `"9.12.0"`
- [x] D-8: 本ファイル完了チェック
- [x] D-9: `memory/MEMORY.md` に v9.12.0 完了を記録
- [x] D-10: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `interface` / `impl ... for` を含む `.fav` が `fav run` で動作する | |
| `with UserDefinedIface` の impl 漏れを E0014 で検出できる | |
| 未定義インターフェース参照を E0015 で検出できる | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |
| Rune 関数（`http.get` 等）の定義ジャンプが `rune_modules/<ns>/<ns>.fav` へ飛ぶ | |
| `seq` 内の stage 名ジャンプが動作する | |
| `cargo test v9120` — 6 件通過 | |

---

## スコープ外（将来版へ延期）

- `interface` の型制約付きジェネリクス（`fn f<T with Show>(v: T)` の型チェック）
- `impl` ブロック内メソッドの完全な型シグネチャ一致チェック（メソッド名の存在確認のみ）
- Rune ジャンプのキャッシュ（毎回ファイル読み込み）
- LSP のインクリメンタル解析
