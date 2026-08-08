# v69.9.0 実装計画

## 前提

- ベース: 3553 tests（v69.8.0 完了後）
- 目標: 3555 tests（+2）
- 変更するファイルはドキュメント・driver.rs のみ（Rust ソース変更なし）

---

## 実装ステップ

### Step 1: `driver.rs` — `v69900_tests` モジュール追加

`v69800_tests` の直前に `v69900_tests` モジュールを挿入する（降順ルール: v69900 → v69800 → v69700 → ...）。

```rust
#[cfg(test)]
mod v69900_tests {
    // use super::*; は不要（include_str! のみ使用。他バージョン同様）
    #[test]
    fn code_freeze_v699_v70_roadmap_has_milestone_declaration() {
        let src = include_str!("../../versions/roadmap/roadmap-v69.1-v70.0.md");
        assert!(
            src.contains("Intelligent ETL 1.0 宣言"),
            "roadmap should declare Intelligent ETL 1.0"
        );
        assert!(
            src.contains("3559"),
            "roadmap should document v70.0 test target 3559"
        );
    }

    #[test]
    fn code_freeze_v699_playground_etl_samples_complete() {
        let src = include_str!("../../site/content/playground/etl-samples.mdx");
        assert!(
            src.contains("schema Order"),
            "etl-samples.mdx should contain schema Order"
        );
        assert!(
            src.contains("bind"),
            "etl-samples.mdx should contain bind syntax"
        );
    }
}
```

### Step 2: `roadmap-v69.1-v70.0.md` — テスト数推移テーブル更新

v69.9.0 行を追加し、テスト数 3555 (+2) を確定する。
状態列を「完了 ✓」にする。

### Step 3: `versions/current.md` — 進行中バージョン更新

`v69.8.0` → `v69.9.0` に更新する。

---

## 依存関係

Step 1 は独立して実施可能。
Step 2, 3 は Step 1 のテスト通過後（cargo test 3555 確認後）に実施。
