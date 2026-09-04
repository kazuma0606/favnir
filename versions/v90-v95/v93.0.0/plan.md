# Plan: v93.0.0 — SAP QueryBuilder 1.0 宣言 ★クリーンアップ

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test` を実行し、4,116 tests, 0 failures であることを確認する。

### Step 2: `cargo clean`

```bash
cargo clean
```

target/ ディレクトリを一掃する。`fav/tmp/hello.fav` は target/ 外のため影響なし。

### Step 3: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "92.0.0"` を `version = "93.0.0"` に変更する。

### Step 4: `driver.rs` の旧バージョン参照を一括更新

driver.rs 内の `"92.0.0"` を `"93.0.0"` に一括置換する（sed を使用）。

```bash
sed -i 's/"92\.0\.0"/"93.0.0"/g' src/driver.rs
```

### Step 5: `CHANGELOG.md` に v93.0.0 エントリを追加

先頭（`## [v92.9.0]` の前）に v93.0.0 エントリを追加する。
**注**: `changelog_has_v93_0_0` テストがこのエントリを参照するため、テストモジュール追加より先に行う。

### Step 6: `MILESTONE.md` を更新

SAP QueryBuilder 1.0 宣言セクションを追加する。

### Step 7: `README.md` を更新

QueryBuilder に関する記述を追加する。

### Step 8: `versions/current.md` を更新

最新安定版を v93.0.0 に更新する。

### Step 9: `mod v93000_tests` を追加

ファイル末尾の最終テストモジュール `mod v92900_tests { ... }` の直後に以下を追加する:

```rust
#[cfg(test)]
mod v93000_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn cargo_toml_version_is_93_0_0() {
        let content = std::fs::read_to_string("Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(content.contains("93.0.0"), "Cargo.toml version should be 93.0.0");
    }

    #[test]
    fn changelog_has_v93_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("v93.0.0"), "CHANGELOG.md should mention v93.0.0");
    }

    #[test]
    fn milestone_has_sap_query_builder() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(
            content.contains("SAP QueryBuilder"),
            "MILESTONE.md should mention SAP QueryBuilder"
        );
    }

    #[test]
    fn readme_mentions_query_builder() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(
            content.contains("QueryBuilder"),
            "README.md should mention QueryBuilder"
        );
    }
}
```

### Step 10: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,120 tests, 0 failures を確認する。

### Step 11: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
