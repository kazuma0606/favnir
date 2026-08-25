# Plan: v81.0.0 — Test-Driven Data 1.0 宣言 ★クリーンアップ

## Step 1: 前提確認

- `cargo test` を実行し、3837 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v80900_tests` が存在することを確認する

## Step 2: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "80.0.0"` を `version = "81.0.0"` に変更する。

## Step 3: ドキュメント更新

### CHANGELOG.md

先頭に v81.0.0 エントリを追加する（T4 を T3 より前に実施 — `changelog_has_v81_0_0` テストがあるため）。

### MILESTONE.md

Test-Driven Data 1.0 達成を宣言するセクションを追加する。
`"Test-Driven Data"` という文字列が含まれていれば要件を満たす。

### README.md

`fav test` コマンドの言及を追加する（既存の機能一覧に 1 行追加する形で可）。

### versions/current.md

現在バージョンを v81.0.0 に更新する。

### versions/roadmap/roadmap-v80.1-v85.0.md

Sprint 1 バージョン一覧テーブル（v80.1.0〜v81.0.0）の「状態」列を全行「完了」に更新する。

## Step 4: `fav/src/driver.rs` に `mod v81000_tests` を追加

`mod v80900_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81000_tests {
    use std::fs;

    #[test]
    fn cargo_toml_version_is_81_0_0() {
        let toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");
        assert!(toml.contains("version = \"81.0.0\""),
            "Cargo.toml should have version 81.0.0: {toml}");
    }

    #[test]
    fn changelog_has_v81_0_0() {
        let log = fs::read_to_string("../CHANGELOG.md").expect("CHANGELOG.md must exist");
        assert!(log.contains("v81.0.0"),
            "CHANGELOG.md should mention v81.0.0");
    }

    #[test]
    fn milestone_has_test_driven_data() {
        let ms = fs::read_to_string("../MILESTONE.md").expect("MILESTONE.md must exist");
        assert!(ms.contains("Test-Driven Data"),
            "MILESTONE.md should mention Test-Driven Data");
    }

    #[test]
    fn readme_mentions_fav_test() {
        let readme = fs::read_to_string("../README.md").expect("README.md must exist");
        assert!(readme.contains("fav test"),
            "README.md should mention fav test");
    }
}
```

## Step 5: `cargo clean` + `fav/tmp/hello.fav` 復元

ビルドキャッシュをリセットする（宣言バージョン慣例）。

```
# fav/ ディレクトリで実行
cargo clean
```

**`cargo clean` 後は `fav/tmp/hello.fav` を必ず復元すること。**
`cargo clean` によりこのファイルが削除され、`bootstrap_c2_artifact_roundtrip` テストが FAIL する既知問題（v30.0.0 クリーンアップで判明）。

復元内容:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

## Step 6: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3841 tests, 0 failures（最大ビルド時間がかかる）
```

## Step 7: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
# fav/ ディレクトリで実行
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
