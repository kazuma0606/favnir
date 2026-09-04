# Spec: v94.6.0 — OSS 整備（SAP コミュニティ向けドキュメント）

## Background

v94.5.0 で `fav bench --sap` による SAP Advanced Benchmark Suite を完成させた。
v94.6.0 では SAP Integration に関する OSS コミュニティ向けドキュメントを整備し、
外部コントリビューターが SAP Rune を試験・利用しやすい環境を構築する。

対象ドキュメントは以下の 3 点:
1. `runes/sap-odata/README.md`（新規）— SAP OData Rune の概要・使い方・セットアップ手順
2. `CONTRIBUTING.md`（追記）— SAP テスト環境セットアップ手順
3. `.github/ISSUE_TEMPLATE/sap-bug.md`（新規）— SAP 向けバグ報告テンプレート

## Goals

1. `runes/sap-odata/README.md` を新規作成する
   - SAP OData Rune の概要・使い方・設定方法（Setup セクション必須）
2. `CONTRIBUTING.md` に SAP テスト環境セットアップ手順を追加する
3. `.github/ISSUE_TEMPLATE/sap-bug.md` を新規作成する
4. `driver.rs` に `mod v94600_tests` を追加する（2 件）

## Syntax/API Examples

```markdown
<!-- runes/sap-odata/README.md（概要） -->
# SAP OData Rune

Favnir の SAP S/4HANA OData 統合 Rune です。

## Setup

fav.toml に以下を追加してください:

[sap]
base_url = "https://your-sap-host.example.com/sap/opu/odata/sap/"
client_id = "100"
username = "${SAP_USER}"
password = "${SAP_PASS}"

## Usage

...
```

```markdown
<!-- .github/ISSUE_TEMPLATE/sap-bug.md -->
---
name: SAP OData Bug Report
about: SAP OData Rune に関するバグ報告
labels: bug, sap-odata
---

**環境**
- Favnir バージョン:
- SAP S/4HANA バージョン:
...
```

## Success Criteria

- `runes/sap-odata/README.md` が存在する
- `runes/sap-odata/README.md` に `Setup` または `setup` が含まれる
- `driver.rs` の `mod v94600_tests` が pass する
  - `sap_odata_rune_readme_exists`: `../runes/sap-odata/README.md` が存在する
  - `sap_odata_rune_readme_has_setup`: README に `Setup` が含まれる
- `cargo test 2>&1 | grep "test result"` が 4,154 tests, 0 failures を示す（着手前: 4,152）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/README.md` | **新規作成** | SAP OData Rune の概要・使い方・Setup 手順 |
| `CONTRIBUTING.md` | **追記** | SAP テスト環境セットアップ手順 |
| `.github/ISSUE_TEMPLATE/sap-bug.md` | **新規作成** | SAP バグ報告テンプレート |
| `fav/src/driver.rs` | **追加** | `mod v94600_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.6.0 エントリ |
