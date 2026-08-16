# v77.0.0 仕様書 — Data Provenance 1.0 宣言 ★クリーンアップ

Date: 2026-08-15
Status: 計画中

---

## Background

v76.1.0〜v76.9.0 で実装した Data Provenance 1.0 スプリントの完成を宣言する。データの来歴が型となり、PII 追跡・GDPR 消去・OpenLineage 統合・リネージグラフ・コントラクト検証が Favnir の型システムとして確立された。★クリーンアップとして `cargo clean` を実施し、ビルド環境を初期化する。

**宣言文:**
> 「データの来歴が型となった。どこから来て、何を経て、PII がどこで消えたかを
>  Favnir が型で追跡する。GDPR はコンパイル時に通る。」

---

## Goals

1. `cargo clean` を実行してビルド成果物を削除する（★クリーンアップ）
2. `fav/tmp/hello.fav` を復元する（cargo clean で消えるため）
3. `MILESTONE.md` の先頭に v77.0.0 エントリを追加する
4. `README.md` の先頭に v77.0 — Data Provenance 1.0 宣言セクションを追加する
5. `CHANGELOG.md` の先頭に v77.0.0 エントリを追加する（テストより先）
6. `fav/Cargo.toml` のバージョンを `76.9.0` → `77.0.0` に更新する（テスト追加より先）
7. `driver.rs` 内の `76.9.0` バージョン文字列を `77.0.0` に一括更新する（replace_all）
8. `v77000_tests` モジュール（4件）を追加する — `use super::*` **不要**
9. `cargo test` が 3736 tests all pass であることを確認する

---

## テスト仕様

`v77000_tests` — `use super::*` **不要**（外部ファイル参照のみ）

```rust
#[cfg(test)]
mod v77000_tests {
    #[test]
    fn cargo_toml_version_is_77_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"77.0.0\""));
    }

    #[test]
    fn changelog_has_v77_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v77.0.0]"));
    }

    #[test]
    fn milestone_has_data_provenance() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Data Provenance"));
    }

    #[test]
    fn readme_mentions_provenance() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Provenance") || content.contains("provenance"));
    }
}
```

---

## MILESTONE.md 追加エントリ（先頭に挿入）

```markdown
## v77.0.0（2026-08-15）— Data Provenance 1.0 宣言

> 「データの来歴が型となった。どこから来て、何を経て、PII がどこで消えたかを
>  Favnir が型で追跡する。GDPR はコンパイル時に通る。」

**Data Provenance 1.0** の宣言バージョン。v76.1〜v76.9 で実装した
Data Provenance 基盤の完成を宣言した。

**v76.1〜v76.9 達成内容:**
- `DataSource` / `DataSourceType` / `ProvenanceTag` / `format_provenance_tag`（来歴型基盤）— v76.1.0
- `TracedData` / `map_traced` / `merge_provenance`（来歴付きデータ型）— v76.2.0
- `PiiProvenanceReport` / `detect_pii_in_tag` / `ErasurePlan` / `generate_erasure_plan`（PII・GDPR）— v76.3.0
- `OpenLineageFacet` / `provenance_to_openlineage` / `format_openlineage_json`（OpenLineage 統合）— v76.4.0
- `LineageNodeType` / `LineageNode` / `LineageEdge` / `LineageGraph` / `format_lineage_dot`（グラフ可視化）— v76.5.0
- `PipelineProvenanceChain` / `chain_provenance` / `format_chain_report`（Cross-pipeline）— v76.6.0
- `DataProductSla` / `ProvenancePolicy` / `DataProduct` / `validate_data_product`（Data product 型）— v76.7.0
- `PiiPolicy` / `ProvenanceContract` / `validate_provenance_contract`（Provenance contracts）— v76.8.0
- 安定化・E2E テスト（`provenance_full_sprint_all_stable` / `provenance_e2e_pipeline_valid`）— v76.9.0
```

---

## README.md 追加セクション（v76.0 セクションの直前に挿入）

```markdown
## v77.0 — Data Provenance 1.0 宣言（2026-08-15）

Favnir v77.0 で **Data Provenance 1.0** を宣言しました。
データの来歴が型となり、どこから来て、何を経て、PII がどこで消えたかを
Favnir が型で追跡します。
`ProvenanceTag` が来歴をファーストクラス型として表現し、
`validate_provenance_contract` が入力ソースと PII ポリシーをコンパイル時に検証します。
`format_openlineage_json` が OpenLineage 標準ファセットを生成し、
`format_lineage_dot` が Graphviz DOT 形式でデータフローを可視化します。
```

---

## Success Criteria

- `cargo clean` 後に `cargo test` が正常完了する
- `MILESTONE.md` の先頭が `## v77.0.0` から始まり `Data Provenance` を含む
- `README.md` が `Provenance` を含む
- `CHANGELOG.md` の先頭が `[v77.0.0]` である
- `Cargo.toml` のバージョンが `77.0.0` である
- `cargo_toml_version_is_77_0_0` が pass
- `changelog_has_v77_0_0` が pass
- `milestone_has_data_provenance` が pass
- `readme_mentions_provenance` が pass
- `cargo test` が 3736 tests all pass

---

## 変更ファイル

- `fav/src/driver.rs` — `v77000_tests` モジュール追加、`76.9.0` → `77.0.0` 一括更新
- `MILESTONE.md` — v77.0.0 エントリを先頭に追加
- `README.md` — v77.0 Data Provenance 1.0 宣言セクションを先頭付近に追加
- `CHANGELOG.md` — v77.0.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.9.0` → `77.0.0` に更新

---

## 依存（既実装・v76.1〜v76.9）

- `DataSource` / `ProvenanceTag` / `TracedData` / `OpenLineageFacet` / `LineageGraph`
- `PipelineProvenanceChain` / `DataProduct` / `ProvenanceContract` / `PiiPolicy`
- 全来歴関連関数（v76.1〜v76.9 完了済み）
