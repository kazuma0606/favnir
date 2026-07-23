# Plan: v48.0.0 — Standard Library 2.0 宣言 ★クリーンアップ

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v48000_tests` モジュール追加（4テスト）|
| `fav/Cargo.toml` | version → `"48.0.0"` |
| `CHANGELOG.md` | v48.0.0 マイルストーン宣言エントリ追加 |
| `MILESTONE.md` | v48.0.0 Standard Library 2.0 エントリ追加 |
| `README.md` | `"Standard Library 2.0"` への言及を追加 |
| `versions/current.md` | v48.0.0 に更新、進行中 v48.1.0 |
| `versions/v45-v50/v48.0.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### `fav/src/driver.rs` — `v48000_tests`

挿入位置: `v479000_tests` モジュールの直前。

```rust
// -- v48000_tests (v48.0.0) -- Standard Library 2.0 宣言 --
#[cfg(test)]
mod v48000_tests {
    #[test]
    fn cargo_toml_version_is_48_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"48.0.0\""),
            "Cargo.toml version should be 48.0.0"
        );
    }

    #[test]
    fn changelog_has_v48_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v48.0.0]"),
            "CHANGELOG.md should have v48.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_stdlib_v2() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Standard Library 2.0"),
            "MILESTONE.md should mention 'Standard Library 2.0'"
        );
    }

    #[test]
    fn readme_mentions_stdlib_v2() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Standard Library 2.0"),
            "README.md should mention 'Standard Library 2.0'"
        );
    }
}
```

### `MILESTONE.md`

先頭（既存の v47.0.0 エントリの直前）に追加:

    ## v48.0.0 — Standard Library 2.0（2026-07-18）

    > 「List・String・Float・Option・Result・Map の主要操作が揃い、
    >  外部ライブラリなしに実務的なデータ変換が書ける。
    >
    >  これが Favnir v48.0 — Standard Library 2.0 の姿である。」

    v48.0.0 をもって、Favnir の **Standard Library 2.0** を正式に宣言する。

    ### 達成コンポーネント（v47.1〜v47.9）

    | コンポーネント | バージョン | 内容 |
    |---|---|---|
    | `List.zip` / `List.chunk` | v47.1.0 | 2 リストのペア化・n 要素分割 |
    | `List.flat_map` / `List.group_by` / `List.dedupe` | v47.2.0 | flatten+map・グループ化・重複除去 |
    | `List.scan` / `List.take_while` / `List.drop_while` | v47.3.0 | 累積値リスト・先頭条件フィルタ |
    | `String.pad_left` / `String.trim_start` / `String.repeat` | v47.4.0 | パディング・トリム・繰り返し |
    | `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` | v47.5.0 | 浮動小数点・整数拡張 |
    | `Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` | v47.6.0 | Option コンビネータ |
    | `Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` | v47.7.0 | Result コンビネータ |
    | `Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` | v47.8.0 | Map 拡充 |
    | stdlib ドキュメント（`float.mdx` / `v2.mdx` / 各 MDX 更新） | v47.9.0 | Standard Library 2.0 全関数索引 |

    ---

### `README.md`

v47.0 エントリ（line 122〜123 付近）の直後に追記:

    **v48.0（2026-07-18）で、[Standard Library 2.0](./MILESTONE.md) マイルストーンを宣言しました。**
    List / String / Float / Option / Result / Map の主要操作が揃い、外部ライブラリなしに実務的なデータ変換が書ける Standard Library 2.0 が完成しました。

### `CHANGELOG.md`

先頭（v47.9.0 エントリの直前）に追加:

    ## [v48.0.0] — 2026-07-18 — Standard Library 2.0 宣言

    ### Added
    - `MILESTONE.md` に v48.0.0 Standard Library 2.0 エントリ追加
    - `README.md` に `"Standard Library 2.0"` 言及を追加
    - `driver.rs`: `v48000_tests` 追加（`cargo_toml_version_is_48_0_0` / `changelog_has_v48_0_0` / `milestone_has_stdlib_v2` / `readme_mentions_stdlib_v2` 4テスト）

    ### Changed
    - `Cargo.toml` version: `47.9.0` → `48.0.0`

---

## 実装順序

1. `MILESTONE.md` に v48.0.0 エントリを追加（`"Standard Library 2.0"` を含む）
2. `README.md` に `"Standard Library 2.0"` を追加
3. `driver.rs` に `v48000_tests` を `v479000_tests` 直前に追加
4. `Cargo.toml` version → `"48.0.0"`
5. `CHANGELOG.md` v48.0.0 エントリ追加
6. `cargo test` で 3045 passed, 0 failed を確認
7. `cargo clippy -- -D warnings` クリーン確認
8. `versions/current.md` 更新（v48.0.0、次 v48.1.0）
9. `versions/roadmap/roadmap-v45.1-v50.0.md` に v48.0.0 完了を反映
10. `tasks.md` COMPLETE に更新
11. **`cargo clean`** ★クリーンアップ実施
12. `cargo clean` 後に `fav/tmp/hello.fav` の存在を確認（消えていた場合は復元）
13. `cargo test` 再実行（クリーン後も 3045 passed, 0 failed を確認）

---

## 注意事項

- **`include_str!` パスまとめ（`fav/src/driver.rs` 起点）**:

  | インクルード先 | パス |
  |---|---|
  | `fav/Cargo.toml` | `"../Cargo.toml"` |
  | `favnir/CHANGELOG.md` | `"../../CHANGELOG.md"` |
  | `favnir/MILESTONE.md` | `"../../MILESTONE.md"` |
  | `favnir/README.md` | `"../../README.md"` |

- `cargo_toml_version_is_48_0_0` テストは Cargo.toml 更新（step 4）より前は FAIL する。step 6 で全通過を確認。
- `cargo clean` は全テスト・clippy 通過確認後に実施する。
- `cargo clean` 後は `fav/tmp/hello.fav` が消えていないか確認（消えた場合は `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` の内容で復元）。
- マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は step 9 で実施する（v48.0.0 完了時）。
