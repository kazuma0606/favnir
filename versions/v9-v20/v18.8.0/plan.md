# v18.8.0 実装計画 — 型駆動 API 生成

Date: 2026-06-16

## 実装順序

```
T1: ast.rs — ApiAnnotation + FnDef.api_annotation 追加
T2: parser.rs — #[api(...)] パース実装
T3: driver.rs — 波及ファイル修正（exhaustive match / FnDef struct リテラル）
T4: driver.rs — cmd_generate_api / build_openapi_json / build_graphql_sdl 実装
T5: driver.rs — cmd_serve + route_table 実装（tiny_http 追加）
T6: driver.rs — v188000_tests 追加
T7: Cargo.toml — バージョン更新、tiny_http 依存追加
T8: site/content/docs/api/generate.mdx + serve.mdx 作成
```

---

## T1: ast.rs

### 追加する型

```rust
// ── ApiAnnotation (v18.8.0) ───────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ApiAnnotation {
    pub method: String,  // "GET", "POST", "PUT", "DELETE", "PATCH"
    pub path: String,    // "/users/:id"
    pub span: Span,
}
```

### FnDef へのフィールド追加

```rust
pub struct FnDef {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub effects: Vec<Effect>,
    pub body: Block,
    pub span: Span,
    pub api_annotation: Option<ApiAnnotation>,  // v18.8.0: #[api(...)]
}
```

**波及**: `FnDef { ... }` struct リテラルが多数のファイルに存在するため、`api_annotation: None` を全箇所に追加が必要。Grep で全件確認してから一括修正。

---

## T2: parser.rs

### `parse_api_annotation` メソッド

```rust
/// Parse `#[api(method = "...", path = "...")]` before a fn definition.
/// Returns None if the next token is not `#`.
fn parse_api_annotation(&mut self) -> Result<Option<ApiAnnotation>, ParseError> {
    if self.peek() != &TokenKind::Hash {
        return Ok(None);
    }
    // lookahead: is it #[api(...)] ?
    // peek(1) = LBracket, peek(2) = Ident("api")
    let is_api = matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "api"));
    if !is_api {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance(); // #
    self.expect(&TokenKind::LBracket)?;
    self.expect_ident()?; // "api"
    self.expect(&TokenKind::LParen)?;
    // method = "..."
    self.expect_ident_name("method")?;
    self.expect(&TokenKind::Eq)?;
    let method = self.expect_str()?;
    self.expect(&TokenKind::Comma)?;
    // path = "..."
    self.expect_ident_name("path")?;
    self.expect(&TokenKind::Eq)?;
    let path = self.expect_str()?;
    // optional trailing comma
    if self.peek() == &TokenKind::Comma { self.advance(); }
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(ApiAnnotation { method, path, span: self.span_from(&start) }))
}
```

ヘルパーメソッドとして以下を追加:
- `expect_ident_name(&str)` — 指定名の Ident を消費
- `expect_str()` — `TokenKind::Str(s)` を消費して `s` を返す

### `parse_item` の変更

`parse_item` の冒頭（`fn` / `public fn` の前）で `parse_api_annotation()` を呼び出し、`FnDef` に付与:

```rust
let api_annotation = self.parse_api_annotation()?;
// ... parse visibility, async, fn keyword ...
// FnDef::parse result に api_annotation を設定
fd.api_annotation = api_annotation;
```

### `parse_fn_def` の変更

`FnDef { ..., api_annotation: None }` を構築している箇所に `api_annotation: None` を追加（T3 の波及修正と合わせて実施）。

---

## T3: driver.rs その他 — 波及修正

### FnDef struct リテラルへの追加

Grep で `FnDef {` を検索して全件に `api_annotation: None` を追加:

```bash
grep -n "FnDef {" src/driver.rs src/middle/checker.rs src/middle/compiler.rs src/fmt.rs
```

主な対象ファイル:
- `fav/src/frontend/parser.rs` — `parse_fn_def` の戻り値
- `fav/src/middle/ast_lower_checker.rs` — FnDef を作成する箇所（あれば）
- `fav/src/driver.rs` — テスト内でのインライン構築（あれば）
- その他 Grep で発見した箇所

### fmt.rs / emit_python.rs / compiler.rs / checker.rs

`FnDef` フィールドを pattern match している箇所（`fd.api_annotation` にアクセスしないなら変更不要）。struct フィールドのデストラクチャリングをしている場合のみ修正が必要。

---

## T4: driver.rs — API 生成コマンド実装

### 主要関数

```rust
/// Collect all #[api]-annotated fn defs from a program.
fn collect_api_fns(prog: &Program) -> Vec<(&FnDef, &ApiAnnotation)>

/// Convert a Favnir return type to OpenAPI response schema (JSON string).
/// Result<T, E> → 200: T schema, 400: { error: string }
fn openapi_response_schema(ret_ty: &TypeExpr) -> serde_json::Value

/// Convert a Favnir TypeExpr to OpenAPI schema object.
fn type_expr_to_openapi_schema(te: &TypeExpr) -> serde_json::Value

/// Build OpenAPI 3.0 JSON from a list of API functions and type definitions.
fn build_openapi_json(api_fns: &[(&FnDef, &ApiAnnotation)], prog: &Program) -> serde_json::Value

/// Build GraphQL SDL from a list of API functions and type definitions.
fn build_graphql_sdl(api_fns: &[(&FnDef, &ApiAnnotation)], prog: &Program) -> String

/// Extract component schemas (named record types) from the program.
fn collect_component_schemas(prog: &Program) -> serde_json::Map<String, serde_json::Value>

/// Convert `:param` path style (Favnir) to `{param}` path style (OpenAPI).
fn path_to_openapi(path: &str) -> String  // "/users/:id" → "/users/{id}"
```

### `build_openapi_json` の構造

```json
{
  "openapi": "3.0.0",
  "info": { "title": "Favnir API", "version": "1.0.0" },
  "paths": {
    "/users/{id}": {
      "get": {
        "parameters": [...],
        "responses": { "200": {...}, "400": {...} }
      }
    }
  },
  "components": {
    "schemas": {
      "User": { "type": "object", "properties": {...} }
    }
  }
}
```

### `cmd_generate_api` の実装

```rust
pub fn cmd_generate_api(source: &str, format: &str, out: Option<&str>, as_json: bool)
```

- `format = "openapi"` → `build_openapi_json` → YAML（`serde_yaml`）または JSON 出力
- `format = "graphql"` → `build_graphql_sdl` → テキスト出力
- `out = None` → stdout 出力

### CLI への追加（`cmd_generate` の拡張）

既存の `fav generate` コマンド（lineage 等）に `api` サブコマンドを追加:

```bash
fav generate api [--format openapi|graphql] [--json] [--out <file>] <source>
```

---

## T5: driver.rs — `fav serve` 実装

### `tiny_http` 依存追加

`Cargo.toml` に:
```toml
tiny_http = { version = "0.12", optional = true }
```

feature gate で追加（`fav serve` 使用時のみ）または常時依存。

### ルートテーブル

```rust
struct Route {
    method: String,
    path_pattern: Vec<PathSegment>,  // "/users/:id" → [Literal("users"), Param("id")]
    fn_name: String,
}

enum PathSegment {
    Literal(String),
    Param(String),  // ":id" → Param("id")
}

fn build_route_table(api_fns: &[(&FnDef, &ApiAnnotation)]) -> Vec<Route>

fn match_route<'a>(routes: &'a [Route], method: &str, path: &str)
    -> Option<(&'a Route, HashMap<String, String>)>
```

### `cmd_serve` の実装

```rust
pub fn cmd_serve(source: &str, port: u16)
```

1. ソースをパース・コンパイルして artifact を構築
2. `collect_api_fns` でルートテーブルを構築
3. `tiny_http::Server::http(format!("0.0.0.0:{}", port))` でリスナー起動
4. リクエストループ: マッチ → パラメータ抽出 → VM 実行 → JSON レスポンス

### テストでの検証方法

HTTP バインドは `serve_routes_request` テストでは行わず、
`build_route_table` と `match_route` の単体テストで確認:

```rust
fn serve_routes_request() {
    let routes = build_route_table(&api_fns);
    let (route, params) = match_route(&routes, "GET", "/users/42").unwrap();
    assert_eq!(route.fn_name, "get_user");
    assert_eq!(params.get("id"), Some(&"42".to_string()));
}
```

---

## T6: v188000_tests

### テスト内容

```rust
mod v188000_tests {
    // 1. version_is_18_8_0 — Cargo.toml チェック
    // 2. api_annotation_parses — #[api(method="GET", path="/users/:id")] fn ... パース確認
    // 3. openapi_generates — build_openapi_json の出力に "paths" / "components" が含まれる
    // 4. graphql_generates — build_graphql_sdl の出力に "type Query" が含まれる
    // 5. serve_routes_request — build_route_table + match_route が動作する
}
```

テスト用入力のサンプル:

```favnir
type User = { id: Int, name: String }

#[api(method = "GET", path = "/users/:id")]
fn get_user(id: Int) -> Result<User, String> { Result.err("stub") }
```

---

## T7: Cargo.toml 更新

```toml
[package]
version = "18.8.0"

[dependencies]
# 既存の serde_yaml は v18.4.0 で追加済みなので再追加不要
tiny_http = "0.12"
```

また、`v187000_tests::version_is_18_7_0` に `#[ignore]` を追加。

---

## T8: ドキュメント

- `site/content/docs/api/generate.mdx` — `fav generate api` ガイド
- `site/content/docs/api/serve.mdx` — `fav serve` ガイド

---

## 実装上の注意点

### `#[api(...)]` パース時の lookahead

`parse_api_annotation` は `#[api(` を lookahead で確認してから消費する。
`#[deprecated]` や `#[test]` などの既存アノテーション（field attrs）と衝突しないよう、
`Ident("api")` であることを確認してからパース開始する。

### `serde_yaml` の存在確認

`serde_yaml` は v18.4.0 で `schema_loader` のために追加済み。
OpenAPI YAML 生成（`serde_yaml::to_string`）でそのまま利用可能。

### `FnDef` struct リテラルの波及

`FnDef { ... }` を直接構築している箇所は少ない（ほとんどは parser.rs のみ）。
`api_annotation: None` の追加はシンプルな一行追加。

### ルートマッチの実装

`:param` セグメントを `PathSegment::Param(name)` として格納。
`match_route` はパスを `/` で split して各セグメントを比較:
- `Literal(s)` → 完全一致
- `Param(name)` → 任意文字列（`params` に name → value で挿入）

### GraphQL SDL 生成の既存コードとの関係

`fav/src/driver.rs` にすでに `graphql_type_from_type_expr_nonnull` 関数が存在。
`build_graphql_sdl` はこれを内部で利用し、`type Query { ... }` ブロックを生成する。

---

## 工数見積もり

| タスク | 難易度 | 備考 |
|---|---|---|
| T1 ast.rs | 低 | struct 追加のみ |
| T2 parser.rs | 中 | key=value arg パース |
| T3 波及修正 | 低〜中 | FnDef リテラル数が少ないはず |
| T4 generate | 中 | JSON 構築が主な作業 |
| T5 serve | 中〜高 | tiny_http 依存追加、ルートマッチ |
| T6 tests | 低 | 既存パターン踏襲 |
| T7 Cargo.toml | 最低 | 2行変更 |
| T8 docs | 低 | MDX 2ファイル |
