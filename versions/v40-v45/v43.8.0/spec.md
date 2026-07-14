# v43.8.0 仕様書 — 双方向型推論（Bidirectional / top-down）

## 概要

ロードマップ: "期待型の下向き伝播。関数が `Int -> Bool` を期待すれば `|x| x > 0` の `x: Int` が確定。"

### v43.8.0 スコープ

v43.8.0 は **バリデーションリリース**。

v43.5.0 で実装した `infer_list_lambda_call` が「リスト要素型からラムダ引数型を下向き伝播する」という双方向型推論の核心部分をすでに実現している。
本バージョンはそれをロードマップ例で明示的に検証し、**ネスト呼び出し式** での型伝播も確認する。

`checker.fav` への変更は不要。

### スコープ外（→ 将来バージョン）

- 匿名レコードリテラル（`{ name: "Alice", age: 30 }`）の文脈推論：`ERecordLit` の `tname = ""` 時に期待型から型名を補完する仕組みが未実装
- 関数型引数（`(Int -> Bool)` 型注釈から `|x| x > 0` の `x: Int` を決定）の一般化
- `ELambda` が `"Fn"` 固定でなく具体的な `Fn<A,B>` を返す型表現

---

## 現状確認

### `infer_list_lambda_call`（checker.fav line 1848）

```favnir
// v43.5.0: contextual lambda inference — propagate list element type to lambda param
fn infer_list_lambda_call(fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    match args {
        EArgList({ _0: list_expr, _1: rest_args }) => {
            match infer_expr(list_expr, env) {   // ← 引数のリスト型を推論（bottom-up）
                Ok(list_ty) => {
                    bind elem_ty <- type_str_inner(list_ty);  // ← "List<Int>" → "Int"
                    ...
                    ELambda({ _0: param, _1: body }) => {
                        bind lam_env <- env_insert(env, param, param_ty);  // ← ラムダ env に伝播（top-down）
                        Result.and_then(infer_expr(body, lam_env), |ret_ty|
                            if fname == "map" { Result.ok(wrap_in("List", ret_ty)) }
                            else { Result.ok(list_ty) })
```

この仕組みにより:
- `List.filter(xs, |x| x > 0)` — `xs: List<Int>` → `x: Int` を下向き伝播 ✓
- `List.map(xs, |x| x + 1)` → `List<Int>` を返す。その結果を `List.filter(_, |y| y > 0)` に渡すと `y: Int` が伝播 ✓（ネスト式）

---

## 事前確認（T0）

実装前に以下を手動確認する:
- `cargo test` → 2922 passed; 0 failed
- `Cargo.toml` version = `43.7.0`
- `driver.rs` に `v43800_tests` モジュールが存在しないこと
- `checker.fav` に `fn infer_list_lambda_call` が存在すること（現在 line 1849 付近）

---

## フィールド区切り記法

Favnir レコードリテラルのフィールド区切りはスペース（コンマ不要）。本バージョンではレコードリテラルを使わないため参考情報のみ。

---

## テスト設計

### v43.7.0 との差分

v43.7.0 は「名前付きレコードリテラルの型名一致（shallow check）」を検証した。
v43.8.0 は「**ネスト呼び出し式**でリスト要素型が正しくラムダ引数に伝播する」ことを検証する。

### `v43800_tests`（3 件）

#### `cargo_toml_version_is_43_8_0`

バージョン確認テスト（次バージョン bump 時にスタブ化）。

#### `bidirectional_filter_infers_elem_type`

ロードマップ記載の例を直接検証する:

```rust
let src = r#"
fn filter_positive(xs: List<Int>) -> List<Int> {
    List.filter(xs, |x| x > 0)
}
"#;
```

- `xs: List<Int>` → elem_ty = `"Int"` → `x: Int` として lambda body を検証
- `infer_list_lambda_call("filter", ...)` → `"List<Int>"` を返す
- `run_checker_fav` → `Ok(())`

#### `bidirectional_nested_map_filter_expression`

ネスト呼び出し式（bind なし）での型伝播を検証する:

```rust
let src = r#"
fn transform(xs: List<Int>) -> List<Int> {
    List.filter(List.map(xs, |x| x + 1), |y| y > 0)
}
"#;
```

評価経路:
1. `infer_list_lambda_call("filter", EArgList(ECall("List","map",...), ...), env)` を呼ぶ
2. `list_expr = ECall("List","map",...)` → `infer_expr` → `infer_call` → `infer_list_lambda_call("map",...)` → `"List<Int>"`
3. `type_str_inner("List<Int>")` = `"Int"` → `y: Int` として lambda body `y > 0` を評価
4. filter は `list_ty = "List<Int>"` を返す

根拠: `infer_op_with_newtypes(Add, "Int", "Int")` → `"Int"` は v43.5.0/v43.6.0 の `|x| x * 2` / `|x| x + 1` 系テストで間接的に検証済み。
ネスト ECall を `list_expr` に渡す直接パスは `infer_expr` の ECall アームが `infer_call` を呼ぶため動作する。
テストが失敗した場合は `infer_expr(ECall("List","map",...))` が `"Unknown"` を返していないかを最初に確認すること。

- `run_checker_fav` → `Ok(())`

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v43800_tests` 追加（3 件） |
| `fav/Cargo.toml` | version 43.7.0 → 43.8.0 |
| `CHANGELOG.md` | v43.8.0 エントリ追加 |
| `versions/current.md` | v43.8.0 最新安定版に更新 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.8.0 を COMPLETE に更新 |

**`fav/self/checker.fav` は変更不要**: v43.5.0 の `infer_list_lambda_call` で双方向推論が既に動作する。

---

## 完了条件

- `cargo test` 2925 tests passed, 0 failed（2922 + 3）
- `v43800_tests` 3 件 pass
- `bidirectional_filter_infers_elem_type`: ロードマップ記載例が `Ok(())`
- `bidirectional_nested_map_filter_expression`: ネスト式が `Ok(())`

---

## 影響範囲

- **checker.fav 変更なし**
- **既知制限**:
  - 匿名レコードリテラルの文脈推論は非対応（将来バージョン）
  - `ELambda` は `"Fn"` 固定（具体的な関数型表現は将来）
  - `EAccess`（フィールドアクセス）は `"Unknown"` 固定（将来）

---

## 前提条件

- v43.7.0 COMPLETE（2922 tests）
- `infer_list_lambda_call`（checker.fav line 1848）が `List.map` / `List.filter` のラムダ引数型伝播を担当
- `type_str_inner("List<Int>")` → `"Int"`（line ~1700）
- `wrap_in("List", ty)` → `"List<ty>"`（line ~1700）
