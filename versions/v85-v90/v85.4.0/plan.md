# Plan: v85.4.0 — `runes/sap-odata/` 骨格 + `rune.toml`

## Step 1: 前提確認

- `cargo test` を実行し、3,937 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85300_tests` が存在することを確認する（v85.3.0 完了済みの証拠）
- `runes/sap-odata/types.fav` が存在することを確認する（v85.2.0 作成済み）

## Step 2: `runes/sap-odata/rune.toml` を作成

```toml
[rune]
name        = "sap-odata"
version     = "85.4.0"
entry       = "sap_odata.fav"
description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"
```

## Step 3: `runes/sap-odata/sap_odata.fav` を作成

```favnir
-- sap-odata Rune エントリポイント（v85.4.0）
-- 後続バージョンで client.fav / error.fav 等を use 追加する。

use sap_odata.types

-- re-export: 利用者が sap_odata.SapConfig / sap_odata.sap_config_from_env を使えるようにする
public type SapConfig   = types.SapConfig
public fn   sap_config_from_env() -> Result<SapConfig, String> {
    types.sap_config_from_env()
}
```

## Step 4: `runes/sap-odata/sap_odata.test.fav` を作成

```favnir
-- sap-odata Rune テスト骨格（v85.4.0）
-- 後続バージョンで E2E テストを追加する。

fn test_sap_config_fields_exist() -> Bool {
    -- SapConfig のフィールドが定義されていることを確認する（骨格テスト）
    True
}
```

## Step 5: `fav/src/driver.rs` に `mod v85400_tests` を追加

`mod v85300_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85400_tests {
    #[test]
    fn sap_odata_rune_toml_exists() {
        assert!(
            std::path::Path::new("../runes/sap-odata/rune.toml").exists(),
            "runes/sap-odata/rune.toml should exist"
        );
    }

    #[test]
    fn sap_odata_rune_entry_exists() {
        assert!(
            std::path::Path::new("../runes/sap-odata/sap_odata.fav").exists(),
            "runes/sap-odata/sap_odata.fav should exist"
        );
    }
}
```

## Step 6: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3939 tests, 0 failures
```

## Step 7: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.4.0 エントリを追加する。

## Step 8: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
