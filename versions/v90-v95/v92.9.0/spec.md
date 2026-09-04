# Spec: v92.9.0 — 安定化・コードフリーズ

Status: TODO

---

## Background

v92.1.0〜v92.8.0 で `QueryBuilder<T>` API・`Page<T>` 型・`fetch_all_pages` スタブ・W060 N+1 lint ルール・E2E デモパイプライン・ベンチマーク・サイトドキュメントを構築した。
v92.9.0 は全機能の整合性を確認する安定化スプリント。新機能追加はなし。バグ修正のみ受け入れる。

---

## Goals

1. `runes/sap-odata/query_builder.fav` に `with_select` / `with_expand` / `with_filter` / `with_top` / `with_skip` / `with_order_by` の全 6 チェーン関数が揃っていることをテストで確認する
2. `runes/sap-odata/query_builder.fav` に `Page<T>` 型が定義されていることをテストで確認する
3. W060 N+1 lint の誤検知がないことを手動確認する（テストなし）
4. `fav/src/driver.rs` に `mod v92900_tests`（2 件）を追加する

---

## 確認観点

| 観点 | 確認方法 |
|---|---|
| `QueryBuilder<T>` 全チェーン関数 | Rust テスト（ファイル内容検索） |
| `Page<T>` 型 | Rust テスト（ファイル内容検索） |
| W060 lint 誤検知なし | `cargo test` 全 pass で確認 |
| fetch_all_pages スタブ | 既存テスト（v92.4.0〜v92.8.0）で確認済み |

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `mod v92900_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,116 tests, 0 failures**（4,114 + 2）
- `mod v92900_tests` 内の 2 テストが pass する:
  - `query_builder_smoke_all_chains`: `runes/sap-odata/query_builder.fav` に `with_select` / `with_expand` / `with_filter` / `with_top` / `with_skip` / `with_order_by` が含まれる
  - `query_builder_page_type_in_rune_dir`: `runes/sap-odata/query_builder.fav` が存在し、`Page` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4103（4101+2）だが、v92.8.0 の実測が 4,114 のため、本バージョンは 4,114 + 2 = **4,116** が目標。

> **W060 vs W020**: ロードマップの「W020 lint の誤検知がないことを確認」は実装上 **W060** が正しい（W020 は v24.4.0 で `check_w020_deprecated_call` として実装済み）。

> **新機能なし**: v92.9.0 はコードフリーズ。テストのみ追加する。
