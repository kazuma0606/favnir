# Plan: v91.0.0 — SAP Ctx 統合 1.0 宣言 ★クリーンアップ

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（cargo clean）
    ↓
Step 3（Cargo.toml バージョン更新）
    ↓
Step 4（CHANGELOG 追加）          ← changelog_has_v91_0_0 テストより先
    ↓
Step 5（MILESTONE 更新）          ← milestone_has_sap_ctx_integration テストより先
    ↓
Step 6（README 更新）             ← readme_mentions_ctx_sap テストより先
    ↓
Step 7（versions/current.md 更新）
    ↓
Step 8（driver.rs cargo_toml_version テスト更新）
    ↓
Step 9（deprecated 関数削除）
    ↓
Step 10（mod v91000_tests 追加）
    ↓
Step 11（cargo test）
    ↓
Step 12（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `cargo test 2>&1 | grep "test result"` で 4,061 tests, 0 failures を確認する
- `runes/sap-odata/sap_odata.fav` の deprecated 関数（4 件）を確認する
- 各個別 Rune ファイル（business_partner.fav / sales_order.fav / material.fav / journal_entry.fav）の `cfg: SapConfig` 受け取り関数を確認する

### Step 2: `cargo clean`

```bash
cargo clean
```

これにより `target/` ディレクトリが削除される。`fav/tmp/hello.fav` は消えないが念のため確認する。

### Step 3: `Cargo.toml` バージョンを `91.0.0` に更新

`fav/Cargo.toml` の `version = "90.0.0"` を `version = "91.0.0"` に変更する。

### Step 4: `CHANGELOG.md` に v91.0.0 エントリを追加

`## [v90.9.0]` の前に v91.0.0 エントリを追加する。
`SAP Ctx 統合 1.0` / `ctx.sap` / `4,065` を含める。
（`changelog_has_v91_0_0` テストより先に追加すること）

### Step 5: `MILESTONE.md` を更新

`v91.0 — SAP Ctx 統合 1.0` のエントリを **完了** に更新する。
`SAP Ctx 統合 1.0` という文字列が含まれることを確認する。

### Step 6: `README.md` に `ctx.sap` 言及を追加

README の SAP 関連セクションに `ctx.sap` パターンの説明を追加または更新する。

### Step 7: `versions/current.md` を v91.0.0 に更新

- 「最新安定版」を `v91.0.0` に更新する
- テスト数 `4,065` を記載する

### Step 8: `driver.rs` の `cargo_toml_version` テストを更新

`cargo_toml_version_is_90_0_0` を `cargo_toml_version_is_91_0_0` に更新する。
（`replace_all` オプションで全件一括更新）

### Step 9: deprecated 関数を削除

#### 9-1: `runes/sap-odata/sap_odata.fav`

v90.5.0 で deprecated コメントを付けた 4 関数を削除する:
- `business_partners_cfg(cfg: SapConfig, filter: BusinessPartnerFilter)`
- `sales_orders_cfg(cfg: SapConfig, filter: SalesOrderFilter)`
- `materials_cfg(cfg: SapConfig, filter: MaterialFilter)`
- `journal_entries_cfg(cfg: SapConfig, filter: JournalFilter)`

#### 9-2: 個別 Rune ファイル

各ファイルを読み込み、`cfg: SapConfig` を受け取る関数 variants を確認・削除する。
削除対象は実態確認後に決定する。

### Step 10: `mod v91000_tests` を `driver.rs` に追加

`mod v90900_tests { ... }` の直後に追加（4 件）:

```rust
#[cfg(test)]
mod v91000_tests {
    use std::fs;

    #[test]
    fn cargo_toml_version_is_91_0_0() {
        let content = fs::read_to_string("../fav/Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(
            content.contains("version = \"91.0.0\""),
            "Cargo.toml should have version 91.0.0"
        );
    }

    #[test]
    fn changelog_has_v91_0_0() {
        let content = fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(
            content.contains("v91.0.0"),
            "CHANGELOG.md should mention v91.0.0"
        );
    }

    #[test]
    fn milestone_has_sap_ctx_integration() {
        let content = fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(
            content.contains("SAP Ctx 統合 1.0"),
            "MILESTONE.md should mention SAP Ctx 統合 1.0"
        );
    }

    #[test]
    fn readme_mentions_ctx_sap() {
        let content = fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(
            content.contains("ctx.sap"),
            "README.md should mention ctx.sap"
        );
    }
}
```

### Step 11: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4,065 tests, 0 failures を確認する

### Step 12: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
