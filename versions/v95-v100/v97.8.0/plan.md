# Plan: v97.8.0 — サイトドキュメント（Workflow / Approval パターンガイド）

## 実装ステップ

### Step 1: `site/content/docs/guides/sap-workflow.mdx` 新規作成

既存の `sap-integration.mdx`（order: 10）の次の order: 11 として作成する。

フロントマター:
```yaml
---
title: "SAP Workflow & Approval Guide"
order: 11
category: "Guide"
description: "..."
---
```

セクション構成:
1. **全体像** — v97.1〜v97.7 の機能一覧テーブル（バージョン / 型 / 概要）
2. **承認フローの型設計** — `TaskDecision` ADT / `ApprovalClient` interface 説明
3. **`!Approval` エフェクト型の使い方** — pipeline シグネチャの意味 / `route_by_approval_result` 解説
4. **iFlow connector の設定** — `IFlowClient` / `IFlowMessage` / `iflow_send` の使い方
5. **E2E デモのウォークスルー** — `workflow_demo/` ディレクトリ構成 / `bash run.sh` 手順
6. **テスト戦略** — `MockWorkflowClient` / `Ctx.mock_workflow` の使い方

### Step 2: `fav/src/driver.rs` に `mod v97800_tests` を追加

`mod v97700_tests` の直後に追加（`std::fs::read_to_string` パターン）。

パス根拠: `cargo test` 実行時の cwd は `fav/`。`../site/` は `favnir/site/` を指す。
（既存の `"../runes/..."` / `"../infra/..."` パターンと同じ相対パス規約）

```rust
#[cfg(test)]
mod v97800_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_workflow_mdx_exists() {
        let _ = std::fs::read_to_string(
            "../site/content/docs/guides/sap-workflow.mdx",
        )
        .expect("site/content/docs/guides/sap-workflow.mdx should exist (v97.8.0)");
    }
    #[test]
    fn sap_workflow_mdx_has_approval_client() {
        let content = std::fs::read_to_string(
            "../site/content/docs/guides/sap-workflow.mdx",
        )
        .expect("site/content/docs/guides/sap-workflow.mdx should exist");
        assert!(
            content.contains("ApprovalClient"),
            "sap-workflow.mdx should document ApprovalClient"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

テスト数: 4,227 + 2 = 4,229

### Step 4: `CHANGELOG.md` に v97.8.0 エントリを追加

先頭に追加。

### Step 5: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v97.8.0` に更新
- 最新安定版を `v97.8.0 — 4,229 tests` に更新

### Step 6: CI 事前確認（Clippy / Self-fmt）

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
