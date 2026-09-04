# Plan: v98.8.0 — サイトドキュメント（Analytics / KPI パターンガイド）

## 実装順序

### Step 1: sap-analytics.mdx を新規作成

`site/content/docs/guides/sap-analytics.mdx` を作成。

フロントマター:
```
---
title: "SAP Analytics Guide"
order: 12
category: "Guide"
description: "KPI 定義・BW/4HANA クエリ・SAC データプッシュの完全ガイド"
---
```

セクション構成:
1. 概要（KPI 監視 pipeline フロー図: Fetch → Evaluate → Push → Alert）
2. KPI 定義パターン（`KpiDefinition` / `KpiThreshold` / `KpiSnapshot` / `KpiStatus` 型説明 + コード例）
3. BW/4HANA クエリ（`ctx.sap.bw_query()` の使い方）
4. SAC データプッシュ（`SacDataset` / `sac_push_mock` 設定）
5. `fav report --sap` コマンドリファレンス（フラグ一覧テーブル）

コード例は Favnir 構文（`bind`、`--` コメント、`|>` stage）を使用する。

既存ガイド（`sap-integration.mdx`）の構成スタイルに従う。

---

### Step 2: driver.rs に mod v98800_tests を追加

`mod v98700_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98800_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_analytics_guide_exists() {
        let _ = std::fs::read_to_string(
            "../site/content/docs/guides/sap-analytics.mdx",
        )
        .expect("sap-analytics.mdx should exist (v98.8.0)");
    }

    #[test]
    fn sap_analytics_guide_has_kpi_definition() {
        let content = std::fs::read_to_string(
            "../site/content/docs/guides/sap-analytics.mdx",
        )
        .expect("sap-analytics.mdx should exist");
        assert!(
            content.contains("KpiDefinition"),
            "sap-analytics.mdx should document KpiDefinition (v98.8.0)"
        );
    }
}
```

---

### Step 3: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,251 tests, 0 failures

---

### Step 4: CHANGELOG.md に v98.8.0 エントリを追加

---

### Step 5: versions/current.md 更新

最新安定版を `v98.8.0` に更新（テスト数 4,251）。

---

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
