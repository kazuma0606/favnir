# v76.4.0 タスクリスト — OpenLineage 統合強化

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.3.0` であることを確認
- [x] `cargo test` が全 pass（3720 tests）であることを確認（v76.4.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.4.0: OpenLineage 統合強化 ---` コメントを追加する
- [x] `OpenLineageFacet` 構造体を追加する（fields: producer, data_source_uri, transforms）
- [x] `provenance_to_openlineage(tag: &ProvenanceTag) -> OpenLineageFacet` を追加する
  - `producer`: `"favnir/v76"` 固定
  - `data_source_uri`: `tag.source.uri.clone()`
  - `transforms`: `tag.transforms.clone()`
- [x] `format_openlineage_json(facet: &OpenLineageFacet) -> String` を追加する
  - 手書き JSON フォーマット（serde_json 不使用）
  - 空 transforms → `[]`、非空 → `["t1","t2"]` 形式
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.4.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v764000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `openlineage_facet_from_provenance` テストを実装する
  - `provenance_to_openlineage` で producer・URI・transforms を検証
- [x] `openlineage_json_format` テストを実装する
  - `_producer`・`dataSource`・`uri`・transforms 要素を検証
  - 空 transforms で `"transforms":[]` を検証
- [x] `cargo test v764000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.3.0"` → `"76.4.0"` に変更する
- [x] `driver.rs` 内の `76.3.0` バージョン文字列アサーションを `76.4.0` に一括更新

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.4.0 に更新する
- [x] 「次に切る版」を v76.5.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3722 tests）
- [x] `cargo test v764000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.4.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.4.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `openlineage_facet_from_provenance` が pass
- [x] `openlineage_json_format` が pass
- [x] テスト総数: 3722（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_4_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.4.0 エントリ追加自体は T2 で必須
