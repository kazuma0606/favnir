# v32.8.0 — Spec: 型駆動 API 生成 確認・テスト補強

## 概要

v32.8.0 は **型駆動 API 生成（Type-Driven API Generation）** の確認・テスト補強バージョン。

ロードマップ v32.5〜v32.9 候補「型駆動 API 生成（`fav generate api --format openapi`）」として、
`#[api(method, path)]` アノテーション・OpenAPI JSON 生成・ルートテーブルマッチングが
仕様通りに動作することを確認する。
実際にはすでに v18.8.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `ApiAnnotation` struct（`method` / `path` フィールド）| ✓ | v18.8.0 |
| `FnDef.api_annotation: Option<ApiAnnotation>` | ✓ | v18.8.0 |
| `#[api(method = "...", path = "...")]` パース | ✓ | v18.8.0 |
| `collect_api_fns` — API アノテーション付き関数を収集 | ✓ | v18.8.0 |
| `build_openapi_json` — OpenAPI 3.0 JSON 生成 | ✓ | v18.8.0 |
| `build_graphql_sdl` — GraphQL SDL 生成 | ✓ | v18.8.0 |
| `build_route_table` / `match_route` — ルートテーブル構築・マッチング | ✓ | v18.8.0 |
| `v188000_tests` — 6 件のテスト（うち 2 件 `#[ignore]`） | ✓ | v18.8.0 |

v32.8.0 では、型駆動 API 生成の動作を `v328000_tests` で明示的に確認し、
バージョンと CHANGELOG を更新する。

---

## 型駆動 API 生成 仕様確認

### 構文

```favnir
// API アノテーション付き関数
#[api(method = "GET", path = "/items/:id")]
fn get_item(id: Int) -> String { "ok" }
```

### 生成機能

| 機能 | 関数 | 出力 |
|---|---|---|
| OpenAPI 3.0 JSON | `build_openapi_json` | `{ "paths": { "/items/{id}": ... }, "components": ... }` |
| ルートテーブル | `build_route_table` + `match_route` | `GET /items/42` → `{ fn_name: "get_item", params: { "id": "42" } }` |

---

## 追加するテスト（v328000_tests — 4 件）

`v328000_tests` は v32.1.0〜v32.7.0 と同じパターン:
- `use super::*` **なし**
- `use crate::frontend::parser::Parser;` を使用
- ルートテーブル・OpenAPI は `super::` で driver.rs の関数を呼び出す

テスト名は v188000_tests（`api_annotation_parses` / `openapi_generates` / `graphql_generates` / `serve_routes_request`）と被らないよう `api_ann_` プレフィックスを使用する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_8_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.8.0"), "Cargo.toml must contain '32.8.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_8_0_exists() {
    let src = include_str!("../../benchmarks/v32.8.0.json");
    assert!(src.contains("32.8.0"), "benchmarks/v32.8.0.json must contain '32.8.0'");
}
```

### テスト 3: `#[api]` アノテーションのパース確認

```rust
fn api_ann_get_items_path_parses() {
    // /items/:id エンドポイント（v188000_tests は /users/:id）
    // (テスト名は v188000_tests::api_annotation_parses と異なる)
    let src = r#"
#[api(method = "GET", path = "/items/:id")]
fn get_item(id: Int) -> String { "ok" }
"#;
    let prog = Parser::parse_str(src, "v328000_test.fav").expect("parse");
    if let crate::ast::Item::FnDef(fd) = &prog.items[0] {
        let ann = fd.api_annotation.as_ref().expect("expected api_annotation");
        assert_eq!(ann.method, "GET");
        assert_eq!(ann.path, "/items/:id");
    } else {
        panic!("expected FnDef");
    }
}
```

### テスト 4: OpenAPI JSON の paths キー確認

```rust
fn api_ann_openapi_items_path_exists() {
    // /items/{id} が OpenAPI JSON の paths に含まれることを確認
    // (テスト名は v188000_tests::openapi_generates と異なる)
    let src = r#"
type Item = {
    id: Int
    name: String
}
#[api(method = "GET", path = "/items/:id")]
fn get_item(id: Int) -> Item { Item { id: 0, name: "" } }
"#;
    let prog = Parser::parse_str(src, "v328000_test.fav").expect("parse");
    let api_fns = super::collect_api_fns(&prog);
    let json = super::build_openapi_json(&api_fns, &prog);
    let paths = json["paths"].as_object().expect("expected paths");
    assert!(
        paths.contains_key("/items/{id}"),
        "expected /items/{{id}} in OpenAPI paths, got: {:?}",
        paths.keys().collect::<Vec<_>>()
    );
}
```

---

## テストモジュールの配置

`v328000_tests` は `v327000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"32.8.0"`
- `cargo_toml_version_is_32_7_0` が空スタブになっていること
- `api_ann_get_items_path_parses` テストが PASS
- `api_ann_openapi_items_path_exists` テストが PASS
- `cargo test --bin fav v328000` — 4/4 PASS
- `cargo test` — 全件 PASS（2488 件、0 failures）
- `CHANGELOG.md` に `[v32.8.0]` セクション
- `benchmarks/v32.8.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.8.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.8.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（型駆動 API 生成は v18.8.0 で完成済み）
