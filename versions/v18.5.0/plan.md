# v18.5.0 実装計画 — 線形型（Linear Types）

## 実装ステップ

### Step 1: AST 拡張（`TypeExpr::LinearArrow` / `Type::LinearFn`）

**対象ファイル**: `fav/src/ast.rs`

1. `TypeExpr` enum に `LinearArrow(Box<TypeExpr>, Box<TypeExpr>, Span)` を追加
2. `TypeExpr::span()` に `LinearArrow(_, _, s) => s` を追加
3. `Type` enum に `LinearFn(Box<Type>, Box<Type>)` を追加

追加後は多数の exhaustive match エラーが発生するので、Step 2 で全て修正する。

---

### Step 2: 波及ファイル修正

`TypeExpr::LinearArrow` と `Type::LinearFn` を追加したことで発生する exhaustive match エラーを全修正。

**`TypeExpr::LinearArrow` を追加する箇所:**

| ファイル | 関数 / 箇所 | 追加内容 |
|---|---|---|
| `fav/src/fmt.rs` | `type_expr()` / `fmt_type_expr_simple` | `LinearArrow(a, b, _) => format!("{} -o {}", ...)` |
| `fav/src/emit_python.rs` | TypeExpr match | `LinearArrow(_, _, _) => "Callable"` |
| `fav/src/middle/ast_lower_checker.rs` | `lower_te` / `te_to_string` | `LinearArrow` → `"TeLinearArrow"` |
| `fav/src/middle/compiler.rs` | `lower_type_expr*` / `substitute_self_in_type_expr` | `LinearArrow` → `Type::LinearFn(...)` |
| `fav/src/middle/checker.rs` | `resolve_type_expr_with_subst` / `resolve_type_expr_with_self` / `validate_type_expr_arity` | `LinearArrow` → `Type::LinearFn(...)` |
| `fav/src/driver.rs` | `format_type_expr` / `favnir_type_display` / `graphql_type_from_type_expr_nonnull` / `proto_type_from_type_expr_nonwrapper` / `favnir_type_to_sql_from_expr` | `LinearArrow` → 適切な文字列 |

**`Type::LinearFn` を追加する箇所:**

| ファイル | 関数 / 箇所 | 追加内容 |
|---|---|---|
| `fav/src/middle/checker.rs` | `Type` を match している全箇所 | `LinearFn(a, b) => ...` |
| `fav/src/backend/codegen.rs` | `lower_type` 等 | `LinearFn` → `Arrow` として扱う（実行時は同じ） |
| `fav/src/driver.rs` | `favnir_type_display` 等 | `LinearFn(a, b) => format!("{} -o {}", ...)` |

---

### Step 3: パーサー修正（`-o` トークンの追加）

**対象ファイル**: `fav/src/frontend/lexer.rs` / `fav/src/frontend/parser.rs`

1. **Lexer**: `TokenKind::LinearArrow` を追加（`-o` を字句解析）
   - `lexer.rs` の `next_token` で `-` の次が `o`（スペースなし）のとき `LinearArrow` を返す
   - 注意: `-` と `o` の間にスペースがある場合は `Minus` + `Ident("o")` として扱う

2. **Parser**: `parse_type_expr_inner` / `parse_base_type` の後処理で `TokenKind::LinearArrow` を認識
   - `->` の後処理と同様に、`-o` の後ろの型を解析して `TypeExpr::LinearArrow` を構築

---

### Step 4: チェッカー拡張（線形型検査）

**対象ファイル**: `fav/src/middle/checker.rs`

#### 4-1: フィールド追加

```rust
pub struct Checker {
    // 既存フィールド...
    /// 組み込み線形型名のセット（v18.5.0: "Connection" / "Tx"）
    linear_types: HashSet<String>,
    /// 現在のスコープの線形変数使用状況
    linear_env: HashMap<String, LinearState>,
}

enum LinearState { Available, Consumed }
```

#### 4-2: 線形変数の追跡

- `check_fn_def` / `check_stage_def` の開始時に `linear_env` を初期化
- `bind x <- expr` のチェック時: `expr` の型が線形型（`linear_types` に含まれる）なら `linear_env.insert(x, Available)`
- `EVar` のチェック時: `linear_env.get(&name)` が `Some(Consumed)` なら **E0332**。`Some(Available)` なら `Consumed` に更新。
- 関数終了時: `linear_env` に `Available` が残っていれば **E0333**

#### 4-3: `-o` 適用の検査

- `Expr::Apply` で関数型が `Type::LinearFn(param_ty, ret_ty)` の場合:
  - 引数の変数名を取得し `linear_env[name]` が `Consumed` なら **E0332**
  - `Consumed` でなければ消費してから型を返す

#### 4-4: エラー定義

```rust
// E0332: 線形変数の二重消費
type_error!("E0332", span, "linear variable `{}` has already been consumed", name)

// E0333: 線形変数の未消費
type_error!("E0333", span, "linear variable `{}` must be consumed before function returns", name)
```

---

### Step 5: テスト追加（`v185000_tests`、5件）

**対象ファイル**: `fav/src/driver.rs`

1. `version_is_18_5_0` — Cargo.toml に "18.5.0" が含まれる
2. `linear_type_parses` — `fn f(g: Int -o String) -> String` が AST として解析される
3. `linear_use_once_ok` — 線形変数を 1 回使うとエラーなし
4. `linear_double_use_error` — 線形変数を 2 回使うと E0332
5. `linear_unused_error` — 線形変数を使わずに関数が終わると E0333

---

### Step 6: バージョン更新

- `fav/Cargo.toml`: `18.4.0` → `18.5.0`
- `v184000_tests::version_is_18_4_0` に `#[ignore]` を追加

---

### Step 7: ドキュメント作成

- `site/content/docs/language/linear-types.mdx` 新規作成

---

## 技術的注意事項

### `-o` のトークナイズ

`-o` は単一のトークンとして扱う必要がある。しかし `- o` はマイナス演算子 + 変数名になる。
スペースなしの `-o` のみを `LinearArrow` として認識する。

**実装方針**: lexer で `-` を読んだ直後に次の文字が `o`（英字）かどうかを peek で確認。
`o` の後が英数字でない（単語境界がある）場合のみ `LinearArrow` として返す。
それ以外は `Minus` として返す。

### match 分岐での線形変数

v18.5.0 では match 分岐での線形変数追跡はシンプルに実装する:
- match の全分岐で同じ線形変数が消費されることを要求するのは複雑すぎる
- v18.5.0 では match を「分岐によらず全分岐で消費」とみなし、match ブロック後は常に `Consumed` とする

### `Type::LinearFn` の実行時扱い

VM（実行エンジン）は `LinearFn` と通常の `Arrow` を区別しない。
線形型チェックはコンパイル時のみの制約であり、バイトコードには影響しない。
したがって、codegen では `LinearFn` を `Arrow` と同様に扱う。

### 既存の `Connection` / `Tx` 型との整合

現在、`Connection` / `Tx` は通常の型として扱われている。
v18.5.0 では Checker の `linear_types` セットに登録するだけで、
既存の API 定義を変更せず線形性チェックを追加する。

---

## リスク

| リスク | 対処 |
|---|---|
| `-o` が既存の式（e.g. `x - offset`）に誤検知される | トークナイズで単語境界チェックを実施 |
| `linear_env` スコープ管理が複雑になる | 関数単位でリセット、ネストは未対応 |
| match 分岐での消費追跡が不完全 | v18.5.0 では保守的に全消費とみなす |
| `Type::LinearFn` の exhaustive match 漏れ | Step 2 で `cargo build` が通ることを確認してから次に進む |

---

## ファイル変更一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/ast.rs` | `TypeExpr::LinearArrow` / `Type::LinearFn` 追加 |
| `fav/src/frontend/lexer.rs` | `TokenKind::LinearArrow` 追加 |
| `fav/src/frontend/parser.rs` | `-o` 構文のパース追加 |
| `fav/src/middle/checker.rs` | `linear_types` / `linear_env` / E0332 / E0333 追加 |
| `fav/src/middle/compiler.rs` | `LinearArrow` / `LinearFn` の exhaustive match 追加 |
| `fav/src/middle/ast_lower_checker.rs` | `LinearArrow` 追加 |
| `fav/src/emit_python.rs` | `LinearArrow` 追加 |
| `fav/src/fmt.rs` | `LinearArrow` の表示追加 |
| `fav/src/driver.rs` | `LinearArrow` / `LinearFn` 追加 + テスト |
| `fav/Cargo.toml` | バージョン更新 |
| `site/content/docs/language/linear-types.mdx` | 新規作成 |
