# v71.8.0 タスクリスト — 型推論強化（型注釈省略可能範囲の拡大）

Date: 2026-08-11
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.7.0` であることを確認
- [x] `cargo test` が 3604 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v717000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v718000_tests` が未存在であることを確認
- [x] `checker.rs` の `Stmt::Bind` ハンドラ（line ~4301）で `annotated_ty = None` 時に `effective_ty` で `check_pattern_bindings` が呼ばれることを確認
- [x] `checker.rs` の `Expr::Closure` ハンドラ（line ~5471）でパラメータが `Type::Unknown` に初期化されることを確認
- [x] `List.fold`（checker.rs line ~6416）と `List.length`（line ~6353）が `check_builtin_apply` の match アームに存在することを確認
- [x] `List.of` は `check_builtin_apply` 専用アームなし（`Unknown` 返却）→ `expect_list_arg` が `Unknown` を受け入れることを確認（line ~7861）

---

## T1: `v718000_tests` モジュール追加（`driver.rs`）

- [x] `v717000_tests` モジュールの直後に `v718000_tests` モジュールを追加した
- [x] `#[cfg(test)]` のみ（`#[cfg(not(target_arch = "wasm32"))]` は不要）
- [x] `use crate::frontend::parser::Parser` を追加した
- [x] `use crate::middle::checker::Checker` を追加した
- [x] `type_infer_local_var_omit_annotation` テストを実装した
  - `fn get_values() -> List<Int>` + `bind items <- get_values()` (注釈なし)
  - `Checker::check_program` で errors.is_empty() を assert
- [x] `type_infer_closure_arg_omit` テストを実装した
  - `List.fold(items, 0, |acc, x| acc + x)` (引数型注釈なし)
  - `Checker::check_program` で errors.is_empty() を assert
- [x] `cargo build` でエラーがないことを確認

---

## T2: `cargo_toml_version` テスト文字列を更新

- [x] `driver.rs` 内の `"71.7.0"` バージョンアサーション文字列を `"71.8.0"` に更新した（replace_all）

---

## T3: `fav/Cargo.toml` バージョン更新

- [x] `version = "71.7.0"` → `version = "71.8.0"` に変更した

---

## T4: 部分テスト確認（新規テストのみ）

- [x] `cargo test v718000` で 2 件 pass することを確認

---

## T5: CHANGELOG.md 更新

- [x] `## [v71.8.0]` エントリを先頭に追加した

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.8.0`（型推論強化）に更新した
- [x] 「次に切る版」を `v71.9.0` に更新した

---

## T7: 最終確認

- [x] `cargo test v718000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3606 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.8.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- `fresh_var` / `unify` の大規模リファクタリング: 既存実装で動作するため対象外
- `fav check --show-types` の型表示拡張: v12.5.0 で実装済み、対象外
- ポリモーフィック関数の推論強化: v72.x 以降
- `checker.fav`（セルフホスト型チェッカー）への反映: スコープ外
- `site/` MDX 追加: 機能追加なし（テストのみ）のため対象外

---

## コードレビュー指摘対応

（実装後に記録）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | ロードマップ「`fresh_var`/`unify` 改善」が spec スコープ外と矛盾 | ロードマップを「改善なし（既存挙動確認のみ）」に更新 |
| [HIGH] | ロードマップ「`fav check --show-types`」が spec スコープ外と矛盾 | ロードマップを「v12.5.0 実装済み、本バージョン変更なし」に更新 |
| [MED] | `register_builtins` 説明が不正確（実態は `check_builtin_apply`） | spec/plan/tasks を `check_builtin_apply` line 6353/6416 ベースに修正 |
| [MED] | ロードマップのテスト数 3597 vs spec 3606 の不一致 | ロードマップを 3604+2=3606 に更新 |
| [LOW] | tasks.md T0 の確認先が `register_builtins`（不正確） | `check_builtin_apply` の match アームを確認するよう修正 |
| [MED] | test1: `List.of` が `Unknown` 返却→ `bind` 推論ではなく Unknown 互換でパス | `current_count() -> Int` + `bind n <- current_count()` に変更し宣言戻り型推論を明示 |
| [MED] | test2: `List.of` の `items` が `Unknown` → クロージャ引数推論の確認が不十分 | `sum(items: List<Int>)` 関数パラメータで `items` を明示型にし `\|acc, x\|` の推論を確実に検証 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `type_infer_local_var_omit_annotation` が pass
- [x] `type_infer_closure_arg_omit` が pass
- [x] テスト総数: 3606（+2、実績ベース: 3604 + 2）
