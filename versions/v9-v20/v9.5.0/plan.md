# Favnir v9.5.0 実装計画

Date: 2026-06-01
Theme: HTTP / gRPC / GraphQL — 型付き API アクセス層の整備

---

## 前提確認

| 項目 | 現状 |
|---|---|
| `runes/http/` | v4.2.0。`get`/`post`/`put`/`delete`/`patch` が `!Network` で実装済み |
| `runes/grpc/` | v4.2.0。`call`/`call_stream`/`call_typed` が `!Rpc` で実装済み（型なし Map ベース） |
| `runes/graphql/` | 存在しない。`fav build --graphql` はコード生成のみ |
| `Effect` enum | `ast.rs`: Network / Rpc は存在。`Http` は未定義（`Unknown("Http")`になる） |
| `!Network` 使用箇所 | runes/ 内 40 箇所。変更しない |
| Effect 変更ファイル | ast.rs / parser.rs / fmt.rs / lineage.rs / driver.rs / ast_lower_checker.rs / checker.rs / reachability.rs |
| `Http.get_body_raw` | 未実装。フィールドアクセス回避のために追加が必要 |
| `Http.post_body_raw` | 未実装。POST レスポンス body 取得のために追加が必要 |

---

## フェーズ構成

```
Phase A: Rust — Effect::Http 追加（8ファイル）
Phase B: Rust — vm.rs に Http.get_body_raw / Http.post_body_raw 追加
Phase C: Rust — checker.rs に Http/GraphQL 型情報追加
Phase D: checker.fav — http_fn / grpc_fn / graphql_fn 追加
Phase E: http Rune 拡張（get_text / get_json<T> / post_json_typed<T,R>）
Phase F: grpc Rune 拡張（call_json<T> / call_list<T>）
Phase G: graphql Rune 新規作成
Phase H: 統合テスト（driver.rs）
Phase I: バージョン更新・commit
```

---

## Phase A: Effect::Http 追加（Rust 8 ファイル）

### A-1: `src/ast.rs`
```rust
pub enum Effect {
    ...
    Network,
    Http,   // 追加
    Rpc,
    ...
}
```

### A-2: `src/frontend/parser.rs`
`parse_effect_ann` の match に追加：
```rust
"Http" => {
    self.advance();
    Effect::Http
}
```
`"Network"` ブロックの直前（アルファベット順）または直後に追加。

### A-3: `src/fmt.rs`
```rust
Effect::Http => Some("!Http".to_string()),
```
`Effect::Network` の近くに追加。

### A-4: `src/lineage.rs`
```rust
Http => "!Http".into(),
```

### A-5: `src/driver.rs`
`effect_to_string` 関数（または `format_effect` 関数）に：
```rust
ast::Effect::Http => "Http".into(),
```

### A-6: `src/middle/ast_lower_checker.rs`
```rust
ast::Effect::Http => "Http".to_string(),
```

### A-7: `src/middle/checker.rs`
- `BUILTIN_EFFECTS` に `"Http"` 追加
- `Http.*` 呼び出し effect チェックを `Network | Http` に緩和：
  ```rust
  if !self.has_effect(|e| matches!(e, Effect::Network | Effect::Http)) {
      // E0003
  }
  ```

### A-8: `src/middle/reachability.rs`
```rust
Effect::Http => {
    // Network と同じパターン
}
```

---

## Phase B: vm.rs — 新 primitive 追加

### B-1: `Http.get_body_raw(url: String) -> Result<String, String>`
HTTP GET してレスポンス body のみ返す。`HttpResponse` 構造体を経由しない。
```rust
"Http.get_body_raw" => {
    let url = vm_string(&args[0])?;
    match http_get(&url) {
        Ok(resp) => Ok(VMValue::Str(resp.body)),
        Err(e) => ok_err_str(e.to_string()),
    }
}
```
→ `Result<String, String>` として返す（Result.ok/err ラップ）。

### B-2: `Http.post_body_raw(url: String, body: String, ct: String) -> Result<String, String>`
HTTP POST してレスポンス body のみ返す。
```rust
"Http.post_body_raw" => {
    let url = vm_string(&args[0])?;
    let body = vm_string(&args[1])?;
    let ct = vm_string(&args[2])?;
    match http_post(&url, &body, &ct) {
        Ok(resp) => Ok(VMValue::Str(resp.body)),
        Err(e) => ok_err_str(e.to_string()),
    }
}
```

### B-3: `checker.rs` に型シグネチャ追加
```rust
("Http", "get_body_raw") => {
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Http", "post_body_raw") => {
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
```

---

## Phase C: checker.rs — GraphQL 型情報

GraphQL Rune は内部で `Http.*` primitive を使うため、新規 builtin は不要。
checker.rs への追加は B-3 のみ。

---

## Phase D: checker.fav 更新

### D-1: `fn http_fn` 追加（Phase A-6 の後）

```favnir
fn http_fn(fname: String) -> String {
    if fname == "get_raw"           { "Result" }
    else if fname == "post_raw"     { "Result" }
    else if fname == "put_raw"      { "Result" }
    else if fname == "delete_raw"   { "Result" }
    else if fname == "patch_raw"    { "Result" }
    else if fname == "get_body_raw" { "Result" }
    else if fname == "post_body_raw" { "Result" }
    else { "Result" }
}
```

### D-2: `fn grpc_fn` 追加

```favnir
fn grpc_fn(fname: String) -> String {
    if fname == "call_raw"             { "Result" }
    else if fname == "call_typed_raw"  { "Result" }
    else if fname == "call_stream_raw" { "List" }
    else if fname == "encode_raw"      { "String" }
    else if fname == "decode_raw"      { "Map" }
    else { "Unknown" }
}
```

### D-3: `builtin_ret_ty` に Http / Grpc 追加

```favnir
else if ns == "Http"  { http_fn(fname) }
else if ns == "Grpc"  { grpc_fn(fname) }
```

### D-4: `ns_to_effect` に Http 追加

```favnir
else if ns == "Http" { "Http" }
```
（Grpc は既存の `!Rpc` が `ns_to_effect` に未登録なので確認して追加）

---

## Phase E: http Rune 拡張（`runes/http/request.fav`）

### E-1: `get_text`

```favnir
public fn get_text(url: String) -> Result<String, String> !Http {
    Http.get_body_raw(url)
}
```

### E-2: `get_json<T>`

```favnir
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
```

### E-3: `post_json_typed<T, R>`

```favnir
public fn post_json_typed<T, R>(url: String, body: T) -> Result<R, String> !Http {
    match Http.post_body_raw(url, Schema.to_json(body, type_name_of<T>()), "application/json") {
        Err(e) => Result.err(e)
        Ok(resp_body) =>
            match Json.parse_raw(resp_body) {
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

### E-4: `http.fav` の use 文更新

```favnir
use request.{ get, post, post_json, get_body, get_text, get_json, post_json_typed }
```

---

## Phase F: grpc Rune 拡張（`runes/grpc/client.fav`）

### F-1: `call_json<T>`

gRPC レスポンス（`Map<String,String>`）を型 T に変換するラッパー。
`Grpc.encode_raw` で Map を JSON にして `Schema.adapt_one` で変換。

```favnir
public fn call_json<T>(host: String, method: String, payload: Map<String, String>) -> Result<T, String> !Rpc {
    match Grpc.call_raw(host, method, payload) {
        Err(e) => Result.err(e.message)
        Ok(row) =>
            match Schema.adapt_one(Json.parse_raw(Grpc.encode_raw(type_name_of<T>(), row)), type_name_of<T>()) {
                Err(_) => Result.err("call_json: schema error")
                Ok(v)  => Result.ok(v)
            }
    }
}
```

注: `Json.parse_raw(Grpc.encode_raw(...))` のネストが Rust pipeline で問題になる可能性あり。
問題が出た場合は `Grpc.call_raw` → `Grpc.encode_raw` の 2 ステップに分ける（bind で）。

### F-2: `call_list<T>`

gRPC ストリームレスポンスをそのまま返す（型 T への変換はユーザー側）。

```favnir
public fn call_list<T>(host: String, method: String, payload: Map<String, String>) -> List<T> !Rpc {
    Grpc.call_stream_raw(host, method, payload)
}
```

注: `Grpc.call_stream_raw` が `List<Map<String,String>>` を返すため、`List<T>` との型不一致が checker で出る可能性あり。
その場合は `List<Map<String, String>>` を明示する。

### F-3: `grpc.fav` の use 文更新

```favnir
use client.{ call, call_stream, call_typed, call_json, call_list }
```

---

## Phase G: graphql Rune 新規作成（`runes/graphql/`）

### G-1: `rune.toml` 作成

```toml
name = "graphql"
version = "9.5.0"
entry = "graphql.fav"
```

### G-2: `client.fav` 作成

```favnir
// runes/graphql/client.fav — GraphQL クライアント (v9.5.0)

// GraphQL リクエストを組み立てて POST する内部関数
fn gql_post(url: String, query_str: String) -> Result<String, String> !Http {
    Http.post_body_raw(url,
        String.concat("{\"query\":\"", String.concat(query_str, "\"}")),
        "application/json")
}

// query<T> — GraphQL クエリを実行してレスポンスを型 T に変換
public fn query<T>(url: String, gql: String) -> Result<T, String> !Http {
    match gql_post(url, gql) {
        Err(e) => Result.err(e)
        Ok(body) =>
            match Json.parse_raw(body) {
                Err(e) => Result.err(String.concat("graphql.query: ", e))
                Ok(raw) =>
                    match Schema.adapt_one(raw, type_name_of<T>()) {
                        Err(_) => Result.err("graphql.query: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}

// mutate<T> — GraphQL ミューテーションを実行（query と同一実装）
public fn mutate<T>(url: String, gql: String) -> Result<T, String> !Http {
    match gql_post(url, gql) {
        Err(e) => Result.err(e)
        Ok(body) =>
            match Json.parse_raw(body) {
                Err(e) => Result.err(String.concat("graphql.mutate: ", e))
                Ok(raw) =>
                    match Schema.adapt_one(raw, type_name_of<T>()) {
                        Err(_) => Result.err("graphql.mutate: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}
```

### G-3: `graphql.fav` 作成（エントリポイント）

```favnir
// runes/graphql/graphql.fav — Favnir graphql rune public API (v9.5.0)
// GraphQL クライアント: HTTP POST ベース、!Http エフェクト。

use client.{ query, mutate }
```

---

## Phase H: 統合テスト（driver.rs）

新規モジュール `v950_tests`：

- `http_effect_http_accepted` — `!Http` 宣言関数で Http.* 呼び出しが E0003 を出さない
- `http_effect_missing_errors` — `!Http`/`!Network` 未宣言で E0003 が出る
- `http_get_body_raw_bad_url_err` — 不正 URL で `Result.err` を返す
- `lineage_http_effect_in_sources` — `!Http` stage が lineage Sources に表示される
- `grpc_checker_fav_grpc_fn` — `builtin_ret_ty("Grpc", "call_raw") == "Result"`
- `graphql_rune_test_file_passes` — graphql.test.fav が全テスト通過

---

## Phase I: バージョン更新・commit

- `fav/Cargo.toml` → `"9.5.0"`
- `fav/self/cli.fav` → `"9.5.0"`
- tasks.md 完了チェック
- MEMORY.md 更新
- commit

---

## リスク管理

| リスク | 対策 |
|---|---|
| `resp.body` フィールドアクセスで stack overflow | Phase B で `Http.get_body_raw` / `Http.post_body_raw` primitive を追加して回避 |
| `Effect::Http` の match 漏れ | `cargo build` で exhaustive match エラーが出るので確実に全箇所修正 |
| `call_list<T>` の型不一致 | 型パラメータ T を Map に制約するか、`List<Map<String,String>>` で返す設計に変更 |
| GraphQL 変数（Map）の JSON 変換 | v9.5.0 では variables を String として渡す簡易設計にする |
