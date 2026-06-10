# v13.9.0 Plan — 型状態パターン統合 + lineage 更新 実装計画

Date: 2026-06-11

---

## 実装アプローチの概要

2 本立て:
1. **E0024 型状態チェック**: `lint.rs` に `check_type_state_errors()` を追加。
   関数シグネチャを走査して `A → B` の型状態シーケンスを推論し、
   型ミスマッチ呼び出しを検出する。
2. **lineage.rs 更新**: 分類ロジックを `ast::Effect` ベースから
   「関数パラメータの型名」ベースに切り替える。

`fav doc --builtins --format json` の capability フィールド追加は独立した小変更。

**変更ファイル一覧**:

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0024 エントリ追加 |
| `fav/src/lint.rs` | `check_type_state_errors()` 追加 |
| `fav/src/driver.rs` | E0024 ヘルプテキスト、`cmd_check` 統合、`v139000_tests` 追加 |
| `fav/src/lineage.rs` | `LineageEntry.kind` + `capability` フィールド追加、分類ロジック更新 |
| `fav/src/driver.rs` | `BuiltinPrimitive` に `capability` フィールド追加 |
| `fav/Cargo.toml` | `version = "13.9.0"` |

---

## Phase A — E0024 エラーカタログ追加

### A-1: `fav/src/error_catalog.rs`

E0023 エントリの直後に追加:

```rust
ErrorEntry {
    code: "E0024",
    title: "type state mismatch",
    category: "types",
    description: "A value of type A was passed to a function that expects type B, \
                  where A and B are consecutive stages in a type state sequence \
                  inferred from function signatures in this file.",
    example: "fn validate(d: Loaded) -> Result<Validated, String>\n\
              fn transform(d: Validated) -> Result<Transformed, String>\n\n\
              transform(rows)  // E0024: got Loaded, expected Validated",
    fix: "Call the intermediate transformation function first. \
          Type state sequence: Loaded → Validated → Transformed.",
},
```

### A-2: `fav/src/driver.rs` の `get_help_text`

```rust
"E0024" => &[
    "call the intermediate transformation function first",
    "type state sequence: Loaded → Validated → Transformed",
    "use `--legacy` flag to downgrade E0024 to W011 during migration",
],
```

---

## Phase B — lint.rs: E0024 型状態チェック実装

### B-1: 型状態シーケンス推論

```rust
/// (A, B) のペア: 関数 `fn f(d: A) -> Result<B, _>` から収集
fn collect_type_state_edges(program: &Program) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            // 単一パラメータで型状態らしい名目型の場合のみ対象
            // 戻り値が Result<B, _> または B（命名型）
            if let Some(from_ty) = extract_single_named_param(&fd.params) {
                if let Some(to_ty) = extract_named_ret_ty(&fd.ret_ty) {
                    if from_ty != to_ty {
                        edges.push((from_ty, to_ty));
                    }
                }
            }
        }
    }
    edges
}
```

**判定基準** — 名目型かどうかの判定:
- `type Loaded(...)` のような `type` 宣言がファイル内にある型名
- パラメータが 1〜2 個（ctx を除く）
- 戻り値型が `Result<TypeName, _>` または `TypeName`（プリミティブ型を除く）

### B-2: 型状態違反の検出

```rust
pub fn check_type_state_errors(program: &Program) -> Vec<LintError> {
    let edges = collect_type_state_edges(program);
    // A → B が存在する場合、B を要求する関数に A を渡す呼び出しを検出
    // expected_map: "B" → "A"（前フェーズ）
    let expected_map: HashMap<String, String> =
        edges.iter().map(|(a, b)| (b.clone(), a.clone())).collect();

    let mut errors = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            collect_type_state_in_block(&fd.body, &expected_map, program, &mut errors);
        }
    }
    errors
}
```

### B-3: 呼び出し箇所のスキャン

`collect_type_state_in_expr`:
- `Expr::Apply(callee, args, span)` を検出
- `callee` が `fn f(d: B) -> ...` の関数名
- `args[0]`（または ctx を除いた最初の引数）の型が `A`（`B` の前フェーズ）
- → E0024 を emit

**注意**: `args` の型推論は lint.rs スコープ外。
代わりに `args[0]` が `Ident(name)` の場合に `name` の束縛型を追跡する
（シンプルな型追跡のみ実装、複雑なケースは無視）。

### B-4: `--legacy` モード降格

E0024 は E0023 と同様に `cmd_check` 内の `if !legacy_check { ... }` ブロック内で呼ぶ。

---

## Phase C — driver.rs: cmd_check への統合

### C-1: E0024 チェックブロック

E0023 チェックブロックの直後に追加:

```rust
if !legacy_check && !json {
    // ... E0023 チェック ...

    // E0024: type state check
    if let Some(prog) = &parsed_prog {
        let e0024s = crate::lint::check_type_state_errors(prog);
        if !e0024s.is_empty() {
            for e in &e0024s {
                eprintln!("error[E0024]: {}", e.message);
                // ... source context ...
            }
            eprintln!("error: {} type state mismatch(es) (E0024)", e0024s.len());
            process::exit(1);
        }
    }
}
```

**注意**: `parsed_prog` を E0023 と共有して二重パースを避けること。

---

## Phase D — lineage.rs 更新

### D-1: `LineageEntry` に `kind` と `capability` を追加

```rust
#[derive(Debug, Clone, Serialize)]
pub struct LineageEntry {
    pub name: String,
    pub kind: String,              // "read" | "write" | "transform" | "sink" | "io"
    pub capability: Option<String>, // "DbRead" | "DbWrite" | "StorageWrite" | null
    pub effects: Vec<String>,      // 旧フィールド（後方互換で残存）
    pub sources: Vec<String>,
    pub sinks: Vec<String>,
}
```

### D-2: capability ベース分類ロジック

```rust
fn classify_capability(params: &[Param]) -> (&'static str, Option<&'static str>) {
    // パラメータ型名から capability を判定
    let types: Vec<&str> = params.iter()
        .map(|p| type_name_str(&p.ty))
        .collect();

    if types.iter().any(|t| *t == "DbWrite" || *t == "WriteCtx" || *t == "MigrateCtx") {
        return ("write", Some("DbWrite"));
    }
    if types.iter().any(|t| *t == "StorageWrite") {
        return ("sink", Some("StorageWrite"));
    }
    if types.iter().any(|t| *t == "DbRead" || *t == "LoadCtx") {
        return ("read", Some("DbRead"));
    }
    if types.iter().any(|t| *t == "AppCtx") {
        // AppCtx はすべての capability を持つ → 本体に DB 呼び出しがあれば read/write
        return ("read", Some("DbRead")); // 保守的に read と分類
    }
    if types.iter().any(|t| *t == "Io" || *t == "CommonCtx") {
        return ("io", Some("Io"));
    }
    ("transform", None)
}
```

### D-3: 旧エフェクトベース分類の後方互換

`--legacy` モードでは旧 `effects` フィールドを `format_effects()` で生成して出力。
標準モードでは `kind` + `capability` を主フィールドとして出力し、`effects` は空配列。

### D-4: `fav explain --lineage` の出力形式

```
$ fav explain --lineage pipeline.fav

=== Lineage Report: pipeline.fav ===

Transformations:
  load_rows      [read]   DbRead  sources: [orders]
  validate       [transform]      (pure)
  aggregate      [transform]      (pure)
  save_result    [sink]   StorageWrite  sinks: [s3://results/]

Pipelines:
  Pipeline: load_rows → validate → aggregate → save_result
    sources: [orders]
    sinks:   [s3://results/]
```

---

## Phase E — fav doc --builtins --format json 更新

### E-1: `BuiltinPrimitive` に `capability` フィールド追加

```rust
struct BuiltinPrimitive {
    namespace:      &'static str,
    name:           &'static str,
    signature:      &'static str,
    effects:        Vec<&'static str>,
    returns_result: bool,
    description:    &'static str,
    capability:     Option<&'static str>,  // ← 追加
    impls:          Vec<&'static str>,     // ← 追加
}
```

マクロを拡張して `capability`/`impls` を渡せるようにするか、
既存エントリはデフォルト値（`None` / `vec![]`）を使う。

### E-2: DbRead / DbWrite / StorageWrite 等のエントリに capability を追加

```rust
p!("DbRead", "query", "(sql: String, params: List<String>) -> Result<List<Row>, String>",
   [], true, "Execute a read query",
   capability: Some("DbRead"), impls: vec!["PostgresDb", "SnowflakeDb", "MockDb"]),
```

既存の `IO.*` / `Postgres.*` 等のエントリは `capability: None` のまま。

---

## Phase F — テスト追加

### F-1: `v139000_tests` モジュール（driver.rs 末尾）

```rust
#[cfg(test)]
mod v139000_tests {
    fn version_is_13_9_0()
    fn e0024_type_state_skip_phase()       // Loaded を transform に渡す → E0024
    fn e0024_correct_sequence_no_error()   // Loaded → Validated → Transformed → OK
    fn e0024_pure_fn_not_affected()        // Int 引数の純粋関数は E0024 なし
    fn e0024_legacy_mode_no_error()        // --legacy では E0024 なし
    fn lineage_db_read_node()              // DbRead パラメータ → kind: "read"
    fn lineage_pure_transform_node()       // capability なし → kind: "transform"
    fn lineage_storage_write_sink()        // StorageWrite → kind: "sink"
    fn doc_builtins_capability_field()     // JSON 出力に capability フィールドあり
}
```

### F-2: 既存テストのリグレッション確認

```bash
cargo test v139000   # 9/9 パス確認
cargo test           # 全件パス（1484 + α）
```

---

## Phase G — バージョンバンプ + コミット

```bash
# Cargo.toml: version = "13.8.0" → "13.9.0"
cargo test -- --test-threads=1
git add -A
git commit -m "feat: v13.9.0 — 型状態パターン統合 + lineage 更新 (E0024)"
```

---

## 実装上の注意点・リスク

### R-1: 型状態推論の誤検知

`fn load(ctx: LoadCtx, path: String) -> Result<Loaded, String>` は ctx + String の 2 引数。
「ctx を除いた最初の引数が Named 型」という条件で絞り込めば誤検知を抑制できる。
ただし ctx 以外の Named 型パラメータを持つすべての関数が誤って対象になる可能性あり。
初期実装は保守的に: 「ファイル内に `type X(...)` 宣言が存在する X のみ型状態型と見なす」。

### R-2: `type_name_str` ヘルパー

`Type::Named(name)` の name を取り出す関数が既存の checker.rs や lint.rs にあるか確認。
なければ簡単なマッチ関数を追加する。

### R-3: lineage.rs の `LineageEntry.kind` フィールド追加による serde 変化

`kind` フィールドの追加は JSON 出力の変化を伴う。
既存の `v11000_tests` の lineage テストは `effects` フィールドを検証しているが、
`kind` フィールドは既存テストに影響しない（追加フィールドのため）。

### R-4: `BuiltinPrimitive` マクロの拡張

`p!` マクロのシグネチャ変更は全 `BuiltinPrimitive` 登録箇所（数百エントリ）に影響する。
`capability` / `impls` はオプションフィールドとして追加し、
既存エントリへのデフォルト値補完（`None` / `vec![]`）を確実に行うこと。
または `impl Default for BuiltinPrimitive` を追加して既存マクロを変更しない方法も有効。

### R-5: E0024 と E0023 の parser 共有

`cmd_check` で `Parser::parse_str` を 2 回呼ぶ二重パースを避けるため、
E0023 のパース結果を `Option<Program>` 変数に保持して E0024 でも再利用すること。

---

## 実装順序（推奨）

```
A（error_catalog + help text）
→ B（lint.rs: check_type_state_errors）
  → B の単体テスト（e0024_type_state_skip_phase）で動作確認
→ C（driver.rs: cmd_check 統合）
→ D（lineage.rs 更新）
  → lineage テスト（lineage_db_read_node 等）で動作確認
→ E（doc builtins capability フィールド）
→ F（全テスト追加）→ cargo test v139000
→ G（バージョンバンプ + cargo test 全件）
```
