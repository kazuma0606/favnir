# Spec: v93.0.0 — SAP QueryBuilder 1.0 宣言 ★クリーンアップ

Status: TODO

---

## Background

v92.1.0〜v92.9.0 で `QueryBuilder<T>` API・`Page<T>` 型・`fetch_all_pages` スタブ・W060 N+1 lint ルール・E2E デモパイプライン・ベンチマーク・サイトドキュメントを構築した。
v93.0.0 は「SAP QueryBuilder 1.0」の宣言バージョン。`cargo clean` によるビルドアーティファクト一掃と、バージョン番号の `93.0.0` への更新を行う。

---

## 宣言文

> 「`query<SalesOrder>() |> with_filter(Eq("SoldToParty", "CUST-001")) |> with_top(50)` と書けば、
>  型安全な OData クエリが組み立てられる。
>  ページネーションは `fetch_all_pages` で自動化され、N+1 は W060 で防がれる。
>  それが、Favnir SAP QueryBuilder 1.0 である。」

---

## Goals

1. `cargo clean` でビルドアーティファクトを一掃する
2. `fav/Cargo.toml` のバージョンを `93.0.0` に更新する
3. `fav/src/driver.rs` 内の旧 `cargo_toml_version` テスト（`"92.0.0"` 参照）を `"93.0.0"` に一括更新する
4. `CHANGELOG.md` に v93.0.0 エントリを追加する
5. `MILESTONE.md` に SAP QueryBuilder 1.0 宣言を追加する
6. `README.md` に QueryBuilder に関する言及を追加する
7. `versions/current.md` を v93.0.0 に更新する
8. `fav/src/driver.rs` に `mod v93000_tests`（4 件）を追加する

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | バージョンを `93.0.0` に更新 |
| `fav/src/driver.rs` | 旧 `"92.0.0"` 参照を `"93.0.0"` に一括更新 + `mod v93000_tests` 追加 |
| `CHANGELOG.md` | v93.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | SAP QueryBuilder 1.0 宣言セクションを追加 |
| `README.md` | QueryBuilder 言及を追加 |
| `versions/current.md` | v93.0.0 に更新 |

---

## Success Criteria

- `cargo test` 全 pass: **4,120 tests, 0 failures**（4,116 + 4）
- `mod v93000_tests` 内の 4 テストが pass する:
  - `cargo_toml_version_is_93_0_0`: `Cargo.toml` に `"93.0.0"` が含まれる
  - `changelog_has_v93_0_0`: `CHANGELOG.md` に `v93.0.0` が含まれる
  - `milestone_has_sap_query_builder`: `MILESTONE.md` に `SAP QueryBuilder` が含まれる
  - `readme_mentions_query_builder`: `README.md` に `QueryBuilder` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4,107（4,103+4）だが、v92.9.0 の実測が 4,116 のため、本バージョンは 4,116 + 4 = **4,120** が目標。

> **`cargo clean` 必須**: 宣言バージョンでは毎回 `cargo clean` を実施する（target/ ビルドアーティファクトの蓄積を防ぐ）。`cargo clean` 後は `fav/tmp/hello.fav` が消えないことを確認する（`target/` 外のため影響なし）。

> **一括更新**: driver.rs 内の `"92.0.0"` 文字列は `sed` で一括更新する。v92.0.0 時の実績は 88 箇所。
