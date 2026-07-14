# v41.1.0 実装計画

## 概要

Type Precision スプリント第 1 版。
`type Age = Int where (>= 0)` 構文を parser.rs に追加し、
`fav/self/checker.fav` に `check_refinement_alias` スタブを追加する。
AST 変更不要（`TypeDef.invariants` は既存フィールド）。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2845 tests / 0 failures であることを確認
- `Cargo.toml` version が `41.0.0` であることを確認
- `v41000_tests::cargo_toml_version_is_41_0_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `driver.rs` に `v41100_tests` モジュールが存在しないことを確認
- `type Age = Int where (>= 0)` が現時点でパースエラーになることを確認（変更前確認）

### Step 2 — parser.rs: Alias 型に `where` 節を追加
`parse_type_def` の Alias 分岐（line 1394〜1405 付近）を修正。
`TypeDef.invariants` を `vec![]` から `where` 節 parse 結果に変更する。

具体的変更箇所: `invariants: vec![]` → `where` 節検出後に `vec![self.parse_expr()?]` をセット。

### Step 3 — checker.fav: `check_refinement_alias` スタブ追加
`fav/self/checker.fav` に `check_refinement_alias(ty_name, invariants)` スタブ関数を追加。
本体は `true` を返すのみ（v41.2.0 で実装）。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "41.0.0"` → `"41.1.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v41.1.0]` エントリを `[v41.0.0]` の直後に追加。

### Step 6 — driver.rs テストモジュール更新
1. `v41000_tests::cargo_toml_version_is_41_0_0` をスタブ化
2. `v41100_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 不要）

### Step 7 — cargo test 実行
`cargo test` で 2848 tests / 0 failures を確認。

### Step 8 — バージョン管理ドキュメント更新
`versions/current.md`・ロードマップ完了マーク・`tasks.md` COMPLETE 更新。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（parser.rs）
       └→ Step 6（driver.rs — refinement_type_alias_where_parseable）
  └→ Step 3（checker.fav）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_41_1_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v41_1_0）
            └→ Step 7（cargo test）
                 └→ Step 8（docs 更新）
```

Step 2〜5 は相互に独立しており並列実施可能。

---

## リスクと注意点

- **テスト数差異**: ロードマップ記載は「推定 2843」だが、v41.0.0 実績（2845）を起点に 2848 を採用する（spec.md §ロードマップとの差異 参照）。
- `TypeDef.invariants` フィールドは v9.7.5 から存在するため AST 変更不要。`where` 節が absent の場合は従来どおり `invariants: vec![]` が返る。
- Alias 分岐の `where` 節は、既存の Wrapper 型の `where` 節（`type UserId(Int) where ...`）と同じ `TokenKind::Where` + `parse_expr()` パターンを踏襲する。
- `refinement_type_alias_where_parseable` テストは `crate::frontend::parser::Parser::parse_str` を直接呼び出す。`Parser::parse_str` は pub のため `use super::*` 不要。
- `check_refinement_alias` は checker.fav に追加するが、v41.1.0 では呼び出し箇所は設けない（v41.2.0 で統合）。
- `include_str!` パスは `../` = `fav/`、`../../` = `favnir/` ルート（従来パターン通り）。
