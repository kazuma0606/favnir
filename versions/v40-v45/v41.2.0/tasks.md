# v41.2.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2851（前バージョン 2848 + 3）
**実績テスト数**: 2853 passed, 0 failed（2026-07-11、code-reviewer 対応後）

---

## T0 — 事前確認

- [x] `cargo test` が 2848 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.1.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.2.0 を確認
- [x] `v41100_tests::cargo_toml_version_is_41_1_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44531
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v41100_tests` の閉じ `}` の行番号を確認し記録: 行44549
- [x] `driver.rs` に `v41200_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `error_catalog.rs` に E0404 が存在しないことを確認（変更前確認）
- [x] `error_catalog.rs` の E0403 行番号と E05xx セクションコメント行番号を確認し記録: E0403=行468、E05xx=行500
- [x] `checker.rs` の `type_invariants` が TypeDef Alias の invariants を収集済みであることを確認（line 2158-2159）— 新規フィールド追加不要を確認
- [x] `checker.fav` の `TypeDef` 実フィールドリストを確認し記録（`name / is_record / type_params / variants / fields`）
- [x] `checker.fav` の `TypeDef` に `invariants` フィールドが存在しないことを確認（変更前確認）
- [x] `checker.fav` の型定義チェック関数名を確認し記録: `check_item` 内の `IType(td)` 分岐（行2271）
- [x] `checker.fav` 内の `TypeDef` 構築箇所を grep し、`invariants` フィールド追加後にデフォルト値の補完が必要か確認: TypeDef はパターンマッチのみで直接構築なし→補完不要

---

## T1 — error_catalog.rs: E0404〜E0406 追加

- [x] `E0403` 直後・`// ── E05xx` セクションコメント直前に以下を追加（ErrorEntry 形式）
  - `E0404`: "refinement constraint violation"（category: "types"）
  - `E0405`: "ambiguous refinement type"（category: "types"）
  - `E0406`: "refinement constraint type mismatch"（category: "types"）
- [x] `// ── E04xx: Refinement type (v41.2.0)` セクションコメント付きで追加

---

## T2 — checker.fav: TypeDef invariants 統合

- [x] `TypeDef` レコード型末尾に `invariants: List<String>` フィールド追加
- [x] T0 で確認した既存 TypeDef 構築箇所に補完不要を確認（パターンマッチのみ）
- [x] `check_refinement_alias` stub のコメント中 `E0400` → `E0404` に修正
- [x] `check_item` の `IType(td)` 分岐から `check_refinement_alias` の統合呼び出し追加
- [x] `ast_lower_checker.rs` の `lower_type_def` に `("invariants", vm_list(vec![]))` 追加

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "41.1.0"` → `"41.2.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v41.2.0]` エントリを `[v41.1.0]` の直後に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v41100_tests::cargo_toml_version_is_41_1_0` をスタブ化
- [x] `v41200_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 不要）
  - `cargo_toml_version_is_41_2_0`（NOTE コメント付き）
  - `changelog_has_v41_2_0`
  - `error_catalog_has_e0404`（`include_str!("error_catalog.rs")` で `"E0404"` 存在確認）

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2851 を確認（実績: 2851）
- [x] `v41200_tests` 3 件すべて pass を確認
- [x] E0405 / E0406 も `error_catalog.rs` に存在することを目視確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.2.0（最新安定版）・v41.3.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.2.0 を完了済みにマーク（テスト数実績値 2851 に更新・E0404〜E0406 明記）
- [x] `versions/v40-v45/v41.2.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [BUG][MED] E0404 example の `let x: Age = -1` → `bind x: Age <- -1` に修正（Favnir 構文対応）
- [BUG][LOW] `check_item` の IType 分岐にスタブコメント 2 件追加（invariants 常に空リスト・check_refinement_alias stub の旨）
- [BUG][LOW] E0405/E0406 テスト欠落 → `error_catalog_has_e0405` / `error_catalog_has_e0406` を v41200_tests に追加（テスト数 2851 → 2853）
- [STYLE][LOW] E04xx コメント見出し混在・日英混在・IWrapper 設計差分コメント → 次バージョン以降で対処予定として記録

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] 4件 → E0404〜番号確定・type_invariants 既存確認・checker.fav TypeDef 実フィールド対応・テスト数修正 / [MED] 3件・[LOW] 1件 → 全対応）
- [x] code-reviewer 指摘対応済み（[BUG][MED] 1件・[BUG][LOW] 2件 → 全対応 / [STYLE][LOW] 3件 → 次バージョン記録）
