# v36.8.0 実装計画 — `fav schema diff`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追記 | `schema_diff` + `cmd_schema_diff` 追加 / `v36700_tests` スタブ化 / `v36800_tests` 追加 |
| `fav/src/main.rs` | 変更 | `cmd_schema_diff` import 追加 / `Some("schema")` ルーティング追加 / HELP 更新 |
| `fav/Cargo.toml` | 更新 | `version = "36.7.0"` → `"36.8.0"` |
| `CHANGELOG.md` | 追記 | `[v36.8.0]` エントリ追加 |
| `versions/current.md` | 更新 | 最新安定版 v36.8.0、次バージョン v36.9.0 |
| `versions/roadmap/roadmap-v36.1-v37.0.md` | 更新 | v36.8.0 完了済みにマーク（✅）（ロードマップ最小値 2 件を超える 6 件を実装。ロードマップ側件数は更新不要） |

## 実装順序

### Step 1: CHANGELOG.md に [v36.8.0] エントリ追加

`## [v36.7.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

### Step 2: driver.rs — `schema_diff` 追加

挿入位置: `validate_contract_file` 関数の `}` の後、`cmd_contract_check` の前（`// ── fav contract check` セクション末尾の後に `// ── fav schema diff (v36.8.0)` セクションを追加）。

- 純粋関数（`load_file` 不使用）
- 内部クロージャ `parse` で `Parser::parse_str` → `SchemaDef` リストに変換
- `use crate::ast::Item;` はクロージャ内でローカル宣言（driver.rs モジュールスコープに `Item` が未インポートの場合）
- `SchemaDef` の型名比較は `format!("{:?}", ty)` で行う

### Step 3: driver.rs — `cmd_schema_diff` 追加

`schema_diff` の `}` の直後に追加。
`load_file` でソースを読み込み、`schema_diff` に渡して結果を `println!` で出力する。

### Step 4: main.rs — `Some("schema")` ルーティング追加

挿入位置: `Some("contract") =>` アームの直後（`Some("doc") =>` の前）。
`cmd_schema_diff` を `use driver::{ ... }` import に追加する。
`Some("build")` 内の `--schema` フラグとは別物（トップレベルの `match args[0]` のアームとして追加する）。

### Step 5: driver.rs — `v36700_tests::cargo_toml_version_is_36_7_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 36.8.0` に変更。

### Step 6: driver.rs — `v36800_tests` モジュール追加

`v36700_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する。
`use super::schema_diff;` インポートを使用（driver.rs 内 pub fn の慣用パターン）。

### Step 7: Cargo.toml バージョン更新

Step 2〜6 完了・コンパイルエラー解消後に `36.7.0` → `36.8.0` に更新。

## 依存関係

- driver.rs モジュールスコープは `use crate::ast;` のみ（glob インポートなし）— `schema_diff` 内で `use crate::ast::{Item, SchemaDef};` を両方ローカル宣言する（`Item` のみでは `Vec<SchemaDef>` の型推論が失敗）
- `load_file` が driver.rs に既存であることを確認（T0）
- `cmd_schema_diff` の呼び出し引数は `Option<&str>` — main.rs の `args.get(N).map(|s| s.as_str())` で生成

## リスク

| リスク | 対処 |
|---|---|
| `TypeExpr` が `Debug` を impl していない | T0 で `#[derive(Debug)]` の有無を確認 |
| `SchemaDef` が `Clone` でなく `parse` クロージャから所有権を移動できない | T0 で確認。`.collect::<Vec<_>>()` → `into_iter()` で所有権移動する設計 |
| `Some("schema") =>` アームが既存の `Some("schema")` と競合 | T0 で main.rs に `"schema"` アームがないことを確認 |
