# v29.3.0 Tasks — pinecone Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.2.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2324 passed` を含むこと
- [x] `driver.rs` に `mod v293000_tests` が存在しないこと
- [x] `runes/pinecone/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.2.0` → `29.3.0` | [x] |
| T2 | `runes/pinecone/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T3 | `runes/pinecone/pinecone.fav` 作成（5 関数）| [x] |
| T4 | `CHANGELOG.md` に `[v29.3.0]` セクション追加 | [x] |
| T5 | `benchmarks/v29.3.0.json` 作成（test_count: 2330）| [x] |
| T6 | `site/content/docs/runes/pinecone.mdx` 作成 | [x] |
| T7 | `driver.rs` に `v293000_tests` 6 件追加 | [x] |
| T8 | `cargo test --bin fav v293000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2330 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T7）

```rust
// v293000_tests (v29.3.0) -- pinecone Rune
#[cfg(test)]
mod v293000_tests {
    #[test]
    fn pinecone_rune_file_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("upsert"),
            "runes/pinecone/pinecone.fav must define upsert"
        );
    }
    #[test]
    fn pinecone_query_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(src.contains("query"), "pinecone.fav must define query");
    }
    #[test]
    fn pinecone_delete_and_fetch_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("delete") && src.contains("fetch"),
            "pinecone.fav must define delete and fetch"
        );
    }
    #[test]
    fn pinecone_describe_index_stats_fn_exists() {
        let src = include_str!("../../runes/pinecone/pinecone.fav");
        assert!(
            src.contains("describe_index_stats"),
            "pinecone.fav must define describe_index_stats"
        );
    }
    #[test]
    fn pinecone_rune_toml_exists() {
        let src = include_str!("../../runes/pinecone/rune.toml");
        assert!(
            src.contains("pinecone"),
            "runes/pinecone/rune.toml must contain 'pinecone'"
        );
    }
    #[test]
    fn changelog_has_v29_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.3.0]") || src.contains("## v29.3.0"),
            "CHANGELOG.md must contain '[v29.3.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.3.0"
- [x] `runes/pinecone/pinecone.fav` に `upsert` / `query` / `delete` / `fetch` / `describe_index_stats` が存在する
- [x] `runes/pinecone/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `CHANGELOG.md` に `[v29.3.0]` セクションあり
- [x] `benchmarks/v29.3.0.json` 存在（test_count: 2330）
- [x] `site/content/docs/runes/pinecone.mdx` 存在
- [x] `cargo test --bin fav v293000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2330 tests PASS
