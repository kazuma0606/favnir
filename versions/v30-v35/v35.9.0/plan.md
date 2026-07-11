# v35.9.0 実装計画 — v36.0 前調整・安定化

## 前提確認

- `v35900_tests` モジュールは driver.rs に**存在しない**（今回新規作成）
- CHANGELOG に `[v35.9.0]` エントリは**存在しない**（今回新規作成）
- `v35800_tests::cargo_toml_version_is_35_8_0` は現在ライブアサーション → バンプ前にスタブ化が必須
- `examples/lambda-deploy/fav.toml`、`site/content/docs/deploy/lambda.mdx` は存在確認済み

## 実装ステップ

### Step 1: CHANGELOG.md に [v35.9.0] エントリを追加

`## [v35.8.0]` の直前に追加する：

```markdown
## [v35.9.0] — 2026-07-07

### Changed
- 安定化スプリント — v35.1〜v35.8 の機能統合確認・v36.0 前調整

### Verified
- `!Effect` 廃止完結（v35.4〜v35.8）: lsp/completion.rs・docs_server.rs・mcp/mod.rs・error_catalog.rs すべてクリーン
- `examples/lambda-deploy/` — Lambda デプロイデモ（v36.0 前提条件）確認済み
- `site/content/docs/deploy/lambda.mdx` — Lambda デプロイドキュメント確認済み
- `versions/roadmap/roadmap-v35.1-v36.0.md` — Deployment Story 計画確認済み

### Added
- `fav/src/driver.rs` — `v35900_tests`（5 件）追加

---
```

### Step 2: v35800_tests::cargo_toml_version_is_35_8_0 をスタブ化

**ファイル**: `fav/src/driver.rs`

```rust
// before:
fn cargo_toml_version_is_35_8_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.8.0"), "Cargo.toml must contain version 35.8.0");
}

// after:
fn cargo_toml_version_is_35_8_0() {
    // stubbed: version bumped to 35.9.0
}
```

### Step 3: v35900_tests モジュールを driver.rs に追加

**ファイル**: `fav/src/driver.rs`（`v35700_tests` の閉じ `}` = line 42309 の後、ファイル末尾に追加）
> 注: driver.rs の既存モジュール順は `v35600_tests` → `v35800_tests` → `v35700_tests` と非標準。`v35900_tests` はファイル末尾に追加する。

```rust
// ── v35900_tests (v35.9.0) — v36.0 前調整・安定化 ──────────────────────────
#[cfg(test)]
mod v35900_tests {
    #[test]
    fn cargo_toml_version_is_35_9_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("35.9.0"), "Cargo.toml must contain version 35.9.0");
    }

    #[test]
    fn changelog_has_v35_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v35.9.0]"), "CHANGELOG.md must contain [v35.9.0]");
    }

    #[test]
    fn lambda_deploy_example_exists() {
        // v36.0 前提条件: examples/lambda-deploy/ が存在し Lambda 設定を含む
        let manifest = include_str!("../../examples/lambda-deploy/fav.toml");
        assert!(manifest.contains("lambda"), "examples/lambda-deploy/fav.toml must reference lambda");
    }

    #[test]
    fn deploy_docs_exists() {
        // v36.0 前提条件: Lambda デプロイドキュメントが存在する
        let mdx = include_str!("../../site/content/docs/deploy/lambda.mdx");
        assert!(mdx.contains("lambda"), "site/content/docs/deploy/lambda.mdx must reference lambda");
    }

    #[test]
    fn v36_deployment_story_planned() {
        // v36.0 ロードマップが Deployment Story を計画していることを確認
        let roadmap = include_str!("../../versions/roadmap/roadmap-v35.1-v36.0.md");
        assert!(
            roadmap.contains("Deployment Story"),
            "roadmap-v35.1-v36.0.md must contain Deployment Story"
        );
    }
}
```

### Step 4: Cargo.toml バージョンを 35.9.0 に更新

**ファイル**: `fav/Cargo.toml`

```toml
# before:
version = "35.8.0"

# after:
version = "35.9.0"
```

Step 2 完了後（v35800 スタブ化後）に実施すること。

### Step 5: cargo test 全通過を確認

```
cargo test 2>&1 | grep "test result"
```

期待: `test result: ok. N passed; 0 failed` （N ≥ 2646）

### Step 6: v35900_tests 5 件が pass することを確認

```
cargo test v35900 2>&1 | grep "test result"
```

### Step 7: ドキュメント更新

- `versions/v30-v35/v35.9.0/tasks.md` を COMPLETE ステータスに更新
- `versions/current.md` を v35.9.0（最新安定版）・v36.0.0（次バージョン）に更新

差分例:
```
# 最新安定版
- before: **v35.8.0** — !Effect 廃止完結（LSP / エラーカタログ / MCP / help）
+ after:  **v35.9.0** — v36.0 前調整・安定化

# 次に切る版
- before: **v35.9.0** — v36.0 前調整・安定化
+ after:  **v36.0.0** — Deployment Story マイルストーン宣言
```
