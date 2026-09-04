# Plan: v94.8.0 — サイトドキュメント完全化（SAP Advanced Era 総まとめ）

## 依存関係

ドキュメント変更のみのバージョン。Rust コード変更は `driver.rs` のテスト追加のみ。
MDX ファイル → driver.rs テスト → CHANGELOG の順で実施する。

---

## Step 1: `site/content/docs/guides/sap-integration.mdx` 新規作成

新規 MDX ガイドファイルを作成する。

**フロントマター:**
```yaml
---
title: "SAP Integration Guide"
order: 10
category: "Guide"
description: "Favnir で SAP S/4HANA を統合する完全ガイド（ctx.sap / QueryBuilder / $batch / Metadata Infer / SnapStart）"
---
```

**セクション構成:**
1. 概要（SAP Advanced Era の全体像）
2. 前提条件（fav.toml [sap] 設定 / 環境変数）
3. `ctx.sap` パターン（基本的なエンティティ取得）
4. QueryBuilder<T> 型安全クエリ
5. `$batch` 一括操作（`BatchRequest<T>` / `BatchOperation<T>` / `ctx.sap.batch`）
6. Metadata Infer（`fav infer --sap-metadata`）
7. Lambda SnapStart（コールドスタート削減）
8. E2E デモ（`infra/e2e-demo/sap-odata/`）
9. テスト（`MockSapClient` / `Ctx.mock`）

テスト要件: `batch` または `BatchRequest` を含めること（`docs_sap_integration_guide_mentions_batch`）

---

## Step 2: `site/content/docs/runes/sap-odata.mdx` に `$batch` セクション追加

既存 `sap-odata.mdx` の末尾（`## fav infer による型定義自動生成` セクションの直前）に追加する。

**追加内容:**
- `## OData $batch による一括操作（v94.1.0〜）` セクション
  - `BatchOperation<T>` ADT 説明表
  - `batch_request_builder<T>` シグネチャ
  - `ctx.sap.batch(req)` 使用例コードブロック
- `SapClient` メソッド表に `ctx.sap.batch(req)` を追記
- 業務シナリオ表にシナリオ 5 を追記

---

## Step 3: `site/content/docs/cli/infer.mdx` に `--sap-metadata` 追記

既存 `infer.mdx` の末尾または `--from sap` セクションに追記する。

**追加内容:**
- `--sap-metadata <url>` フラグ説明（HTTP エンドポイントから取得）
- `--sap-metadata-file <path>` フラグ説明（ローカルファイルから取得）
- 使用例コードブロック

---

## Step 4: `driver.rs` に `mod v94800_tests` 追加

`mod v94700_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v94800_tests {
    #[test]
    fn docs_sap_integration_guide_exists() {
        assert!(
            std::path::Path::new("../site/content/docs/guides/sap-integration.mdx").exists(),
            "sap-integration.mdx が存在すること"
        );
    }

    #[test]
    fn docs_sap_integration_guide_mentions_batch() {
        let content = std::fs::read_to_string(
            "../site/content/docs/guides/sap-integration.mdx"
        ).expect("sap-integration.mdx を読み込めること");
        assert!(
            content.contains("batch") || content.contains("BatchRequest"),
            "sap-integration.mdx に batch または BatchRequest が含まれること"
        );
    }
}
```

---

## Step 5: `CHANGELOG.md` に v94.8.0 エントリ追記

先頭に追加する。

```markdown
## [v94.8.0] — 2026-08-30 — サイトドキュメント完全化（SAP Advanced Era 総まとめ）

### Added
- `site/content/docs/guides/sap-integration.mdx`（新規作成）— SAP Advanced Era 統合ガイド全体像
  - ctx.sap / QueryBuilder<T> / $batch / Metadata Infer / Lambda SnapStart を 1 ページにまとめ
- `site/content/docs/runes/sap-odata.mdx` — `$batch` セクション追加（BatchOperation<T> / batch_request_builder / ctx.sap.batch）
- `site/content/docs/cli/infer.mdx` — `--sap-metadata` / `--sap-metadata-file` フラグ記述を追記
- `fav/src/driver.rs` — `mod v94800_tests`（テスト 2 件）を追加
  - `docs_sap_integration_guide_exists`: `site/content/docs/guides/sap-integration.mdx` が存在する
  - `docs_sap_integration_guide_mentions_batch`: ガイドに `batch` または `BatchRequest` が含まれる
- 合計テスト数: **4,158**（+2）
```

---

## Step 6: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、4,158 tests, 0 failures を確認する。

---

## Step 7: CI 事前確認（T-last）

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
