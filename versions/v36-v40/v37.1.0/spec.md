# v37.1.0 spec — 境界付きジェネリクス実用強化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.1.0 |
| テーマ | 境界付きジェネリクス実用強化 — `Deserialize` 制約の明示化 + Generic Rune 追加 |
| 前提 | v37.0.0 COMPLETE — Data Quality First マイルストーン宣言済み |
| 完了条件 | `v37100_tests` 全テスト pass・`cargo test` 0 failures（≥ 2707 件） |

## 背景と目的

v32.1 で `T with Ord/Eq/Hash` 形式の境界付きジェネリクスを実装し、
v32.2 以降で `Serialize` / `Clone` 制約も `middle/checker.rs` の `type_implements_bound` に追加済み。

**現状の問題:**
- `Serialize` は `"Eq" | "Serialize" | "Clone" => true` で明示的に有効
- `Deserialize` は `_ => true`（フォールスルー）で通るが **意図が不明確で将来のバグリスク**
- Generic Rune（型パラメータ付き Rune 関数）の具体的な参照実装がない

**今バージョンで行うこと:**
1. `Deserialize` を `type_implements_bound` の明示的な有効制約として追加
2. `T with Deserialize` が型チェックを通ることをテストで保証
3. Generic Rune の参照実装 (`runes/generic/`) を追加

## 実装スコープ

### 1. `fav/src/middle/checker.rs` — `Deserialize` を明示的な有効制約に追加

`type_implements_bound` 関数（行 7596〜7609）を変更:

**変更前（行 7599）:**
```rust
"Eq" | "Serialize" | "Clone" => true,
```

**変更後:**
```rust
"Eq" | "Serialize" | "Deserialize" | "Clone" => true,
```

これにより `T with Deserialize` が `Ord/Eq/Hash/Serialize` と同様に
明示的に有効な型制約として扱われる。

### 2. Generic Rune 追加（`runes/generic/`）

型パラメータ付き汎用関数の参照実装として追加。

**`runes/generic/generic.fav`:**
```favnir
// Generic Rune — 型パラメータ付き汎用 ETL 関数

fn export_json<T with Serialize>(data: List<T>) -> String {
  Json.encode(data)
}

fn import_records<T with Deserialize>(src: String) -> List<T> {
  Json.decode(src)
}

fn sort_records<T with Ord>(records: List<T>) -> List<T> {
  List.sort(records)
}
```

**`runes/generic/rune.toml`:**
```toml
[rune]
name = "generic"
version = "0.1.0"
description = "Generic Rune — 型パラメータ付き汎用 ETL 関数"
```

### 3. `fav/src/driver.rs` — `v37100_tests` モジュール追加

```rust
// ── v37100_tests (v37.1.0) — 境界付きジェネリクス実用強化 ─────────────────────
#[cfg(test)]
mod v37100_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_37_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.1.0"), "Cargo.toml must contain version 37.1.0");
    }
    #[test]
    fn changelog_has_v37_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.1.0]"), "CHANGELOG.md must contain [v37.1.0]");
    }
    #[test]
    fn deserialize_constraint_type_checks() {
        // T with Deserialize が型チェックと実行を通ることを確認
        let result = run(r#"
fn decode<T with Deserialize>(src: String) -> String {
    src
}
fn main() -> String {
    decode("hello")
}
"#);
        assert_eq!(result, Value::Str("hello".to_string()));
    }
    #[test]
    fn generic_rune_file_exists() {
        // Generic Rune ファイルが存在し Deserialize を含むことを確認
        let src = include_str!("../../runes/generic/generic.fav");
        assert!(
            src.contains("Deserialize"),
            "runes/generic/generic.fav must contain Deserialize constraint"
        );
    }
}
```

**注意:** `deserialize_constraint_type_checks` は `run()` を使うため `use super::*;` が必要。

## 注意事項

### `v37000_tests` のスタブ化

`v37000_tests::cargo_toml_version_is_37_0_0` のライブアサーションを
`// Stubbed: version bumped to 37.1.0` に変更する（T4）。

### 実装箇所（`middle/checker.rs`）

`type_implements_bound` は `fav/src/middle/checker.rs` にある（`driver.rs` の `cmd_check` には制約バリデーションリストなし）。
T0 で行番号を確認してから Edit する。

### `use super::*` の要否

`deserialize_constraint_type_checks` は `run()` と `Value` を使うため、`v37100_tests` モジュールには `use super::*;` が必要。
（v36900_tests や v37000_tests は `include_str!` のみで不要だったが、本バージョンは `run()` 使用のため必要）

### Generic Rune テストの自己参照リスクなし

`generic_rune_file_exists` は `include_str!("../../runes/generic/generic.fav")` を読む。
`driver.rs` ソースを読むわけではないため、自己参照の問題は発生しない。

### スコープ外（v37.2 以降）

- `Deserialize` の実際の型推論への統合（制約伝播・実行時デシリアライズ）
- Generic Rune のコンパイラ完全対応（型パラメータの実行時解決）
- `rune.toml` の存在テスト（今バージョンでは `generic.fav` 内容テストのみ）

## ロードマップとの整合

ロードマップ v37.1.0:「v32.1 実装（`T with Ord/Eq/Hash`）に `Serialize` / `Deserialize` 制約と Generic Rune 対応を追加。完了条件: 新制約が型チェックを通る / Rust テスト 4 件」

- `Serialize` は v32.2 時点で実装済み → 本バージョンでは `Deserialize` の明示化に集中
- `Generic Rune` の参照実装を追加
- Rust テスト 4 件（ロードマップ指定通り）

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.1.0` | `cargo_toml_version_is_37_1_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.1.0]` が含まれる | `changelog_has_v37_1_0` テスト |
| 3 | `T with Deserialize` が型チェックと実行を通る | `deserialize_constraint_type_checks` テスト |
| 4 | `runes/generic/generic.fav` が `Deserialize` を含む | `generic_rune_file_exists` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2707） | `cargo test` 実行結果（v37.0.0 実績 2703 + v37100_tests 4 件 = 2707） |
