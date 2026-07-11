# v35.5.0 spec — `!Effect` 廃止 Phase 2

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.5.0 |
| テーマ | `!Effect` 廃止 Phase 2 — `Effect` enum・`effects` フィールド・`parse_effects_acc` を AST/パーサーから完全削除し、`effect` 宣言を no-op 化する |
| 前提 | v35.4.0 COMPLETE（E0374 パースエラー化・W022 lint 削除） |
| 完了条件 | `v35500_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.4.0 で `!Effect` アノテーション構文（`fn f() -> Int !Io { ... }`）をパースエラー（E0374）に変更した。
それと並行して、v35.1〜v35.5 の `!Effect` 廃止スプリントの中で `Effect` 型本体も AST/パーサーから削除済みである。

本バージョンでは以下を**テストによって正式に保証**する:
1. `ast.rs` の `pub enum Effect { ... }` が存在しない（スプリント中に削除済み）
2. `FnDef` 等の `effects: Vec<Effect>` フィールドが存在しない（スプリント中に削除済み）
3. `parser.rs` の `fn parse_effects_acc` が存在しない（スプリント中に削除済み）
4. `checker.rs` の `effect` 宣言処理が no-op である（`effect Foo` は解析されるが何も登録しない）

## ロードマップとの差異

`versions/roadmap/roadmap-v35.1-v36.0.md` では v35.5.0 を
`deploy.fav` 宣言的デプロイ設定と計画していたが、
`!Effect` 廃止作業（v35.4.0 から継続）を優先したため本バージョンで Phase 2 を実施する。

`deploy.fav` 機能は後続バージョンで対応する。

## 実装スコープ

### 対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `pub enum Effect { ... }` 削除済み・`effects: Vec<Effect>` 削除済み（スプリント中に実施 — 本バージョンで確認のみ） |
| `fav/src/frontend/parser.rs` | `fn parse_effects_acc` 削除済み（スプリント中に実施 — 本バージョンで確認のみ） |
| `fav/src/middle/checker.rs` | `effect` 宣言を no-op 化済み（`effect_registry` フィールドは残存、登録処理のみ削除 — 本バージョンで確認のみ） |
| `fav/src/driver.rs` | `v35500_tests` モジュール（6 件）追加・`v35400_tests::cargo_toml_version_is_35_4_0` スタブ化 |
| `fav/Cargo.toml` | バージョン `35.4.0` → `35.5.0` |
| `CHANGELOG.md` | `## [35.5.0]` エントリ追加（既存） |

### 対象外（スコープ外）

- `checker.rs` の `effect_registry` フィールド自体の削除（構造体から完全除去は v35.6.0 以降）
- `deploy.fav` 宣言的デプロイ設定（ロードマップ計画、後続対応）
- W007/W021 lint ルールの削除（v35.6.0 で対応）

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `ast.rs` に `pub enum Effect {` が存在しない | `effect_enum_removed_from_ast` テスト |
| 2 | `ast.rs` に `effects: Vec<Effect>` が存在しない | `effects_field_removed_from_fn_def` テスト |
| 3 | `parser.rs` に `fn parse_effects_acc` が存在しない | `parse_effects_acc_removed_from_parser` テスト |
| 4 | `effect Payment` のパースがエラーなしで通る（no-op 化） | `effect_def_no_longer_registers_in_checker` テスト |
| 5 | `cargo test` 全通過（0 failures） | `cargo test` 実行 |
| 6 | `CHANGELOG.md` に `[35.5.0]` エントリが存在する | `changelog_has_v35_5_0` テスト（`include_str!` による自動確認） |

## 設計決定

- **Effect enum 削除**: `ast.rs` から `pub enum Effect` と派生実装を全削除。コンパイルが通ることで削除を証明する
- **`effects` フィールド削除**: `FnDef`・`EffectDef` 等から `effects: Vec<Effect>` を削除。パーサーは `!Effect` 構文を E0374 で弾くため、このフィールドが設定されることはない
- **`parse_effects_acc` 削除**: v35.4.0 で `!Effect` がパースエラーになったため、エフェクト列を解析する関数は不要
- **no-op 化（checker.rs）**: `effect Foo` 構文は `EffectDef` AST ノードとしてパースされるが、チェッカーは何も行わない。`effect_registry` フィールドの完全削除は v35.6.0 以降に行う
