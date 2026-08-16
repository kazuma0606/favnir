# v76.1.0 タスクリスト — `DataSource` / `ProvenanceTag` 型基盤

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.0.0` であることを確認
- [x] `cargo test` が全 pass（3714 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.1.0: DataSource / ProvenanceTag 型基盤 ---` コメントを追加する
- [x] `DataSourceType` enum を追加する（Snowflake / S3 / Api / Manual / Pipeline）
- [x] `DataSource` 構造体を追加する（name: String, uri: String, source_type: DataSourceType）
- [x] `ProvenanceTag` 構造体を追加する（source: DataSource, transforms: Vec<String>, pii: bool）
- [x] `format_provenance_tag(tag: &ProvenanceTag) -> String` を追加する
  - `source=<name> type=<DataSourceType:?> transforms=[t1,t2,...] pii=<true|false>` フォーマット
  - `transforms` が空の場合は `transforms=[]`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.1.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 2 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v761000_tests` モジュールを追加する（`use super::*` 必須 — `DataSource`・`ProvenanceTag` 等の同ファイル内型を参照するため、v76.0.0 の `include_str!` のみのテストとは異なる）
- [x] `provenance_tag_created` テストを実装する
  - `DataSource { name: "crm", source_type: Snowflake }` + transforms 2 件 + pii=false
  - `format_provenance_tag` が "crm"、"Snowflake"、"mask_pii"、"pii=false" を含む
- [x] `provenance_pii_flagged` テストを実装する
  - pii=true・S3 ソース → "pii=true"、"S3"、"transforms=[]" を含む
  - pii=false・Api ソース → "pii=false" を含む
- [x] `cargo test v761000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.0.0"` → `"76.1.0"` に変更する
- [x] `driver.rs` 内に `76.0.0` をアサートしているテスト（`cargo_toml_version_is_76_0_0` 等）が存在する場合は replace_all で `76.1.0` に更新する（存在しない場合はスキップ）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.1.0 に更新する
- [x] 「次に切る版」を v76.2.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3716 tests）
- [x] `cargo test v761000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.1.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.1.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `provenance_tag_created` が pass
- [x] `provenance_pii_flagged` が pass
- [x] テスト総数: 3716（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] CHANGELOG テスト（`changelog_has_v76_1_0`）: 宣言バージョン限定のため対象外
