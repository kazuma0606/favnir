# v77.0.0 実装計画 — Data Provenance 1.0 宣言 ★クリーンアップ

Date: 2026-08-15

---

## Step 1: ★クリーンアップ — cargo clean + hello.fav 復元

```bash
cargo clean
```

`fav/tmp/hello.fav` が削除されるため復元する:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

## Step 2: MILESTONE.md 更新

`MILESTONE.md` の先頭（`## v76.0.0` の直前）に v77.0.0 エントリを挿入する。
エントリには宣言文と v76.1〜v76.9 達成内容の箇条書きを含める。

---

## Step 3: README.md 更新

`README.md` の `## v76.0` セクションの直前に `## v77.0 — Data Provenance 1.0 宣言` セクションを挿入する。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.0.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: Cargo.toml バージョン更新（テスト追加より先）

`76.9.0` → `77.0.0`

`cargo_toml_version_is_77_0_0` テストが `include_str!("../Cargo.toml")` を参照するため、
テストモジュール追加より **前** に Cargo.toml を更新する必要がある。

---

## Step 6: driver.rs — バージョン文字列一括置換

`driver.rs` 内の `76.9.0` を `77.0.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: driver.rs — v77000_tests モジュール追加

`use super::*` **不要**（外部ファイル参照のみ）。

```rust
#[cfg(test)]
mod v77000_tests {
    #[test]
    fn cargo_toml_version_is_77_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"77.0.0\""));
    }

    #[test]
    fn changelog_has_v77_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v77.0.0]"));
    }

    #[test]
    fn milestone_has_data_provenance() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Data Provenance"));
    }

    #[test]
    fn readme_mentions_provenance() {
        let content = include_str!("../../README.md");
        assert!(content.contains("Provenance") || content.contains("provenance"));
    }
}
```

---

## Step 8: versions/current.md 更新

進行中バージョンを v77.0.0 に、次に切る版を v77.1.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3736 tests all pass であることを確認する。
