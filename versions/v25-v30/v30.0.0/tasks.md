# v30.0.0 Tasks — Ecosystem Maturity マイルストーン宣言

**状態**: COMPLETE
**開始日**: 2026-07-01
**完了日**: 2026-07-01

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.9.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2366 passed` を含むこと
- [x] `driver.rs` に `mod v300000_tests` が存在しないこと
- [x] `MILESTONE.md` に `Ecosystem Maturity` セクションが存在しないこと
- [x] `site/content/docs/ecosystem-maturity.mdx` が存在しないこと
- [x] v29.7.0 / v29.8.0 / v29.9.0 が COMPLETE であること（依存前提）

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.9.0` → `30.0.0` | [x] |
| T2 | `MILESTONE.md` に `## v30.0.0 — Ecosystem Maturity` セクション追加（H1 直後挿入）| [x] |
| T3 | `README.md` に `v30.0` マイルストーン追記 | [x] |
| T4 | `site/content/docs/ecosystem-maturity.mdx` 作成（`fav add stripe` 含む）| [x] |
| T5 | `versions/roadmap/roadmap-v29.1-v30.0.md` に達成宣言（COMPLETE）追記 | [x] |
| T6 | `CHANGELOG.md` に `[v30.0.0]` セクション追加 | [x] |
| T7 | `benchmarks/v30.0.0.json` 作成（test_count: 2372）| [x] |
| T8 | `driver.rs` に `v300000_tests` 6 件追加 | [x] |
| T9 | `cargo test --bin fav v300000` — 6/6 PASS 確認 | [x] |
| T10 | `cargo test --bin fav` — 2372 tests PASS 確認 | [x] |
| T11 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T8）

```rust
// v300000_tests (v30.0.0) -- Ecosystem Maturity マイルストーン宣言
#[cfg(test)]
mod v300000_tests {
    #[test]
    fn milestone_md_has_ecosystem_maturity() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Ecosystem Maturity"),
            "MILESTONE.md must contain 'Ecosystem Maturity'"
        );
    }
    #[test]
    fn readme_mentions_v30() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("v30.0"),
            "README.md must contain 'v30.0'"
        );
    }
    #[test]
    fn changelog_has_v30_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v30.0.0]"),
            "CHANGELOG.md must contain '[v30.0.0]'"
        );
    }
    #[test]
    fn ecosystem_maturity_doc_exists() {
        let src = include_str!("../../site/content/docs/ecosystem-maturity.mdx");
        assert!(
            src.contains("fav add stripe"),
            "site/content/docs/ecosystem-maturity.mdx must contain 'fav add stripe'"
        );
    }
    #[test]
    fn community_rune_shopify_exists() {
        let src = include_str!("../../runes/shopify/shopify.fav");
        assert!(
            src.contains("Shopify.list_orders"),
            "runes/shopify/shopify.fav must contain 'Shopify.list_orders'"
        );
    }
    #[test]
    fn roadmap_v29_v30_declared_complete() {
        let src = include_str!("../../versions/roadmap/roadmap-v29.1-v30.0.md");
        assert!(
            src.contains("COMPLETE"),
            "versions/roadmap/roadmap-v29.1-v30.0.md must contain 'COMPLETE'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "30.0.0"
- [x] `MILESTONE.md` に `Ecosystem Maturity` 宣言セクションが存在する
- [x] `README.md` に `v30.0` が存在する
- [x] `site/content/docs/ecosystem-maturity.mdx` が存在し `fav add stripe` を含む
- [x] `versions/roadmap/roadmap-v29.1-v30.0.md` に `COMPLETE` が存在する
- [x] `CHANGELOG.md` に `[v30.0.0]` セクションあり
- [x] `benchmarks/v30.0.0.json` 存在（test_count: 2372）
- [x] `cargo test --bin fav v300000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2372 tests PASS
- [x] tasks.md を COMPLETE に更新
