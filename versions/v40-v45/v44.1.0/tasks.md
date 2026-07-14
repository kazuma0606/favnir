# v44.1.0 タスク — Refinement type x Streaming 統合

## ステータス: COMPLETE（2026-07-14）— 2944 tests

---

## T0 — 事前確認

- [x] `cargo test` 2941 / 0 確認
- [x] `Cargo.toml` version = `44.0.0` 確認
- [x] `v44100_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `collect_refinement_stream_bindings` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — driver.rs: `collect_refinement_stream_bindings` 追加

- [x] `check_opaque_coerce_violations` の直後に `collect_refinement_stream_bindings` 関数を追加
  - TypeDef の refinement（invariants 非空）を収集
  - FnDef / TrfDef の body.stmts を走査
  - `bind x: Stream<T>` または `bind x: List<T>` の T が refinement type 名と一致するものを収集
  - 返り値: `"<filename>:<line>: <name>: <container><elem>"` 形式の文字列リスト

---

## T2 — driver.rs: `v44100_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44000_tests` の直前に `v44100_tests` を挿入（3 件）
  - `cargo_toml_version_is_44_1_0`
  - `refinement_type_invariant_in_typedef_ast`
  - `collect_refinement_stream_bindings_detects_annotated_bind`
- [x] スタブ化: `v44000_tests::cargo_toml_version_is_44_0_0` に `// Stubbed: version bumped to 44.1.0 in v44.1.0.` コメント追加
- [x] `fav/Cargo.toml` version を `44.0.0` → `44.1.0` に更新

---

## T3 — CHANGELOG.md に v44.1.0 エントリ追加

- [x] v44.1.0 エントリを CHANGELOG.md の先頭に追加（`[v44.1.0]` を含む）
  - Refinement type x Streaming 統合の説明
  - `collect_refinement_stream_bindings` ヘルパー追加

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2944 passed; 0 failed 確認
- [x] `v44100_tests` 3 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.1.0 最新安定版（2944 tests）、次版 v44.2.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.1.0 を `✅ COMPLETE（2026-07-14）`
- [x] `versions/v40-v45/v44.1.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- `Pattern::Bind(String, Span)` が変数束縛の正しいバリアント名（`Pattern::Ident` は存在しない — spec-reviewer [HIGH] 指摘で修正）
- `where (> 0.0)` は無効構文。正しくは `where |v| v > 0.0`（ラムダ式形式）— テスト実行で判明・修正
- `stage` の正しい構文は `stage Name: InType -> OutType = |params| { body }`（`stage Name { }` ではない）— テスト実行で判明・修正
- スタブ化方法: `assert!` を削除し `// Stubbed: version bumped to 44.1.0 in v44.1.0.` に置き換える
- `collect_refinement_stream_bindings` は FnDef と TrfDef の 1 レベルのみ走査（ネスト Block は今回スコープ外）
- `TypeExpr::Named` は 3 フィールド: `(String, Vec<TypeExpr>, Span)`
