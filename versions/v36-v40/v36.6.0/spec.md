# v36.6.0 spec — E0380〜E0384 スキーマ不整合エラーコード

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.6.0 |
| テーマ | E0380〜E0384 スキーマ不整合エラーコード |
| 前提 | v36.5.0 COMPLETE — Data Contract 規約実装済み |
| 完了条件 | `v36600_tests` 全テスト pass・`cargo test` 0 failures（≥ 2686 件） |

## 背景と目的

v36.1〜v36.5 で「スキーマ定義・品質ブロック・lint・validate・contract check」を整備した。
本バージョンは **スキーマ不整合を表す専用エラーコード E0380〜E0384** を `error_catalog.rs` に追加する。

これにより `fav explain E0380` でエラーの詳細な説明と修正方法を表示できるようになる。

### 既存 E050x コードとの関係

`error_catalog.rs` には既存の `E0501`（`schema field missing`）・`E0502`（`schema type mismatch`）が存在する。
これらは **ランタイム CSV/JSON 変換時の外部スキーマ参照エラー**（カテゴリ `modules`）であり、
今回追加する E0380〜E0384 は **コンパイル時のインライン `schema {}` 構文検証エラー**（カテゴリ `schema`）として位置づけが異なる。
用途・カテゴリが異なるため重複ではなく、双方とも維持する。

## エラーコード定義

| コード | タイトル（title） | カテゴリ | 意味 |
|---|---|---|---|
| E0380 | `schema_field_missing` | schema | schema 定義の必須フィールドがデータに存在しない |
| E0381 | `schema_type_mismatch` | schema | schema フィールドの型がデータ値と一致しない |
| E0382 | `schema_constraint_violated` | schema | schema フィールドの `where` 制約をデータ値が満たさない |
| E0383 | `schema_duplicate_key` | schema | schema 定義にフィールド名が重複している |
| E0384 | `schema_extra_field` | schema | データに schema 未定義のフィールドが含まれている |

## 実装スコープ

### 1. `fav/src/error_catalog.rs` — E0380〜E0384 エントリ追加

`ERROR_CATALOG` 配列の末尾（`E0903` エントリの `},` の後、配列の閉じ `];` の前）に追加する。

```rust
    // ── E038x: スキーマ不整合 (v36.6.0) ────────────────────────────────────
    ErrorEntry {
        code: "E0380",
        title: "schema_field_missing",
        category: "schema",
        description: "A required field defined in the schema is missing from the data.",
        example: "schema Orders { id: Int, amount: Float }\n// data: { id: 1 }  // E0380: missing `amount`",
        fix: "Add the missing field to your data source, or remove it from the schema definition.",
    },
    ErrorEntry {
        code: "E0381",
        title: "schema_type_mismatch",
        category: "schema",
        description: "A schema field has a value whose type does not match the declared type.",
        example: "schema Orders { amount: Float }\n// data: { amount: \"not-a-number\" }  // E0381: expected Float",
        fix: "Fix the data value to match the declared type, or update the schema field type.",
    },
    ErrorEntry {
        code: "E0382",
        title: "schema_constraint_violated",
        category: "schema",
        description: "A schema field value violates its `where` constraint.",
        example: "schema Orders { amount: Float where { amount >= 0.0 } }\n// data: { amount: -1.0 }  // E0382",
        fix: "Fix the data value to satisfy the constraint, or relax the schema constraint.",
    },
    ErrorEntry {
        code: "E0383",
        title: "schema_duplicate_key",
        category: "schema",
        description: "A field name appears more than once in the schema definition.",
        example: "schema Orders { id: Int, id: String }  // E0383: duplicate field `id`",
        fix: "Remove or rename the duplicate field in the schema definition.",
    },
    ErrorEntry {
        code: "E0384",
        title: "schema_extra_field",
        category: "schema",
        description: "The data contains a field that is not defined in the schema.",
        example: "schema Orders { id: Int }\n// data: { id: 1, unknown: \"x\" }  // E0384: extra field `unknown`",
        fix: "Remove the extra field from your data, or add it to the schema definition.",
    },
```

### 2. `fav/src/driver.rs` — テストモジュール

`v36500_tests::cargo_toml_version_is_36_5_0` をスタブ化し、`v36600_tests` を追加。

## v36600_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_6_0` | Cargo.toml に `"36.6.0"` が含まれる |
| `changelog_has_v36_6_0` | CHANGELOG.md に `[v36.6.0]` が含まれる |
| `error_catalog_has_schema_codes` | E0380〜E0384 が ERROR_CATALOG に存在する |
| `e0380_lookup_returns_correct_title` | `lookup("E0380")` が `schema_field_missing` を返す |
| `e0384_lookup_returns_correct_title` | `lookup("E0384")` が `schema_extra_field` を返す |

### テスト実装

```rust
// ── v36600_tests (v36.6.0) — E0380〜E0384 スキーマ不整合エラーコード ──────────
#[cfg(test)]
mod v36600_tests {
    use crate::error_catalog::{lookup, ERROR_CATALOG};

    #[test]
    fn cargo_toml_version_is_36_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.6.0"), "Cargo.toml must contain version 36.6.0");
    }
    #[test]
    fn changelog_has_v36_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.6.0]"), "CHANGELOG.md must contain [v36.6.0]");
    }
    #[test]
    fn error_catalog_has_schema_codes() {
        let codes: Vec<&str> = ERROR_CATALOG.iter().map(|e| e.code).collect();
        for code in &["E0380", "E0381", "E0382", "E0383", "E0384"] {
            assert!(codes.contains(code), "ERROR_CATALOG missing {}", code);
        }
    }
    #[test]
    fn e0380_lookup_returns_correct_title() {
        let entry = lookup("E0380").expect("E0380 must be in catalog");
        assert_eq!(entry.title, "schema_field_missing");
        assert_eq!(entry.category, "schema");
    }
    #[test]
    fn e0384_lookup_returns_correct_title() {
        let entry = lookup("E0384").expect("E0384 must be in catalog");
        assert_eq!(entry.title, "schema_extra_field");
        assert_eq!(entry.category, "schema");
    }
}
```

## 注意事項

### `ERROR_CATALOG` の挿入位置

`ERROR_CATALOG` 配列の閉じ `];` の直前に追加する（末尾エントリの後）。
コメントセクション `// ── E038x: スキーマ不整合 (v36.6.0) ──` で区切ること。

### 既存の `ErrorEntry` フィールド

`ErrorEntry` は `code` / `title` / `category` / `description` / `example` / `fix` の6フィールドを持つ。
全フィールドを埋めること（漏れるとコンパイルエラー）。

### スコープ外（v36.7.0 以降）

- E0380〜E0384 を実際に **発行する** ロジックの実装（v36.6.0 はカタログ定義のみ）
- `fav validate` コマンドへの E038x 統合

## ロードマップとの整合

ロードマップ v36.6.0 完了条件:「`error_catalog.rs` に定義済み / Rust テスト 2 件」
本 spec では 5 テストを追加する（ロードマップの最小要件 2 件を上回る）。
ロードマップの完了条件に記載の件数（2 件）は更新しない（最小要件値として維持）。
完了ステータス（✅）の更新は tasks.md T7 で行う。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `error_catalog.rs` に E0380〜E0384 が定義されている | `error_catalog_has_schema_codes` テスト |
| 2 | `CHANGELOG.md` に `[v36.6.0]` が含まれる | `changelog_has_v36_6_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.6.0` | `cargo_toml_version_is_36_6_0` テスト |
| 4 | `lookup("E0380")` が正しい title を返す | `e0380_lookup_returns_correct_title` テスト |
| 5 | `lookup("E0384")` が正しい title を返す | `e0384_lookup_returns_correct_title` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2686） | `cargo test` 実行結果（v36.5.0 実績 2681 + v36600_tests 5 件 = 2686） |
