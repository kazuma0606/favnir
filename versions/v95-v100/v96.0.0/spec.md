# Spec: v96.0.0 — SAP Real-time 1.0 宣言

## Background

v95.1.0〜v95.9.0（SAP Real-time Sprint 1）の完成を受け、**SAP Real-time 1.0** を宣言する。

> 「SAP が、Favnir の時間軸で動き始めた。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに変化を知り、
>  Deep Insert で一気に書き込み、`fav sap-mock` でオフラインでもテストできる。
>
>  それが、Favnir SAP Real-time 1.0 である。」

スプリント成果のまとめ:
- `DeltaResult<T>` / `DeletedEntity`（OData $delta）
- `SapEventClient` / `SapEventMessage`（Event Mesh）
- `create_sales_order_deep`（Deep Insert）
- `function_import<T>` / `action_import`（Function/Action Import）
- `BatchItemResult<T>` / `PartialSuccess<T>` / `batch_with_partial`（部分失敗）
- `fav sap-mock`（オフラインテスト）

## Goals

1. `fav/Cargo.toml` の version を `96.0.0` に更新する
2. `CHANGELOG.md` に `[v96.0.0]` エントリを追加する
3. `MILESTONE.md` の先頭に v96.0.0 エントリを追加する
4. `README.md` に `## v96.0 — SAP Real-time 1.0` セクションを追加する
5. `fav/src/driver.rs` に `mod v96000_tests`（4 件）を追加する
6. `cargo clean` を実施する（★クリーンアップ）
7. `cargo test` で 4,188 tests, 0 failures を確認する（cargo clean 後）

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 修正 | version を `96.0.0` に更新 |
| `CHANGELOG.md` | 修正 | `[v96.0.0]` エントリ追加 |
| `MILESTONE.md` | 修正 | v96.0.0 SAP Real-time 1.0 エントリ追加 |
| `README.md` | 修正 | `## v96.0 — SAP Real-time 1.0` セクション追加 |
| `fav/src/driver.rs` | 修正 | `mod v96000_tests`（4 件）追加 |

## Success Criteria（v96000_tests 4 件）

- `cargo_toml_version_is_96_0_0`: `Cargo.toml` に `version = "96.0.0"` が含まれる
- `changelog_has_v96_0_0`: `CHANGELOG.md` に `v96.0.0` が含まれる
- `milestone_has_sap_realtime`: `MILESTONE.md` に `SAP Real-time` が含まれる
- `readme_mentions_sap_realtime`: `README.md` に `SAP Real-time` が含まれる

## 確認手順

- `cargo test` で 4,188 tests, 0 failures
- `cargo clean` 後に `cargo test` で 4,188 tests, 0 failures を再確認する
