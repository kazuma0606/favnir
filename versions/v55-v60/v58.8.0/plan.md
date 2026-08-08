# v58.8.0 Plan — ドキュメントサイト Governance & Deployment 記事

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.7.0"` → `"58.8.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v58.1-v59.0.md` に以下を行う:
- v58.9.0 のベース数を `3297 → 3302`、目標を `3299 → 3304` に修正
- v59.0.0 の数値修正は v58.9.0 確定後に改めて行う（本バージョンでは v58.9.0 のみ修正）

### Step 3: deployment.mdx 作成

`site/content/docs/enterprise/deployment.mdx` を新規作成。

**frontmatter:**
    ---
    title: Enterprise Deployment — Blue/Green, Canary, HA
    ---

**本文に含めるべき内容（すべて必須）:**
- `# Enterprise Deployment` 見出し
- `Blue/Green` という文字列（テストで検証）
- `--strategy blue-green` の bash 例
- `canary` という文字列
- `--canary-weight` の bash 例
- `HA` / `--ha` の bash 例（`/healthz` 言及）

**`"Blue/Green"` の文字列が必ず含まれていることを確認すること。**

### Step 4: governance.mdx 作成

`site/content/docs/enterprise/governance.mdx` を新規作成。

**frontmatter:**
    ---
    title: Enterprise Governance — Schema, Catalog, Policy
    ---

**本文に含めるべき内容（すべて必須）:**
- `# Enterprise Governance` 見出し
- `Schema Migration` という文字列
- `fav schema migrate` の bash 例
- `Data Catalog` という文字列
- `fav catalog push` / `fav catalog search` の bash 例
- `Policy-as-Code` という文字列（テストで検証）
- `policy` ブロックの favnir コード例
- `fav policy check` の bash 例
- `E0426` という文字列（policy 違反エラーコードとして出力例に含める）

**`"Policy-as-Code"` と `"E0426"` の文字列が必ず含まれていることを確認すること。**

### Step 5: multi-env-pipeline.mdx 作成

`site/content/cookbook/multi-env-pipeline.mdx` を新規作成。

**frontmatter:**
    ---
    title: "Cookbook: Multi-Environment Pipeline"
    ---

**本文に含めるべき内容:**
- `# Multi-Environment Pipeline` 見出し
- `--env` という文字列
- `dev` / `staging` / `prod` 環境設定例（fav.toml の [env] セクション）
- `fav run pipeline.fav --env dev` 等の bash 例

**このファイルは Rust テストの対象外。** 人手確認のみ（ロードマップで `docs_multi_env_page_exists` は要件外）。

### Step 6: driver.rs テストモジュール追加

**注意: T3〜T5（MDX 作成）を必ず先に行うこと。`include_str!` はコンパイル時に解決されるため、MDX ファイルが存在しないとビルドエラーになる。**

`v58800_tests` を `v58700_tests` の直前に挿入:

    // -- v58800_tests (v58.8.0) -- ドキュメントサイト --
    #[cfg(test)]
    mod v58800_tests {
        #[test]
        fn docs_deployment_page_exists() {
            let content = include_str!("../../site/content/docs/enterprise/deployment.mdx");
            assert!(
                content.contains("Blue/Green"),
                "deployment.mdx should contain 'Blue/Green'"
            );
        }

        #[test]
        fn docs_governance_page_exists() {
            let content = include_str!("../../site/content/docs/enterprise/governance.mdx");
            assert!(
                content.contains("Policy-as-Code"),
                "governance.mdx should contain 'Policy-as-Code'"
            );
        }
    }

**`use super::*` は不要（`include_str!` のみ使用）。**

### Step 7: driver.rs ローリングチェック更新

- `version = \"58.7.0\"` → `version = \"58.8.0\"`（5 件、`replace_all`）
- failure メッセージも **5 件すべて**個別に更新（`replace_all` では他バージョン番号との混在リスクあり、目視確認を推奨）:
  - `"Cargo.toml version should be 58.7.0, got: {}"` → `"58.8.0"`（1 件）
  - `"Cargo.toml version should be 58.7.0 (rolling check from v57.0.0), got: {}"` → `"58.8.0"`（1 件）
  - `"Cargo.toml version should be 58.7.0 (rolling check from v56.9.0), got: {}"` → `"58.8.0"`（1 件）
  - その他のローリングメッセージ 2 件も同様に更新（合計 5 件）

---

## 注意点

- main.rs の変更なし（ドキュメント専用バージョン）
- v59.0.0 の目標数修正は v58.9.0 確定後に行う
