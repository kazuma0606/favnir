# Roadmap v18.1.0 〜 v19.0.0 — Type System Maturity

Date: 2026-06-14

## 目標

v18.0.0「Language Power」で「表現できる言語」への転換を果たした。
次のテーマは「**信頼できる言語**」——型システムが現実のデータに追いつくことである。

データエンジニアリングの現実:
- スキーマが実行時に変わる（BigQuery のカラム追加、Snowflake の型変更）
- データソースによって型が違う（CSV は全て String、DB は型あり、API は不定）
- エフェクトを毎回手書きするのは負担になる
- 接続・トランザクションの「使い忘れ」「二重クローズ」は実行時にしか検出できない

これらを型レベルで解決することで、**型チェッカーがデータパイプラインの設計図になる**。

- v18.1: エフェクト自動推論でエフェクト宣言の書き忘れを排除
- v18.2: 行多相で「このフィールドを持つ任意のレコード」を扱う汎用関数を書けるようにする
- v18.3: 関数引数レベルの `where` 制約（Refinement Types）でゼロ除算・空リストをコンパイル時検出
- v18.4: `schema "bigquery:..."` でDB スキーマを型として直接インポート
- v18.5: 線形型 `-o` で接続・トランザクションの安全性をコンパイル時保証
- v18.6: 共変・反変アノテーションでジェネリクスの型安全性を完成させる
- v18.7: 型レベル定数（Const Generics）でバッチサイズ等をコンパイル時検証
- v18.8: 型定義から REST / GraphQL API スキーマを自動生成
- v19.0: Type System Maturity マイルストーン宣言

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| エフェクト推論の実装方式 | HM 型推論の拡張（エフェクト変数を型変数と同様に扱う）。明示宣言は上書きではなく検証として機能 |
| 行多相の構文 | `fn f<R with { id: Int }>(row: R) -> { ...R, ts: String }` — `with { ... }` でレコード型制約 |
| Refinement Types の評価 | コンパイル時（リテラル値の場合）+ 実行時アサーション（変数の場合）のハイブリッド |
| スキーマ型の取得タイミング | `fav check` / `fav build` 実行時に接続してスキーマを取得し、`.fav.schema-cache` にキャッシュ |
| 線形型の構文 | `-o` を「linear arrow」とする（`fn(T) -> U` は通常、`fn(T) -o U` は線形） |
| 共変・反変アノテーション | `+T`（共変）/ `-T`（反変）/ `T`（不変、デフォルト）— interface 定義のみ |
| Const Generics の制限 | v18.7 では `Int` 型定数のみ対応（Float / String は v19.x 以降） |
| API 生成の対象フォーマット | OpenAPI 3.0（JSON / YAML）と GraphQL SDL。REST と GraphQL の両方をサポート |

---

## バージョン計画

### v18.1.0 — エフェクト推論（Effect Inference）

**テーマ**: エフェクト宣言を自動推論する。
現在は全てのエフェクトを手動で宣言しなければならない。
推論することで記述量を減らし、宣言忘れのエラー（E0314〜E0319）も消える。

**現状と推論後の比較:**

```fav
// 現状: エフェクトを手動で宣言（宣言漏れが E0314 等になる）
fn load_users() -> Result<List<User>, String> !Db !IO {
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- io.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}

// 推論後: エフェクト宣言が不要（型チェッカーが自動推論）
fn load_users() -> Result<List<User>, String> {
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- io.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}

// stage も同様に推論
stage LoadUsers -> List<User> {  // !Db !IO が自動推論される
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- io.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}
```

**明示宣言の維持ユースケース:**

```fav
// interface の実装では明示が必要（外部からの契約として）
interface Loader {
  fn load() -> Result<List<Row>, String> !Db
}

// 純粋関数であることを明示・保証したい場合
// （エフェクトが推論されないことをコンパイル時に確認）
fn pure_transform(row: Row) -> Row /* !pure */ {
  { ...row, score: compute(row) }
}
// もし Db 呼び出しが混入していたらコンパイルエラー
```

**エフェクト推論の仕組み:**

```
1. 関数ボディを解析してプリミティブ呼び出しを収集
   (Postgres.* → !Db, IO.* → !IO, S3.* → !AWS, ...)

2. 呼び出す関数のエフェクトを再帰的に収集

3. 推論結果を型シグネチャに付与

4. 明示宣言がある場合は推論結果との整合性を検査
   - 明示 ⊇ 推論 → OK（余分な宣言は警告 W010）
   - 明示 ⊂ 推論 → E0330（宣言漏れ）
```

**実装内容:**

- `fav/src/middle/checker.rs`:
  - `infer_effects(fn_def: &FnDef, env: &Env) -> EffectSet` 関数追加
  - `EffectSet`: エフェクトの集合（`HashSet<Effect>`）
  - HM 型推論の `infer_hm` にエフェクト変数を統合
  - 推論エフェクトをシグネチャに付与して `fav check --show-types` で表示

- `self/checker.fav`:
  - `infer_effects` の Favnir 実装追加

- `fav/src/driver.rs`:
  - `fav check --show-effects`: 推論されたエフェクトを表示
  - W010: 明示宣言が推論結果のスーパーセット（余分な宣言）

- テスト: `v181000_tests`（5件）:
  - `version_is_18_1_0`
  - `effect_inference_db`（`Postgres.query_raw` を含む fn に `!Db` が推論される）
  - `effect_inference_multi`（複数エフェクトが正しく推論される）
  - `effect_inference_pure`（副作用なし fn のエフェクトが空集合になる）
  - `effect_inference_transitive`（エフェクトありの fn を呼ぶ fn でも推論される）

**完了条件（PASS=5）:**
1. `!Db` を宣言しなくても `Postgres.*` を呼ぶ fn のチェックが通る
2. 複数エフェクト（`!Db !IO !AWS`）が正しく推論される
3. 純粋関数（副作用なし）のエフェクトが空集合として推論される
4. エフェクトありの fn を呼ぶ fn でも推論が伝播する
5. `fav check --show-effects` で推論結果を確認できる

---

### v18.2.0 — 行多相（Row Polymorphism）

**テーマ**: 「このフィールドを持つ任意のレコード型」を型安全に扱える汎用関数を書けるようにする。
パイプラインで「どんな行でも timestamp を追加できる」関数が書きたい、という問題を解決する。

**設計上の重要な制約:**

スプレッド構文（`{ ...row, field: val }`）は**値の位置のみ**に使用する（v16.3 と同じルール）。
戻り型の位置には `&`（交差型）を使う。これにより呼び出し元が戻り型を明確に読める。

```fav
// ❌ 型の位置でのスプレッドは使わない
fn add_timestamp<R with { id: Int }>(row: R) -> { ...R, timestamp: String } { }

// ✅ 型の位置では & （交差型）を使う
fn add_timestamp<R with { id: Int }>(row: R) -> R & { timestamp: String } {
  { ...row, timestamp: DateTime.format_iso(DateTime.now()) }
  // 値の組み立ては spread、型の宣言は & で分離
}
```

**構文:**

```fav
// 戻り型は & で表現: 「R のフィールド全部 + timestamp フィールド」
fn add_timestamp<R with { id: Int }>(row: R) -> R & { timestamp: String } {
  { ...row, timestamp: DateTime.format_iso(DateTime.now()) }
}

// 使用例: 型が違うレコードに同じ関数を適用できる
let user_with_ts  = add_timestamp({ id: 1, name: "Alice" })
let order_with_ts = add_timestamp({ id: 42, amount: 100.0, status: "pending" })
// user_with_ts の型:  UserRow & { timestamp: String }
// order_with_ts の型: OrderRow & { timestamp: String }
// → 呼び出し元はエラーメッセージでもこの形式で型を確認できる

// フィールドを読む関数（戻り型が R そのままなら & 不要）
fn get_id<R with { id: Int }>(row: R) -> Int {
  row.id
}

// 複数フィールドの制約
fn validate_user<R with { id: Int, email: String }>(row: R) -> Result<R, String> {
  if String.contains(row.email, "@")
    { Result.ok(row) }
  else
    { Result.err(f"invalid email: {row.email}") }
}

// パイプラインでの活用（戻り型を & で明示）
stage Enrich<R with { id: Int, created_at: String }>(rows: List<R>) -> List<R & { age_days: Int }> {
  Result.ok(List.map(rows, |row| {
    let age = DateTime.diff_days(DateTime.parse(row.created_at), DateTime.now())
    { ...row, age_days: age }
  }))
}
```

**`&`（交差型）の読み方:**

```fav
R & { timestamp: String }
// = 「R が持つ全フィールド」かつ「timestamp: String フィールド」を持つ型

UserRow & { score: Float }
// = { id: Int, name: String, email: String, score: Float }
// エラー表示でも展開形が見えるのでデバッグしやすい
```

**実装内容:**

- `fav/src/ast.rs`:
  - `TypeConstraint::HasField { name: String, ty: Type }` 追加
  - `GenericParam.bounds` に `TypeConstraint` を追加（既存の `Interface` bound と共存）

- `fav/src/ast.rs`:
  - `Type::Intersection(Box<Type>, Box<Type>)` 追加（`R & { field: Type }` 構文）
  - `TypeConstraint::HasField { name: String, ty: Type }` 追加

- `fav/src/frontend/parser.rs`:
  - `parse_type` で `T & { ... }` の交差型を解析
  - `&` を型演算子として認識（式の `&` ビット演算子とは型文脈で区別）

- `fav/src/middle/checker.rs`:
  - `check_row_constraint`: `R with { field: Type }` の型検査
  - `unify_row(ty: &Type, constraint: &RecordConstraint) -> SubstResult` 追加
  - レコード型の包含チェック（制約フィールドが全てある → OK）
  - `check_intersection_type`: `R & { field: Type }` を展開して具体型に解決
    - `R` が具体的なレコード型の場合 → フィールドをマージして検査
    - 戻り値の `{ ...row, field: val }` が `R & { field: Type }` と一致するか確認
  - E0329: `R & { field: Type }` の `field` が `R` にも存在する場合（フィールド重複）

- `self/checker.fav`:
  - `check_row_polymorphism` / `check_intersection_type` 関数を Favnir 実装に追加

- テスト: `v182000_tests`（5件）:
  - `version_is_18_2_0`
  - `row_poly_single_field`（`R with { id: Int }` が動作）
  - `row_poly_different_records`（異なるレコード型に同じ関数を適用）
  - `row_poly_intersection_return`（`-> R & { ts: String }` の戻り型が正しい）
  - `row_poly_field_missing`（制約フィールドがない型を渡すと型エラー）

**完了条件（PASS=5）:**
1. `fn f<R with { id: Int }>(row: R) -> R` が動作する
2. 異なるフィールドを持つ複数のレコード型に同じ関数を適用できる
3. `-> R & { timestamp: String }` の交差型が静的に解決される
4. 制約フィールドを持たない型を渡すと型エラーになる
5. 複数フィールド制約 `R with { id: Int, email: String }` が動作する

---

### v18.3.0 — 改良版 `where` 制約（Refinement Types）

**テーマ**: 現在の `where` は型定義時のバリデーションのみ。
関数の引数レベルでも使えるようにし、ゼロ除算・空リスト・不正範囲を型レベルで防ぐ。

**現状と拡張後の比較:**

```fav
// 現状: 型定義での where（v9.7.5 実装済み）
type Amount(Float) where { self > 0.0 }

// 拡張: 関数引数での where（v18.3 新規）
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
  a / b
}

fn process(rows: List<Row> where { List.length(rows) > 0 }) -> Result<Summary, String> {
  let head = List.head(rows)
  ...
}

fn percentage(n: Int where { n >= 0 && n <= 100 }) -> String {
  f"{n}%"
}
```

**コンパイル時チェック（リテラル値の場合）:**

```fav
divide(10, 0)       // E0331: constraint violation: b != 0 (got 0)
divide(10, 2)       // OK: 2 != 0 は静的に確認可能
percentage(150)     // E0331: constraint violation: n <= 100 (got 150)
percentage(50)      // OK
```

**実行時チェック（変数の場合）:**

```fav
// b の値がコンパイル時不明な場合は実行時アサーションを挿入
fn safe_divide(a: Int, b: Int) -> Result<Int, String> {
  divide(a, b)  // → コンパイラが自動的にアサーションを挿入
                // → b == 0 なら Result.err("constraint violation: b != 0")
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `FnParam { name: String, ty: Type, constraint: Option<Expr> }` に拡張
  - `fn f(x: T where { expr })` を解析

- `fav/src/frontend/parser.rs`:
  - `parse_fn_param_with_constraint`: `name: Type where { expr }` の解析

- `fav/src/middle/checker.rs`:
  - `check_refinement_call`: 呼び出し時に引数が制約を満たすかチェック
  - リテラル引数 → コンパイル時評価 → E0331
  - 変数引数 → 実行時チェック用の `RefinementAssert` opcode を挿入

- `fav/src/backend/vm.rs`:
  - `RefinementAssert { constraint: String }` opcode 追加
  - 制約違反時に `RuntimeError::ConstraintViolation(msg)` を発生

- テスト: `v183000_tests`（5件）:
  - `version_is_18_3_0`
  - `refinement_literal_pass`（`divide(10, 2)` がコンパイル・実行できる）
  - `refinement_literal_fail`（`divide(10, 0)` が E0331 でコンパイルエラー）
  - `refinement_runtime_check`（変数で呼ぶと実行時アサーションが動作する）
  - `refinement_range_constraint`（`n >= 0 && n <= 100` の複合制約が動作）

**完了条件（PASS=5）:**
1. `fn f(b: Int where { b != 0 })` が定義できる
2. リテラル違反（`f(0)`）がコンパイル時 E0331 になる
3. 変数渡し時に実行時アサーションが挿入される
4. 実行時制約違反で適切なエラーメッセージが出る
5. 複合制約（`n >= 0 && n <= 100`）が動作する

---

### v18.4.0 — スキーマ型（Schema Types）

**テーマ**: BigQuery / Snowflake / Postgres のスキーマを型として直接インポートする。
`fav infer` で型を手書きする手間を排除し、スキーマ変更を型エラーで検出できるようにする。

**構文:**

```fav
// BigQuery テーブルから型を生成（コンパイル時にスキーマを取得）
type UsersRow = schema "bigquery:my-project.my_dataset.users"
// → { id: Int, name: String, email: String, created_at: String }

// Postgres テーブルから型を生成
type OrderRow = schema "postgres:orders"
// → { id: Int, user_id: Int, amount: Float, status: String, created_at: String }

// Snowflake テーブルから型を生成
type ProductRow = schema "snowflake:MY_DB.MY_SCHEMA.PRODUCTS"
// → { PRODUCT_ID: Int, NAME: String, PRICE: Float }

// JSON Schema ファイルから型を生成
type EventPayload = schema "file:schemas/event.json"

// 使用例: 型が実テーブルと自動的に同期
stage LoadUsers -> List<UsersRow> !Db {
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  Result.ok(rows)  // UsersRow 型として型チェックされる
}
```

**fav.toml での事前定義（ビルド時に取得・キャッシュ）:**

```toml
[[schema]]
name = "UsersRow"
source = "bigquery:my-project.my_dataset.users"

[[schema]]
name = "OrderRow"
source = "postgres:orders"
```

**`fav infer` との統合:**

```bash
# 従来: 型を手書きする必要があった
fav infer --from bigquery --table users
# → type UsersRow = { id: Int, name: String, ... } を出力（コピペが必要）

# v18.4 以降: schema 型として参照するだけ
type UsersRow = schema "bigquery:my-project.my_dataset.users"
# → ビルド時に自動取得
```

**スキーマキャッシュ:**

```
.fav/
  schema-cache/
    bigquery__my-project__my_dataset__users.json   # キャッシュ済みスキーマ
    postgres__orders.json
```

- `fav check --refresh-schemas`: キャッシュを破棄して再取得

**実装内容:**

- `fav/src/ast.rs`:
  - `Type::Schema(String)` 追加（`schema "source:identifier"` 構文）

- `fav/src/middle/resolver.rs`:
  - `resolve_schema_type`: `schema "..."` 文字列を解析して実際の型に展開
  - `SchemaSource::BigQuery / Postgres / Snowflake / JsonFile` enum
  - キャッシュ読み書き（`.fav/schema-cache/`）

- `fav/src/middle/checker.rs`:
  - スキーマ展開後の型チェックは通常のレコード型として処理

- `fav/src/driver.rs`:
  - `cmd_check` で `--refresh-schemas` フラグ対応

- テスト: `v184000_tests`（5件）:
  - `version_is_18_4_0`
  - `schema_type_parses`（`type X = schema "..."` が AST として解析される）
  - `schema_cache_creates`（スキーマキャッシュファイルが生成される）
  - `schema_file_source`（`schema "file:path.json"` が動作する）
  - `schema_type_checks`（スキーマ型のフィールドアクセスが型チェックされる）

**完了条件（PASS=5）:**
1. `type X = schema "file:schema.json"` が動作する（ファイルソースでテスト）
2. スキーマキャッシュが `.fav/schema-cache/` に生成される
3. スキーマ型のフィールドアクセスが型チェックされる
4. 存在しないフィールドへのアクセスが型エラーになる
5. `fav check --refresh-schemas` でキャッシュが再取得される

---

### v18.5.0 — 線形型（Linear Types）によるリソース安全性

**テーマ**: 接続・ファイルハンドル・トランザクションを「使い忘れ」「二重クローズ」から守る。
`-o`（linear arrow）で「ちょうど 1 回使われる」ことをコンパイル時に保証する。

**構文と使用例:**

```fav
// Connection は linear 型: 必ず 1 回だけ使われる（ドロップか明示クローズ）
// -o は「linear arrow」: 引数を必ずちょうど 1 回消費する

fn with_connection<T>(f: Connection -o Result<T, String>) -> Result<T, String> !Db {
  bind conn   <- Postgres.connect()    // conn: Connection (linear)
  let result  = f(conn)               // conn は f に「移動」（以後使用不可）
  result                              // conn は f の中でクローズされた
}

// トランザクション安全性
fn transact<T>(f: Tx -o Result<T, String>) -> Result<T, String> !Db {
  bind tx <- Postgres.begin()         // tx: Tx (linear)
  match f(tx) {
    Result.ok(v)  => { Postgres.commit(tx); Result.ok(v) }
    Result.err(e) => { Postgres.rollback(tx); Result.err(e) }
  }
}

// 使用例: コネクションの使い方が型安全
fn do_work() -> Result<Int, String> !Db {
  with_connection(|conn| {
    bind rows <- Postgres.query_with_conn(conn, "SELECT COUNT(*) FROM users", [])
    // conn はここで自動クローズ（f が完了）
    Result.ok(rows)
  })
}

// コンパイルエラー例
fn wrong_usage() -> Result<Int, String> !Db {
  bind conn <- Postgres.connect()
  let _ = Postgres.query_with_conn(conn, "SELECT 1", [])
  let _ = Postgres.query_with_conn(conn, "SELECT 2", [])  // E0332: conn は既に消費済み
  Result.ok(0)
}
```

**線形型ルール:**

| 操作 | 通常型 | 線形型 |
|---|---|---|
| 同じ変数を 2 回使う | OK | E0332（二重消費） |
| 使わずに捨てる | OK | E0333（未消費） |
| 関数に渡す | コピー渡し | 移動（以後使用不可） |

**実装内容:**

- `fav/src/ast.rs`:
  - `Type::LinearFn { param: Box<Type>, ret: Box<Type> }` 追加（`T -o U`）
  - `LinearType` マーカー（`#[linear]` アノテーション or 組み込み型の指定）

- `fav/src/middle/checker.rs`:
  - `LinearEnv`: 線形変数の使用状況トラッキング（`HashMap<String, UseCount>`）
  - `check_linear_use`: 変数が消費済みかチェック → E0332
  - `check_linear_drop`: 関数終了時に未消費の線形変数をチェック → E0333
  - 組み込み線形型: `Connection` / `Tx`（Postgres トランザクション）

- `fav/src/backend/vm.rs`:
  - `LinearDrop` opcode: 線形値の明示的破棄（クローズ操作）

- テスト: `v185000_tests`（5件）:
  - `version_is_18_5_0`
  - `linear_type_parses`（`T -o U` 型が解析される）
  - `linear_use_once_ok`（1 回使用で OK）
  - `linear_double_use_error`（2 回使用で E0332）
  - `linear_unused_error`（未消費で E0333）

**完了条件（PASS=5）:**
1. `T -o U` 線形関数型が解析される
2. 線形変数を 1 回使うと正常にコンパイルされる
3. 線形変数を 2 回使うと E0332 が出る
4. 線形変数を使わずに関数が終わると E0333 が出る
5. `with_connection` パターンが正しく型チェックされる

---

### v18.6.0 — 共変・反変アノテーション（Variance）

**テーマ**: ジェネリクスの型安全性を完成させる。
`List<Cat>` が `List<Animal>` として扱えるか、という問題を型レベルで正しく扱う。

**構文:**

```fav
// covariant (+T): 出力のみ（Producer パターン）
// List<Cat> は List<Animal> として渡せる
interface Source<+T> {
  fn next() -> Option<T>
}

// contravariant (-T): 入力のみ（Consumer パターン）
// Sink<Animal> は Sink<Cat> として渡せる
interface Sink<-T> {
  fn write(val: T) -> Result<Unit, String>
}

// invariant（デフォルト）: 入出力両方
// List<Cat> は List<Animal> として渡せない
interface Transform<T> {
  fn apply(val: T) -> T
}

// 使用例
fn process_cats(source: Source<Cat>, sink: Sink<Animal>) -> Result<Unit, String> {
  // source: Source<Cat> → Source<Animal> の代わりに使える（共変）
  // sink: Sink<Animal> → Sink<Cat> の代わりに使える（反変）
  ...
}
```

**分散ルール（リスコフ置換原則の型システム実装）:**

| 型引数の使われ方 | 分散 | サブタイピング方向 |
|---|---|---|
| 出力のみ（戻り値）| 共変 `+T` | `List<Cat> <: List<Animal>` |
| 入力のみ（引数）| 反変 `-T` | `Sink<Animal> <: Sink<Cat>` |
| 入出力両方 | 不変（デフォルト）| 同一型のみ |

**実装内容:**

- `fav/src/ast.rs`:
  - `GenericParam.variance: Variance` 追加（`Covariant / Contravariant / Invariant`）
  - `+T` / `-T` を `GenericParam` の分散アノテーションとして解析

- `fav/src/frontend/parser.rs`:
  - `parse_generic_params` で `+T` / `-T` を認識

- `fav/src/middle/checker.rs`:
  - `check_variance`: interface 実装時に宣言した分散と実際の使われ方が一致するか確認
  - `is_subtype_with_variance`: 分散を考慮したサブタイプチェック
  - E0334: 分散違反（`+T` の型引数位置に入力として使用）

- テスト: `v186000_tests`（5件）:
  - `version_is_18_6_0`
  - `variance_covariant_parses`（`interface X<+T> { ... }` が解析される）
  - `variance_contravariant_parses`（`interface X<-T> { ... }` が解析される）
  - `variance_subtype_covariant`（共変型でのサブタイピングが動作）
  - `variance_violation_error`（分散違反で E0334）

**完了条件（PASS=5）:**
1. `+T`（共変）/ `-T`（反変）アノテーションが解析される
2. 共変型引数でサブタイピングが機能する
3. 反変型引数で逆向きサブタイピングが機能する
4. 分散アノテーションと実際の使われ方が矛盾すると E0334 が出る
5. デフォルト（不変）では従来通りの型チェックが動作する

---

### v18.7.0 — 型レベル定数（Const Generics）

**テーマ**: バッチサイズ・配列サイズ等の定数をコンパイル時型情報として扱う。
「バッチサイズが 0 のまま本番デプロイ」のような設定ミスをコンパイル時に検出する。

**構文:**

```fav
// Int 型定数を型パラメータとして使用
type FixedBatch<T, const N: Int> = { items: List<T>, size: Int }

// N が 0 でないことを型レベルで保証
fn process_batch<T, const N: Int where { N > 0 }>(batch: FixedBatch<T, N>) -> List<Output> {
  List.chunk(batch.items, N) |> ...
}

// バッチサイズを型で表現
stage ProcessBatch<const BATCH_SIZE: Int where { BATCH_SIZE > 0 }>(
  rows: List<Row>
) -> List<Output> {
  let batches = List.chunk(rows, BATCH_SIZE)
  Result.ok(List.flat_map(batches, process_chunk))
}

// 使用例: 型引数でサイズを指定
let result = process_batch::<100>(my_batch)
// BATCH_SIZE = 0 を渡すと E0335: const constraint violation
let bad = process_batch::<0>(my_batch)  // E0335
```

**実装内容:**

- `fav/src/ast.rs`:
  - `GenericParam::Const { name: String, ty: Type, constraint: Option<Expr> }` 追加
  - `const N: Int` をジェネリクス宣言内で認識

- `fav/src/frontend/parser.rs`:
  - `parse_generic_params` で `const Name: Type` を認識

- `fav/src/middle/checker.rs`:
  - `eval_const_expr`: `where { N > 0 }` をコンパイル時に評価
  - `subst_const`: 定数型引数を式中の定数名に代入
  - E0335: const 制約違反

- `fav/src/middle/compiler.rs`:
  - `const` 型引数を具体的な整数値に置換して IR 生成

- テスト: `v187000_tests`（5件）:
  - `version_is_18_7_0`
  - `const_generic_parses`（`fn f<const N: Int>()` が解析される）
  - `const_generic_use`（型引数に整数を渡して動作）
  - `const_generic_constraint`（`where { N > 0 }` の制約が動作）
  - `const_generic_violation`（制約違反で E0335）

**完了条件（PASS=5）:**
1. `fn f<const N: Int>(...)` が定義できる
2. `f::<100>(...)` で定数型引数を渡せる
3. `where { N > 0 }` 制約が定数リテラルに対してコンパイル時評価される
4. 制約違反（`f::<0>()` で `N > 0` が成立しない）が E0335 になる
5. `const N` を型内の式で使用できる（`List.chunk(xs, N)` 等）

---

### v18.8.0 — 型駆動 API 生成（`fav generate` / `fav serve`）

**テーマ**: 型定義から REST / GraphQL API スキーマを自動生成する。
データパイプラインに HTTP エンドポイントを追加する際の、手書き OpenAPI の手間を排除する。

**構文:**

```fav
// #[api] アノテーションで HTTP エンドポイントを宣言
#[api(method = "GET", path = "/users/:id")]
fn get_user(id: Int) -> Result<User, String> !Db {
  bind rows <- Postgres.query_raw(f"SELECT * FROM users WHERE id = {id}", [])
  match rows {
    [user] => Result.ok(user)
    _      => Result.err("not found")
  }
}

#[api(method = "POST", path = "/orders")]
fn create_order(req: CreateOrderRequest) -> Result<Order, String> !Db {
  bind id <- Postgres.execute_raw("INSERT INTO orders ...", [])
  Result.ok({ id: id, ...req })
}

#[api(method = "GET", path = "/pipeline/status")]
fn get_pipeline_status() -> Result<PipelineStatus, String> {
  Result.ok({ running: true, last_run: DateTime.format_iso(DateTime.now()) })
}
```

**CLI コマンド:**

```bash
# OpenAPI 3.0 仕様書生成
fav generate api --format openapi --out api.yaml
fav generate api --format openapi --out api.json

# GraphQL スキーマ生成
fav generate api --format graphql --out schema.graphql

# 開発用 HTTP サーバー起動（ホットリロード）
fav serve src/api.fav --port 8080
# → GET /users/:id, POST /orders を自動でルーティング

# 型チェック込みの生成（スキーマ型と API が整合しているか確認）
fav generate api --check-schemas
```

**生成される OpenAPI の例:**

```yaml
openapi: "3.0.0"
info:
  title: "Favnir API"
  version: "1.0.0"
paths:
  /users/{id}:
    get:
      parameters:
        - name: id
          in: path
          schema: { type: integer }
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"
components:
  schemas:
    User:
      type: object
      properties:
        id: { type: integer }
        name: { type: string }
        email: { type: string }
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Annotation::Api { method: String, path: String }` 追加
  - `TopLevel::FnDef` に `annotations: Vec<Annotation>` フィールド追加

- `fav/src/frontend/parser.rs`:
  - `#[api(method = "...", path = "...")]` のアノテーション解析

- `fav/src/driver.rs`:
  - `cmd_generate_api(format: &str, out: &str, check_schemas: bool)` 実装
  - `cmd_serve(path: &str, port: u16)` 実装（`ureq`/`tiny_http` ベースの軽量 HTTP サーバー）

- `fav/src/codegen/openapi.rs`（新規）:
  - Favnir 型 → OpenAPI スキーマ変換
  - `#[api]` アノテーション → `paths` セクション生成

- `fav/src/codegen/graphql.rs`（新規）:
  - Favnir 型 → GraphQL SDL 変換
  - レコード型 → `type Query { ... }` 変換

- テスト: `v188000_tests`（5件）:
  - `version_is_18_8_0`
  - `api_annotation_parses`（`#[api(...)]` が解析される）
  - `openapi_generates`（`fav generate api --format openapi` が YAML を生成）
  - `graphql_generates`（`fav generate api --format graphql` が SDL を生成）
  - `serve_routes_request`（`fav serve` で HTTP リクエストが処理される）

**完了条件（PASS=5）:**
1. `#[api(method = "GET", path = "/...")]` アノテーションが解析される
2. `fav generate api --format openapi` が有効な OpenAPI 3.0 YAML を生成する
3. `fav generate api --format graphql` が有効な GraphQL SDL を生成する
4. `fav serve` で宣言したエンドポイントが HTTP リクエストを受け付ける
5. スキーマ型（v18.4）との整合性チェック（`--check-schemas`）が動作する

---

### v19.0.0 — Type System Maturity マイルストーン宣言

**テーマ**: v18.x シリーズの集大成。「信頼できる言語」への転換を宣言する。

**実装内容:**

- `Cargo.toml`: バージョンを `19.0.0` に更新

- `CHANGELOG.md`: v18.1.0〜v18.8.0 の全エントリ追加

- `README.md`:
  - 「現在の状態」を v19.0.0 に更新
  - Type System Maturity 達成を記載（effect inference / schema types / linear types / API generation）
  - バージョン履歴表に v18.1.0〜v19.0.0 エントリ追加

- `site/content/docs/`:
  - `language/effect-inference.mdx` 新規作成
  - `language/row-polymorphism.mdx` 新規作成
  - `language/refinement-types.mdx` 新規作成
  - `language/schema-types.mdx` 新規作成
  - `language/linear-types.mdx` 新規作成
  - `language/variance.mdx` 新規作成
  - `language/const-generics.mdx` 新規作成
  - `api/generate.mdx` 新規作成（`fav generate` ガイド）
  - `api/serve.mdx` 新規作成（`fav serve` ガイド）

- テスト: `v190000_tests`（5件）:
  - `version_is_19_0_0`
  - `changelog_has_v18_entries`（CHANGELOG に v18.x エントリが含まれる）
  - `readme_mentions_effect_inference`（README にエフェクト推論が記載されている）
  - `readme_mentions_schema_types`（README にスキーマ型が記載されている）
  - `api_docs_exist`（`site/content/docs/api/generate.mdx` が存在する）

**完了条件:**

| 確認項目 | 状態 |
|---|---|
| エフェクトが自動推論される | [ ] |
| 行多相レコード関数が動作する | [ ] |
| 関数引数の `where` 制約が動作する | [ ] |
| `schema "file:..."` から型が生成される | [ ] |
| 線形型による接続安全性が動作する | [ ] |
| 共変・反変アノテーションが動作する | [ ] |
| `const` 型パラメータが動作する | [ ] |
| `fav generate api` が動作する | [ ] |
| `cargo test v190000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 依存関係

```
v18.0.0（Language Power）✅
    |
    v18.1.0（エフェクト推論）← 後続 fn 定義のエフェクト宣言省略に恩恵
    |
    v18.2.0（行多相）         v18.3.0（Refinement Types）  ← 並列実施可能
    |                          |
    v18.4.0（スキーマ型）← 行多相と組み合わせて「スキーマの一部を使う関数」を書ける
    |
    v18.5.0（線形型）          v18.6.0（分散アノテーション）  ← 並列実施可能
    |                          |
    v18.7.0（Const Generics）← 線形型の「1 回使用」カウントに型レベル定数を活用
    |
    v18.8.0（API 生成）← スキーマ型（v18.4）と組み合わせて完全な型駆動 API 生成
    |
    v19.0.0（マイルストーン）
```

v18.1.0（エフェクト推論）は最優先（後続全バージョンでエフェクト宣言が楽になる）。
v18.2.0 と v18.3.0 は独立して並列実施可能。
v18.5.0 と v18.6.0 は独立して並列実施可能。
v18.8.0（API 生成）は v18.4.0（スキーマ型）の後に着手することで完成度が上がる。

---

## 新規 Cargo 依存（予定）

| Crate | 用途 | 追加バージョン |
|---|---|---|
| `tiny_http 0.12` | `fav serve` の軽量 HTTP サーバー | v18.8.0 |
| `serde_yaml 0.9` | OpenAPI YAML 生成 | v18.8.0 |
| その他 | なし（型システム拡張は Rust コードのみ） | — |

---

## 実装ノート

- **エフェクト推論とセルフホスト**: `checker.fav` にエフェクト推論を追加する際、`checker.fav` 自身のエフェクト推論も適用されることになる。段階的に適用し、`--legacy` モードでのフォールバックを維持する。
- **行多相の実装限界**: v18.2 では「フィールドの存在」制約のみ対応。フィールドの「不在」制約（`R without { error: String }`）は v19.x 以降で検討。
- **Refinement Types の SMT ソルバー非使用方針**: 完全なリファインメント型（Z3 等）は複雑すぎるため、v18.3 では「コンパイル時に評価可能なリテラル」と「実行時アサーション」の 2 モードのみ対応。
- **スキーマキャッシュの整合性**: DB スキーマが変わっても Favnir のキャッシュが古い場合、型エラーが出ない。`fav check --refresh-schemas` を CI で定期実行することを推奨ドキュメントに記載する。
- **線形型の実装方針**: v18.5 では `Connection` / `Tx` の 2 型のみを組み込み線形型とする。ユーザー定義の線形型は v19.x 以降（`#[linear]` アノテーション）。
- **分散アノテーションの推論**: v18.6 では明示アノテーション（`+T` / `-T`）のみ対応。自動推論（Scala の `@covariant`に相当）は v19.x 以降。
- **`fav serve` のパフォーマンス**: 開発用途のみを想定。本番は `fav deploy` で Lambda/ECS にデプロイする。`tiny_http` の選択は実装シンプルさ優先。
- **OpenAPI 生成の型マッピング**: `Int` → `integer`, `Float` → `number`, `String` → `string`, `Bool` → `boolean`, `List<T>` → `array`, レコード型 → `object`。`Result<T, E>` は `T` の型として生成（エラーは HTTP ステータスコードでハンドル）。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/roadmap-master.md` | v17.0〜v20.0 の全体戦略 |
| `versions/roadmap-v17.1-v18.0.md` | 直前ロードマップ（形式参照） |
| `fav/src/middle/checker.rs` | 型チェッカー（エフェクト推論・行多相・線形型追加対象） |
| `fav/src/ast.rs` | AST（新型・アノテーション追加対象） |
| `fav/src/middle/resolver.rs` | スキーマ型の取得・キャッシュ対象 |
| `fav/src/driver.rs` | CLI（generate / serve コマンド追加対象） |
| `self/checker.fav` | セルフホスト型チェッカー（拡張対象） |
| `site/content/docs/language/` | ドキュメント追加対象 |
