# v76.6.0 タスクリスト — Cross-pipeline provenance

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.5.0` であることを確認
- [x] `cargo test` が全 pass（3724 tests）であることを確認（v76.6.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.6.0: Cross-pipeline provenance ---` コメントを追加する
- [x] `PipelineProvenanceChain` 構造体を追加する（pipelines: Vec<String>, merged_tag: ProvenanceTag）
- [x] `chain_provenance(upstream: &ProvenanceTag, pipeline_name: &str) -> ProvenanceTag` を追加する
  - `source`: upstream.source.clone()
  - `transforms`: upstream.transforms + pipeline_name を末尾追加
  - `pii`: upstream.pii そのまま引き継ぐ
- [x] `format_chain_report(chain: &PipelineProvenanceChain) -> String` を追加する
  - 形式: `pipelines=[p1,p2,...] source=<name> pii=<bool>`
- [x] `cargo test` で既存 3724 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.6.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v766000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `cross_pipeline_provenance_chained` テストを実装する
  - `chain_provenance` で source・pii 引き継ぎと transforms への pipeline_name 追加を検証
  - `PipelineProvenanceChain` 構造体の構築を検証
- [x] `cross_pipeline_pii_propagated` テストを実装する
  - pii=true の upstream から chain_provenance → pii=true が引き継がれる
  - `format_chain_report` の出力に `pipelines=`・`source=crm`・`pii=true` が含まれる
- [x] `cargo test v766000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.5.0"` → `"76.6.0"` に変更する
- [x] `driver.rs` 内の `76.5.0` バージョン文字列アサーションを `76.6.0` に一括更新（`replace_all: true` で全件置換すること）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.6.0 に更新する
- [x] 「次に切る版」を v76.7.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3726 tests）
- [x] `cargo test v766000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.6.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.6.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `cross_pipeline_provenance_chained` が pass
- [x] `cross_pipeline_pii_propagated` が pass
- [x] テスト総数: 3726（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_6_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.6.0 エントリ追加自体は T2 で必須
