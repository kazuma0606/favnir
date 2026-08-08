# v68.0.0 実装計画

## 実施順序

依存関係: Cargo.toml → MILESTONE.md → README.md → CHANGELOG.md → driver.rs → cargo clean → テスト

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
# 変更前
version = "67.0.0"
# 変更後
version = "68.0.0"
```

`cargo build` でエラーなしを確認。

---

## Step 2: `MILESTONE.md` — v68.0.0 エントリを先頭に追加

挿入前に実際の見出し文字列を確認する:

```bash
grep "## v67.0.0" MILESTONE.md
```

確認した見出しの直前に v68.0.0 エントリを挿入。
`"Developer Intelligence"` が含まれることを確認。

---

## Step 3: `README.md` — v68.0.0 宣言を追加

v67.0.0 の記述の直前に追加。
`"Developer Intelligence"` または `"v68.0"` を含めること（テスト要件）。

---

## Step 4: `CHANGELOG.md` — v68.0.0 エントリを先頭に追加

`## [v67.0.0]` の直前に挿入。
`"v68.0.0"` が含まれることを確認。

---

## Step 5: `driver.rs` — `v68000_tests` 追加

挿入位置: `// -- v67900_tests (v67.9.0) --` の直前

```rust
// -- v68000_tests (v68.0.0) -- Developer Intelligence 宣言 --
#[cfg(test)]
mod v68000_tests {
    #[test]
    fn cargo_toml_version_is_68_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"68.0.0\""),
            "Cargo.toml should have version 68.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v68_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v68.0.0"), "CHANGELOG.md should mention v68.0.0");
    }

    #[test]
    fn milestone_has_dev_intelligence() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Developer Intelligence"),
            "MILESTONE.md should contain 'Developer Intelligence'"
        );
    }

    #[test]
    fn readme_mentions_dev_intelligence() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Developer Intelligence") || readme.contains("v68.0"),
            "README.md should mention Developer Intelligence or v68.0"
        );
    }
}
```

---

## Step 6: `cargo clean` + `fav/tmp/hello.fav` 復元

```bash
cargo clean
```

復元内容（`fav/tmp/hello.fav`）:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

## Step 7: フルビルド・テスト

```bash
cargo test --bin fav v68000_tests  # 4 件 PASS を確認
cargo test -j 8 -- --test-threads=8  # 3519 tests passed を確認
```

---

## 注意事項

- `driver.rs` への `v68000_tests` 挿入は Step 1〜4 完了後（Cargo.toml / CHANGELOG / MILESTONE / README が更新されてから）に行う
- `cargo clean` は Step 5（テスト挿入）の後に実施する
- `cargo clean` 後は必ず `fav/tmp/hello.fav` を復元してから `cargo test` を実行する
