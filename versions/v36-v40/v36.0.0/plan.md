# v36.0.0 実装計画 — Deployment Story マイルストーン宣言

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.0.0]` エントリを追加（`## [v35.9.0]` の直前） |
| S2 | `MILESTONE.md` | Deployment Story 宣言セクションを追加 |
| S3 | `fav/src/driver.rs` | `v35900_tests::cargo_toml_version_is_35_9_0` をスタブ化 |
| S4 | `fav/src/driver.rs` | `v36000_tests` モジュール（5 件）を追加 |
| S5 | `fav/Cargo.toml` | バージョンを `35.9.0` → `36.0.0` に更新 |
| S6 | `cargo test` | 全通過確認（≥ 2656 件） |
| S7 | `cargo clean` | ★クリーンアップ（x.0.0 必須） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v35.9.0]` の直前に挿入（日付は実装当日の日付を記入すること）:

```markdown
## [v36.0.0] — 2026-07-07

### Milestone: Deployment Story

- `fav deploy --target lambda` で Lambda に自動デプロイ（bootstrap.zip パッケージング）
- `fav deploy --target docker` で Docker イメージ生成
- `fav ci init` で GitHub Actions CI ワークフロー自動生成
- `!Effect` 廃止（v35.4〜v35.8）により全 API が ctx: AppCtx ベースに統一
- ★ v35.1〜v35.9 スプリント完了、Deployment Story マイルストーン宣言

```

### S2: MILESTONE.md

末尾（または v35.0 Production Ready セクションの後）に追加:

```markdown
## v36.0 — Deployment Story（2026-07-07）

v35.1〜v35.9 スプリントで実装した機能を統合し、Deployment Story マイルストーンを宣言する。

- `fav deploy --target lambda` — Lambda 自動デプロイ・bootstrap.zip パッケージング（v35.1）
- `fav deploy --target docker` — Dockerfile 自動生成・docker build 実行（v35.2）
- `fav ci init` — GitHub Actions CI ワークフロー自動生成（v35.3）
- `!Effect` 廃止完結（v35.4〜v35.8）— すべての API が ctx: AppCtx ベースに統一
- E2E 動作確認・lambda-deploy デモ確認完了（v35.9）

これが Favnir v36.0 — Deployment Story の姿である。
```

### S3: driver.rs — v35900_tests スタブ化

対象行:
```rust
fn cargo_toml_version_is_35_9_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.9.0"), "Cargo.toml must contain version 35.9.0");
}
```

変更後:
```rust
fn cargo_toml_version_is_35_9_0() {
    // stubbed: version bumped to 36.0.0
}
```

### S4: driver.rs — v36000_tests モジュール追加

`v35900_tests` モジュールの閉じ `}` の後（= ファイル末尾）に追加:

```rust
// ── v36000_tests (v36.0.0) — Deployment Story マイルストーン宣言 ──────────────
#[cfg(test)]
mod v36000_tests {
    #[test]
    fn cargo_toml_version_is_36_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.0.0"), "Cargo.toml must contain version 36.0.0");
    }
    #[test]
    fn changelog_has_v36_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.0.0]"), "CHANGELOG.md must contain [v36.0.0]");
    }
    #[test]
    fn milestone_has_deployment_story() {
        let m = include_str!("../../MILESTONE.md");
        assert!(m.contains("Deployment Story"), "MILESTONE.md must contain Deployment Story");
    }
    #[test]
    fn deploy_lambda_fn_exists() {
        let src = include_str!("driver.rs");
        assert!(src.contains("pub fn cmd_deploy"), "driver.rs must contain pub fn cmd_deploy");
    }
    #[test]
    fn ci_init_yaml_fn_exists() {
        let src = include_str!("driver.rs");
        assert!(src.contains("generate_ci_yaml"), "driver.rs must contain generate_ci_yaml");
    }
}
```

### S5: Cargo.toml バージョン更新

**必ず S3（スタブ化）完了後に実行すること**（S3 より先に実行すると `cargo_toml_version_is_35_9_0` のライブアサーションが失敗する）。

`version = "35.9.0"` → `version = "36.0.0"`

### S6: cargo test

期待値: 2651（現在）+ 5（v36000_tests）= **2656 件** pass、0 failures

### S7: cargo clean

x.0.0 規約に従い `cargo clean` を実行（ビルドキャッシュ削除）。

## モジュール順序の注意

driver.rs の末尾は現在:
```
... v35800_tests ... (sprint batch で v35700 より前に定義)
... v35700_tests ... (sprint batch 末尾)
... v35900_tests ... (v35.9.0 で追加)
```

`v36000_tests` は `v35900_tests` の閉じ `}` の後に追加する（ファイル真末尾）。

## MILESTONE.md 確認

spec.md テスト `milestone_has_deployment_story` が `"Deployment Story"` を検索するため、
MILESTONE.md のセクション見出しに必ず `"Deployment Story"` を含めること。
