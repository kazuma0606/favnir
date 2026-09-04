# Spec: v94.0.0 — SAP Metadata Infer 1.0 宣言 ★クリーンアップ

## Background

v93.1.0〜v93.9.0 で実装した SAP Metadata Infer 機能の完成を宣言する。
`fav infer --from sap --metadata <url>` で SAP の `$metadata` から Favnir 型定義が自動生成され、
EntityType は `type` に、EnumType は ADT に、NavigationProperty は ExpandClause ヘルパーに変換される。

## 宣言文

> 「`fav infer --sap-metadata <url>` と打てば、SAP の $metadata から Favnir 型定義が自動生成される。
>  EntityType は `type` に、EnumType は ADT に、NavigationProperty は ExpandClause ヘルパーに変換される。
>  それが、Favnir SAP Metadata Infer 1.0 である。」

## Goals

1. `Cargo.toml` バージョンを `94.0.0` に更新する。
2. `driver.rs` 内の全 `cargo_toml_version_is_X_0_0` テストが `version = "94.0.0"` を検証するよう一括更新する。
3. `CHANGELOG.md` / `MILESTONE.md` / `README.md` に v94.0.0 宣言エントリを追加する。
4. `versions/current.md` を v94.0.0 に更新する。
5. `driver.rs` に `v94000_tests`（4 件）を追加し、4,142 tests を達成する。
6. `cargo clean` を実施してビルド成果物をリセットする。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | バージョンを `93.0.0` → `94.0.0` に更新 |
| `fav/src/driver.rs` | 全 `cargo_toml_version_is_X_0_0` テストの assert 文字列を `"94.0.0"` に一括更新 / `v94000_tests` 追加 |
| `CHANGELOG.md` | v94.0.0 エントリを追加 |
| `MILESTONE.md` | v94.0.0 — SAP Metadata Infer 1.0 エントリを先頭に追加 |
| `README.md` | v94.0 宣言セクションを追加 |
| `versions/current.md` | v94.0.0 に更新 |

## `v94000_tests` テスト一覧

```rust
#[cfg(test)]
mod v94000_tests {
    fn cargo_toml_version_is_94_0_0() { ... }   // Cargo.toml に "94.0.0" が含まれる
    fn changelog_has_v94_0_0()        { ... }   // CHANGELOG.md に "v94.0.0" が含まれる
    fn milestone_has_sap_metadata_infer() { ... } // MILESTONE.md に "SAP Metadata Infer" が含まれる
    fn readme_mentions_metadata_infer() { ... }  // README.md に "metadata_infer" または関連文字列が含まれる
}
```

## Success Criteria

- `cargo test 2>&1 | grep "test result"` → `4142 tests, 0 failures`
- `cargo clippy --locked -- -D warnings` → pass
- `./target/debug/fav fmt --check self/compiler.fav` → pass
- `./target/debug/fav fmt --check self/checker.fav` → pass

## Notes

- **サイト MDX は v93.8.0 で完了済みのため本バージョンでは変更不要**（`site/content/docs/cli/infer.mdx` / `sap-odata.mdx` は既に更新済み）。
- **CHANGELOG 更新は v94000_tests 追加より前に行うこと**（`changelog_has_v94_0_0` テストが先に通る必要があるため）。
- **`cargo clean` 後に `fav/tmp/hello.fav` が消える場合がある**。消えていた場合は内容 `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` で復元すること。
- **`cargo clean` 後は `./target/debug/fav` が消える**ため、T-last の `fav fmt --check` は `cargo build` より後に実行すること。
- `driver.rs` の `cargo_toml_version_is_X_0_0` 全テストは `replace_all: true` で `"93.0.0"` → `"94.0.0"` に一括置換する（assert 文字列のみ、テスト名は変更しない）。
