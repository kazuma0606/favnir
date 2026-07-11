# v36.5.0 spec — Data Contract 規約

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.5.0 |
| テーマ | Data Contract 規約 |
| 前提 | v36.4.0 COMPLETE — `fav validate` コマンド実装済み |
| 完了条件 | `v36500_tests` 全テスト pass・`cargo test` 0 failures（≥ 2681 件） |

## 背景と目的

v36.4.0 で `fav validate` による CSV/スキーマ整合性検証を実装した。
本バージョンは「Data Contract」の概念をプロジェクト規約として確立する:

1. **`contracts/` ディレクトリ規約** — プロジェクト内に `contracts/*.fav` ファイルを置く規約を策定する
2. **`fav new --template data-contract`** — Data Contract プロジェクトのスキャフォルドを追加する
3. **`fav contract check`** — `contracts/` 内のすべての `.fav` ファイルに schema 定義が存在するか検証する

## `contracts/` ディレクトリ規約

```
my-project/
├── contracts/
│   ├── orders.fav      # schema Orders { ... }
│   └── users.fav       # schema Users { ... }
├── src/
│   └── main.fav
└── fav.toml
```

- `contracts/*.fav` はそれぞれ少なくとも1つの `schema` 定義を含む必要がある
- `fav contract check` が規約の遵守を静的に検証する

## 実装スコープ

### 1. `fav/src/driver.rs` — `validate_contract_file` と `cmd_contract_check`

#### コアロジック（純粋関数 — テスト可能）

```rust
/// contracts ファイルの内容を検証する純粋関数。
/// schema 定義が存在しない場合にエラーメッセージを返す（空 Vec = OK）。
pub fn validate_contract_file(src: &str, file: &str) -> Vec<String> {
    use crate::ast::Item; // `Item` はモジュールスコープ未インポートのためローカル use が必要
    let program = match Parser::parse_str(src, file) {
        Ok(p) => p,
        Err(e) => return vec![format!("{}: parse error: {}", file, e)],
    };
    let has_schema = program.items.iter().any(|item| matches!(item, Item::SchemaDef(_)));
    if has_schema {
        vec![]
    } else {
        vec![format!(
            "{}: no `schema` definition found \
             (data contracts must define at least one schema)",
            file
        )]
    }
}
```

#### `cmd_contract_check` 関数

```rust
// ── fav contract check (v36.5.0) ──────────────────────────────────────────────

pub fn cmd_contract_check(dir: Option<&str>) {
    let contracts_dir = dir.unwrap_or("contracts");
    let path = std::path::Path::new(contracts_dir);

    if !path.exists() {
        eprintln!("error: `{}` directory not found", contracts_dir);
        process::exit(1);
    }

    let mut fav_files: Vec<std::path::PathBuf> = std::fs::read_dir(path)
        .unwrap_or_else(|e| {
            eprintln!("error: cannot read `{}`: {}", contracts_dir, e);
            process::exit(1);
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("fav") {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    fav_files.sort(); // 結果の一貫性のためソート

    if fav_files.is_empty() {
        eprintln!("error: no .fav files found in `{}`", contracts_dir);
        process::exit(1);
    }

    let mut has_errors = false;
    for file_path in &fav_files {
        let path_str = file_path.to_string_lossy();
        let src = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
            eprintln!("error: cannot read {}: {}", path_str, e);
            process::exit(1);
        });
        let errors = validate_contract_file(&src, &path_str);
        if errors.is_empty() {
            println!("{}: ok", path_str);
        } else {
            has_errors = true;
            for err in &errors {
                eprintln!("{}", err);
            }
        }
    }

    if has_errors {
        process::exit(1);
    }
}
```

#### `create_data_contract_project` 関数

```rust
fn create_data_contract_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("contracts/orders.fav"), &format!(
        "// Data Contract: Orders\n\
         schema Orders {{\n\
         \    id:          Int\n\
         \    customer_id: Int\n\
         \    amount:      Float\n\
         \    status:      String\n\
         \    created_at:  String\n\
         }}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nData Contract project.\n\n## Usage\n\n```bash\nfav contract check\n```\n"
    ))?;
    Ok(())
}
```

#### `TEMPLATE_GALLERY` へのエントリ追加（5 エントリに）

```rust
pub const TEMPLATE_GALLERY: &[(&str, &str)] = &[
    ("etl-csv-to-db",    "CSV → DB ETL パイプライン"),
    ("api-gateway",      "HTTP API ゲートウェイ"),
    ("lambda-scheduled", "スケジュール実行 Lambda ジョブ"),
    ("distributed-etl",  "分散並列 ETL パイプライン"),
    ("data-contract",    "Data Contract スキーマ定義プロジェクト"),  // v36.5.0
];
```

#### `try_cmd_new` へのアーム追加（`other =>` の前に追加）

```rust
"data-contract" => create_data_contract_project(&root, name),
other => Err(format!(
    "unknown template `{other}` \
     (expected script|pipeline|lib|postgres-etl|\
     etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl|data-contract)"
)),
```

### 2. `fav/src/main.rs` — `fav contract check` ルーティングと import

#### import 追加

`use driver::{ ... }` に `cmd_contract_check` を追加する。

#### `Some("contract") =>` アーム（`Some("validate") =>` の直後に追加）

```rust
Some("contract") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("check") => {
            let dir = args.get(3).map(|s| s.as_str());
            cmd_contract_check(dir);
        }
        sub => {
            eprintln!(
                "error: unknown contract subcommand `{}`; expected: check",
                sub.unwrap_or("(none)")
            );
            process::exit(1);
        }
    }
}
```

#### HELP 定数への追加

`validate` コマンド説明の後に追加:

```
    contract check [dir]
                  Check that all .fav files in contracts/ (or [dir]) contain
                  at least one schema definition.
                  Default directory: ./contracts/
```

### 3. `fav/src/driver.rs` — 既存テスト更新とスタブ化

#### スタブ化: `v36400_tests::cargo_toml_version_is_36_4_0`

```rust
#[test]
fn cargo_toml_version_is_36_4_0() {
    // stubbed: version bumped to 36.5.0
}
```

#### 更新: `v248000_tests::template_gallery_has_4_entries`

`TEMPLATE_GALLERY` が 5 エントリになるため、テストを更新する（スタブ化ではなく内容修正）:

```rust
#[test]
fn template_gallery_has_4_entries() {
    // v36.5.0 で data-contract を追加したため 5 エントリ
    assert_eq!(TEMPLATE_GALLERY.len(), 5,
        "TEMPLATE_GALLERY must have 5 entries, got {}", TEMPLATE_GALLERY.len());
    let names: Vec<&str> = TEMPLATE_GALLERY.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"etl-csv-to-db"),     "missing etl-csv-to-db");
    assert!(names.contains(&"api-gateway"),        "missing api-gateway");
    assert!(names.contains(&"lambda-scheduled"),   "missing lambda-scheduled");
    assert!(names.contains(&"distributed-etl"),    "missing distributed-etl");
    assert!(names.contains(&"data-contract"),      "missing data-contract");
}
```

### 4. `fav/src/driver.rs` — `v36500_tests` モジュール

## v36500_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_5_0` | Cargo.toml に `"36.5.0"` が含まれる |
| `changelog_has_v36_5_0` | CHANGELOG.md に `[v36.5.0]` が含まれる |
| `data_contract_template_in_try_cmd_new` | driver.rs に `data-contract` と `create_data_contract_project` が含まれる |
| `validate_contract_file_fires` | schema なし .fav でエラーが返る |
| `validate_contract_file_silent` | schema あり .fav でエラーなし |

### テスト実装

```rust
// ── v36500_tests (v36.5.0) — Data Contract 規約 ───────────────────────────────
#[cfg(test)]
mod v36500_tests {
    use super::validate_contract_file;

    #[test]
    fn cargo_toml_version_is_36_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.5.0"), "Cargo.toml must contain version 36.5.0");
    }
    #[test]
    fn changelog_has_v36_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.5.0]"), "CHANGELOG.md must contain [v36.5.0]");
    }
    #[test]
    fn data_contract_template_in_try_cmd_new() {
        let src = include_str!("driver.rs");
        assert!(
            src.contains("\"data-contract\""),
            "driver.rs must contain data-contract template arm"
        );
        assert!(
            src.contains("create_data_contract_project"),
            "driver.rs must contain create_data_contract_project"
        );
    }
    #[test]
    fn validate_contract_file_fires() {
        let errors = validate_contract_file("fn foo() -> Int { 1 }", "test.fav");
        assert!(!errors.is_empty(), "expected error for file without schema");
        assert!(
            errors.iter().any(|e| e.contains("schema")),
            "error must mention `schema`: {:?}", errors
        );
    }
    #[test]
    fn validate_contract_file_silent() {
        let errors = validate_contract_file("schema Orders { id: Int }", "test.fav");
        assert!(errors.is_empty(), "expected no errors for valid contract: {:?}", errors);
    }
}
```

## 注意事項

### `Parser` と `Item` の import

- `Parser` は driver.rs ファイル先頭（`use crate::frontend::parser::Parser;`）でモジュールスコープにインポート済みのため、`validate_contract_file` 内でのローカル use は不要。
- `Item` は driver.rs モジュールスコープに **インポートされていない**。`validate_contract_file` 内で `use crate::ast::Item;` をローカル宣言すること（`cmd_validate` の `use crate::ast::Item;` と同パターン）。

### `validate_contract_file` の制限（v36.5.0 スコープ外）

- `contracts/` サブディレクトリの再帰検索（トップレベルの `.fav` のみが対象）
- schema フィールドの型検証（存在確認のみ）
- `expect` ブロックの実行時評価

### `validate_contract_file_silent` テストの前提

テスト入力 `"schema Orders { id: Int }"` の1行形式は v36.1.0 のインライン schema パーサーが受け付けることを前提とする。パーサーが改行を要求する場合はテスト入力を複数行形式に変更すること。

### `cmd_contract_check` 内の `process::exit`

`cmd_contract_check` はプロセス終了を伴うため、`validate_contract_file` を純粋関数として分離してテストする設計としている。

### テスト名の関数名不変性

`v248000_tests::template_gallery_has_4_entries` は関数名を変えずに（`has_4_entries` のまま）内容のみ 5 エントリ版に更新する。関数名を変えると他のドキュメント参照と不整合になるため。

## ロードマップとの整合

ロードマップ v36.5.0 完了条件:「Rust テスト 2 件」
本 spec では 5 テストを追加する（ロードマップの最小要件 2 件を上回る）。
ロードマップの件数は更新しない（最小要件値として維持）。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `driver.rs` に `create_data_contract_project` と `validate_contract_file` が含まれる | `data_contract_template_in_try_cmd_new` テスト |
| 2 | CHANGELOG.md に `[v36.5.0]` が含まれる | `changelog_has_v36_5_0` テスト |
| 3 | Cargo.toml バージョンが `36.5.0` | `cargo_toml_version_is_36_5_0` テスト |
| 4 | schema なし .fav でエラーが返る | `validate_contract_file_fires` テスト |
| 5 | schema あり .fav でエラーなし | `validate_contract_file_silent` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2681） | `cargo test` 実行結果 |
