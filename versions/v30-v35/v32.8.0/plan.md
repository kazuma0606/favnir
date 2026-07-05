# v32.8.0 — Plan: 型駆動 API 生成 確認・テスト補強

## 実装方針

型駆動 API 生成（`ApiAnnotation` / `build_openapi_json` / `build_route_table` 等）は
v18.8.0 で完成済み。v32.8.0 は v32.1.0〜v32.7.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.7.0"` → `"32.8.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_7_0` スタブ化 + `v328000_tests` 追加 |
| `CHANGELOG.md` | `[v32.8.0]` セクションを先頭に追記 |
| `benchmarks/v32.8.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.8.0 に更新 |
| `versions/v30-v35/v32.8.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_7_0` をスタブ化

```rust
// v327000_tests 内（既存の #[test] fn を空スタブに置き換える）
fn cargo_toml_version_is_32_7_0() {
    // Stubbed: version bumped to 32.8.0 in v32.8.0.
}
```

### ② `v328000_tests` を挿入

挿入位置: `v327000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。
（`#[cfg(test)]` も含む v31.7.0 ブロック開始行より前）

```rust
// ── v32.8.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v328000_tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn cargo_toml_version_is_32_8_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.8.0"), "Cargo.toml must contain '32.8.0'");
    }

    #[test]
    fn benchmark_v32_8_0_exists() {
        let src = include_str!("../../benchmarks/v32.8.0.json");
        assert!(src.contains("32.8.0"), "benchmarks/v32.8.0.json must contain '32.8.0'");
    }

    #[test]
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

    #[test]
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
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.7.0 完了時点 | — | 2484 |
| `cargo_toml_version_is_32_7_0` スタブ化 | 0（テストは残る） | 2484 |
| `v328000_tests` 追加（4 件） | +4 | **2488** |

---

## CHANGELOG 追記内容

```markdown
## [v32.8.0] — 2026-07-03

### Added
- `v328000_tests`: 型駆動 API 生成（Type-Driven API Generation）動作確認テスト 4 件
  - `cargo_toml_version_is_32_8_0` — バージョン確認
  - `benchmark_v32_8_0_exists` — ベンチマークファイル存在確認
  - `api_ann_get_items_path_parses` — `#[api]` アノテーション `/items/:id` のパース確認
  - `api_ann_openapi_items_path_exists` — OpenAPI JSON の `/items/{id}` paths キー確認

### Notes
- `ApiAnnotation` / `build_openapi_json` / `build_route_table` 等は v18.8.0 実装済み
- v32.8.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
