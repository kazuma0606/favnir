# Spec: v99.9.0 — コードフリーズ・最終確認

## Background

v99.1〜v99.8 で SAP Platform の全機能とドキュメントが完成した。
v99.9.0 は v100.0.0 宣言前の最終コードフリーズバージョンである。
全テスト通過・CI チェック・ガイドドキュメントのリンク整合性確認・次バージョン欄の更新を行い、
v100.0.0 宣言への準備を完了する。

前提: `versions/v95-v100/v99.9.0/` ディレクトリが存在すること（存在しなければ作成する）。

## Goals

1. `fav/src/driver.rs` に `mod v99900_tests`（2 テスト）を追加する
2. 全 SAP ガイドドキュメントの相互リンク整合性を確認する
3. `versions/current.md` の「次に切る版」欄を `v100.0.0` に更新する
4. 全テスト 4,275 pass・CI チェック完全通過を確認する

## 成果物仕様

### mod v99900_tests（driver.rs）

```rust
#[cfg(test)]
mod v99900_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_guide_docs_all_exist() {
        // v99.8.0 で作成した 3 ガイドすべての存在を最終確認
        std::fs::read_to_string("../site/content/docs/guides/sap-platform.mdx")
            .expect("sap-platform.mdx should exist (v99.9.0)");
        std::fs::read_to_string("../site/content/docs/guides/sap-migration.mdx")
            .expect("sap-migration.mdx should exist (v99.9.0)");
        std::fs::read_to_string("../site/content/docs/guides/sap-enterprise-checklist.mdx")
            .expect("sap-enterprise-checklist.mdx should exist (v99.9.0)");
    }
    #[test]
    fn current_md_next_version_is_v100() {
        let content = std::fs::read_to_string("../versions/current.md")
            .expect("current.md should exist (v99.9.0)");
        assert!(
            content.contains("v100"),
            "versions/current.md should mention v100 as next version (v99.9.0)"
        );
    }
}
```

### versions/current.md 更新

「次に切る版」セクションを `v100.0.0 — Favnir SAP Platform 1.0 宣言` に更新する。
「次バージョン欄」の `v100` キーワードが含まれること（テストで検証）。

## Success Criteria

- `mod v99900_tests` の 2 テストが pass する
- `sap_guide_docs_all_exist`: 3 ガイド MDX ファイルがすべて存在する
- `current_md_next_version_is_v100`: `versions/current.md` に `v100` が含まれる
- 全テスト通過: 4,275（4,273 + 2）
- `sap-platform.mdx` が `sap-migration` と `sap-enterprise-checklist` への参照を含む
- `sap-migration.mdx` が `sap-platform` と `sap-enterprise-checklist` への参照を含む
- `sap-enterprise-checklist.mdx` が `sap-platform` と `sap-migration` への参照を含む
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass

## Files to Modify

| ファイル | 操作 |
|---|---|
| `fav/src/driver.rs` | `mod v99900_tests` 追加 |
| `versions/current.md` | 次に切る版を `v100.0.0` に更新 + 最新安定版を `v99.9.0` に更新 |
| `CHANGELOG.md` | v99.9.0 エントリ追加 |
