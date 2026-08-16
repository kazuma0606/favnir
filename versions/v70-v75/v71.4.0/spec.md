# v71.4.0 仕様書 — Const / Compile-Time Evaluation

Date: 2026-08-09
Status: 計画中

---

## Background

v71.1.0 で `Vec<T>[N]` 依存型を導入したが、次元数はリテラル整数のみ（`TypeExpr::ConstInt`）。
実運用では `EMBED_DIM = 1536` のような定数を定義してコード全体で共有したい。
また、定数算術式（`EMBED_DIM / 2`）のコンパイル時評価も求められる。

---

## Goals

1. `const NAME: Type = expr` 宣言をトップレベルに追加
2. 整数・文字列・算術式（`+`, `-`, `*`, `/`）のコンパイル時評価
3. `Vec<T>[NAME]` など型注釈の次元位置で定数名を参照可能にする
4. 既存テスト（3592 件）を全 pass のまま維持し、+2 件追加（3594 件）

---

## Syntax / API

```favnir
// 整数定数
const MAX_BATCH_SIZE: Int = 1024
const EMBED_DIM: Int      = 1536

// 文字列定数
const API_BASE_URL: String = "https://api.favnir.dev"

// 算術定数式（コンパイル時評価）
const HALF_DIM: Int = EMBED_DIM / 2   // 768

// 依存型の次元パラメータとして使用
stage EmbedText: String -> Vec<Float>[EMBED_DIM] = |text| {
    OpenAI.embed(text, dim: EMBED_DIM)
}
```

---

## 実装スコープ

### 1. AST（`fav/src/ast.rs`）

#### `ConstDef` 構造体（新規）

```rust
/// `const NAME: Type = expr` — コンパイル時定数宣言（v71.4.0）
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub ty: TypeExpr,
    pub value: Expr,
    pub span: Span,
}
```

#### `Item::ConstDef` バリアント追加

```rust
pub enum Item {
    // ...既存...
    ConstDef(ConstDef),  // v71.4.0
}
```

#### `TypeExpr::ConstName` バリアント追加

```rust
pub enum TypeExpr {
    // ...既存...
    ConstName(String, Span),  // v71.4.0: 型位置で定数名を参照（Vec<Float>[EMBED_DIM]）
}
```

### 2. パーサー（`fav/src/frontend/parser.rs`）

- `parse_item` に `TokenKind::Ident("const")` ブランチ追加
- `parse_const_def` 新規関数: `const NAME: Type = expr` を解析し `ConstDef` を返す
- `parse_base_type` の `[...]` サフィックス処理: リテラル整数 → `ConstInt`、識別子 → `ConstName`

### 3. チェッカー（`fav/src/middle/checker.rs`）

- `Checker` に `const_env: HashMap<String, StaticValue>` フィールド追加
- `register_item_signatures` / 先頭 const pre-pass:
  - `Item::ConstDef(cd)` → `eval_static_expr(&cd.value, &HashMap::new())` → `const_env.insert(cd.name, val)`
  - 型チェック: `resolve_type_expr(&cd.ty)` との互換確認、不一致は E0247 / E0248
- `resolve_type_expr` に `TypeExpr::ConstName` 分岐追加:
  - `const_env.get(name)` → `StaticValue::Int(n)` → `Type::Int`（型推論用）
  - 依存型チェック時に次元値として解決（`is_dim_annotated_name_mismatch` と連携）
- `check_type_def` の dim mismatch 解決: `ConstName` → `const_env` 参照で `ConstInt` に展開

### 4. fmt.rs（`fav/src/fmt.rs`）

- `TypeExpr::ConstName(name, _)` → `format!("{}", name)` の arm 追加
- トップレベル const の整形: `const {name}: {ty} = {expr}`

### 5. その他ファイル（網羅性対応）

以下のファイルに `TypeExpr::ConstName` 対応アームを追加（コンパイルエラー解消）:
- `fav/src/middle/compiler.rs` — `resolve_type_expr` で `ConstName` → `Type::Int`
- `fav/src/middle/ast_lower_checker.rs` — `lower_type_expr` で `ConstName` → 整数値
- `fav/src/lint.rs` — ConstName は警告対象外としてスキップ
- `fav/src/emit_python.rs` — `ConstName` → `"int"` にフォールバック
- `fav/src/driver.rs` — 各 `ty_to_str` 系関数に `ConstName` → name 文字列

---

## Error Codes

| コード | 内容 | 例 |
|---|---|---|
| E0247 | 未定義定数参照 | `Vec<Float>[UNKNOWN_DIM]` — `UNKNOWN_DIM` が未定義 |
| E0250 | 定数型不一致 | `const N: Int = "hello"` — String を Int 定数に代入 |

**注意**: E0248 / E0249 は既存の checker.rs（abstract seq スロット型不一致）で使用中のため使用不可。
E0247 は未使用（確認済み）、E0250 は未使用（確認済み）。

---

## Success Criteria

- [x] `const MAX_BATCH_SIZE: Int = 1024` がパース・型チェックで通る
- [x] `const HALF_DIM: Int = EMBED_DIM / 2` が `768` にコンパイル時評価される（前方宣言順に評価）
- [x] `Vec<Float>[EMBED_DIM]` の型注釈で定数名が解決される
- [x] 未定義定数参照で E0247 が発生する
- [x] 定数型不一致で E0250 が発生する
- [x] テスト総数: 3592 + 2 = 3594 件（実績 3592 ベース）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `ConstDef` 構造体・`Item::ConstDef`・`TypeExpr::ConstName` 追加 |
| `fav/src/frontend/parser.rs` | `parse_const_def`・`ConstName` parsing 追加 |
| `fav/src/middle/checker.rs` | `const_env` フィールド・const pre-pass・`resolve_type_expr` ConstName 対応 |
| `fav/src/fmt.rs` | `ConstName` arm・`Item::ConstDef` フォーマット |
| `fav/src/middle/compiler.rs` | `ConstName` arm（`Type::Int` フォールバック） |
| `fav/src/middle/ast_lower_checker.rs` | `ConstName` arm |
| `fav/src/lint.rs` | `ConstName` スキップ arm |
| `fav/src/emit_python.rs` | `ConstName` arm |
| `fav/src/driver.rs` | `v714000_tests` 追加・`ConstName` arm（各 ty_to_str 関数） |
| `fav/src/lsp/references.rs` | `collect_in_item` の `_ => {}` アームが既存のため自動スキップ（確認のみ） |
| `fav/src/lineage.rs` | `Item::ConstDef` を透過（skip）するか確認 |
| `fav/Cargo.toml` | `version = "71.4.0"` |
| `CHANGELOG.md` | v71.4.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン更新 |
