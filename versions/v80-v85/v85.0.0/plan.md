# plan: v85.0.0 — Favnir 4.0 宣言 ★クリーンアップ

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,927 tests, 0 failures を確認する（前提: v84.9.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
- `fav/src/driver.rs` に `mod v84900_tests` が存在することを確認する

### Step 2: cargo clean

- `cargo clean` を `fav/` ディレクトリで実行する
- `fav/tmp/hello.fav` が消えていないことを確認する（`bootstrap_c2_artifact_roundtrip` テスト用）
  - 消えていた場合は以下の内容で復元する:
    ```
    fn add(a: Int, b: Int) -> Int { a + b }
    fn main() -> Bool { add(1, 2) == 3 }
    ```

### Step 3: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "84.0.0"` を `version = "85.0.0"` に更新する

### Step 4: CHANGELOG 更新

- `CHANGELOG.md` の先頭に v85.0.0 エントリを追加する:
  ```
  ## v85.0.0 — Favnir 4.0 宣言 ★クリーンアップ（2026-08-22）
  - cargo clean でビルドキャッシュを削除
  - Cargo.toml version を 85.0.0 に更新
  - MILESTONE.md / README.md に Favnir 4.0 宣言を追記
  - versions/current.md を v85.0.0 に更新
  - roadmap-v84.1-v85.0.md Sprint 5 テーブルを全行「完了」に更新
  - v85000_tests 4 件追加（3927 → 3931）
  ```

### Step 5: MILESTONE.md 更新

- `MILESTONE.md` の先頭に v85.0.0 / Favnir 4.0 宣言のマイルストーンを追加する
- `"Favnir 4.0"` という文字列が含まれるようにする

### Step 6: README.md 更新

- `README.md` の Latest 欄を v85.0.0 / Favnir 4.0 宣言に更新する
- `"Favnir 4.0"` という文字列が含まれるようにする

### Step 7: versions/current.md 更新

- `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認する
- `versions/current.md` の現行バージョンを `v85.0.0` に更新する

### Step 8: ロードマップ更新

- `versions/roadmap/roadmap-v84.1-v85.0.md` の Sprint 5 バージョン一覧テーブルを全行「完了」に更新する
- `roadmap-v84.1-v85.0.md` 完了条件のテスト数を `3927 + 4 = 3931` に修正する
- `roadmap-v84.1-v85.0.md` テスト数推移テーブルの v85.0.0 行を `3,931` に修正する
- `versions/roadmap/roadmap-v80.1-v85.0.md` の Sprint 5 テーブル v85.0.0 行を「完了」に更新する
- `roadmap-v80.1-v85.0.md` の v85.0.0 完了条件テスト数を `3915 + 4 = 3919` → `3927 + 4 = 3931` に修正する

### Step 9: driver.rs に v85000_tests を追加

`mod v84900_tests { ... }` の直後に以下を追加する:

```rust
#[cfg(test)]
mod v85000_tests {
    #[test]
    fn cargo_toml_version_is_85_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"85.0.0\""), "Cargo.toml should have version = \"85.0.0\"");
    }

    #[test]
    fn changelog_has_v85_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v85.0.0"), "CHANGELOG.md should mention v85.0.0");
    }

    #[test]
    fn milestone_has_favnir_4() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Favnir 4.0"), "MILESTONE.md should mention Favnir 4.0");
    }

    #[test]
    fn readme_mentions_favnir_4() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Favnir 4.0"), "README.md should mention Favnir 4.0");
    }
}
```

> **パス注記**: `include_str!` のパス起点は `fav/src/`。
> - `Cargo.toml` は `fav/Cargo.toml` → `../Cargo.toml`（1 段上の `fav/` に存在）
> - ルートファイルは `../../CHANGELOG.md` 等

### Step 10: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,931 tests, 0 failures を確認する。

### Step 11: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
