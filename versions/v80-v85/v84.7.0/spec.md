# spec: v84.7.0 — OSS 公開強化・コミュニティ整備 v2

## Background

> **テスト数注記**: ロードマップ計画値は 3,909/3,911 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,921 tests**（v84.6.0 完了時点）。
> v84.7.0 完了目標は **3,923 tests**（+2）。

v84.6.0 で Favnir 4.0 の 4 スプリント全機能ドキュメントを完成させた。v84.7.0 では
Quality-First 機能（QualityRule / IoContract）に対応した OSS コントリビュートガイドと
コミュニティ整備を実施する。既存の `CONTRIBUTING.md`・`SECURITY.md`・`CODE_OF_CONDUCT.md`
を v4 対応に更新し、品質フィードバック用 Issue テンプレートを新規追加する。

## Goals

1. `CONTRIBUTING.md` を v4 対応に更新する
   - `QualityRule` 追加手順（`test_framework.rs` に型を追加し `driver.rs` でテスト）
   - `IoContract` 追加手順（`test_framework.rs` に型を追加）
2. `.github/ISSUE_TEMPLATE/quality-feedback.md` を新規作成する
   - 品質フィードバック（QualityRule / QualityGate / AnomalyDetector の問題報告）専用テンプレート
3. `SECURITY.md` を最新に更新する（バージョン表記を v4 対応に）
4. `CODE_OF_CONDUCT.md` を確認・最新化する（内容に変更がなければ確認のみ）
5. Rust テスト 2 件で OSS ファイルの内容を検証する
   - `oss_contributing_v4_exists` — `CONTRIBUTING.md` に `QualityRule` が含まれること
   - `oss_issue_template_quality_exists` — `.github/ISSUE_TEMPLATE/quality-feedback.md` が存在すること

## Files 構成

### `CONTRIBUTING.md` への追加（v4 対応セクション）

既存の `CONTRIBUTING.md` 末尾に以下のセクションを追加する：

```markdown
## Favnir 4.0 機能の追加手順

### QualityRule を追加する

1. `fav/src/test_framework.rs` に `QualityRule` 構造体を追加する
2. `fav/src/driver.rs` に `#[cfg(test)]` テストモジュールを追加して動作確認する
3. `site/content/docs/v4/data-quality.mdx` にドキュメントを追加する

### IoContract を追加する

1. `fav/src/test_framework.rs` に `IoContract` 構造体を追加する
2. `infra/e2e-demo/favnir4-showcase/pipeline.fav` にショーケース用コードを追加する
3. `site/content/docs/v4/pipeline-contracts.mdx` にドキュメントを追加する
```

### `.github/ISSUE_TEMPLATE/quality-feedback.md`

```markdown
---
name: Quality Feedback
about: QualityRule / QualityGate / AnomalyDetector に関するフィードバック
title: "[Quality] "
labels: quality, feedback
assignees: ""
---

## フィードバック種別

- [ ] QualityRule の誤動作
- [ ] QualityGate の閾値問題
- [ ] AnomalyDetector の検知精度
- [ ] その他

## 再現手順

```favnir
-- 問題が発生するコードを貼り付けてください
```

## 期待する動作

## 実際の動作

## 環境

- Favnir バージョン:
- OS:
```

## Rust テスト（v84700_tests）

```rust
#[cfg(test)]
mod v84700_tests {
    #[test]
    fn oss_contributing_v4_exists() {
        let content = include_str!("../../CONTRIBUTING.md");
        assert!(content.contains("QualityRule"), "CONTRIBUTING.md should mention QualityRule for v4");
    }

    #[test]
    fn oss_issue_template_quality_exists() {
        assert!(
            std::path::Path::new("../.github/ISSUE_TEMPLATE/quality-feedback.md").exists(),
            ".github/ISSUE_TEMPLATE/quality-feedback.md should exist"
        );
    }
}
```

**パス起点:**
- `include_str!("../../CONTRIBUTING.md")` — `fav/src/` 起点 → `favnir/CONTRIBUTING.md`
- `Path::new("../.github/ISSUE_TEMPLATE/quality-feedback.md")` — `fav/` 起点 → `favnir/.github/...`

## Success Criteria

- `CONTRIBUTING.md` に `QualityRule` および `IoContract` の追加手順が含まれること
- `.github/ISSUE_TEMPLATE/quality-feedback.md` が存在すること
- `SECURITY.md` のバージョン表記が v4（v84.x）対応であること
- `cargo test` が 3,923 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはドキュメント・テンプレート更新のみ）

## Files to Modify / Create

### 更新
- `CONTRIBUTING.md` — v4 対応セクション追加
- `SECURITY.md` — バージョン表記を v4 対応に更新

### 新規作成
- `.github/ISSUE_TEMPLATE/quality-feedback.md`

### 追記
- `fav/src/driver.rs` — `v84700_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.7.0 エントリ追加

### パス起点

- `oss_contributing_v4_exists`: `include_str!("../../CONTRIBUTING.md")`（`fav/src/` 起点）
- `oss_issue_template_quality_exists`: `Path::new("../.github/ISSUE_TEMPLATE/quality-feedback.md")`（`fav/` 起点）
