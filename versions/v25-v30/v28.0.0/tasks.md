# v28.0.0 Tasks — Data Lakehouse マイルストーン宣言

Status: COMPLETE
test_count: 2226

## 事前確認（T0）

実装開始前に以下を確認する:

- [x] `Cargo.toml` の version が `27.9.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2220 tests` を含むこと
- [x] `driver.rs` に `mod v280000_tests` が存在しないこと
- [x] 前提 Rune が存在すること:
  - `runes/delta-lake/` ディレクトリ
  - `runes/iceberg/` ディレクトリ
  - `runes/dbt/` ディレクトリ（v27.8 で作成）
  - `runes/sqlite/` ディレクトリ（v27.9 で作成）

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `27.9.0` → `28.0.0` | [x] |
| T2 | MILESTONE.md に "Data Lakehouse" セクション追加 | [x] |
| T3 | README.md に v28.0 参照追記 | [x] |
| T4 | `site/content/docs/data-lakehouse.mdx` 新規作成 | [x] |
| T5 | `versions/roadmap/roadmap-v27.1-v28.0.md` 完了マーク追記 | [x] |
| T6 | CHANGELOG.md に `[v28.0.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.0.0.json` 新規作成（test_count: 2226） | [x] |
| T8 | `driver.rs` に `v280000_tests` 6 件追加 | [x] |
| T8.5 | `cargo test --bin fav v280000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` 全体 — 2226 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
#[cfg(test)]
mod v280000_tests {
    use super::*;

    #[test]
    fn milestone_md_mentions_data_lakehouse() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Data Lakehouse"));
    }

    #[test]
    fn milestone_md_lists_sqlite_rune() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("sqlite") || src.contains("SQLite"));
    }

    #[test]
    fn milestone_md_lists_dbt_rune() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("dbt") || src.contains("Dbt"));
    }

    #[test]
    fn readme_mentions_v28() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v28.0") || src.contains("v28.0.0"));
    }

    #[test]
    fn site_data_lakehouse_page_exists() {
        let src = include_str!("../../site/content/docs/data-lakehouse.mdx");
        assert!(src.contains("Data Lakehouse") && (src.contains("delta-lake") || src.contains("Delta Lake")));
    }

    #[test]
    fn changelog_has_v28_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.0.0]") || src.contains("## v28.0.0"));
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.0.0"
- [x] MILESTONE.md に "Data Lakehouse" セクションあり
- [x] README.md に `v28.0` または `v28.0.0` の記述あり
- [x] `site/content/docs/data-lakehouse.mdx` 存在（"Data Lakehouse" + "Delta Lake" を含む）
- [x] `CHANGELOG.md` に `[v28.0.0]` セクションあり
- [x] `benchmarks/v28.0.0.json` 存在（test_count: 2226）
- [x] `cargo test --bin fav v280000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2226 tests PASS

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [MED] | `data-lakehouse.mdx` の `../../../MILESTONE.md` リンクは Next.js 静的サイト上で解決されない | リンクを削除してテキスト参照（バッククォート）に変更 |
| [MED] | mdx 達成コンポーネント表に `list_snapshots`（iceberg）と `create_table`（bigquery）が欠落 | mdx の各行に追加して MILESTONE.md と同期 |
| [LOW] | その他 Rust コード変更なし・セキュリティ/正確性問題なし | 対応不要 |
