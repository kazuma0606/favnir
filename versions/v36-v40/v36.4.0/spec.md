# v36.4.0 spec — `fav validate` コマンド

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.4.0 |
| テーマ | `fav validate` コマンド |
| 前提 | v36.3.0 COMPLETE — W025 schema_mismatch lint ルール実装済み |
| 完了条件 | `v36400_tests` 全テスト pass・`cargo test` 0 failures（≥ 2676 件） |

## 背景と目的

v36.1〜v36.3 で「スキーマ定義・品質ブロック・lint 静的検査」を整備した。
本バージョンは **実行時の CSV データとスキーマ定義の整合性を検証する** `fav validate` コマンドを追加する。

```bash
fav validate --schema orders.fav data.csv
```

- `orders.fav` に含まれる `schema Orders { ... }` 定義を読み込む
- `data.csv` のヘッダー行とスキーマのフィールド名を照合する
- 欠損カラムをエラーとして報告する（exit 1）
- すべてのフィールドが揃っている場合は `ok` を表示（exit 0）

## 実装スコープ

### 1. `fav/src/driver.rs` — `validate_schema_against_headers` と `cmd_validate`

#### コアロジック（純粋関数 — テスト可能）

```rust
/// schema フィールド名リストと CSV ヘッダーリストを照合する（純粋関数）
pub fn validate_schema_against_headers(
    schema_field_names: &[String],
    csv_headers: &[String],
) -> Vec<String> {
    let mut errors = Vec::new();
    for field in schema_field_names {
        if !csv_headers.contains(field) {
            errors.push(format!("missing column: `{}`", field));
        }
    }
    errors
}
```

#### CSV ヘッダー読み取り（プライベートヘルパー）

```rust
fn read_csv_headers(path: &str) -> Vec<String> {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {}", path, e);
        process::exit(1);
    });
    let first_line = content.lines().next().unwrap_or("");
    first_line.split(',').map(|h| h.trim().to_string()).collect()
}
```

#### `cmd_validate` 関数

```rust
// ── fav validate ──────────────────────────────────────────────────────────────

pub fn cmd_validate(schema_file: Option<&str>, data_file: Option<&str>) {
    use crate::ast::Item;

    let schema_path = schema_file.unwrap_or_else(|| {
        eprintln!("error: --schema <file.fav> is required");
        process::exit(1);
    });
    let data_path = data_file.unwrap_or_else(|| {
        eprintln!("error: data file (CSV) is required");
        process::exit(1);
    });

    // 1. スキーマファイルをパース
    let schema_src = load_file(schema_path);
    let program = Parser::parse_str(&schema_src, schema_path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // 2. SchemaDef を収集
    let schema_defs: Vec<_> = program.items.iter().filter_map(|item| {
        if let Item::SchemaDef(sd) = item { Some(sd) } else { None }
    }).collect();

    if schema_defs.is_empty() {
        eprintln!("error: no `schema` definitions found in {}", schema_path);
        process::exit(1);
    }

    // 3. CSV ヘッダーを読み取る
    let csv_headers = read_csv_headers(data_path);

    // 4. 各スキーマ定義を照合してレポート
    let mut has_errors = false;
    for sd in &schema_defs {
        let field_names: Vec<String> = sd.fields.iter().map(|(n, _)| n.clone()).collect();
        let errors = validate_schema_against_headers(&field_names, &csv_headers);
        if errors.is_empty() {
            println!("{}: schema `{}`: ok", data_path, sd.name);
        } else {
            has_errors = true;
            for err in &errors {
                eprintln!("{}: schema `{}`: {}", data_path, sd.name, err);
            }
        }
    }

    if has_errors {
        process::exit(1);
    }
}
```

### 2. `fav/src/main.rs` — `fav validate` ルーティングと import

#### import 追加

`use driver::{ ..., cmd_validate, ... };` に `cmd_validate` を追加する。

#### ルーティング追加

`Some("lint") =>` アームの直後に `Some("validate") =>` アームを追加する:

```rust
// 引数形式: fav validate --schema <schema.fav> <data.csv>
// （--schema フラグは data ファイルより前に置くこと）
Some("validate") => {
    let mut schema_file: Option<&str> = None;
    let mut data_file: Option<&str> = None;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--schema" => {
                i += 1;
                schema_file = args.get(i).map(|s| s.as_str());
            }
            a if !a.starts_with('-') => {
                data_file = Some(a);
            }
            _ => {}
        }
        i += 1;
    }
    cmd_validate(schema_file, data_file);
}
```

### 3. `fav/src/driver.rs` — テストモジュール

`v36300_tests::cargo_toml_version_is_36_3_0` をスタブ化し、`v36400_tests` を追加。

## v36400_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_4_0` | Cargo.toml に `"36.4.0"` が含まれる |
| `changelog_has_v36_4_0` | CHANGELOG.md に `[v36.4.0]` が含まれる |
| `cmd_validate_in_driver_rs` | driver.rs に `cmd_validate` が含まれる |
| `validate_missing_column_reported` | 欠損カラムがあるとエラーリストに報告される |
| `validate_all_columns_present_ok` | 全カラムが揃うとエラーなし |

### テスト実装

```rust
// ── v36400_tests (v36.4.0) — fav validate コマンド ────────────────────────────
#[cfg(test)]
mod v36400_tests {
    // driver.rs 内のテストは `super::` で同ファイルの pub fn を参照する（既存テストと統一）
    use super::validate_schema_against_headers;

    #[test]
    fn cargo_toml_version_is_36_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.4.0"), "Cargo.toml must contain version 36.4.0");
    }
    #[test]
    fn changelog_has_v36_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.4.0]"), "CHANGELOG.md must contain [v36.4.0]");
    }
    #[test]
    fn cmd_validate_in_driver_rs() {
        let src = include_str!("driver.rs");
        assert!(src.contains("cmd_validate"), "driver.rs must contain cmd_validate");
    }
    #[test]
    fn validate_missing_column_reported() {
        let fields = vec!["id".to_string(), "amount".to_string()];
        let headers = vec!["id".to_string()]; // `amount` が欠損
        let errors = validate_schema_against_headers(&fields, &headers);
        assert!(!errors.is_empty(), "expected error for missing column");
        assert!(
            errors.iter().any(|e| e.contains("amount")),
            "error must mention `amount`: {:?}", errors
        );
    }
    #[test]
    fn validate_all_columns_present_ok() {
        let fields = vec!["id".to_string(), "amount".to_string()];
        let headers = vec!["id".to_string(), "amount".to_string(), "extra".to_string()];
        let errors = validate_schema_against_headers(&fields, &headers);
        assert!(errors.is_empty(), "expected no errors: {:?}", errors);
    }
}
```

## 注意事項

### `load_file` の参照

`cmd_validate` 内で使用する `load_file(path)` は driver.rs 内で定義済みのプライベートヘルパー（`fn load_file`）。
`cmd_validate` は必ず driver.rs に直接追加すること（別ファイルからは `load_file` にアクセスできない）。

### `Parser` の import について

`Parser` は driver.rs ファイル先頭（`use crate::frontend::parser::Parser;`）でモジュールスコープ全体にインポート済み。
`cmd_validate` 関数内でローカル `use` を書く必要はない（書いても重複 import エラーにはならないが不要）。

### `read_csv_headers` のカバレッジ制限

`read_csv_headers` は `split(',')` による単純なパース（RFC 4180 準拠のクォーテッドフィールド非対応）。
v36.4.0 では純粋関数 `validate_schema_against_headers` のみをユニットテストし、`read_csv_headers` 自体のテストはスコープ外とする。この制限は既知として文書化する。

### テストモジュール内の `use` パス

driver.rs 内の `#[cfg(test)] mod v36400_tests` から同ファイルの関数を参照する場合は `super::` を使う:
```rust
use super::validate_schema_against_headers;
```
`crate::driver::validate_schema_against_headers` 形式でも動作するが、既存テストモジュールと統一するため `super::` を使うこと。

### スコープ外（v36.5.0 以降）

- Parquet ファイルのヘッダー検証（Arrow RecordBatch の schema 読み取り）
- フィールドの**型**照合（v36.4.0 はカラム名のみが対象）
- `expect` ルールの実行時評価（`not_empty` / `all(|r| ...)` 等）
- `--export ge`（Great Expectations 形式出力）— v36.7.0 で実装予定

## ロードマップとの整合

ロードマップ v36.4.0 完了条件:「`fav validate` コマンドが動作する / Rust テスト 2 件」
本 spec では 5 テストを追加する（ロードマップの最小要件 2 件を上回る）。
ロードマップの件数は更新しない（最小要件値として維持）。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `driver.rs` に `cmd_validate` と `validate_schema_against_headers` が含まれる | `cmd_validate_in_driver_rs` テスト |
| 2 | `CHANGELOG.md` に `[v36.4.0]` が含まれる | `changelog_has_v36_4_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.4.0` | `cargo_toml_version_is_36_4_0` テスト |
| 4 | 欠損カラムがあるとエラーが報告される | `validate_missing_column_reported` テスト |
| 5 | 全カラムが揃うとエラーなし | `validate_all_columns_present_ok` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2676） | `cargo test` 実行結果 |
