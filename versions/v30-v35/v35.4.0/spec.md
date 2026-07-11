# v35.4.0 spec — `!Effect` アノテーション廃止 Phase 1

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.4.0 |
| テーマ | `!Effect` アノテーション廃止 Phase 1 — E0374 パースエラー化・W022 lint 削除・error_catalog 登録 |
| 前提 | v35.3.0 COMPLETE（`fav ci init` 実装済み） |
| 完了条件 | `v35400_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.3.0 時点で `!Effect` アノテーション構文（例: `fn f() -> Int !Io { 1 }`）は
パーサー側で既に E0374 エラーを返すが、エラーカタログへの正式登録と
W022 lint ルール（deprecated `!Effect` annotation）の削除が未完了だった。

本バージョンでは以下を正式完結させる:

1. **E0374 パースエラー化の正式化**: `!Effect` 構文が `[E0374]` エラーを返すことを保証
2. **W022 lint 削除**: `check_w022_deprecated_effect_annotation` 関数が lint.rs に存在しないことを保証
3. **ctx:AppCtx 旁通の確認**: `!Effect` アノテーション廃止後、effect チェックは `ctx: AppCtx` の有無で代替される設計のため、`ctx: AppCtx` 引数を持つ関数が E0107（effect 違反）を発生させないことをこのバージョンで同時確認する
4. **error_catalog.rs への E0374 登録**: `error_catalog.rs` に `E0374` エントリが存在することを保証

## ロードマップとの差異

`versions/roadmap/roadmap-v35.1-v36.0.md` では v35.4.0 を
`fav deploy --target k8s`（Deployment + Service + ConfigMap 生成）と計画していたが、
`!Effect` 廃止作業を優先したためこのバージョンで Effect 廃止 Phase 1 を実施する。

k8s Manifest 生成（CronJob）は `cmd_deploy_k8s` として v30.1〜v35.0 スプリント中に先行実装済み
（ロードマップ外の先行実装）。Deployment + Service + ConfigMap への拡張は後続バージョンで対応する。

## 実装スコープ

### 対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/parser.rs` | `!` トークン検出時に E0374 エラーを返す（確認のみ・変更なし） |
| `fav/src/lint.rs` | `check_w022_deprecated_effect_annotation` を削除済み（確認のみ・変更なし） |
| `fav/src/error_catalog.rs` | E0374 エントリ登録済み（確認のみ・変更なし） |
| `fav/src/driver.rs` | `v35400_tests` モジュール（5 件: 機能 4 件 + CHANGELOG 1 件）追加・バージョン bump |
| `fav/Cargo.toml` | バージョン `35.3.0` → `35.4.0` |
| `CHANGELOG.md` | `## [35.4.0]` エントリ追加 |

### 対象外（スコープ外）

- k8s Deployment + Service + ConfigMap YAML 生成（後続対応）
- `fav deploy --target k8s` の既存 CronJob 動作への変更なし
- `!Effect` 廃止の残り作業（Effect enum 削除等）は v35.5.0 以降

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `fn f() -> Int !Io { 1 }` のパースが E0374 エラーを返す | `effect_annotation_is_parse_error_e0374` テスト |
| 2 | `ctx: AppCtx` 関数が E0107 を発生させない | `ctx_appctx_bypasses_effect_check` テスト |
| 3 | `check_w022_deprecated_effect_annotation` が lint.rs に存在しない | `w022_lint_removed` テスト |
| 4 | `error_catalog.rs` に `E0374` エントリが存在する | `e0374_in_error_catalog` テスト |
| 5 | `cargo test` 全通過（0 failures） | `cargo test` 実行 |
| 6 | `CHANGELOG.md` に `[35.4.0]` エントリが存在する | `changelog_has_v35_4_0` テスト（`include_str!` による自動確認） |

## 設計決定

- **実装場所**: `parser.rs` に直接 E0374 を追加（`deploy/k8s.rs` 等の別モジュール化は不要）
- **W022 削除**: `check_w022_deprecated_effect_annotation` を削除し、コメントのみ残す（削除証跡として）
- **error_catalog 登録**: 既存の `ErrorCatalogEntry` パターンに従い E0374 を追加
