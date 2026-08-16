# v71.9.0 タスクリスト — 安定化・コードフリーズ（Type System 2.0 前調整）

Date: 2026-08-11
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.8.0` であることを確認
- [x] `cargo test` が 3606 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v718000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v719000_tests` が未存在であることを確認
- [x] `Vec<Float>[384]` 型注釈が parser で通ることを確認（v71.1 実装済み）
- [x] `type Score = Float where self >= 0.0` 構文が parser で通ることを確認（`BinOp::GtEq` は checker.rs line 4799 に実装済み）
- [x] `interface Sortable { key: Self -> Int }` 構文（フィールド記法）が parser/checker で通ることを確認（`fn` 付き記法は parser 非対応のため使わない）

---

## T1: `v719000_tests` モジュール追加（`driver.rs`）

- [x] `v718000_tests` モジュールの直後に `v719000_tests` モジュールを追加した
- [x] `#[cfg(test)]` のみ（`#[cfg(not(target_arch = "wasm32"))]` は不要）
- [x] `use crate::frontend::parser::Parser` + `use crate::middle::checker::Checker` を追加した
- [x] `type_system_2_all_stable` テストを実装した
  - v71.1: `Vec<Float>[1536]` 依存型引数
  - v71.2: `type PositiveFloat = Float where self > 0.0`
  - v71.3: `type UserId = phantom String`
  - v71.4: `const EMBED_DIM: Int = 1536`
  - v71.5: `interface Sortable { key: Self -> Int }` + `fn top_item<T: Sortable>`（フィールド記法を使用）
  - v71.6（AOT）/ v71.7（WASM）: 実行バイナリが必要なためスコープ外（`aot_native_binary_runs_hello` / `wasm_target_runs_simple_pipeline` で既にカバー済み）
  - v71.8: `bind n <- current_count()`
  - `errors.is_empty()` を assert
- [x] `dependent_refined_phantom_e2e` テストを実装した
  - `const VEC_DIM: Int = 384` + `Vec<Float>[384]` 依存型
  - `type UserId = phantom String` + `UserId("u-123")` コンストラクタ
  - `type Score = Float where self >= 0.0` refined type
  - `errors.is_empty()` を assert
- [x] `cargo build` でエラーがないことを確認

---

## T2: `cargo_toml_version` テスト文字列を更新

- [x] `driver.rs` 内の `"71.8.0"` バージョンアサーション文字列を `"71.9.0"` に更新した（replace_all）

---

## T3: `fav/Cargo.toml` バージョン更新

- [x] `version = "71.8.0"` → `version = "71.9.0"` に変更した

---

## T4: 部分テスト確認

- [x] `cargo test v719000` で 2 件 pass することを確認

---

## T5: CHANGELOG.md 更新

- [x] `## [v71.9.0]` エントリを先頭に追加した

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.9.0`（安定化・コードフリーズ）に更新した
- [x] 「次に切る版」を `v72.0.0` に更新した

---

## T7: 最終確認

- [x] `cargo test v719000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3608 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.9.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- v71.6.0（AOT）バイナリ実行テスト: `aot_native_binary_runs_hello` で既にカバー済み
- v71.7.0（WASM）実行テスト: `wasm_target_runs_simple_pipeline` で既にカバー済み
- `v72.0.0` 宣言・`MILESTONE.md` 更新: v72.0.0 スコープ
- `site/` MDX 追加: 機能追加なしのため対象外

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `interface Sortable { fn key(self: Self) -> Int }` が parser 非対応（panic リスク） | `{ key: Self -> Int }` フィールド記法に修正 |
| [HIGH] | ロードマップテスト数 3599 vs spec 3608 の乖離 | ロードマップを 3606+2=3608 に更新 |
| [MED] | `>=` 確認が「はず」表現 | 「checker.rs line 4799 に実装済み、確認不要」と断定 |
| [MED] | tasks.md T1 に v71.6/v71.7 除外根拠がない | 既存テストでカバー済みと明記 |
| [LOW] | plan.md に全体 `cargo test` 確認ステップがない | Step 7 を追加 |
| [MED] | `VEC_DIM` 定数が `similarity` 型引数で未使用（リテラル `[384]`） | `Vec<Float>[VEC_DIM]` に変更し const → 依存型連結を実際に検証 |
| [MED] | `Score` 型が宣言のみで未使用（デッドコード） | `fn apply_score(s: Score) -> Float` を追加して refined type の実利用を検証 |
| [LOW] | v71.5 の `&` 複数境界・`impl Trait` が `type_system_2_all_stable` に含まれない | `interface Comparable` + `fn top_item<T: Sortable & Comparable>` に変更 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `type_system_2_all_stable` が pass
- [x] `dependent_refined_phantom_e2e` が pass
- [x] テスト総数: 3608（+2、実績ベース: 3606 + 2）
