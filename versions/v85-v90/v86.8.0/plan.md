# Plan: v86.8.0 — Rune Registry デプロイ（sap-odata）

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.8.0 エントリ追加

v86.7.0 エントリの直前（先頭）に v86.8.0 エントリを追加する。

### Step 2: `runes/sap-odata/rune.toml` を更新

`version` を現在値から `86.8.0` に更新する。

```toml
[rune]
name        = "sap-odata"
version     = "86.8.0"
entry       = "sap_odata.fav"
description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"
```

### Step 3: `fav/src/driver.rs` に `mod v86800_tests` 追加

`mod v86700_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v86800_tests {
    #[test]
    fn sap_odata_rune_version_matches_cargo() {
        let content = std::fs::read_to_string("../runes/sap-odata/rune.toml")
            .expect("runes/sap-odata/rune.toml should exist");
        assert!(
            content.contains("version") && content.contains("86."),
            "rune.toml version should start with 86."
        );
    }

    #[test]
    fn sap_odata_rune_entry_file_is_sap_odata_fav() {
        let content = std::fs::read_to_string("../runes/sap-odata/rune.toml")
            .expect("runes/sap-odata/rune.toml should exist");
        assert!(
            content.contains("entry") && content.contains("sap_odata.fav"),
            "rune.toml entry should be sap_odata.fav"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

ベース: v86.7.0 完了時点で 3967 tests。

```
cargo test 2>&1 | grep "test result"
```

期待: `3969 tests, 0 failures`

### Step 5: Rune Registry デプロイ（手動）

`deploy-registry` スキルを使用して `sap-odata` Rune をデプロイする。
（ユーザーが手動で `/deploy-registry` を実行する）

デプロイ後、以下を手動確認する:
- DynamoDB (`favnir-rune-registry`) に `name = "sap-odata"` のエントリが存在すること
- S3 (`favnir-rune-packages`) に `sap-odata/` 配下の `.fav` ファイルが存在すること

### Step 6: CI 事前確認

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
