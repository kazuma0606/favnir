# Plan: v87.0.0 — SAP Master Data 1.0 宣言 ★クリーンアップ

## 実装ステップ

### Step 0: 着手前確認

- `cargo test 2>&1 | grep "test result"` が 3971 tests, 0 failures であることを確認する
- `fav/src/driver.rs` に `mod v86900_tests` が存在することを確認する（v86.9.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `86.0.0` であることを確認する

### Step 1: `CHANGELOG.md` に v87.0.0 エントリ追加

v86.9.0 エントリの直前（先頭）に v87.0.0 宣言エントリを追加する。
`changelog_has_v87_0_0` テストが Step 6 の `cargo test` 前に通る必要があるため、
必ずテストモジュール追加（Step 4）より先に実施すること。

### Step 2: `fav/Cargo.toml` のバージョン更新

`version = "86.0.0"` → `version = "87.0.0"` に変更する。

### Step 3: `driver.rs` の `cargo_toml_version` テスト群を一括更新

`cargo_toml_version_is_86_0_0` を含む既存テストを `87.0.0` に更新する。
`replace_all: true` で `"86.0.0"` を `"87.0.0"` に置換するか、対象テストを個別に更新する。

### Step 4: `mod v87000_tests` を追加

`mod v86900_tests { ... }` の直後に追加する。

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

### Step 5: `MILESTONE.md` / `README.md` / `versions/current.md` 更新

- `MILESTONE.md`: SAP Master Data 1.0（v87.0.0）マイルストーンを追加
- `README.md`: 最新バージョンを v87.0.0 に更新
- `versions/current.md`: 最終更新を v87.0.0 に更新

### Step 6: `cargo test` で全 pass 確認

ベース: v86.9.0 完了時点で 3971 tests。

```
cargo test 2>&1 | grep "test result"
```

期待: `3975 tests, 0 failures`

### Step 7: `cargo clean` 実施（★クリーンアップ）

```
cargo clean
```

### Step 8: `cargo test` で全 pass 再確認（clean 後）

clean 後にビルド・テストが正常完了することを確認する。

### Step 9: CI 事前確認

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
