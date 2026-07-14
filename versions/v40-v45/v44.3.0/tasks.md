# v44.3.0 タスク — Stream join x Opaque type

## ステータス: COMPLETE（2026-07-14）— 2951 tests（code-review 対応で +1）

---

## T0 — 事前確認

- [x] `cargo test` 2947 / 0 確認
- [x] `Cargo.toml` version = `44.2.0` 確認
- [x] `v44300_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `collect_opaque_alias_groups` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — driver.rs: `collect_opaque_alias_groups` 追加

- [x] `collect_cep_expr_refinement_refs` の直後（`bare_inner_literal_line` の直前）に `collect_opaque_alias_groups` を追加
  - `opaque type Name = Inner`（型引数なし）の TypeDef を収集
  - inner type 名でグループ化（`HashMap<String, Vec<(name, line)>>`）
  - 2 件以上のグループのみレポート
  - 名前アルファベット順ソート
  - 返り値: `"<filename>:<line>: <inner>: <Name1>, <Name2>"` 形式

---

## T2 — driver.rs: `v44300_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44200_tests` の直前に `v44300_tests` を挿入（3 件）
  - `cargo_toml_version_is_44_3_0`
  - `opaque_alias_group_detected`
  - `non_opaque_type_excluded_from_groups`
- [x] スタブ化: `v44200_tests::cargo_toml_version_is_44_2_0` の `assert!` を削除し `// Stubbed: version bumped to 44.3.0 in v44.3.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.2.0` → `44.3.0` に更新

---

## T3 — CHANGELOG.md に v44.3.0 エントリ追加

- [x] v44.3.0 エントリを CHANGELOG.md の先頭に追加（`[v44.3.0]` を含む）
  - Stream join x Opaque type の説明
  - `collect_opaque_alias_groups` ヘルパー追加

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2950 passed; 0 failed 確認
- [x] `v44300_tests` 3 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.3.0 最新安定版（2950 tests）、次版 v44.4.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.3.0 を `✅ COMPLETE（2026-07-14）`、推定テスト数 `2940` → `2950` に修正、「MVP: AST レベル opaque グループ検出のみ、checker.fav E0413 統合は将来版」注記を追記
- [x] `versions/v40-v45/v44.3.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- `check_opaque_coerce_violations` の opaque 収集パターン（`is_opaque + TypeBody::Alias + TypeExpr::Named + params.is_empty()`）をそのまま流用
- `inner_types.sort()` と `names.sort()` で出力を安定させ、テストの `any()` チェックが順序非依存で動作
- `min().unwrap_or(0)` は保守的だが `entries.len() >= 2` 条件後なので実際には `None` にならない
- code-review 対応: MVP コメント追加（ジェネリック alias の無言スキップ説明）、`single_opaque_alias_does_not_form_group` テスト追加（2950→2951）
