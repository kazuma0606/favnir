# v36.7.0 spec — Great Expectations 互換エクスポート

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.7.0 |
| テーマ | Great Expectations 互換エクスポート |
| 前提 | v36.6.0 COMPLETE — E0380〜E0384 スキーマ不整合エラーコード実装済み |
| 完了条件 | `v36700_tests` 全テスト pass・`cargo test` 0 failures（≥ 2687 件） |

## 背景と目的

Great Expectations（GE）は Python エコシステムで広く使われているデータ品質フレームワーク。
`fav validate` の出力を GE の Expectation Suite 形式（JSON）にエクスポートすることで、
既存の GE ワークフローと Favnir の schema 定義を連携できるようになる。

`fav validate --schema orders.fav data.csv --export ge --output suite.json` を実行すると、
`suite.json` に GE 互換の JSON が書き出される。

## エクスポート形式（GE Expectation Suite）

生成する JSON の構造:

```json
{
  "expectation_suite_name": "<schema_name>",
  "expectations": [
    { "expectation_type": "expect_column_to_exist", "kwargs": { "column": "<field1>" } },
    { "expectation_type": "expect_column_to_exist", "kwargs": { "column": "<field2>" } }
  ],
  "meta": {
    "great_expectations_version": "0.18.0",
    "generated_by": "fav validate"
  }
}
```

`schema_name` は `.fav` ファイル内の `schema Name { ... }` の `Name` 部分。
各フィールドにつき `expect_column_to_exist` expectation を1件生成する。

## 実装スコープ

### 1. `fav/src/driver.rs` — `export_ge_suite` 追加

`cmd_validate` の直前（`// ── fav validate (v36.4.0)` セクション内）に純粋関数として追加:

```rust
/// schema フィールドリストから Great Expectations Expectation Suite JSON を生成する。
/// GE バージョン 0.18.0 互換形式（expect_column_to_exist のみ）。
pub fn export_ge_suite(schema_name: &str, field_names: &[String]) -> String {
    let expectations: Vec<String> = field_names
        .iter()
        .map(|f| {
            format!(
                r#"    {{ "expectation_type": "expect_column_to_exist", "kwargs": {{ "column": "{}" }} }}"#,
                f
            )
        })
        .collect();
    format!(
        r#"{{
  "expectation_suite_name": "{}",
  "expectations": [
{}
  ],
  "meta": {{
    "great_expectations_version": "0.18.0",
    "generated_by": "fav validate"
  }}
}}"#,
        schema_name,
        expectations.join(",\n")
    )
}
```

### 2. `fav/src/driver.rs` — `cmd_validate` シグネチャ変更

`cmd_validate(schema_file: Option<&str>, data_file: Option<&str>)` に
`export_fmt: Option<&str>` と `output_file: Option<&str>` を追加:

```rust
pub fn cmd_validate(
    schema_file: Option<&str>,
    data_file: Option<&str>,
    export_fmt: Option<&str>,
    output_file: Option<&str>,
)
```

`has_errors` が false（全スキーマ照合 OK）かつ `export_fmt == Some("ge")` の場合のみエクスポートを実行:

```rust
if !has_errors {
    if export_fmt == Some("ge") {
        let out_path = output_file.unwrap_or("suite.json");
        // 最初のスキーマ定義を使用してエクスポート
        if let Some(sd) = schema_defs.first() {
            let field_names: Vec<String> = sd.fields.iter().map(|(n, _)| n.clone()).collect();
            let json = export_ge_suite(&sd.name, &field_names);
            // write_text_file は &Path 引数・Result 返り値のため変換が必要
            write_text_file(std::path::Path::new(out_path), &json)
                .unwrap_or_else(|e| eprintln!("error writing {}: {}", out_path, e));
            println!("exported GE suite to {}", out_path);
        }
    }
}
```

### 3. `fav/src/main.rs` — `--export` / `--output` フラグ追加

`Some("validate") =>` アームの引数解析に追加:

```rust
"--export" => {
    i += 1;
    export_fmt = args.get(i).cloned();
}
"--output" => {
    i += 1;
    output_file = args.get(i).cloned();
}
```

`cmd_validate` 呼び出しを:
```rust
cmd_validate(schema_file.as_deref(), data_file.as_deref(), export_fmt.as_deref(), output_file.as_deref());
```
に更新。

### 4. `fav/src/driver.rs` — `v36700_tests` テストモジュール

`v36600_tests` の閉じ `}` の後に追加。

## v36700_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_7_0` | Cargo.toml に `"36.7.0"` が含まれる |
| `changelog_has_v36_7_0` | CHANGELOG.md に `[v36.7.0]` が含まれる |
| `ge_suite_export_generates_json` | `export_ge_suite` が GE 互換 JSON を生成する |

### テスト実装

```rust
// ── v36700_tests (v36.7.0) — Great Expectations 互換エクスポート ──────────────
#[cfg(test)]
mod v36700_tests {
    use super::export_ge_suite;

    #[test]
    fn cargo_toml_version_is_36_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.7.0"), "Cargo.toml must contain version 36.7.0");
    }
    #[test]
    fn changelog_has_v36_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.7.0]"), "CHANGELOG.md must contain [v36.7.0]");
    }
    #[test]
    fn ge_suite_export_generates_json() {
        let fields = vec!["id".to_string(), "amount".to_string()];
        let json = export_ge_suite("Orders", &fields);
        assert!(json.contains("\"expectation_suite_name\": \"Orders\""));
        assert!(json.contains("expect_column_to_exist"));
        assert!(json.contains("\"column\": \"id\""));
        assert!(json.contains("\"column\": \"amount\""));
        assert!(json.contains("\"great_expectations_version\": \"0.18.0\""));
        assert!(json.contains("\"generated_by\": \"fav validate\""));
    }
}
```

## 注意事項

### `export_ge_suite` は純粋関数

ファイル I/O を行わない純粋関数とする。`write_text_file` の呼び出しは `cmd_validate` 内でのみ行う。
これによりテストが副作用なしで実行できる。

### `cmd_validate` シグネチャ変更の影響

`cmd_validate` のシグネチャ変更により `main.rs` の呼び出し箇所を必ず更新する。
テストモジュール（`v36400_tests::cmd_validate_in_driver_rs`）は `pub fn cmd_validate` の存在確認のみで
シグネチャは検証しないため、影響なし。

### スコープ外（v36.8.0 以降）

- `expect_column_values_to_not_be_null` 等の追加 expectation 型
- `--export` フォーマットとして `ge` 以外（dbt、Soda 等）
- GE Cloud との直接連携

## ロードマップとの整合

ロードマップ v36.7.0 完了条件:「Rust テスト 1 件」
本 spec では 3 テストを追加する（ロードマップの最小要件 1 件を上回る）。
ロードマップの完了条件に記載の件数（1 件）は更新しない（最小要件値として維持）。
完了ステータス（✅）の更新は tasks.md T7 で行う。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `export_ge_suite` が GE 互換 JSON を生成する | `ge_suite_export_generates_json` テスト |
| 2 | `CHANGELOG.md` に `[v36.7.0]` が含まれる | `changelog_has_v36_7_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.7.0` | `cargo_toml_version_is_36_7_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2687） | `cargo test` 実行結果（v36.6.0 実績 2686 + v36700_tests 3 件 = 2689 ※ 余裕を持たせ ≥ 2687 とする） |

> 注: v36600_tests スタブ化はテスト件数を変えないため（関数ボディが空になるだけで関数自体は残る）、純粋に +3 件増加し 2689 件となる。≥ 2687 を完了条件とする。
