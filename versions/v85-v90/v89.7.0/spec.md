# Spec: v89.7.0 — OSS 整備

## Background

v89.6.0 で sap-odata Rune の公式ドキュメントが完成した。
本バージョンでは外部コントリビューターが SAP Rune に新エンティティを追加できるよう
`CONTRIBUTING.md` への手順追記と GitHub Issue テンプレートを整備する。

### 現行状態確認

| ファイル | 状態 |
|---|---|
| `CONTRIBUTING.md` | 存在するが SAP Rune 追加手順のセクションなし |
| `.github/ISSUE_TEMPLATE/bug_report.md` | 存在 |
| `.github/ISSUE_TEMPLATE/feature_request.md` | 存在 |
| `.github/ISSUE_TEMPLATE/quality-feedback.md` | 存在 |
| `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` | **本バージョンで追加** |

## Goals

1. `CONTRIBUTING.md` に SAP Rune エンティティ追加手順セクションを追記する
   - 新エンティティの追加ステップ:
     1. 型定義（`runes/sap-odata/<entity>.fav`）
     2. 関数実装（スタブ → 実装）
     3. `sap_odata.fav` への re-export 追加
     4. `fav/src/driver.rs` テスト追加
     5. Rune Registry デプロイ（`/deploy-registry` コマンド）
2. `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` を作成する
3. `fav/src/driver.rs` に `mod v89700_tests` を追加する（2 件）

## Issue テンプレート仕様

```yaml
---
name: SAP Integration Feedback
about: SAP OData Rune（sap-odata）に関するフィードバック・不具合報告
title: "[SAP] "
labels: sap-integration, feedback
assignees: ""
---

## フィードバック種別

- [ ] エンティティ取得の誤動作
- [ ] 型定義の不一致
- [ ] 認証エラー
- [ ] 新エンティティのリクエスト
- [ ] その他

## 詳細

（詳細を記述してください）

## 再現手順

（再現手順があれば記述してください）

## 環境情報

- Favnir バージョン:
- SAP バージョン（S/4HANA Cloud / Business One / ECC）:
```

## Success Criteria（Rust テストで担保）

- `contributing_has_sap_section`:
  `CONTRIBUTING.md` に `"SAP Rune"` を含む
- `issue_template_sap_feedback_exists`:
  `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` が存在する
- `cargo test` で 4,033 tests, 0 failures（4,031 + 2）

## Files to Modify / Create

| ファイル | 変更種別 |
|---|---|
| `CONTRIBUTING.md` | 追記（SAP Rune エンティティ追加手順セクション） |
| `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` | 新規作成 |
| `fav/src/driver.rs` | `mod v89700_tests` 追加 |

**前提確認**:
- `CONTRIBUTING.md` は存在するが SAP 関連セクションなし — 末尾に追記
- `.github/ISSUE_TEMPLATE/` ディレクトリは存在（`bug_report.md` 等が存在）
- `quality-feedback.md` のフロントマター形式（key: name / about / title / labels / assignees）を参照パターンとして使用
  - セクション構成は SAP 向けに意図的に変更: 「詳細」「再現手順」「環境情報（SAP バージョン含む）」の 3 セクション構成（quality-feedback.md とは異なる）

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
