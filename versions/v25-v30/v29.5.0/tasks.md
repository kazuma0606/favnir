# v29.5.0 Tasks — github Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.4.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2336 passed` を含むこと
- [x] `driver.rs` に `mod v295000_tests` が存在しないこと
- [x] `runes/github/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.4.0` → `29.5.0` | [x] |
| T2 | `runes/github/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T3 | `runes/github/github.fav` 作成（5 関数）| [x] |
| T4 | `CHANGELOG.md` に `[v29.5.0]` セクション追加 | [x] |
| T5 | `benchmarks/v29.5.0.json` 作成（test_count: 2342）| [x] |
| T6 | `site/content/docs/runes/github.mdx` 作成 | [x] |
| T7 | `driver.rs` に `v295000_tests` 6 件追加 | [x] |
| T8 | `cargo test --bin fav v295000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2342 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T7）

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

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.5.0"
- [x] `runes/github/github.fav` に `create_comment` / `create_issue` / `update_issue` / `list_prs` / `get_pr` が存在する
- [x] `runes/github/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `CHANGELOG.md` に `[v29.5.0]` セクションあり
- [x] `benchmarks/v29.5.0.json` 存在（test_count: 2342）
- [x] `site/content/docs/runes/github.mdx` 存在
- [x] `cargo test --bin fav v295000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2342 tests PASS
