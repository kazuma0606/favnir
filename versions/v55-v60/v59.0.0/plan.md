# v59.0.0 Plan — Governance & Deployment 2.0 宣言 ★クリーンアップ

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.9.0"` → `"59.0.0"` に変更。

### Step 2: MILESTONE.md 更新

`MILESTONE.md` の先頭（現在の `## v58.0.0` エントリの前）に v59.0.0 エントリを挿入。

**含めるべき内容（テスト検証あり）:**
- `"Governance & Deployment 2.0"` という文字列（必須）
- v59.0 の宣言文（ロードマップ記載の引用文をそのまま使用）:

      「パイプラインは Blue/Green で無停止デプロイされ、
       カナリアは段階的にトラフィックを引き受ける。
       スキーマはバージョン管理され、データはカタログで検索できる。
       ポリシーはコードで記述され、コンプライアンスは自動で証明される。
       Favnir のパイプラインは運用チームに信頼される。

       これが Favnir v59.0 — Governance & Deployment 2.0 の姿である。」

- v58.1〜v58.9 の達成内容:
  - v58.1（Blue/Green デプロイ）
  - v58.2（カナリアデプロイ）
  - v58.3（Schema Migration）
  - v58.4（Data Catalog）
  - v58.5（Policy-as-Code・E0426）
  - v58.6（マルチ環境設定）
  - v58.7（HA / DR）
  - v58.8（ドキュメントサイト記事）
  - v58.9（安定化・コードフリーズ）

**`"Governance & Deployment 2.0"` の文字列が必ず含まれることを確認すること。**

### Step 3: README.md 更新

README.md に `"Governance & Deployment 2.0"` を含む言及を追加。
既存のマイルストーン進捗テーブル（v36.0〜v40.0 以降の行）に v59.0.0 行を追記する。

**含めるべき内容（テスト検証あり）:**
- `"Governance & Deployment 2.0"` という文字列（必須）

### Step 4: CHANGELOG.md 更新（テストモジュール追加の前に実施）

CHANGELOG.md に v59.0.0 エントリを追加。**`"v59.0.0"` という文字列を必ず含めること**（`changelog_has_v59_0_0` テストが参照）。

---

### Step 5: driver.rs テストモジュール追加

**注意: Step 2〜4（MILESTONE.md・README.md・CHANGELOG.md への追記）をすべて先に行うこと。`include_str!` はコンパイル時に解決されるため、参照先の文字列が存在しないとテストが失敗する。**

`v59000_tests` を `v58900_tests` の直前に挿入:

    // -- v59000_tests (v59.0.0) -- Governance & Deployment 2.0 宣言 --
    #[cfg(test)]
    mod v59000_tests {
        #[test]
        fn cargo_toml_version_is_59_0_0() {
            // rolling check: function name is frozen at v59.0.0 by convention,
            // but this assertion is updated each release to the current version.
            let cargo_toml = include_str!("../Cargo.toml");
            assert!(
                cargo_toml.contains("version = \"59.0.0\""),
                "Cargo.toml version should be 59.0.0, got: {}",
                cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
            );
        }

        #[test]
        fn changelog_has_v59_0_0() {
            let changelog = include_str!("../../CHANGELOG.md");
            assert!(
                changelog.contains("v59.0.0"),
                "CHANGELOG.md should have a v59.0.0 entry"
            );
        }

        #[test]
        fn milestone_has_governance_deployment2() {
            let milestone = include_str!("../../MILESTONE.md");
            assert!(
                milestone.contains("Governance & Deployment 2.0"),
                "MILESTONE.md should contain Governance & Deployment 2.0 declaration"
            );
        }

        #[test]
        fn readme_mentions_governance_deployment2() {
            let readme = include_str!("../../README.md");
            assert!(
                readme.contains("Governance & Deployment 2.0"),
                "README.md should mention Governance & Deployment 2.0"
            );
        }
    }

**`use super::*` は不要（`include_str!` のみ使用）。**

### Step 6: driver.rs ローリングチェック更新

既存ローリングチェック 5 件（`version = "58.9.0"`）を更新:

- `version = \"58.9.0\"` → `version = \"59.0.0\"`（5 件、`replace_all`）
- failure メッセージも **5 件すべて**個別に更新:
  - `"Cargo.toml version should be 58.9.0, got: {}"` → `"59.0.0"`（`cargo_toml_version_is_58_0_0` 用）
  - `"Cargo.toml version should be 58.9.0, got: {}"` → `"59.0.0"`（`cargo_toml_version_is_57_9_0` 用）
  - `"Cargo.toml version should be 58.9.0 (rolling check from v57.0.0), got: {}"` → `"59.0.0 (rolling check from v57.0.0)"`
  - `"Cargo.toml version should be 58.9.0 (rolling check from v56.9.0), got: {}"` → `"59.0.0 (rolling check from v56.9.0)"`
  - `"Cargo.toml version should be 58.9.0, got: {}"` → `"59.0.0"`（`cargo_toml_version_is_56_3_0` 用）

**更新後のローリングチェック総数: 6 件**（既存 5 + v59000_tests の `cargo_toml_version_is_59_0_0` 1 件）

### Step 7: テスト実行

`cargo test -j 8 -- --test-threads=8` を実行し、3308 tests passed, 0 failed を確認。

### Step 8: ★クリーンアップ（テスト全通過後）

`cargo clean` を実行してビルドキャッシュを初期化する。

---

## 注意点

- main.rs の変更なし（宣言専用バージョン）
- CHANGELOG.md への v59.0.0 追記は Step 4（テストモジュール追加）の **前** に行うこと
- `cargo clean` はテストが全通過してから実施する（Step 8）
