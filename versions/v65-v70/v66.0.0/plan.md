# v66.0.0 実装計画 — Math & Science Foundation 宣言 ★クリーンアップ

Version: 66.0.0
Status: 未着手
Base tests: 3471
Target tests: 3475

---

## 実装ステップ

### Step 1: `driver.rs` — `v66000_tests` 追加

`// -- v65900_tests (v65.9.0)` コメントの直前に `v66000_tests` を挿入（4 テスト関数）。

**重要**: 4 テストは Cargo.toml / CHANGELOG.md / MILESTONE.md / README.md の更新後でないとすべて PASS しない。
Step 2〜4 の更新を先に行い、最後にテストで確認する。

### Step 2: `fav/Cargo.toml` — version 更新

```toml
version = "66.0.0"
```

### Step 3: `MILESTONE.md` — v66.0.0 エントリを先頭に追加

既存 `## v65.0.0` エントリの直前に v66.0.0 セクションを挿入。
"Math & Science" を含む必要がある。

### Step 4: `README.md` — v66.0.0 宣言を追加

既存バージョン履歴の先頭（または適切な箇所）に v66.0.0 の言及を追加。
`"Math & Science"` または `"v66.0"` を含む必要がある。

### Step 5: `CHANGELOG.md` — v66.0.0 エントリを先頭に追加

既存 `## [v65.0.0]` エントリの直前に v66.0.0 エントリを挿入。
v65.1〜v65.9 の全変更を一括追記（CHANGELOG 方針: サブバージョンでは追記せず宣言時に一括）。
`"v66.0.0"` を含む必要がある。

### Step 6: テスト確認

```bash
cargo test --bin fav v66000_tests   # 4 件 PASS 確認
```

### Step 7: `cargo clean` ★クリーンアップ

```bash
cargo clean
```

**実行後、`fav/tmp/hello.fav` を必ず確認・復元すること。**

`hello.fav` の正しい内容:
```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

### Step 8: フルテスト

```bash
cargo test -j 8 -- --test-threads=8
```

3475 tests passed, 0 failed を確認。

---

## `driver.rs` 挿入コード

```rust
// -- v66000_tests (v66.0.0) -- Math & Science Foundation 宣言 --
#[cfg(test)]
mod v66000_tests {
    #[test]
    fn cargo_toml_version_is_66_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"66.0.0\""),
            "Cargo.toml should have version 66.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v66_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v66.0.0"), "CHANGELOG.md should mention v66.0.0");
    }

    #[test]
    fn milestone_has_math_science() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Math & Science"),
            "MILESTONE.md should contain 'Math & Science'"
        );
    }

    #[test]
    fn readme_mentions_math_science() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Math & Science") || readme.contains("v66.0"),
            "README.md should mention Math & Science Foundation or v66.0"
        );
    }
}
```

---

## include_str! パス一覧

| ファイル | driver.rs 相対パス | 場所 |
|---|---|---|
| `fav/Cargo.toml` | `../Cargo.toml` | `fav/src/` の1つ上 |
| `CHANGELOG.md` | `../../CHANGELOG.md` | リポジトリルート |
| `MILESTONE.md` | `../../MILESTONE.md` | リポジトリルート |
| `README.md` | `../../README.md` | リポジトリルート |

---

## リスク・注意点

- **`cargo clean` 後の `hello.fav` 消失**: `fav/tmp/hello.fav` が削除される場合がある。復元を忘れると `bootstrap_c2_artifact_roundtrip` FAIL。
- **テスト数 +4**: マイルストーン宣言は +4（サブバージョンの +2 と異なる）
- **フルテストコマンド**: `cargo test -j 8 -- --test-threads=8`（`--bin fav` なし。全テストを実行）
- **Cargo.toml の更新は本バージョンのみ**: sub-version（v66.1〜v66.9）では更新しない
