# Favnir v10.3.0 Spec

Date: 2026-06-04
Theme: Effect::Snowflake 追加（8 ファイル更新）

---

## 概要

`!Http`（v9.5.0）・`!Llm`（v9.6.0）と同じ 8 ファイル更新パターンで
`!Snowflake` エフェクト型を言語に追加する。

```favnir
stage Query: String -> List<Row> !Snowflake = |sql| {
  match Snowflake.query_raw(sql) {
    Ok(json) -> Json.decode<List<Row>>(json)
    Err(e)   -> List.empty()
  }
}
```

このバージョンで追加するのはエフェクト型の宣言・型チェック・リネージ出力のみ。
Rune 実装（`runes/snowflake/`）は v10.6.0、checker.fav 対応は v10.4.0、
compiler.fav 対応は v10.5.0 で実施する。

---

## 前提（v10.2.0 完了時点）

- `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` 実装済み
- `compiler.rs` の Rust builtin NS リスト 2 箇所に `"Snowflake"` 追加済み
- `cargo test` 1264 件通過

---

## 変更対象ファイル（8 件）

```
fav/src/ast.rs
fav/src/frontend/parser.rs
fav/src/fmt.rs
fav/src/lineage.rs
fav/src/driver.rs
fav/src/middle/ast_lower_checker.rs
fav/src/middle/checker.rs
fav/src/middle/reachability.rs
```

加えて:
```
fav/src/error_catalog.rs   (E0314 エントリ追加)
```

---

## 各ファイルの変更仕様

### 1. `ast.rs` — Effect 列挙体に Snowflake 追加

```rust
pub enum Effect {
    // ... 既存 ...
    Http,
    Llm,
    Snowflake,   // ← 追加
    // ...
}
```

### 2. `parser.rs` — `"Snowflake"` トークンを Effect::Snowflake に解析

既存の `"Llm"` ブランチの直後に追加:

```rust
"Llm" => {
    self.advance();
    Effect::Llm
}
"Snowflake" => {   // ← 追加
    self.advance();
    Effect::Snowflake
}
```

### 3. `fmt.rs` — Effect の文字列変換に追加

```rust
Effect::Http      => Some("!Http".to_string()),
Effect::Llm       => Some("!Llm".to_string()),
Effect::Snowflake => Some("!Snowflake".to_string()),   // ← 追加
```

### 4. `lineage.rs` — lineage 出力の Effect 表示に追加

```rust
Http      => "!Http".into(),
Llm       => "!Llm".into(),
Snowflake => "!Snowflake".into(),   // ← 追加
```

### 5. `driver.rs` — effect 名表示（2 箇所）

effect → display string 変換と effect → short name 変換の両方に追加:

```rust
Http      => "!Http".into(),
Llm       => "!Llm".into(),
Snowflake => "!Snowflake".into(),   // ← 追加（表示用）

// および

ast::Effect::Http      => "Http".into(),
ast::Effect::Llm       => "Llm".into(),
ast::Effect::Snowflake => "Snowflake".into(),   // ← 追加（短縮名）
```

### 6. `ast_lower_checker.rs` — lowering 時の Effect 変換に追加

```rust
ast::Effect::Http      => "Http".to_string(),
ast::Effect::Llm       => "Llm".to_string(),
ast::Effect::Snowflake => "Snowflake".to_string(),   // ← 追加
```

### 7. `checker.rs` — 型チェックに Snowflake を組み込む

#### 7a. builtin NS ホワイトリスト（2 箇所）

`checker.rs` には builtin namespace リストが 2 箇所ある。両方に `"Snowflake"` を追加:

```rust
// 1 箇所目（〜line 1256）
"Http",
"Llm",
"Snowflake",   // ← 追加

// 2 箇所目（〜line 2124）
"Http",
"Llm",
"Snowflake",   // ← 追加
```

#### 7b. effects ホワイトリスト（2 箇所）

```rust
| "Http"
| "Llm"
| "Snowflake"   // ← 追加
```

#### 7c. `require_snowflake_effect` 関数を追加

`require_llm_effect` の直後に:

```rust
fn require_snowflake_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Snowflake)) {
        self.type_error(
            "E0314",
            "Snowflake.* call requires `!Snowflake` effect on enclosing fn/stage",
            span,
        );
    }
}
```

#### 7d. `Snowflake.*` の型シグネチャ追加

Llm セクションの直後（〜line 5400）に追加:

```rust
// Snowflake (v10.3.0) — require !Snowflake effect
("Snowflake", "execute_raw") => {
    self.require_snowflake_effect(span);
    Some(Type::Result(
        Box::new(Type::String),
        Box::new(Type::String),
    ))
}
("Snowflake", "query_raw") => {
    self.require_snowflake_effect(span);
    Some(Type::Result(
        Box::new(Type::String),
        Box::new(Type::String),
    ))
}
```

### 8. `reachability.rs` — 到達可能性解析に追加

```rust
Effect::Http => {
    effects_required.insert("Http".to_string());
}
Effect::Llm => {
    effects_required.insert("Llm".to_string());
}
Effect::Snowflake => {   // ← 追加
    effects_required.insert("Snowflake".to_string());
}
```

### 9. `error_catalog.rs` — E0314 エントリ追加

E0313 の直後に:

```rust
ErrorEntry {
    code: "E0314",
    title: "undeclared !Snowflake effect",
    category: "effects",
    description: "A Snowflake operation was used in a function that does not declare `!Snowflake`.",
    example: "fn run(sql: String) -> String {\n    Snowflake.execute_raw(sql)  // E0314: !Snowflake not declared\n}",
    fix: "Add `!Snowflake` to the function signature: `fn run(sql: String) -> String !Snowflake`.",
},
```

---

## テスト設計（`driver.rs` 末尾に `v10300_tests` モジュールを追加）

### テスト 1: `snowflake_execute_requires_effect`

```favnir
fn run(sql: String) -> Result<String, String> {
  Snowflake.execute_raw(sql)
}
```

→ E0314 が出ること。

### テスト 2: `snowflake_execute_with_effect_ok`

```favnir
fn run(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.execute_raw(sql)
}
```

→ E0314 が出ないこと。

### テスト 3: `snowflake_lineage_shows_effect`

```favnir
stage RunQuery: String -> String !Snowflake = |sql| {
  match Snowflake.query_raw(sql) {
    Ok(json) -> json
    Err(e)   -> e
  }
}
seq Pipeline = RunQuery
```

→ `fav explain --lineage` 出力に `!Snowflake` が含まれること。

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.3.0"`
- `fav/self/cli.fav`: `run_version` → `"10.3.0"`

---

## スコープ外（後続バージョン）

| バージョン | 内容 |
|---|---|
| v10.4.0 | checker.fav に `snowflake_fn` / `ns_to_effect` 追加 |
| v10.5.0 | compiler.fav の builtin NS に `"Snowflake"` 追加 |
| v10.6.0 | `runes/snowflake/` 実装 |
| v10.7.0 | `fav.toml` `[snowflake]` セクション対応 |
