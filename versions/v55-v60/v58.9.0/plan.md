# v58.9.0 Plan — 安定化・コードフリーズ（Governance & Deployment 2.0 前調整）

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.8.0"` → `"58.9.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v58.1-v59.0.md` に以下を行う:
- v59.0.0 の `ベース 3299 + 4 = 3303` → `ベース 3304 + 4 = 3308` に修正
- v59.0.0 の `テスト数 ≥ 3303` → `テスト数 ≥ 3308` の記述も更新する（行 266 相当）

（v58.9.0 の実績欄は T6 テスト確認後に記入）

### Step 3: governance-overview.mdx 作成

`site/content/docs/governance-overview.mdx` を新規作成。

**frontmatter:**

    ---
    title: Governance & Deployment — Overview
    ---

**本文に含めるべき内容（すべて必須）:**
- `# Governance & Deployment Overview` 見出し
- `"Governance & Deployment"` という文字列（タイトル・見出しで自動充足）
- v58.1〜v58.8 の機能一覧（Blue/Green・カナリア・HA・Schema Migration・Data Catalog・Policy-as-Code・マルチ環境設定）
- 各機能ページへの内部リンク（`/docs/enterprise/deployment`・`/docs/enterprise/governance` 等）

**`"Governance & Deployment"` の文字列が必ず含まれていることを確認すること。**

### Step 4: driver.rs テストモジュール追加

**注意: Step 3（MDX 作成）を必ず先に行うこと。`include_str!` はコンパイル時に解決されるため、MDX ファイルが存在しないとビルドエラーになる。**

`v58900_tests` を `v58800_tests` の直前に挿入:

    // -- v58900_tests (v58.9.0) -- 安定化 --
    #[cfg(test)]
    mod v58900_tests {
        #[test]
        fn cargo_toml_version_is_58_9_0() {
            let cargo_toml = include_str!("../Cargo.toml");
            assert!(
                cargo_toml.contains("version = \"58.9.0\""),
                "Cargo.toml version should be 58.9.0, got: {}",
                cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
            );
        }

        #[test]
        fn governance_overview_exists() {
            let content = include_str!("../../site/content/docs/governance-overview.mdx");
            assert!(
                content.contains("Governance & Deployment"),
                "governance-overview.mdx should contain 'Governance & Deployment'"
            );
        }
    }

**`use super::*` は不要（`include_str!` のみ使用）。**

### Step 5: driver.rs ローリングチェック更新

- `version = \"58.8.0\"` → `version = \"58.9.0\"`（5 件、`replace_all`）
- failure メッセージも **5 件すべて**個別に更新（全件を列挙）:
  - `"Cargo.toml version should be 58.8.0, got: {}"` → `"58.9.0, got: {}"`（`cargo_toml_version_is_58_0_0` 用）
  - `"Cargo.toml version should be 58.8.0, got: {}"` → `"58.9.0, got: {}"`（`cargo_toml_version_is_57_9_0` 用）
  - `"Cargo.toml version should be 58.8.0 (rolling check from v57.0.0), got: {}"` → `"58.9.0 (rolling check from v57.0.0), got: {}"`
  - `"Cargo.toml version should be 58.8.0 (rolling check from v56.9.0), got: {}"` → `"58.9.0 (rolling check from v56.9.0), got: {}"`
  - `"Cargo.toml version should be 58.8.0, got: {}"` → `"58.9.0, got: {}"`（`cargo_toml_version_is_56_3_0` 用）

---

## 注意点

- main.rs の変更なし（安定化専用バージョン）
- v58900_tests の `cargo_toml_version_is_58_9_0` は **ローリングチェックではなく**、v58.9.0 固有のスナップショットテスト
- v59.0.0 のカスケード修正を Step 2 で実施する（ベース 3299→3304、目標 3303→3308）
