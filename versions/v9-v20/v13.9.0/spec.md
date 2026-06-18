# v13.9.0 Spec — 型状態パターン統合 + lineage 更新

Date: 2026-06-11

---

## 概要

capability-context 設計の「型状態パターン」側を整備し、
lineage 解析を新しい capability 型に対応させる。

2 つの柱:

1. **E0024 — 型状態フェーズ違反**: `Loaded` / `Validated` / `Transformed` のフェーズを
   飛ばした呼び出しをコンパイルエラーとして検出する。
2. **lineage 解析更新**: `fav explain --lineage` の分類を `!Effect` ベースから
   capability 型（`DbRead` / `DbWrite` 等）ベースに移行する。

オプション機能として `fav doc --builtins --format json` の出力に
`capability` フィールドを追加する。

---

## 現状（v13.8.0）

### 型状態パターンの問題

```fav
// 型状態パターン（設計上の慣用）
fn load(ctx: LoadCtx, path: String) -> Result<Loaded, String>
fn validate(d: Loaded) -> Result<Validated, String>
fn transform(d: Validated) -> Result<Transformed, String>

// 問題: フェーズを飛ばしても現在はコンパイルエラーにならない
fn broken(ctx: LoadCtx, path: String) -> Result<Transformed, String> {
    chain rows  <- load(ctx, path)
    transform(rows)   // Loaded を渡しているが Validated を期待: 今は実行時エラー
}
```

### lineage 解析の問題

```
$ fav explain --lineage pipeline.fav

transformations:
  - name: load_rows
    effects: ["!Postgres(read)"]    ← エフェクト型ベース（旧設計）
  - name: validate
    effects: ["Pure"]
  - name: save_result
    effects: ["!AWS"]               ← capability 種別が不明
```

capability 型移行後は `!Postgres(read)` ではなく `DbRead` capability を持つ
ステージとして表現すべきだが、現在の `lineage.rs` は `ast::Effect` を直接参照している。

---

## 新しい動作

### E0024 — 型状態フェーズ違反

```
$ fav check pipeline.fav

error[E0024]: type state mismatch — expected `Validated`, got `Loaded`
  --> pipeline.fav:12:15
11 |
12 |     transform(rows)
   |               ^^^^ type `Loaded` cannot be used where `Validated` is expected
   |
   = help: call `validate(rows)` before `transform`
   = help: type state sequence: Loaded → Validated → Transformed
error: 1 type state mismatch(es) (E0024)
```

### lineage 解析の新出力形式

```yaml
transformations:
  - name: load_rows
    kind: read          ← capability ベース分類
    capability: DbRead
    sources: ["orders"]
  - name: validate
    kind: transform     ← capability なし = 純粋
    capability: null
  - name: save_result
    kind: sink          ← StorageWrite
    capability: StorageWrite
    sinks: ["s3://results/"]
  - name: aggregate
    kind: transform     ← 純粋変換
    capability: null
```

---

## E0024 の適用ルール

### 型状態名前の規則

以下の命名規則を持つ名目型を「型状態型」として扱う:

```fav
// 単純な型状態（wrapper ありなし問わず）
type Loaded(List<Row>)
type Validated(List<Row>)
type Transformed(List<Row>)
```

型状態型の同定: 同一ファイル内で「型状態シーケンス」として宣言されたもの。
v13.9.0 では明示的シーケンス宣言（`type_state_seq` ディレクティブ）は導入せず、
**関数シグネチャの引数型と戻り値型から暗黙的にシーケンスを推論**する。

### 推論ルール

```
fn f(d: A) -> Result<B, E>  → A が B の前フェーズ候補
fn g(d: B) -> Result<C, E>  → B が C の前フェーズ候補
```

`A → B → C` のシーケンスが推論された場合:
- `g(d_of_type_a)` は E0024（A を渡しているが B を期待）

### エラーになる例

```fav
type Loaded(List<Row>)
type Validated(List<Row>)

fn validate(d: Loaded) -> Result<Validated, String>
fn transform(d: Validated) -> Result<Transformed, String>

fn broken(rows: Loaded) -> Result<Transformed, String> {
    transform(rows)  // E0024: got Loaded, expected Validated
}
```

### エラーにならない例

```fav
// 正しい順序
fn correct(rows: Loaded) -> Result<Transformed, String> {
    chain v <- validate(rows)
    transform(v)  // OK: Validated → Transformed
}

// 型状態シーケンスに含まれない型は対象外
fn pure_fn(x: Int) -> Int {
    x * 2  // OK
}
```

### v13.9.0 の適用範囲

- 同一ファイル内で定義された型状態シーケンスのみ対象
- クロスファイルの型状態チェックは v14.0.0 以降
- `--legacy` モードでは E0024 を W011 に降格（後方互換）

---

## lineage 解析の更新

### 新しい分類ロジック

| 分類 | 判定条件 | 表示 |
|---|---|---|
| `read` | 関数パラメータに `DbRead` を持つ（または `LoadCtx`/`AppCtx` を持ち DB 読み取り呼び出しあり） | `kind: "read"` |
| `write` | 関数パラメータに `DbWrite` を持つ | `kind: "write"` |
| `sink` | 関数パラメータに `StorageWrite` を持つ（S3 put 等） | `kind: "sink"` |
| `transform` | capability 引数なし（純粋関数） | `kind: "transform"` |
| `io` | 関数パラメータに `Io` を持つ（stdout 出力のみ） | `kind: "io"` |

### 後方互換

- `--legacy` モードでは旧 `effects` フィールドも出力（`effects: ["!Postgres(read)"]`）
- 標準出力では `kind` + `capability` に統一
- `fav explain --lineage --json` の JSON フォーマット変更あり（`effects` → `kind`/`capability`）

---

## `fav doc --builtins --format json` の更新

`BuiltinPrimitive` 構造体に `capability: Option<&'static str>` フィールドを追加:

```json
{
  "namespace": "DbRead",
  "name": "query",
  "signature": "(sql: String, params: List<String>) -> Result<List<Row>, String>",
  "capability": "DbRead",
  "impls": ["PostgresDb", "SnowflakeDb", "MockDb"],
  "returns_result": true,
  "description": "Execute a read query and return rows"
}
```

`impls` は v13.9.0 では static リストとして実装（型システム解析は後回し）。

---

## エラーコード

### E0024: type state mismatch

```
E0024: type state mismatch
  --> pipeline.fav:12:15
   |
12 |     transform(rows)
   |               ^^^^ expected `Validated`, got `Loaded`
   |
   = help: call `validate(rows)` before `transform`
```

トリガー条件:
- 推論された型状態シーケンス `A → B → C` において、`B` を要求する関数に `A` を渡した場合
- 非 legacy モードのみ（`--legacy` では W011 に降格）

---

## スコープ外（v13.9.0 では実装しない）

- 型状態シーケンスの明示的宣言ディレクティブ（`type_state_seq`）
- クロスファイル型状態チェック
- `fav migrate` ツール（v13.10.0）
- `!` 記法廃止（v13.10.0）
- lineage グラフの可視化（Mermaid 出力等）

---

## 後方互換性

- `--legacy` モードで E0024 は W011 に降格（ゼロエラー）
- lineage 出力の旧フォーマットは `--legacy` で保持
- 既存の W008 / E0023 テストは全件パスを維持
