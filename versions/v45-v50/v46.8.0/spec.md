# Spec: v46.8.0 — `fav explain --types`

## 概要

`fav explain --types <file>` コマンドを追加し、
パイプライン各ステージの宣言型を一覧表示する。

---

## 問題

`fav explain` コマンドはパイプラインの構造を可視化するが、
各ステージの型情報は出力されない。
開発者はソースコードを読まないとステージの入出力型がわからない。

---

## 解決策

`fav explain --types <file>` を実装する。

- `TrfDef`（`trf` ステージ）の宣言型を AST から直接読み取り、
  `stage Name: InputType -> OutputType` 形式で stdout に出力する。
- ジェネリック型パラメーターを持つステージは型変数を名前に付加して表示する。
- 型チェッカーの実行は不要（AST 上の `input_ty`/`output_ty` を使用）。

---

## 出力形式

```
stage ParseCsv:   String -> List<Row>
stage FilterRows: List<Row> -> List<Row>
stage SaveToDb:   List<Row> -> Result<Int>
```

ジェネリックステージの例:

```
stage Map<T, U>: List<T> -> List<U>
```

---

## スコープ

| 対象 | 内容 |
|---|---|
| `TrfDef` | `stage Name: InputType -> OutputType` を出力（必須） |
| ジェネリック | 型パラメーター `<T, U>` を名前に付加して表示（必須） |
| `FnDef` | Phase 2 スコープ外（今回は出力しない） |
| 型推論 | AST の宣言型のみ（checker 実行なし） |

---

## 実装詳細

### `driver.rs`

1. `fn type_expr_str(ty: &ast::TypeExpr) -> String`（private ヘルパー）を追加。
   `fmt.rs` の `fmt_type_expr_simple` と同等のロジックを `driver.rs` 内に複製する。
   （`fmt_type_expr_simple` は private のため pub 化せず独立実装する）

2. `pub fn cmd_explain_types(file: Option<&str>)` を追加:
   - ファイルを解析し `ast::Program` を取得
   - `program.items` をイテレートして `ast::Item::TrfDef` を収集
   - 各 `TrfDef` を `stage Name<params>: InputType -> OutputType` 形式で出力
   - ステージが 0 件の場合は `"(no stages found)\n"` を出力

3. `v468000_tests` モジュールを追加（テスト 2 件）。

### `main.rs`

`Some("explain")` ブランチに `--types` ガード（`--lineage` ガードの前）を追加:

```rust
if args.iter().any(|a| a == "--types") {
    let file = args.iter().skip(2).find(|a| !a.starts_with('-')).map(|s| s.as_str());
    cmd_explain_types(file);
    return;
}
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `explain_types_shows_stage_types` | 3 ステージのソースで `stage Name: InputType -> OutputType` が正しく出力される |
| `explain_types_generic_instantiation` | ジェネリックステージで型パラメーター付き `stage Map<T, U>: List<T> -> List<U>` が出力される |

### テストソース例（`explain_types_shows_stage_types`）

```favnir
trf ParseCsv: String -> List<Row> { List.empty() }
trf FilterRows: List<Row> -> List<Row> { input }
trf SaveToDb: List<Row> -> Result<Int> { Ok(0) }
```

期待出力:
- `"stage ParseCsv: String -> List<Row>"` を含む
- `"stage FilterRows: List<Row> -> List<Row>"` を含む
- `"stage SaveToDb: List<Row> -> Result<Int>"` を含む

### テストソース例（`explain_types_generic_instantiation`）

```favnir
trf Map<T, U>: List<T> -> List<U> { List.empty() }
```

期待出力:
- `"stage Map<T, U>: List<T> -> List<U>"` を含む

---

## ロードマップ表記との対応

ロードマップ v46.8.0 は「推論型を表示」「ジェネリック型の実体化結果も表示」と記述しているが、
本バージョンでは **AST 上の宣言型のみを実装**（型チェッカー実行なし）。

- `trf Map<T, U>: List<T> -> List<U>` の場合、表示は `stage Map<T, U>: List<T> -> List<U>`（型変数のまま）
- 型変数が具体型に実体化された後の推論結果（例: `String -> Int` に特化されたシグネチャ）は Phase 2 スコープ
- ロードマップの「実体化結果」は `List<Row>` のような**具体型引数を持つ宣言型**のことを指しており、
  `trf FilterRows: List<Row> -> List<Row>` のように宣言されていれば `List<Row>` がそのまま出力される

---

## 後方互換性

- 既存の `fav explain` コマンドへの影響なし（`--types` フラグは新規追加）
- JSON/text/mermaid 形式への影響なし

---

## 完了条件

- `cargo test` 3009 passed, 0 failed（3007 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"46.8.0"`
- `CHANGELOG.md` に v46.8.0 エントリ追加
- `versions/current.md` を v46.8.0（3009 tests）に更新
- `tasks.md` を COMPLETE に更新
