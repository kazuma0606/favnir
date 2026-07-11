# v38.3.0 spec — `fav generate --from csv` 強化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.3.0 |
| テーマ | `fav generate --from csv` — CSV から Favnir type + schema + expect ブロックを生成 |
| 前提 | v38.2.0 COMPLETE — `fav generate --from sql` 実装済み（`Some("--from")` アーム追加済み） |
| 完了条件 | `v38300_tests` 全テスト pass・`cargo test` 0 failures（≥ 2754 件） |

## 背景と目的

v10.8.0 の `fav infer` は CSV から Favnir 型定義を生成する。
v38.3.0 では `fav generate --from csv <file>` として `schema` ブロック + `expect` バリデーションブロックまで含む
完全な出力を生成する。既存の `Some("--from")` アームに `"csv"` 分岐を追加する。

**想定動作**:
```bash
$ fav generate --from csv data.csv
// Generated from CSV
type Row = {
    id: String
    name: String
    email: String
}

schema Row {
    id: String
    name: String
    email: String
}

expect {
    all rows: Row -> rows.id != ""
}
```

## 実装スコープ

### 1. `fav/src/generate_csv.rs` — 新規作成

```rust
/// v38.3.0 — fav generate --from csv: CSV から Favnir type + schema + expect を生成する

pub fn csv_to_favnir(csv_path: &str) -> Result<String, String> {
    // パス traversal ガード（v38.1.0 suggest.rs と同パターン）
    if csv_path.contains("..") {
        return Err(format!("invalid path (must not contain '..'): {}", csv_path));
    }
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read {}: {}", csv_path, e))?;
    let headers = parse_headers(&content)?;
    Ok(generate_from_headers(&headers))
}

/// テスト用: ファイルパスではなく CSV 文字列から直接生成する
/// `pub(crate)` — binary crate 内のテスト専用（外部公開不要、generate_sql との対称性）
pub(crate) fn csv_to_favnir_from_str(csv_str: &str) -> Result<String, String> {
    let headers = parse_headers(csv_str)?;
    Ok(generate_from_headers(&headers))
}

fn parse_headers(csv: &str) -> Result<Vec<String>, String> {
    let first_line = csv.lines().next().ok_or("CSV is empty")?;
    Ok(first_line.split(',').map(|h| h.trim().to_string()).collect())
}

fn generate_from_headers(headers: &[String]) -> String {
    let fields = headers
        .iter()
        .map(|h| format!("    {}: String", h))
        .collect::<Vec<_>>()
        .join("\n");
    let first_col = headers.first().map(|s| s.as_str()).unwrap_or("id");
    // `fields` は `type Row` と `schema Row` の両ブロックで同じフィールド列を共有するため 2 回使用
    format!(
        "// Generated from CSV\ntype Row = {{\n{}\n}}\n\nschema Row {{\n{}\n}}\n\nexpect {{\n    all rows: Row -> rows.{} != \"\"\n}}\n",
        fields, fields, first_col
    )
}
```

**エクスポート関数**:
- `pub fn csv_to_favnir(csv_path: &str) -> Result<String, String>` — ファイルから生成
- `pub(crate) fn csv_to_favnir_from_str(csv_str: &str) -> Result<String, String>` — 文字列から生成（テスト用・binary crate 内限定）

**出力内容**:
| ブロック | 内容 |
|---|---|
| `type Row = { ... }` | CSV ヘッダーを `String` フィールドとして定義 |
| `schema Row { ... }` | 同じフィールドのスキーマ宣言 |
| `expect { all rows: ... }` | 最初の列が空でないことを検証 |

### 2. `fav/src/main.rs` — `pub(crate) mod generate_csv;` 追加 + `"csv"` 分岐追加

#### `pub(crate) mod generate_csv;` 追加

`pub(crate) mod generate_sql;` の直後に追加:
```rust
pub(crate) mod generate_sql;
pub(crate) mod generate_csv;
```

#### `"csv"` 分岐追加

既存の `match fmt` ブロック内の `_ =>` catch-all（line 2441 付近）の**直前**に追加:

```rust
"csv" => {
    let csv_path = args.get(4).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("error: `fav generate --from csv` requires a CSV file path");
        eprintln!("usage: fav generate --from csv <file.csv>");
        process::exit(1)
    });
    match generate_csv::csv_to_favnir(csv_path) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("fav generate error: {}", e);
            process::exit(1);
        }
    }
}
```

**args インデックス根拠**（v38.2.0 の `"sql"` 分岐と同一構造）:
- `args[1]` = `"generate"` / `args[2]` = `"--from"` / `args[3]` = `"csv"` / `args[4]` = CSV ファイルパス

**注意**: `_ =>` catch-all のエラーメッセージも `"Supported: sql, csv"` に更新すること。

### 3. `driver.rs` — テストモジュール追加

#### `v38200_tests::cargo_toml_version_is_38_2_0` のスタブ化

```rust
// Stubbed: version bumped to 38.3.0 — assertion intentionally removed
```

#### `v38300_tests` モジュール新規追加（4 テスト）

```rust
// ── v38300_tests (v38.3.0) — fav generate --from csv ─────────────────────────
#[cfg(test)]
mod v38300_tests {
    // include_str! および crate::generate_csv 直接参照

    #[test]
    fn cargo_toml_version_is_38_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.3.0"), "Cargo.toml must contain version 38.3.0");
    }

    #[test]
    fn changelog_has_v38_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.3.0]"), "CHANGELOG.md must contain [v38.3.0]");
    }

    #[test]
    fn generate_csv_fn_exists() {
        // driver.rs と generate_csv.rs は同じ fav/src/ ディレクトリに置かれる
        let src = include_str!("generate_csv.rs");
        assert!(src.contains("pub fn csv_to_favnir"), "generate_csv.rs must contain pub fn csv_to_favnir");
        assert!(src.contains("pub(crate) fn csv_to_favnir_from_str"), "generate_csv.rs must contain pub(crate) fn csv_to_favnir_from_str");
    }

    #[test]
    fn csv_to_favnir_basic() {
        let result = crate::generate_csv::csv_to_favnir_from_str(
            "id,name,email\n1,Alice,alice@example.com"
        ).unwrap();
        assert!(
            result.contains("type Row") && result.contains("schema") && result.contains("expect"),
            "CSV generation should produce type, schema, and expect blocks: got {:?}", result
        );
    }
}
```

**アクセス可能性**: `generate_csv.rs` は `pub(crate) mod generate_csv;` で binary crate に追加される。
driver.rs は同 binary crate のモジュールのため `crate::generate_csv` にアクセス可能（v38.2.0 の `crate::generate_sql` と同構造）。

### 4. `CHANGELOG.md` — `[v38.3.0]` エントリ追加

```
## [v38.3.0] — 2026-07-10

### Added
- `fav/src/generate_csv.rs` — `fav generate --from csv <file>` コマンド追加
- `csv_to_favnir`: CSV ヘッダーから `type Row` + `schema` + `expect` ブロックを生成
- `v38300_tests` 4 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `38.2.0` → `38.3.0`
- `versions/current.md`: 最新安定版 → v38.3.0、次バージョン → v38.4.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.3.0 を ✅ 完了済みにマーク・テスト件数を 4 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.2.0 | 2750 |
| v38.3.0 追加分 | +4 |
| v38.3.0 期待値 | 2754 |

ロードマップは「Rust テスト 2 件」と記載しているが、meta 2 件 + functional 2 件の計 4 件を追加し、T9 でロードマップを 4 件に更新する。

## ロードマップとの整合

ロードマップ v38.3.0:
- v10.8.0 `fav infer` を `schema` + `expect` ブロック出力に強化
- Rust テスト 2 件（→ 4 件に更新）

## 注意事項

### `_ =>` catch-all のエラーメッセージ更新

v38.2.0 で追加した `_ =>` catch-all のメッセージは `"Supported: sql"` となっている。
v38.3.0 で `"csv"` を追加するため `"Supported: sql, csv"` に更新する。

### パス traversal ガード

`csv_to_favnir` は `std::fs::read_to_string` を使うため、v38.1.0 の `suggest.rs` と同様に `..` を含むパスへのアクセスをガードすること。

```rust
if csv_path.contains("..") {
    return Err(format!("invalid path (must not contain '..'): {}", csv_path));
}
```

### フィールド名にスペースが含まれる CSV

`"first name"` のようなフィールド名は Favnir の識別子として無効。v38.3.0 では変換を行わず（スペースはそのまま出力）、ユーザーが手修正することを想定する。

### `gen` 予約語（Rust 2024）

`generate_csv.rs` の変数名には `headers`、`fields`、`first_col` 等を使用する — `gen` は使わないこと。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `generate_csv.rs` に `pub fn csv_to_favnir` が含まれる | `generate_csv_fn_exists` テスト |
| 2 | CSV から `type Row` + `schema` + `expect` ブロックが生成される | `csv_to_favnir_basic` テスト |
| 3 | `CHANGELOG.md` に `[v38.3.0]` が含まれる | `changelog_has_v38_3_0` テスト |
| 4 | `Cargo.toml` バージョンが `38.3.0` | `cargo_toml_version_is_38_3_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2754） | `cargo test` 実行結果 |
| 6 | `roadmap-v38.1-v39.0.md` の v38.3.0 が ✅ かつテスト件数が 4 件 | T9 後に目視確認 |
