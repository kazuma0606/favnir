# v29.6.0 Tasks — pagerduty Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.5.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2342 passed` を含むこと
- [x] `driver.rs` に `mod v296000_tests` が存在しないこと
- [x] `runes/pagerduty/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.5.0` → `29.6.0` | [x] |
| T2 | `runes/pagerduty/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T3 | `runes/pagerduty/pagerduty.fav` 作成（4 関数）| [x] |
| T4 | `CHANGELOG.md` に `[v29.6.0]` セクション追加 | [x] |
| T5 | `benchmarks/v29.6.0.json` 作成（test_count: 2348）| [x] |
| T6 | `site/content/docs/runes/pagerduty.mdx` 作成 | [x] |
| T7 | `driver.rs` に `v296000_tests` 6 件追加 | [x] |
| T8 | `cargo test --bin fav v296000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2348 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T7）

```rust
// v296000_tests (v29.6.0) -- pagerduty Rune
#[cfg(test)]
mod v296000_tests {
    #[test]
    fn pagerduty_rune_file_exists() {
        let src = include_str!("../../runes/pagerduty/pagerduty.fav");
        assert!(
            src.contains("PagerDuty.create_incident"),
            "runes/pagerduty/pagerduty.fav must define PagerDuty.create_incident"
        );
    }
    #[test]
    fn pagerduty_resolve_fn_exists() {
        let src = include_str!("../../runes/pagerduty/pagerduty.fav");
        assert!(src.contains("PagerDuty.resolve"), "pagerduty.fav must define PagerDuty.resolve");
    }
    #[test]
    fn pagerduty_acknowledge_fn_exists() {
        let src = include_str!("../../runes/pagerduty/pagerduty.fav");
        assert!(src.contains("PagerDuty.acknowledge"), "pagerduty.fav must define PagerDuty.acknowledge");
    }
    #[test]
    fn pagerduty_add_note_fn_exists() {
        let src = include_str!("../../runes/pagerduty/pagerduty.fav");
        assert!(src.contains("PagerDuty.add_note"), "pagerduty.fav must define PagerDuty.add_note");
    }
    #[test]
    fn pagerduty_rune_toml_exists() {
        let src = include_str!("../../runes/pagerduty/rune.toml");
        assert!(
            src.contains("pagerduty"),
            "runes/pagerduty/rune.toml must contain 'pagerduty'"
        );
    }
    #[test]
    fn changelog_has_v29_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.6.0]") || src.contains("## v29.6.0"),
            "CHANGELOG.md must contain '[v29.6.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.6.0"
- [x] `runes/pagerduty/pagerduty.fav` に `create_incident` / `resolve` / `acknowledge` / `add_note` が存在する
- [x] `runes/pagerduty/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `CHANGELOG.md` に `[v29.6.0]` セクションあり
- [x] `benchmarks/v29.6.0.json` 存在（test_count: 2348）
- [x] `site/content/docs/runes/pagerduty.mdx` 存在
- [x] `cargo test --bin fav v296000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2348 tests PASS
