# Tasks: v94.8.0 — サイトドキュメント完全化（SAP Advanced Era 総まとめ）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,156 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94700_tests` が存在することを確認する（v94.7.0 完了済みの証拠）
- [x] `infra/e2e-demo/sap-odata/pipeline_advanced.fav` が存在することを確認する（v94.7.0 完了済みの証拠）

## T1: `site/content/docs/guides/sap-integration.mdx` を新規作成する

- [x] `site/content/docs/guides/sap-integration.mdx` を新規作成する
- [x] フロントマター（title / order / category / description）を記載する
- [x] 以下のセクションを含める:
  - [x] 概要（SAP Advanced Era の全体像）
  - [x] 前提条件（fav.toml [sap] 設定 / 環境変数）
  - [x] `ctx.sap` パターン（基本的なエンティティ取得）
  - [x] QueryBuilder<T> 型安全クエリ
  - [x] `$batch` 一括操作（`BatchRequest<T>` / `BatchOperation<T>` / `ctx.sap.batch`）
  - [x] Metadata Infer（`fav infer --sap-metadata`）
  - [x] Lambda SnapStart（コールドスタート削減）
  - [x] E2E デモ（`infra/e2e-demo/sap-odata/` への参照）
  - [x] テスト（`MockSapClient` / `Ctx.mock`）
- [x] `batch` または `BatchRequest` が含まれることを確認する（テスト要件: `docs_sap_integration_guide_mentions_batch`）
- [x] Favnir コード例は `bind` を使う（`let` は禁止）
- [x] Favnir コメントは `--` を使う（`//` は禁止）

## T2: `site/content/docs/runes/sap-odata.mdx` に `$batch` セクションを追加する

- [x] `## OData $batch による一括操作（v94.1.0〜）` セクションを追加する（`## fav infer による型定義自動生成` の直前）
- [x] `BatchOperation<T>` ADT（BatchCreate / BatchUpdate / BatchDelete）の説明表を追加する
- [x] `batch_request_builder<T>` のシグネチャを追記する
- [x] `ctx.sap.batch(req)` の使用例コードブロックを追加する
- [x] `SapClient` メソッド表に `ctx.sap.batch(req)` を追記する
- [x] 業務シナリオ表にシナリオ 5（`advanced_sap_pipeline`）を追記する
- [x] Favnir コード例は `bind` を使う（`let` は禁止）

## T3: `site/content/docs/cli/infer.mdx` に `--sap-metadata` を追記する

- [x] `infer.mdx` を読んで既存の `--from sap` セクションを確認する
- [x] `--sap-metadata <url>` フラグの説明を追記する（HTTP エンドポイントから EDMX を取得）
- [x] `--sap-metadata-file <path>` フラグの説明を追記する（ローカルファイルから取得）
- [x] 使用例コードブロック（bash）を追加する

## T4: `driver.rs` に `mod v94800_tests` を追加する

- [x] `mod v94700_tests { ... }` の直後に `#[cfg(test)] mod v94800_tests { ... }` を追加する（2 テスト）
- [x] `docs_sap_integration_guide_exists`: `"../site/content/docs/guides/sap-integration.mdx"` が存在することを確認する
- [x] `docs_sap_integration_guide_mentions_batch`: ファイルに `"batch"` または `"BatchRequest"` が含まれることを確認する

## T5: `CHANGELOG.md` に v94.8.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.8.0 エントリを追加する

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,158 tests, 0 failures であることを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T6 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
