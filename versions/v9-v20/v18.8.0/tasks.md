# v18.8.0 — 型駆動 API 生成タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — ApiAnnotation 追加 + FnDef 拡張

- [x] `ApiAnnotation` struct を追加（`method: String`, `path: String`, `span: Span`）
- [x] `FnDef` に `pub api_annotation: Option<ApiAnnotation>` フィールドを追加
- [x] `cargo build` でコンパイルエラーが生じることを確認（T2/T3 で修正）

---

### T2: `fav/src/frontend/parser.rs` — `#[api(...)]` パース実装

- [x] `expect_ident_name(name: &str) -> Result<(), ParseError>` ヘルパー追加
- [x] `expect_str() -> Result<String, ParseError>` ヘルパー追加（`TokenKind::Str(s)` を消費）
- [x] `parse_api_annotation() -> Result<Option<ApiAnnotation>, ParseError>` 追加:
  - `peek() != Hash` なら `Ok(None)` を即返却
  - lookahead で `#[api(` を確認（`#[deprecated]` 等と区別）
  - `method = "..."`, `,`, `path = "..."` の順でパース
  - `ApiAnnotation { method, path, span }` を返す
- [x] `parse_item` の先頭で `parse_api_annotation()` を呼び出し
- [x] `parse_fn_def` の戻り値（`FnDef { ... }`）に `api_annotation: None` を追加（T3 と合わせて実施）
- [x] `parse_api_annotation` から取得した値を `Item::FnDef` に付与する

---

### T3: 波及ファイル修正（`FnDef` struct リテラル）

Grep で `FnDef {` を検索し、`api_annotation: None` を追記:

- [x] `fav/src/frontend/parser.rs` — `parse_fn_def` 内の `FnDef { ... }` 構築（1〜2箇所）
- [x] その他 Grep で発見した `FnDef { ... }` 構築箇所（driver.rs / checker.rs / compiler.rs 等）
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

---

### T4: `fav/src/driver.rs` — API 生成実装

**4-A: ヘルパー関数群**

- [x] `collect_api_fns<'a>(prog: &'a Program) -> Vec<(&'a FnDef, &'a ApiAnnotation)>`
  - `prog.items` をスキャンし、`Item::FnDef(fd)` かつ `fd.api_annotation.is_some()` を収集
- [x] `path_to_openapi(path: &str) -> String`
  - `/users/:id` → `/users/{id}` の変換（`:` を `{` `}` に置換）
- [x] `type_expr_to_openapi_schema(te: &ast::TypeExpr) -> serde_json::Value`
  - `Int` → `{"type":"integer"}`
  - `Float` → `{"type":"number"}`
  - `String` → `{"type":"string"}`
  - `Bool` → `{"type":"boolean"}`
  - `List<T>` → `{"type":"array","items":<T のスキーマ>}`
  - 名前付き型 → `{"$ref":"#/components/schemas/<name>"}`
  - その他 → `{"type":"string"}`（フォールバック）
- [x] `openapi_fn_to_path_item(fd: &FnDef, ann: &ApiAnnotation, prog: &Program) -> serde_json::Value`
  - `:param` パラメータを `parameters` に変換
  - `return_ty` から `responses` を生成（`Result<T, E>` → 200/400）
- [x] `collect_component_schemas(prog: &Program) -> serde_json::Map<String, serde_json::Value>`
  - `Item::TypeDef` で `TypeBody::Record(fields, _)` を持つものを収集
  - フィールドを OpenAPI `properties` に変換
- [x] `build_openapi_json(api_fns: &[(&FnDef, &ApiAnnotation)], prog: &Program) -> serde_json::Value`
  - `openapi`, `info`, `paths`, `components` を組み立てて JSON を返す

**4-B: GraphQL SDL 生成**

- [x] `build_graphql_sdl(api_fns: &[(&FnDef, &ApiAnnotation)], prog: &Program) -> String`
  - 各 API fn を `type Query { fn_name(param: Type): ReturnType }` に変換
  - 既存の `graphql_type_from_type_expr_nonnull` を内部で利用
  - 使用するレコード型を `type TypeName { ... }` として出力

**4-C: CLI コマンド**

- [x] `pub fn cmd_generate_api(source: &str, format: &str, as_json: bool, out: Option<&str>)`:
  - ソースをパース → `collect_api_fns` → `build_openapi_json` or `build_graphql_sdl`
  - `format = "openapi"` かつ `!as_json` → `serde_yaml::to_string(&json_val)` で YAML 出力
  - `format = "openapi"` かつ `as_json` → `serde_json::to_string_pretty` で JSON 出力
  - `format = "graphql"` → SDL テキスト出力
  - `out = Some(path)` → ファイル書き込み、`out = None` → stdout
- [x] `fav generate api` を `cmd_generate` のサブコマンドとして登録

---

### T5: `fav/src/driver.rs` — `fav serve` 実装

**5-A: Cargo.toml への依存追加**（T7 で実施）

**5-B: ルートテーブル**

- [x] `PathSegment` enum 追加（`Literal(String)` / `Param(String)`）
- [x] `Route` struct 追加（`method`, `path_pattern`, `fn_name`）
- [x] `parse_path_pattern(path: &str) -> Vec<PathSegment>`
  - `/users/:id/orders` → `[Literal("users"), Param("id"), Literal("orders")]`
- [x] `pub fn build_route_table<'a>(api_fns: &[(&'a FnDef, &'a ApiAnnotation)]) -> Vec<Route>`
- [x] `pub fn match_route<'a>(routes: &'a [Route], method: &str, path: &str) -> Option<(&'a Route, HashMap<String, String>)>`
  - メソッドとパスセグメントを順番にマッチ
  - Param セグメントは任意文字列としてマッチし、`HashMap` に挿入

**5-C: HTTP サーバー（tiny_http 使用）**

- [x] `pub fn cmd_serve(source: &str, port: u16)`:
  - ソースをパース・コンパイル → artifact 構築
  - `collect_api_fns` でルートテーブルを構築
  - `tiny_http::Server::http(format!("0.0.0.0:{}", port))` でサーバー起動
  - ループ: リクエスト受信 → `match_route` → パラメータ抽出 → VM で fn 実行 → JSON レスポンス
  - 実行結果が `Value::Record` / `Value::Str` / `Value::Int` → JSON にシリアライズ
  - `match_route` が None → 404 レスポンス
- [x] `fav serve <source> [--port <n>]` を CLI に登録

---

### T6: `fav/src/driver.rs` — `v188000_tests` 追加

- [x] `v187000_tests::version_is_18_7_0` に `#[ignore]` を追加
- [x] `v188000_tests` モジュールを追加（5件）:

  ```rust
  #[test]
  fn version_is_18_8_0() {
      let cargo = include_str!("../Cargo.toml");
      assert!(cargo.contains("\"18.8.0\""));
  }

  #[test]
  fn api_annotation_parses() {
      // #[api(method = "GET", path = "/users/:id")] fn get_user(id: Int) -> Result<User, String> { ... }
      // → FnDef.api_annotation = Some(ApiAnnotation { method: "GET", path: "/users/:id" })
  }

  #[test]
  fn openapi_generates() {
      // build_openapi_json の出力 JSON に "paths" と "components" キーが含まれる
      // paths の中に "/users/{id}" が含まれる
  }

  #[test]
  fn graphql_generates() {
      // build_graphql_sdl の出力に "type Query" が含まれる
      // get_user フィールドが含まれる
  }

  #[test]
  fn serve_routes_request() {
      // build_route_table + match_route の単体テスト（HTTP バインドなし）
      // match_route(&routes, "GET", "/users/42") → Some((route, {"id": "42"}))
      // match_route(&routes, "GET", "/not-found") → None
  }
  ```

---

### T7: `fav/Cargo.toml` 更新

- [x] `version = "18.7.0"` → `"18.8.0"` に更新
- [x] `tiny_http = "0.12"` を `[dependencies]` に追加
- [x] `serde_yaml` が既存にあることを確認（v18.4.0 で追加済みのはず）

---

### T8: `site/content/docs/api/` ドキュメント作成

- [x] `site/content/docs/api/generate.mdx` — `fav generate api` コマンドガイド
  - OpenAPI/GraphQL 生成の使い方
  - 型マッピング表
  - `#[api(...)]` アノテーション構文
- [x] `site/content/docs/api/serve.mdx` — `fav serve` コマンドガイド
  - `:param` パスパラメータ
  - 開発用途であることの注記

---

## テスト（v188000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_8_0` | Cargo.toml に "18.8.0" が含まれる |
| `api_annotation_parses` | `#[api(method = "GET", path = "/users/:id")]` が `ApiAnnotation` としてパースされる |
| `openapi_generates` | `build_openapi_json` の出力に `paths` / `components` が含まれる |
| `graphql_generates` | `build_graphql_sdl` の出力に `type Query` が含まれる |
| `serve_routes_request` | `build_route_table` + `match_route` が `/users/:id` → `{id: "42"}` を返す |

---

## 完了条件チェックリスト

- [x] `ApiAnnotation` struct が `ast.rs` に存在する
- [x] `FnDef.api_annotation: Option<ApiAnnotation>` フィールドが存在する
- [x] `#[api(method = "GET", path = "/users/:id")] fn ...` がパースされる
- [x] `build_openapi_json` が `paths` / `components` を含む JSON を返す
- [x] `build_graphql_sdl` が `type Query { ... }` を含む SDL を返す
- [x] `build_route_table` + `match_route` が `:param` 付きパスを正しくマッチする
- [x] `cargo test v188000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/api/generate.mdx` が存在する
- [x] `site/content/docs/api/serve.mdx` が存在する

---

## 優先度

```
T1（ast.rs 型追加）                 ← 最初
T2（parser.rs #[api(...)] パース）  ← T1 完了後
T3（波及修正）                       ← T2 完了後（cargo build が通るまで）
T4（generate コマンド）             ← T3 完了後
T5（serve コマンド）                ← T4 完了後（または並列）
T6（v188000_tests）                 ← T4/T5 完了後
T7（Cargo.toml）                    ← T6 と並列可
T8（ドキュメント）                   ← T7 と並列可
```

---

## 重要な技術ノート

### `parse_api_annotation` の lookahead

`#[deprecated]` / `#[test]` / `#[ignore]` 等の既存アトリビュートと衝突しないよう、
`tokens[pos+2]` が `Ident("api")` であることを lookahead で確認してから消費する。

### `serde_yaml` の確認

`fav/Cargo.toml` で `serde_yaml` が既存依存にあれば追加不要。
`use serde_yaml` のインポートがあるファイルを Grep で確認してから判断。

### `tiny_http` の Windows 動作確認

`tiny_http 0.12` は Windows/Linux 両対応。`cargo add tiny_http` で追加可能。
テストでは実際の HTTP バインドは不要（`cmd_serve` 内の HTTP 部分はユニットテスト対象外）。

### OpenAPI `parameters` の生成

パスに `:id` が含まれる場合、対応する `fn` の引数型を `parameters` に変換:
- `:id` → `id` という名前の引数を `fn` の params から探す
- 見つかった param の型を `in: path` パラメータとして出力

POST などの場合、body パラメータは request body として出力（最初の引数をリクエストボディに）。
