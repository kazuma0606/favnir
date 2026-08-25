# plan: v84.7.0 — OSS 公開強化・コミュニティ整備 v2

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,921 tests, 0 failures を確認する（前提: v84.6.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
- `fav/src/driver.rs` に `mod v84600_tests` が存在することを確認する

> 注: ロードマップ計画値は 3,909/3,911 だが、code-reviewer 対応の累積で実績ベースは 3,921/3,923。

### Step 2: `CONTRIBUTING.md` に v4 対応セクションを追加

既存 `CONTRIBUTING.md` の末尾に「Favnir 4.0 機能の追加手順」セクションを追加する。

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

### Step 3: `.github/ISSUE_TEMPLATE/quality-feedback.md` を新規作成

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

### Step 4: `SECURITY.md` のバージョン表記を v4 対応に更新

`SECURITY.md` の「サポートバージョン」テーブルを確認し、v84.x（v4）を追加する。

### Step 5: `CODE_OF_CONDUCT.md` を確認

内容の変更が不要であれば確認のみ。バージョン表記・連絡先に変更が必要な場合のみ更新する。

### Step 6: driver.rs に v84700_tests を追加

`mod v84600_tests` の直後に `#[cfg(test)] mod v84700_tests` を追加する。

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

### Step 7: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,923 tests, 0 failures を確認する。

### Step 8: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.7.0 エントリを追加する。

### Step 9: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
