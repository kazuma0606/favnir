# Spec: v87.0.0 — SAP Master Data 1.0 宣言 ★クリーンアップ

## 宣言文

> 「SAP の BusinessPartner が、Favnir の型になった。
>  得意先も仕入先も、`business_partners()` で型安全に取得できる。」

## Background

v86.1.0〜v86.9.0 で SAP Master Data Sprint 2 の全機能を実装し、安定化した。

| バージョン | 内容 |
|---|---|
| v86.1.0 | BusinessPartner / BusinessPartnerAddress / BusinessPartnerCategory 型定義 |
| v86.2.0 | business_partners() フィルタ検索 + BusinessPartnerFilter |
| v86.3.0 | business_partner_by_id() 単件取得 |
| v86.4.0 | create_business_partner() 作成 + NewBusinessPartner |
| v86.5.0 | update_business_partner() 更新 + BusinessPartnerPatch |
| v86.6.0 | E2E パイプライン（BusinessPartner → S3） |
| v86.7.0 | CRUD テスト（sap_odata.test.fav） |
| v86.8.0 | Rune Registry デプロイ（rune.toml v86.8.0、`!Effect` 注釈修正） |
| v86.9.0 | 安定化・コードフリーズ、Rune Registry 登録確認 |

v87.0.0 は宣言バージョン。★クリーンアップを実施し、Cargo.toml / CHANGELOG / MILESTONE / README / current.md を更新する。

## Goals

1. `fav/Cargo.toml` のバージョンを `87.0.0` に更新する
2. `cargo clean` でビルドキャッシュをクリアする
3. `CHANGELOG.md` に v87.0.0 宣言エントリを追加する
4. `MILESTONE.md` に SAP Master Data 1.0 マイルストーンを追加する
5. `README.md` の最新バージョン記述を更新する
6. `versions/current.md` を v87.0.0 に更新する
7. `driver.rs` 内の `cargo_toml_version` テスト群（計 33 件）を `87.0.0` に一括更新する
8. `mod v87000_tests` を追加する（テスト 4 件）

## Rust テスト（`mod v87000_tests`）

```rust
#[cfg(test)]
mod v87000_tests {
    #[test]
    fn cargo_toml_version_is_87_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"87.0.0\""), "Cargo.toml version should be 87.0.0");
    }

    #[test]
    fn changelog_has_v87_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("[v87.0.0]"), "CHANGELOG.md should have v87.0.0 entry");
    }

    #[test]
    fn milestone_has_sap_master_data() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(content.contains("SAP Master Data"), "MILESTONE.md should have SAP Master Data milestone");
    }

    #[test]
    fn sap_odata_rune_toml_has_name_sap_odata() {
        let content = std::fs::read_to_string("../runes/sap-odata/rune.toml")
            .expect("runes/sap-odata/rune.toml should exist");
        assert!(content.contains("name        = \"sap-odata\""), "rune.toml should have name sap-odata");
    }
}
```

## Files to Modify

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version を `86.0.0` → `87.0.0` に更新 |
| `CHANGELOG.md` | v87.0.0 宣言エントリ追加（先頭） |
| `MILESTONE.md` | SAP Master Data 1.0 エントリ追加 |
| `README.md` | 最新バージョン記述更新 |
| `versions/current.md` | v87.0.0 に更新 |
| `fav/src/driver.rs` | `cargo_toml_version` テスト群を `87.0.0` に一括更新 + `mod v87000_tests` 追加 |

## Success Criteria

- `cargo test 2>&1 | grep "test result"` が 3975 tests, 0 failures を出力する
- `fav/Cargo.toml` の version が `87.0.0` である
- `CHANGELOG.md` に `[v87.0.0]` エントリが存在する
- `MILESTONE.md` に `SAP Master Data` が含まれる
- `README.md` に `87.0.0` が含まれる
- `versions/current.md` が v87.0.0 を示している
