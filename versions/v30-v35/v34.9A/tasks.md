# v34.9A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.4.0` であること
- [x] v34.8A が COMPLETE であること
- [x] `fn f(x: Int) -> Int !Http { x }` が E0374 を返すこと（v34.8A の成果確認）
- [x] `grep -n "Effect" fav/src/ast.rs` で `Effect` enum が存在すること（削除対象確認）
- [x] `grep -rn "\.effects\b" fav/src/ --include="*.rs" | wc -l` で参照数を確認

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.4.0` → `35.5.0` に更新
- [x] **T2** `fav/src/ast.rs` — `Effect` enum・`impl Effect`・`EffectDef` 構造体・`Item::EffectDef` を削除
- [x] **T3** `fav/src/ast.rs` — `FnDef` / `StageDef` / `SeqDef` / `StageTypeDef` から `effects: Vec<Effect>` フィールドを削除
- [x] **T4** `fav/src/frontend/parser.rs` — `parse_effects_acc` / `parse_effects` 関数と effects 関連コードを削除。E0374 チェックコードも削除（parse error は不要になる）
- [x] **T5** `fav/src/middle/checker.rs` — `.effects` 参照・`Effect::` 参照をすべて削除（常に空を前提に不要コードを除去）
- [x] **T6** `fav/src/middle/compiler.rs` — effects 関連 9 箇所を削除
- [x] **T7** `fav/src/lineage.rs` — Effect ベース lineage トラッキングを削除
- [x] **T8** `fav/src/fmt.rs` — Effect フォーマット関連コードを削除
- [x] **T9** `fav/src/emit_python.rs` — Effect 出力コードを削除
- [x] **T10** `fav/src/backend/wasm_codegen.rs` + `wasm_exec.rs` — `effects:` フィールドを FnDef 構造体リテラルから削除
- [x] **T11** `fav/src/backend/codegen.rs` — Effect 参照 2 箇所を削除
- [x] **T12** `fav/src/middle/reachability.rs` — effects 関連コードを削除
- [x] **T13** `fav/src/middle/ast_lower_checker.rs` — effects 関連コードを削除
- [x] **T14** `fav/src/error_catalog.rs` — E0370（`!Io not declared`）/ E0371（pure fn calls effectful）を削除
       ※ W021 lint が E0371 を使っていないことを確認してから削除
- [x] **T15** `fav/src/driver.rs` — Effect 関連テストをすべてスタブ化、`cargo_toml_version_is_35_4_0` をスタブ化
- [x] **T16** `fav/src/driver.rs` — `v35500_tests`（5 件）を追加（`v35400_tests` 直後に挿入）
- [x] **T17** `CHANGELOG.md` — `[v35.5.0]` セクションを先頭に追記
- [x] **T18** `benchmarks/v35.5.0.json` — 新規作成
- [x] **T19** `versions/current.md` — 最新安定版を v35.5.0 に更新

---

## テスト確認

- [x] **T20** `cargo test --bin fav v35500 2>&1 | tail -8` — 5/5 PASS
- [x] **T21** `cargo test 2>&1 | grep "test result"` — 全件 PASS（0 failures）
- [x] **T22** `cargo clippy --locked -- -D warnings` — PASS
- [x] **T23** `grep -rn "\bEffect\b" fav/src/ --include="*.rs" | grep -v "//"` — 0 件

---

## 完了処理

- [x] **T24** `benchmarks/v35.5.0.json` の `tests_passed` を実測値で確定
- [x] **T25** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = `"35.5.0"`
- [x] `Effect` enum が `ast.rs` に存在しないこと
- [x] `effects:` フィールドが AST 構造体に存在しないこと
- [x] `parse_effects_acc` が `parser.rs` に存在しないこと
- [x] `W022` が `lint.rs` に存在しないこと
- [x] `cargo test --bin fav v35500` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `cargo clippy --locked -- -D warnings` — PASS
- [x] `CHANGELOG.md` に `[v35.5.0]` セクション
- [x] `benchmarks/v35.5.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.5.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `Effect` という識別子がテスト外のコードから消えていること（コメント除く）
- [x] lineage.rs の変更後も lineage 機能（`fav explain --lineage`）が正常に動作すること
- [x] wasm_codegen.rs の FnDef 構造体リテラルがコンパイルエラーなく更新されていること
- [x] W021 lint（`pure_fn_calls_effectful`）が E0371 を参照していないこと（参照していれば合わせて削除）
- [x] `v35500_tests` に `use super::*` が**ない**こと

---

## 完了記録（2026-07-05）

- v35.5.0 として実装完了（tests: 2611 pass, 0 failures）
- Effect enum を ast.rs / checker.rs / compiler.rs / lineage.rs / reachability.rs / ir.rs から完全削除
- checker.fav の check_effects_all を no-op 化
- rune ファイル 95 件から !Effect アノテーション除去
- driver.rs テスト 33 件スタブ化
- cargo clippy -- -D warnings PASS

### コードレビュー指摘
なし（Clippy clean）
