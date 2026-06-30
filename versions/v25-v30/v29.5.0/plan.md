# v29.5.0 Plan — github Rune 追加

**バージョン**: 29.5.0
**日付**: 2026-06-30
**前バージョン**: v29.4.0 (vertex-ai / sagemaker Rune 追加)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.5.0"
```

### T2: runes/github/rune.toml 作成

```toml
[rune]
name        = "github"
version     = "1.0.0"
description = "GitHub REST API 連携（create_comment / create_issue / update_issue / list_prs / get_pr）"
license     = "MIT"
authors     = ["Favnir Team"]
```

### T3: runes/github/github.fav 作成（5 関数）

```favnir
// github Rune -- GitHub REST API 連携（v29.5.0）
// 接続: GITHUB_TOKEN / GITHUB_REPO / GITHUB_BASE_URL 環境変数

// PR にコメントを作成する
fn GitHub.create_comment(config: String, pr_number: String, body: String) -> Result<Unit, String> !Http =
  Http.post_json(
    Env.get_or("GITHUB_BASE_URL", "https://api.github.com") ++ "/repos/" ++ Env.get_or("GITHUB_REPO", config) ++ "/issues/" ++ pr_number ++ "/comments",
    { "body": body }
  )

// Issue を作成し、Issue 番号を返す
fn GitHub.create_issue(config: String, title: String, body: String, labels: List<String>) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("GITHUB_BASE_URL", "https://api.github.com") ++ "/repos/" ++ Env.get_or("GITHUB_REPO", config) ++ "/issues",
    { "title": title, "body": body, "labels": labels }
  )

// Issue を更新する（state: "open" または "closed"）
fn GitHub.update_issue(config: String, issue_number: String, state: String) -> Result<Unit, String> !Http =
  Http.post_json(
    Env.get_or("GITHUB_BASE_URL", "https://api.github.com") ++ "/repos/" ++ Env.get_or("GITHUB_REPO", config) ++ "/issues/" ++ issue_number,
    { "state": state }
  )

// PR 一覧を取得する（state: "open" / "closed" / "all"）
fn GitHub.list_prs(config: String, state: String) -> Result<List<String>, String> !Http =
  Http.get_json(
    Env.get_or("GITHUB_BASE_URL", "https://api.github.com") ++ "/repos/" ++ Env.get_or("GITHUB_REPO", config) ++ "/pulls?state=" ++ state
  )

// PR の詳細を取得する
fn GitHub.get_pr(config: String, pr_number: String) -> Result<String, String> !Http =
  Http.get_json(
    Env.get_or("GITHUB_BASE_URL", "https://api.github.com") ++ "/repos/" ++ Env.get_or("GITHUB_REPO", config) ++ "/pulls/" ++ pr_number
  )
```

### T4: CHANGELOG.md に [v29.5.0] セクション追加

```markdown
## [v29.5.0] — 2026-06-30

### Added
- `runes/github/` — GitHub REST API Rune（create_comment / create_issue / update_issue / list_prs / get_pr）
- `site/content/docs/runes/github.mdx` — GitHub Rune ドキュメント
- テスト数: 2336 → 2342（+6）
```

### T5: benchmarks/v29.5.0.json 作成

```json
{
  "version": "29.5.0",
  "date": "2026-06-30",
  "milestone": "Ecosystem Maturity (phase 5)",
  "test_count": 2342,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T6: site/content/docs/runes/github.mdx 作成

GitHub Rune の使い方・API リファレンス・CI パイプライン統合例を含むドキュメント。

### T7: driver.rs に v295000_tests 6 件追加

```rust
// v295000_tests (v29.5.0) -- github Rune
#[cfg(test)]
mod v295000_tests {
    #[test]
    fn github_rune_file_exists() {
        let src = include_str!("../../runes/github/github.fav");
        assert!(
            src.contains("GitHub.create_comment"),
            "runes/github/github.fav must define GitHub.create_comment"
        );
    }
    #[test]
    fn github_create_issue_fn_exists() {
        let src = include_str!("../../runes/github/github.fav");
        assert!(src.contains("create_issue"), "github.fav must define create_issue");
    }
    #[test]
    fn github_update_issue_fn_exists() {
        let src = include_str!("../../runes/github/github.fav");
        assert!(src.contains("update_issue"), "github.fav must define update_issue");
    }
    #[test]
    fn github_list_prs_fn_exists() {
        let src = include_str!("../../runes/github/github.fav");
        assert!(src.contains("list_prs"), "github.fav must define list_prs");
    }
    #[test]
    fn github_get_pr_fn_exists() {
        let src = include_str!("../../runes/github/github.fav");
        assert!(src.contains("get_pr"), "github.fav must define get_pr");
    }
    #[test]
    fn changelog_has_v29_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.5.0]") || src.contains("## v29.5.0"),
            "CHANGELOG.md must contain '[v29.5.0]'"
        );
    }
}
```

### T8: cargo test --bin fav v295000 — 6/6 PASS 確認

### T9: cargo test --bin fav — 2342 tests PASS 確認

### T10: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.4.0 | 2336 |
| v29.5.0 | **2342** (+6) |
