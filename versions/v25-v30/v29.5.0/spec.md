# v29.5.0 Spec — github Rune 追加

**バージョン**: 29.5.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 5)
**前バージョン**: v29.4.0 (vertex-ai / sagemaker Rune 追加)

---

## 概要

CI パイプラインから GitHub を操作できるようにする。
データ品質チェックの結果を PR にコメントする、データ異常を Issue として自動起票する等、
`stage` として GitHub 操作を組み込める。

> **ポジショニング**: AI/ML Rune 四部作完了後の次フェーズ。
> データパイプラインと DevOps の橋渡しとなる Rune。
> `fav run quality_check.fav` で品質レポートが PR に自動コメントされる。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `runes/github/github.fav` | GitHub Rune 実装（5 関数）|
| `runes/github/rune.toml` | Rune メタデータ |
| `fav/src/driver.rs` | `v295000_tests` 6 件追加 |
| `fav/Cargo.toml` | version 29.4.0 → 29.5.0 |
| `CHANGELOG.md` | `[v29.5.0]` セクション追加 |
| `benchmarks/v29.5.0.json` | ベンチマーク記録 |
| `site/content/docs/runes/github.mdx` | GitHub Rune ドキュメント |

---

## GitHub Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `GitHub.create_comment` | `(config: String, pr_number: String, body: String) -> Result<Unit, String> !Http` | PR コメント作成 |
| `GitHub.create_issue` | `(config: String, title: String, body: String, labels: List<String>) -> Result<String, String> !Http` | Issue 作成（Issue 番号を返す）|
| `GitHub.update_issue` | `(config: String, issue_number: String, state: String) -> Result<Unit, String> !Http` | Issue 更新（close 等）|
| `GitHub.list_prs` | `(config: String, state: String) -> Result<List<String>, String> !Http` | PR 一覧取得 |
| `GitHub.get_pr` | `(config: String, pr_number: String) -> Result<String, String> !Http` | PR 詳細取得 |

### 設定

| 環境変数 | 説明 |
|---|---|
| `GITHUB_TOKEN` | GitHub Personal Access Token または GitHub App トークン（必須）|
| `GITHUB_REPO` | リポジトリ（例: `owner/repo`）|
| `GITHUB_BASE_URL` | API ベース URL（デフォルト: `https://api.github.com`）|

### 使用例

```favnir
import runes/github

// データ品質チェックの結果を PR にコメント
stage PostQualityReport: String -> Unit !Http = |report| {
  bind pr_number <- Env.get("GITHUB_PR_NUMBER")
  GitHub.create_comment(config.github, pr_number, report)
}

// データ異常を Issue として自動起票
stage CreateDataAlert: String -> Unit !Http = |title| {
  bind _ <- GitHub.create_issue(
    config.github,
    "[DATA ALERT] " ++ title,
    "自動検知されたデータ異常です。",
    ["data-alert", "automated"]
  )
  Result.ok(unit)
}
```

---

## テスト戦略

### v295000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `github_rune_file_exists` | `runes/github/github.fav` が存在し `GitHub.create_comment` を含む |
| `github_create_issue_fn_exists` | `github.fav` に `create_issue` が存在する |
| `github_update_issue_fn_exists` | `github.fav` に `update_issue` が存在する |
| `github_list_prs_fn_exists` | `github.fav` に `list_prs` が存在する |
| `github_get_pr_fn_exists` | `github.fav` に `get_pr` が存在する |
| `changelog_has_v29_5_0` | `CHANGELOG.md` に `[v29.5.0]` が存在する |

検証関数カバレッジ: `create_comment`, `create_issue`, `update_issue`, `list_prs`, `get_pr`（5/5 関数 = 100%）

テスト数: 2336 → **2342**（+6）

---

## 完了条件

- [ ] `runes/github/github.fav` に 5 関数が実装されている
- [ ] `runes/github/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `cargo test --bin fav v295000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2342 tests PASS
- [ ] `CHANGELOG.md` に `[v29.5.0]` セクションあり
- [ ] `benchmarks/v29.5.0.json` 存在（test_count: 2342）
- [ ] `site/content/docs/runes/github.mdx` 存在

---

## スコープ外

- GitHub API への実際の HTTP 接続 — インフラ稼働後に有効化
- GitHub App 認証（JWT + Installation Token）— 将来の認証統合フェーズで対応
- Webhook 受信（`stage` として受け取る仕組み）— v30.x+ で対応
- GraphQL API 対応 — REST API のみ実装
