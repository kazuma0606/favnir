# Favnir v4.1.5 Tasks

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.1.5"` に更新
- [x] `fav/Cargo.toml` に `serde_yaml = "0.9"` を追加
- [x] `fav/Cargo.toml` に `regex = "1"` を追加
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `4.1.5` に更新

## Phase 1: `schemas/*.yaml` 読み込み（src/schemas.rs + driver.rs）

- [x] `fav/src/schemas.rs` を新規作成
  - [x] `FieldConstraints` 構造体（`constraints`, `max_length`, `min_length`, `min`, `max`, `pattern`, `nullable`）
  - [x] `type TypeSchema = HashMap<String, FieldConstraints>`
  - [x] `type ProjectSchemas = HashMap<String, TypeSchema>`
  - [x] `load_schemas(project_root: &Path) -> ProjectSchemas` — `schemas/` ディレクトリをスキャン
    - `schemas/` が存在しない場合は空 `HashMap` を返す
    - `.yaml` 以外のファイルはスキップ
    - パースエラーは無視して続行
- [x] `fav/src/main.rs` に `mod schemas;` を追加
- [x] `fav/src/driver.rs` の `cmd_run` 冒頭で `schemas::load_schemas(root)` を呼ぶ
- [x] `fav/src/backend/vm.rs` に `SCHEMA_REGISTRY: RefCell<ProjectSchemas>` スレッドローカルを追加
- [x] `fav/src/backend/vm.rs` に `pub fn set_schema_registry(schemas: ProjectSchemas)` を追加
- [x] `driver.rs` の `cmd_run` で `set_schema_registry(schemas)` を呼ぶ

## Phase 2: Checker — コンパイル時リテラル検査

- [x] `fav/src/middle/checker.rs` に `pub schemas: ProjectSchemas` フィールドを追加
  - `Checker::new` でデフォルト `HashMap::new()`（既存テストを壊さない）
- [x] `ValidationError` 型を checker.rs の事前登録リストに追加
  - フィールド: `field: String`, `constraint: String`, `value: String`
- [x] `check_expr` の `Expr::RecordConstruct` アームに制約チェックを追加
  - `self.schemas.get(type_name)` でスキーマを取得
  - 各フィールドの式に対して `check_field_constraints` を呼ぶ
- [x] `check_field_constraints` プライベートメソッドを実装
  - `Expr::Lit(Lit::Int(n))`: `positive`, `non_negative`, `min`, `max` を検査 → E0510 / E0511
  - `Expr::Lit(Lit::Float(f))`: 同上
  - `Expr::Lit(Lit::Str(s))`: `max_length`, `min_length`, `pattern` を検査 → E0512 / E0513
  - `Expr::BinOp(Sub, Lit::Int(0), operand)`: unary negation として処理
  - その他の式（変数・関数呼び出し等）: スキップ
- [x] checker.rs テスト追加（4件）
  - [x] `schema_constraint_positive_violation_on_literal` — E0510
  - [x] `schema_constraint_max_length_violation_on_literal` — E0513
  - [x] `schema_constraint_min_violation_on_literal` — E0511
  - [x] `schema_constraint_no_violation_passes` — エラーなし

## Phase 3: `Validate.run_raw` VM プリミティブ（vm.rs + checker.rs）

- [x] `fav/src/backend/vm.rs` に `Validate.run_raw` VM プリミティブを追加
  - 引数: `type_name: String`, `raw_map: Map<String, String>`
  - `SCHEMA_REGISTRY` からスキーマを取得
  - スキーマが存在しない場合: `ok_vm(Record([]))` を返す
  - 制約違反を `ValidationError` レコードとして収集
  - エラーなし → `ok_vm(Record)`, エラーあり → `err_vm(List<ValidationError>)`
- [x] `fav/src/middle/checker.rs` に `Validate.run_raw` / `T.validate` のシグネチャを登録
- [x] `fav/src/middle/compiler.rs` に `"Validate"` ネームスペースを追加
- [x] `vm_stdlib_tests.rs` にテスト追加（4件）
  - [x] `validate_run_raw_no_schema_returns_ok`
  - [x] `validate_run_raw_positive_violation`
  - [x] `validate_run_raw_max_length_violation`
  - [x] `validate_run_raw_valid_passes`

## Phase 4: `fav build --schema` — SQL DDL 生成（driver.rs + main.rs）

- [x] `fav/src/main.rs` の引数パースに `--schema` フラグを追加
- [x] `fav/src/driver.rs` に `cmd_build_schema(file, out)` を実装
  - `schemas::load_schemas(root)` を呼ぶ
  - `file` をパースして型定義を収集
  - フィールドごとに SQL 型を決定、制約を CHECK 句として付加
  - `primary_key` → `PRIMARY KEY AUTOINCREMENT`
  - `unique` → `UNIQUE`
  - `nullable: true` → NOT NULL なし
- [x] `to_snake_plural(name)` ユーティリティを実装
- [x] `build_sql_column(fname, ty, fc)` プライベート関数を実装
- [x] `driver.rs` 統合テスト追加（3件）
  - [x] `build_schema_generates_create_table`
  - [x] `build_schema_snake_plural_conversion`
  - [x] `build_schema_with_yaml_constraints_adds_check`

## Phase 5: examples

- [x] `fav/examples/schema_demo/schemas/Order.yaml` 作成
- [x] `fav/examples/schema_demo/main.fav` 作成

## 完了条件

- [x] `schemas/*.yaml` がプロジェクト起動時に自動ロードされる
- [x] リテラル値の制約違反がコンパイルエラーになる（checker テスト 4 件）
- [x] `Validate.run_raw` が手動呼び出しで動作する（vm_stdlib テスト 4 件）
- [x] `fav build --schema` が SQL DDL を生成できる（driver テスト 3 件）
- [x] 全既存テスト（808 件）がパスすること
