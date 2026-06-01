# Favnir v9.5.0 仕様書

Date: 2026-06-01
Theme: HTTP / gRPC / GraphQL — 型付き API アクセス層の整備 + `!Http` / `!GraphQL` エフェクト追加

---

## 背景・現状分析

### 既存実装のギャップ

| Rune | 現状 | ギャップ |
|---|---|---|
| `runes/http/` | `get`/`post`/`put`/`delete` が `!Network` で実装済み | `get_json<T>` / `post_json_typed<T,R>` がない。`!Http` エフェクト未登録 |
| `runes/grpc/` | `call`/`call_stream`/`call_typed` が `!Rpc` で実装済み（`Map<String,String>` 型） | `call_typed<T,R>` ジェネリック版がない。checker.fav が Grpc 未認識 |
| GraphQL | `fav build --graphql`（SDL コード生成）のみ | 実行時クライアント Rune が存在しない |

### 新エフェクト追加方針

**後方互換性を維持しつつ `!Http` / `!GraphQL` を新規追加する。**

- `!Network` / `!Rpc` は削除しない（既存コード 40 箇所に影響）
- `Http.*` primitive は `!Network` または `!Http` どちらでも受け付ける
- `Grpc.*` primitive は `!Rpc` のみ（変更なし）
- GraphQL は HTTP POST ベースなので `!Http` エフェクトを使う（`!GraphQL` は使わない）

---

## HTTP 拡張（v9.5.0 追加分）

### 新 builtin: `Http.get_body_raw`

`HttpResponse.body` フィールドアクセスを避けるため、body String のみ返す primitive を追加。

```rust
// vm.rs
"Http.get_body_raw" => Ok(VMValue::Str(http_get_body(url)?))
```

### 新規 Rune 関数（`runes/http/request.fav` に追加）

```favnir
// HTTP GET → String（エラー型 String）
public fn get_text(url: String) -> Result<String, String> !Http {
    Http.get_body_raw(url)
}

// HTTP GET → 型 T（JSON デコード付き）
public fn get_json<T>(url: String) -> Result<T, String> !Http {
    match Http.get_body_raw(url) {
        Err(e) => Result.err(e)
        Ok(body) =>
            match Json.parse_raw(body) {
                Err(e) => Result.err(String.concat("get_json: ", e))
                Ok(raw) =>
                    match Schema.adapt_one(raw, type_name_of<T>()) {
                        Err(_) => Result.err("get_json: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}

// HTTP POST（型 T → JSON → 送信、レスポンスを型 R に変換）
public fn post_json_typed<T, R>(url: String, body: T) -> Result<R, String> !Http {
    match Http.post_raw(url, Schema.to_json(body, type_name_of<T>()), "application/json") {
        Err(e) => Result.err(e)
        Ok(resp) =>
            match Json.parse_raw(resp.body) {
                Err(e) => Result.err(String.concat("post_json_typed: ", e))
                Ok(raw) =>
                    match Schema.adapt_one(raw, type_name_of<R>()) {
                        Err(_) => Result.err("post_json_typed: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}
```

---

## gRPC 拡張（v9.5.0 追加分）

### 新規 Rune 関数（`runes/grpc/client.fav` に追加）

```favnir
// 型付き単一レスポンス（T にデシリアライズ）
public fn call_json<T>(host: String, method: String, payload: Map<String, String>) -> Result<T, String> !Rpc {
    match Grpc.call_raw(host, method, payload) {
        Err(e) => Result.err(e.message)
        Ok(row) =>
            match Json.parse_raw(Grpc.encode_raw(type_name_of<T>(), row)) {
                Err(e) => Result.err(String.concat("call_json: ", e))
                Ok(raw) =>
                    match Schema.adapt_one(raw, type_name_of<T>()) {
                        Err(_) => Result.err("call_json: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}

// 型付きリスト（List<T> にデシリアライズ）
public fn call_list<T>(host: String, method: String, payload: Map<String, String>) -> List<T> !Rpc {
    Grpc.call_stream_raw(host, method, payload)
}
```

注: `call_json` は `Grpc.encode_raw` + `Schema.adapt_one` で Map → T 変換を行う。
実際のユースケースでは `grpc.call_typed` の後に手動で `Schema.adapt_one` するパターンが多い。

---

## GraphQL Rune（新規 `runes/graphql/`）

### 設計方針

GraphQL はすべて HTTP POST（`Content-Type: application/json`）で送信する。
`!Http` エフェクトをそのまま使用。新規エフェクト `!GraphQL` は追加しない。

### ディレクトリ構成

```
runes/graphql/
  rune.toml
  graphql.fav       # エントリポイント
  client.fav        # query / mutate / subscribe
```

### API

```favnir
// GraphQL クエリ（レスポンスを型 T に変換）
public fn query<T>(url: String, gql: String, variables: Map<String, String>) -> Result<T, String> !Http {
    // { "query": gql, "variables": { ... } } を POST
    bind resp_body <- Http.get_body_raw(url)  // ← 実際は POST
    ...
}

// GraphQL ミューテーション
public fn mutate<T>(url: String, gql: String, variables: Map<String, String>) -> Result<T, String> !Http {
    ...
}

// 変数なしのシンプルクエリ
public fn query_simple<T>(url: String, gql: String) -> Result<T, String> !Http {
    query<T>(url, gql, Map.empty())
}
```

### 内部実装（`Http.post_raw` を直接呼ぶ）

```favnir
public fn query<T>(url: String, gql: String, vars: Map<String, String>) -> Result<T, String> !Http {
    bind resp <- Http.post_raw(url,
        String.concat("{\"query\":\"", String.concat(gql, "\"}")),
        "application/json")
    match Json.parse_raw(resp.body) {
        Err(e) => Result.err(String.concat("graphql.query: ", e))
        Ok(raw) =>
            match Schema.adapt_one(raw, type_name_of<T>()) {
                Err(_) => Result.err("graphql.query: schema error")
                Ok(v)  => Result.ok(v)
            }
    }
}
```

---

## エフェクト設計（Rust 変更）

### `Effect` enum への `Http` 追加（`src/ast.rs`）

```rust
pub enum Effect {
    Pure, Io, Db, DbRead, DbWrite, DbAdmin,
    Network,
    Http,      // 追加
    Rpc,
    File, Checkpoint, Trace,
    Emit(String), EmitUnion(Vec<String>), Unknown(String),
}
```

### 変更が必要なファイル

| ファイル | 変更内容 |
|---|---|
| `src/ast.rs` | `Effect::Http` variant 追加 |
| `src/frontend/parser.rs` | `"Http" => Effect::Http` を parse_effect_ann に追加 |
| `src/fmt.rs` | `Effect::Http => Some("!Http".to_string())` |
| `src/lineage.rs` | `Http => "!Http".into()` |
| `src/driver.rs` | `ast::Effect::Http => "Http".into()` （effect_to_string 関数） |
| `src/middle/ast_lower_checker.rs` | `ast::Effect::Http => "Http".to_string()` |
| `src/middle/checker.rs` | BUILTIN_EFFECTS に `"Http"` 追加、Http.* で `!Http` も受け付け |
| `src/middle/reachability.rs` | `Effect::Http => {...}` を Network と同じパターンで追加 |

### `checker.fav` への追加

```favnir
fn http_fn(fname: String) -> String {
    if fname == "get_raw"         { "Result" }
    else if fname == "post_raw"   { "Result" }
    else if fname == "put_raw"    { "Result" }
    else if fname == "delete_raw" { "Result" }
    else if fname == "patch_raw"  { "Result" }
    else if fname == "get_body_raw" { "Result" }
    else { "Result" }
}

fn grpc_fn(fname: String) -> String {
    if fname == "call_raw"        { "Result" }
    else if fname == "call_typed_raw" { "Result" }
    else if fname == "call_stream_raw" { "List" }
    else if fname == "encode_raw" { "String" }
    else if fname == "decode_raw" { "Map" }
    else { "Unknown" }
}
```

---

## 型シグネチャ一覧（v9.5.0 追加分）

| 関数 | シグネチャ | エフェクト |
|---|---|---|
| `http.get_text` | `String -> Result<String, String>` | `!Http` |
| `http.get_json<T>` | `String -> Result<T, String>` | `!Http` |
| `http.post_json_typed<T, R>` | `(String, T) -> Result<R, String>` | `!Http` |
| `grpc.call_json<T>` | `(String, String, Map) -> Result<T, String>` | `!Rpc` |
| `grpc.call_list<T>` | `(String, String, Map) -> List<T>` | `!Rpc` |
| `graphql.query<T>` | `(String, String, Map) -> Result<T, String>` | `!Http` |
| `graphql.mutate<T>` | `(String, String, Map) -> Result<T, String>` | `!Http` |
| `graphql.query_simple<T>` | `(String, String) -> Result<T, String>` | `!Http` |

---

## 実装上の注意事項

1. **ジェネリック関数内のフィールドアクセス** — `resp.body` は Rust pipeline で stack overflow を引き起こす可能性がある。`Http.get_body_raw` primitive を追加して回避する。

2. **`post_json_typed<T, R>` の `resp.body`** — post_raw が返す `HttpResponse` の body にアクセスする。HTTP の場合は `Http.get_body_raw` が使えるが、POST の場合は新 primitive `Http.post_body_raw(url, body, ct)` が必要になる可能性がある。

3. **GraphQL 変数のシリアライズ** — `variables: Map<String, String>` を JSON に変換する際に `Schema.to_json` が使えない（Map のキーが文字列型のみ）。`Map.to_json_raw` のような builtin が必要。または variables を String として渡すシンプルな設計にする。
