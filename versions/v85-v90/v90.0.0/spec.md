# Spec: v90.0.0 — SAP Integration 1.0 宣言

## Background

v89.1〜v89.9 で SAP Integration の全機能が完成した。
本バージョンは SAP Integration Era（v85.1〜v90.0）の宣言バージョンである。

**宣言文**:
> 「SAP が、Favnir の型になった。
>
>  `business_partners()` で得意先を取得し、
>  `sales_orders()` で受注を集計し、
>  `materials()` で在庫を確認し、
>  `journal_entries()` で支払を照合する。
>
>  世界最大の ERP データが、型安全なパイプラインとして流れる。
>  それが、Favnir SAP Integration 1.0 である。」

## Goals

1. `cargo clean` でビルドキャッシュをクリーンアップする
2. `Cargo.toml` バージョンを `89.0.0` → `90.0.0` に更新する
3. `CHANGELOG.md` に v90.0.0 エントリを追加する
4. `MILESTONE.md` に SAP Integration 1.0 マイルストーンを追加する
5. `README.md` に SAP Integration の言及を追加する
6. `versions/current.md` を v90.0.0 に更新する
7. `driver.rs` 内の `"89.0.0"` 文字列（実測 42 件）を `90.0.0` に一括更新する
8. `versions/roadmap/roadmap-v85.1-v90.0.md` の全エントリを完了マークに更新する
9. `mod v90000_tests` を追加する（4 件）

## Success Criteria（Rust テストで担保）

- `cargo_toml_version_is_90_0_0`:
  `fav/Cargo.toml` の version が `90.0.0` であることを確認
- `changelog_has_v90_0_0`:
  `CHANGELOG.md` に `v90.0.0` が含まれることを確認
- `milestone_has_sap_integration`:
  `MILESTONE.md` に `SAP Integration` が含まれることを確認
- `readme_mentions_sap_integration`:
  `README.md` に `SAP Integration` が含まれることを確認
- `cargo test` で 4,041 tests, 0 failures（4,037 + 4）

## テスト詳細

```rust
#[cfg(test)]
mod v90000_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn cargo_toml_version_is_90_0_0() {
        let content = std::fs::read_to_string("Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(content.contains("version = \"90.0.0\""),
            "Cargo.toml should have version 90.0.0");
    }

    #[test]
    fn changelog_has_v90_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("v90.0.0"),
            "CHANGELOG.md should mention v90.0.0");
    }

    #[test]
    fn milestone_has_sap_integration() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(content.contains("SAP Integration"),
            "MILESTONE.md should mention SAP Integration");
    }

    #[test]
    fn readme_mentions_sap_integration() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(content.contains("SAP Integration"),
            "README.md should mention SAP Integration");
    }
}
```

## Files to Create / Modify

| ファイル | 変更種別 |
|---|---|
| `fav/Cargo.toml` | version `89.0.0` → `90.0.0` |
| `CHANGELOG.md` | v90.0.0 エントリ追加（先頭） |
| `MILESTONE.md` | SAP Integration 1.0 マイルストーン追加 |
| `README.md` | SAP Integration 言及追加 |
| `versions/current.md` | v90.0.0 に更新 |
| `versions/roadmap/roadmap-v85.1-v90.0.md` | 全エントリを完了マークに更新 |
| `fav/src/driver.rs` | `"89.0.0"` 文字列（42 件）を一括置換 + `mod v90000_tests` 追加 |

**前提確認**:
- `cargo clean` は spec.md の Goals に含まれるが、driver.rs の一括更新（214 件）は `sed` または `replace_all: true` Edit で実施
- CHANGELOG の追加（T4）は `mod v90000_tests` 追加（T8）より前に行う（`changelog_has_v90_0_0` テストが通るために必要）
- テストは `fav/` を cwd として実行されるため `"../CHANGELOG.md"` は `CHANGELOG.md` に解決される
