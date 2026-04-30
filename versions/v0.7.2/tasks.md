# Favnir v0.7.2 タスク一覧

更新日: 2026-04-30

> [ ] 未完了 / [x] 完了
>
> **ゴール**: 単一フラット src/ を三相モジュール（frontend/middle/backend）に再編し、全テストが通ること
> **方針**: 機能変更なし・API 変更なし・cargo test 全通過

---

## Phase 1: ディレクトリ・mod.rs

- [x] `src/frontend/` ディレクトリ作成
- [x] `src/middle/` ディレクトリ作成
- [x] `src/backend/` ディレクトリ作成
- [x] `src/frontend/mod.rs` 作成
- [x] `src/middle/mod.rs` 作成
- [x] `src/backend/mod.rs` 作成

## Phase 2: Frontend 移動

- [x] `src/lexer.rs` → `src/frontend/lexer.rs`（import 変更なし）
- [x] `src/parser.rs` → `src/frontend/parser.rs`（`crate::lexer` → `super::lexer`）
- [x] 旧 `src/lexer.rs`, `src/parser.rs` 削除

## Phase 3: Middle 移動

- [x] `src/checker.rs` → `src/middle/checker.rs`（`crate::lexer::Span` → `crate::frontend::lexer::Span`）
- [x] `src/ir.rs` → `src/middle/ir.rs`（`crate::checker` → `super::checker`）
- [x] `src/compiler.rs` → `src/middle/compiler.rs`（`crate::checker` → `super::checker`、`crate::ir` → `super::ir`）
- [x] `src/resolver.rs` → `src/middle/resolver.rs`（複数 import 更新）
- [x] 旧ファイル削除

## Phase 4: Backend 移動

- [x] `src/artifact.rs` → `src/backend/artifact.rs`（`crate::codegen` → `super::codegen`）
- [x] `src/codegen.rs` → `src/backend/codegen.rs`（`crate::artifact` → `super::artifact`、`crate::ir` → `crate::middle::ir`）
- [x] `src/vm.rs` → `src/backend/vm.rs`（`crate::artifact` → `super::artifact`、`crate::codegen` → `super::codegen`）
- [x] 旧ファイル削除

## Phase 5: main.rs 分割

- [x] `src/driver.rs` 新規作成（cmd_* 関数群 + ヘルパー + format 関数）
- [x] `src/main.rs` をエントリポイント + CLI ディスパッチのみに縮小（~160行）
- [x] `mod driver;` を main.rs に追加

## Phase 6: ast.rs 修正

- [x] `ast.rs`: `use crate::lexer::Span` → `use crate::frontend::lexer::Span`

## Phase 7: テスト

- [x] `cargo build` 成功
- [x] `cargo test` 全テスト通過（302件）

## Phase 8: ARCHITECTURE.md

- [x] `fav/ARCHITECTURE.md` 作成（データフロー図 + モジュール責務表）
