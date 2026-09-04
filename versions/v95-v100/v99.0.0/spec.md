# Spec: v99.0.0 — SAP Analytics 1.0 宣言

## Background

v98.1.0〜v98.9.0 で SAP Analytics の全機能（KPI 型定義 / BW クエリ / SAC プッシュ / KPI アラート / CLI / E2E デモ / サイトドキュメント / コードフリーズ）を実装・完成させた。
v99.0.0 では **SAP Analytics 1.0** を正式に宣言する。Cargo.toml のバージョンを `99.0.0` に更新し、MILESTONE.md・README.md を更新する。cargo clean によるクリーンアップも実施する（★クリーンアップ）。

## 宣言文

> 「SAP のデータが、洞察になった。
>
>  `KpiDefinition<SalesOrder>` が売上の健全性を測り、
>  BW クエリの結果が SAC に流れ、
>  閾値を超えた瞬間に Slack が鳴る。
>
>  それが、Favnir SAP Analytics 1.0 である。」

## Goals

1. `fav/Cargo.toml` — version を `99.0.0` に更新
2. `MILESTONE.md` — v99.0.0 エントリを先頭に追加
3. `README.md` — `## v99.0 — SAP Analytics 1.0` セクションを先頭付近に追加
4. `fav/src/driver.rs` — `mod v99000_tests`（4 テスト）追加
5. cargo clean（★クリーンアップ）+ cargo test 再確認

## Tests（mod v99000_tests）

```rust
#[cfg(test)]
mod v99000_tests {
    // use super::* は不要（外部シンボル未使用）
    #[test]
    fn cargo_toml_version_is_99_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("version = \"99.0.0\""),
            "Cargo.toml should declare version 99.0.0"
        );
    }

    #[test]
    fn changelog_has_v99_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(
            content.contains("[v99.0.0]"),
            "CHANGELOG.md should have v99.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_sap_analytics() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("SAP Analytics"),
            "MILESTONE.md should mention SAP Analytics 1.0"
        );
    }

    #[test]
    fn readme_mentions_sap_analytics() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("SAP Analytics"),
            "README.md should mention SAP Analytics 1.0"
        );
    }
}
```

## Success Criteria

- `fav/Cargo.toml` の version が `99.0.0` である
- `MILESTONE.md` に `v99.0.0` エントリと `SAP Analytics` キーワードが含まれる
- `README.md` に `SAP Analytics` が含まれる
- `cargo test -- --test-threads=1` が 4,257 tests, 0 failures で通過する（cargo clean 前後の両方）
- `cargo clippy --locked -- -D warnings` が通過する
- `./target/debug/fav fmt --check self/compiler.fav` が通過する
- `./target/debug/fav fmt --check self/checker.fav` が通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/Cargo.toml` | version を `99.0.0` に更新 |
| `MILESTONE.md` | 先頭に v99.0.0 エントリを追加 |
| `README.md` | `## v99.0 — SAP Analytics 1.0` セクションを追加 |
| `fav/src/driver.rs` | 追記（`mod v99000_tests`、4 テスト） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## クリーンアップ（★必須）

cargo test 全 pass 確認後に `cargo clean` を実施する。
cargo clean 後は `fav/tmp/hello.fav` が削除されるため、必ず復元すること（内容: `fn add(a: Int, b: Int) -> Int { a + b }` と `fn main() -> Bool { add(1, 2) == 3 }`）。
復元後、cargo test を再度実行して 4,257 tests, 0 failures を確認する。
