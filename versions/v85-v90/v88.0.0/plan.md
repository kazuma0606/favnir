# Plan: v88.0.0 — SAP Sales 1.0 宣言 ★クリーンアップ

## 実装ステップ

### Step 1: `cargo clean` 実施

`target/` ディレクトリを削除してクリーンな状態にする。

```bash
cargo clean
```

### Step 2: `Cargo.toml` バージョンを `88.0.0` に更新

`fav/Cargo.toml` の `version = "87.0.0"` を `version = "88.0.0"` に変更する。

### Step 3: `driver.rs` の既存 `cargo_toml_version_is_` テストを一括更新

`driver.rs` 内に存在するすべての `cargo_toml_version_is_` テスト関数は Cargo.toml バージョンを
チェックしており、バージョン更新後にすべてが fail する。`replace_all: true` で
`"87.0.0"` → `"88.0.0"` に一括置換する。

ただし `cargo_toml_version_is_87_0_0` という関数名自体も `cargo_toml_version_is_88_0_0` に
変更する必要がある（テスト名が実バージョンと一致しないと混乱を招くため）。
- 関数名変更の対象は **最新テスト（`cargo_toml_version_is_87_0_0`）のみ**
- 旧バージョン関数名（`cargo_toml_version_is_86_0_0` 等）は変更しない
- バージョン文字列（`"87.0.0"` → `"88.0.0"`）は driver.rs 全体で replace_all: true で置換する

### Step 4: `CHANGELOG.md` に v88.0.0 エントリを追加

先頭（`## [v87.0.0]` の直前）に以下を追加する:

```markdown
## [v88.0.0] — 2026-08-23 — SAP Sales 1.0 宣言

> 「受注が型になった。
>  `sales_orders()` で絞り込み、`sales_order_by_id()` で明細まで取得できる。
>  日次売上レポートが、Favnir の 10 行で書ける。」

### Added
- `fav/src/driver.rs` — `mod v88000_tests`（テスト 4 件）を追加
- `MILESTONE.md` — SAP Sales 1.0 マイルストーンを追加
- 合計テスト数: **3,997**（+4）

### Changed
- `fav/Cargo.toml` — version を `87.0.0` → `88.0.0` に更新
- `versions/current.md` — v88.0.0 に更新
```

### Step 5: `MILESTONE.md` に SAP Sales 1.0 マイルストーンを追加

先頭（`## v87.0.0` の直前）に v88.0.0 マイルストーンエントリを追加する。

### Step 6: `README.md` を更新

最新バージョン（v88.0.0）・テスト数（3,997）を反映する。

### Step 7: `versions/current.md` を更新

`v87.0.0` → `v88.0.0` に更新する。

### Step 8: `driver.rs` に `mod v88000_tests` を追加

`mod v87900_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88000_tests {
    #[test]
    fn cargo_toml_version_is_88_0_0() {
        let content = std::fs::read_to_string("../fav/Cargo.toml")
            .expect("fav/Cargo.toml should exist");
        assert!(content.contains("version = \"88.0.0\""), "Cargo.toml version should be 88.0.0");
    }
    #[test]
    fn changelog_has_v88_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("[v88.0.0]"), "CHANGELOG.md should have v88.0.0 entry");
    }
    #[test]
    fn milestone_has_sap_sales() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(content.contains("SAP Sales"), "MILESTONE.md should have SAP Sales milestone");
    }
    #[test]
    fn sap_odata_rune_has_sales_order_type() {
        let content = std::fs::read_to_string("../runes/sap-odata/sap_odata.fav")
            .expect("runes/sap-odata/sap_odata.fav should exist");
        assert!(content.contains("SalesOrder"), "sap_odata.fav should re-export SalesOrder type");
    }
}
```

### Step 9: `cargo test` で全 pass 確認

**Step 8 完了後に実行すること**（`v88000_tests` の 4 件が追加されていないと 3,997 に届かない）。

3,993 + 4 = 3,997 tests, 0 failures を確認する。
