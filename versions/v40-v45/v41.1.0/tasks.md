# v41.1.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2848（前バージョン 2845 + 3）
**実績テスト数**: 2848 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2845 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.1.0 を確認
- [x] `v41000_tests::cargo_toml_version_is_41_0_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44505
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v41000_tests` の閉じ `}` の行番号を確認し記録: 行44527
- [x] `driver.rs` に `v41100_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `parse_type_def` の Alias 分岐に `where` 節が存在しないことを確認（変更前確認）

---

## T1 — parser.rs: Alias 型に `where` 節を追加

- [x] `parse_type_def` の Wrapper 型 `where` 節（line 1344〜1350）を参照し、同パターンで Alias 分岐に実装することを確認
- [x] `parse_type_def` の Alias 分岐を修正
  ```rust
  } else {
      // type alias: type Name = TypeExpr
      let target = self.parse_type_expr()?;
      // v41.1.0: refinement constraint `where |v| pred` for type aliases
      let invariants = if self.peek() == &TokenKind::Where {
          self.advance(); // consume `where`
          vec![self.parse_expr()?]
      } else {
          vec![]
      };
      return Ok(TypeDef {
          visibility,
          name,
          type_params,
          with_interfaces,
          invariants,
          body: TypeBody::Alias(target),
          span: self.span_from(&start),
      });
  };
  ```

---

## T2 — checker.fav: `check_refinement_alias` スタブ追加

- [x] `fav/self/checker.fav` に `check_refinement_alias` スタブを追加

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "41.0.0"` → `"41.1.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v41.1.0]` エントリを `[v41.0.0]` の直後に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v41000_tests::cargo_toml_version_is_41_0_0` をスタブ化
  ```rust
  #[test]
  fn cargo_toml_version_is_41_0_0() {
      // Stubbed: version bumped to 41.1.0 — assertion intentionally removed
  }
  ```
- [x] `v41100_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 不要）
  - `cargo_toml_version_is_41_1_0`（NOTE コメント付き）
  - `changelog_has_v41_1_0`
  - `refinement_type_alias_where_parseable`（`Parser::parse_str("type Age = Int where |v| v >= 0", "test.fav")` がエラーなく pass することを検証）

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2848 を確認（実績: 2848）
- [x] `v41100_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.1.0（最新安定版）・v41.2.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.1.0 を完了済みにマーク（完了条件テスト数を実績値 2848 に更新）
- [x] `versions/v40-v45/v41.1.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [LOW] Sum 型（`| A | B`）に `where` 節を付けると末尾トークンが残留しパースエラーになる動作が暗黙的 → v41.2.0 の E0400 統合時に明示的エラーメッセージで対処予定。今バージョンでの修正不要（記録のみ）

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] 2件・[LOW] 1件 → 全対応）
- [x] code-reviewer 指摘対応済み（[LOW] 1件 → v41.2.0 で対処予定として記録済み）
