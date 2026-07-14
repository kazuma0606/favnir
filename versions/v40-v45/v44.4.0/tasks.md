# v44.4.0 タスク — 型推論 x パイプライン lineage

## ステータス: COMPLETE（2026-07-14）— 2953 tests

---

## T0 — 事前確認

- [x] `cargo test` 2951 / 0 確認
- [x] `Cargo.toml` version = `44.3.0` 確認
- [x] `v44400_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `collect_annotated_lineage_bindings` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — driver.rs: `collect_annotated_lineage_bindings` 追加

- [x] `collect_opaque_alias_groups` の直後（`bare_inner_literal_line` の直前）に追加
  - TrfDef の body.stmts を走査
  - `Stmt::Bind(b)` かつ `b.annotated_ty.is_some()` の束縛を収集
  - `format_type_expr` で型を文字列化
  - 返り値: `"<filename>:<line>: <stage_name>: <binding_name>: <type>"` 形式

---

## T2 — driver.rs: `v44400_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44300_tests` の直前に `v44400_tests` を挿入（2 件）
  - `cargo_toml_version_is_44_4_0`
  - `annotated_lineage_bindings_detected`
- [x] スタブ化: `v44300_tests::cargo_toml_version_is_44_3_0` の `assert!` を削除し `// Stubbed: version bumped to 44.4.0 in v44.4.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.3.0` → `44.4.0` に更新

---

## T3 — CHANGELOG.md に v44.4.0 エントリ追加

- [x] v44.4.0 エントリを CHANGELOG.md の先頭に追加（`[v44.4.0]` を含む）
  - 型推論 x パイプライン lineage の説明
  - `collect_annotated_lineage_bindings` ヘルパー追加

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2953 passed; 0 failed 確認
- [x] `v44400_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.4.0 最新安定版（2953 tests）、次版 v44.5.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.4.0 を `✅ COMPLETE（2026-07-14）`、推定テスト数 `2942` → `2953` に修正、「MVP: AST レベル型注釈収集、LineageEntry 統合は将来版」注記を追記
- [x] `versions/v40-v45/v44.4.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- `format_type_expr` は driver.rs のプライベート関数 — `super::` なしで同ファイル内から直接呼び出し可
- `stage Name: InType -> OutType = |params| { body }` が正しい stage 構文（v44.1.0 で確認済み）
- `Pattern::Bind(n, _)` が変数束縛の正しいバリアント（v44.1.0 実装時に確認済み）
- 全テスト一発通過（2953 passed; 0 failed）
