# v18.4.0 実装計画 — Schema Types

## 依存関係

```
T1（ast.rs — TypeExpr::Schema 追加）
  └─ T2（parser.rs — schema キーワード解析）
       └─ T3（checker.rs — スキーマ解決・型登録）
            ├─ T4（schema_loader.rs 新規作成 — file: URI 読み込み・キャッシュ）
            └─ T5（v184000_tests）
T6（driver.rs — --refresh-schemas フラグ）  ← T3 完了後
T7（バージョン更新）                         ← T5 完了後
T8（ドキュメント）                           ← T7 と並列可
```

---

## フェーズ別実装計画

### フェーズ 1: AST 変更（T1）

`fav/src/ast.rs` の `TypeExpr` enum に `Schema` variant を追加する。

```rust
/// `schema "source:identifier"` — schema type import (v18.4.0)
Schema(String, Span),
```

`TypeExpr::span()` の match にも追加:
```rust
TypeExpr::Schema(_, s) => s,
```

**波及箇所（exhaustive match）:**
- `checker.rs`: `resolve_type_expr*`, `validate_type_expr_arity`
- `compiler.rs`: `lower_type_expr*`, `substitute_self_in_type_expr`
- `ast_lower_checker.rs`: `lower_te`, `te_to_string`
- `emit_python.rs`: type_expr → Python アノテーション
- `fmt.rs`: `type_expr()` pretty-print
- `driver.rs`: `favnir_type_display`, `format_type_expr` など

→ `cargo build` のコンパイルエラーで全箇所が判明する。

### フェーズ 2: パーサー拡張（T2）

`parse_base_type` に `schema` ident の検出を追加する。

`schema` は `TokenKind::Ident("schema")` として現れる（新トークンは不要）。

```rust
// parse_base_type 内
if matches!(self.peek(), TokenKind::Ident(s) if s == "schema") {
    let span_start = self.peek_span().clone();
    self.advance(); // consume "schema"
    // 次は文字列リテラル
    let uri = match self.advance_token() {
        Token { kind: TokenKind::Str(s), .. } => s,
        tok => return Err(ParseError::expected("string literal after `schema`", tok.span)),
    };
    let span = self.span_from(&span_start);
    return Ok(TypeExpr::Schema(uri, span));
}
```

`parse_alias_decl` は `parse_type_expr()` を呼ぶので、そこから `parse_base_type` が呼ばれ自動的に解析される。

**注意:** `Str` トークンの種類を確認する。lexer が `"..."` を `TokenKind::Str(String)` か `TokenKind::Lit(Lit::Str(String))` で返すかを確認すること。

### フェーズ 3: スキーマローダー（T4）

新規ファイル `fav/src/middle/schema_loader.rs` を作成する。

```rust
pub enum SchemaSource {
    File(PathBuf),
    BigQuery { project: String, dataset: String, table: String },
    Postgres { table: String },
    Snowflake { db: String, schema: String, table: String },
}

pub struct SchemaField {
    pub name: String,
    pub ty: String, // "Int" | "Float" | "String" | "Bool" | "List<...>" | etc.
}

pub fn parse_schema_uri(uri: &str) -> Result<SchemaSource, String> { ... }
pub fn load_schema(source: &SchemaSource, cache_dir: &Path, refresh: bool) -> Result<Vec<SchemaField>, String> { ... }
pub fn cache_key(source: &SchemaSource) -> String { ... }
```

#### `file:` ソースの実装

1. `PathBuf` で JSON ファイルを読み込む
2. `serde_json` でパース（`Value::Object` → `properties` フィールドを走査）
3. 各プロパティ名 + JSON Schema type → `SchemaField { name, ty }` に変換
4. `.fav/schema-cache/{cache_key}` にキャッシュを書き込む

**JSON Schema の例（`schemas/users.json`）:**
```json
{
  "type": "object",
  "properties": {
    "id":         { "type": "integer" },
    "name":       { "type": "string" },
    "email":      { "type": "string" },
    "created_at": { "type": "string" }
  }
}
```

#### キャッシュ形式

```json
{ "fields": [{ "name": "id", "type": "Int" }, { "name": "name", "type": "String" }] }
```

キャッシュ key: `file:` URI の場合 `"file__" + path.replace('/', "__").replace('.', "_")`

#### `load_schema` の流れ

```
1. cache_key を計算
2. refresh=false かつ キャッシュファイルが存在 → キャッシュから読み込み
3. refresh=true またはキャッシュなし:
   - file: → ファイルを読み込んで JSON Schema パース
   - bigquery/postgres/snowflake: → 将来実装（v18.4.0 では Empty fields を返す）
   - キャッシュに書き込む
4. Vec<SchemaField> を返す
```

### フェーズ 4: チェッカー統合（T3）

`checker.rs` で `TypeExpr::Schema` を解決する。

#### `resolve_type_expr_with_subst` への追加

```rust
TypeExpr::Schema(uri, _) => {
    // スキーマを読み込んでレコード型として解決する
    let fields = self.resolve_schema(uri);
    fields
}
```

#### `resolve_schema` の実装

```rust
fn resolve_schema(&mut self, uri: &str) -> Type {
    // schema_loader::load_schema を呼ぶ
    // 成功 → フィールドリストを record_fields に登録して Type::Named(synthetic_name) を返す
    // 失敗 → type_error(E0338 or E0339) を発行して Type::Error を返す
}
```

スキーマから生成された型は「匿名レコード型」として `record_fields` に登録する。
型名は `$schema:{uri}` のような内部名を使う。

#### `validate_type_expr_arity` への追加

```rust
TypeExpr::Schema(_, _) => 0,
```

### フェーズ 5: driver.rs — `--refresh-schemas` フラグ（T6）

`cmd_check` に `--refresh-schemas` フラグを追加する。

- `CheckOpts` / `RunOpts` に `refresh_schemas: bool` フィールドを追加
- フラグが立っていれば `REFRESH_SCHEMAS` thread-local を設定
- `schema_loader::load_schema` が `refresh=true` で呼ばれる

---

## 技術的注意事項

### `serde_json` は既存依存

`fav/Cargo.toml` に `serde_json = "1"` は既に存在する。新しい依存は不要。

### スキーマキャッシュディレクトリの作成

`.fav/schema-cache/` は存在しない可能性がある。`load_schema` 内で `fs::create_dir_all` を呼ぶ。
テスト環境では `tempfile::tempdir()` を使ってキャッシュ先を一時ディレクトリに向ける。

### exhaustive match の波及

`TypeExpr::Schema` を追加すると多数のファイルでコンパイルエラーが発生する。
各箇所での推奨処理:
- `checker.rs`: `TypeExpr::Schema` → `resolve_schema(uri)` を呼ぶ
- `compiler.rs`: `TypeExpr::Schema(_,_)` → `Type::Unknown` に変換（型検査済みなので問題なし）
- `fmt.rs`: `Schema(uri, _)` → `format!("schema \"{}\"", uri)` を表示
- `emit_python.rs`: `Schema(_, _)` → `"Any"` を返す
- `ast_lower_checker.rs`: `Schema(uri, _)` → `v1("TeSimple", "Any")` or `v1("TeSchema", uri)`
- `driver.rs`: 各 match に `Schema(_, _) => ...` を追加

### テストでの `file:` スキーマ

テスト内でインラインでスキーマ JSON を `tempfile` に書き出し、
`schema "file:path"` で参照する。または `tempdir` 内の固定パスを使う。

---

## リスク

| リスク | 対策 |
|---|---|
| `TypeExpr::Schema` の exhaustive match が多い | `cargo build` 後に漏れを一括修正 |
| テスト環境でのファイルパス | `tempfile::tempdir()` で一時ディレクトリを使用 |
| キャッシュディレクトリが存在しない | `create_dir_all` で自動作成 |
| `db:` ソースが未実装 | v18.4.0 では `file:` のみ完全実装、他は空フィールドを返す |
