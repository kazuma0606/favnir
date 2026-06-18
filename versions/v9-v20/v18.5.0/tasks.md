# v18.5.0 — Linear Types タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/ast.rs` — 新型追加

- [x] `TypeExpr` enum に `LinearArrow(Box<TypeExpr>, Box<TypeExpr>, Span)` を追加（`T -o U`）
- [x] `TypeExpr::span()` の match に `LinearArrow(_, _, s) => s` を追加
- [x] `Type` enum に `LinearFn(Box<Type>, Box<Type>)` を追加

### T2: 波及ファイル更新（exhaustive match 修正）

**`TypeExpr::LinearArrow` の追加:**

- [x] `fav/src/fmt.rs` — `LinearArrow(a, b, _) => format!("{} -o {}", type_expr(a), type_expr(b))` 追加
- [x] `fav/src/emit_python.rs` — `LinearArrow(_, _, _) => "Any".to_string()` 追加
- [x] `fav/src/middle/ast_lower_checker.rs` — `lower_te` / `te_to_string` に `LinearArrow` 追加
- [x] `fav/src/middle/compiler.rs` — `lower_type_expr*` / `substitute_self_in_type_expr` に `LinearArrow` 追加（`Type::Arrow(...)` として扱う）
- [x] `fav/src/middle/checker.rs` — `resolve_type_expr_with_subst` / `resolve_type_expr_with_self` / `validate_type_expr_arity` に `LinearArrow` 追加
- [x] `fav/src/driver.rs` — `format_type_expr` / `favnir_type_display` / `graphql_type_from_type_expr_nonnull` / `proto_type_from_type_expr_nonwrapper` / `favnir_type_to_sql_from_expr` に `LinearArrow` 追加

**`Type::LinearFn` の追加:**

- [x] `fav/src/middle/checker.rs` — `display()` / `collect_type_vars_ordered` / resolve 系メソッドに `LinearFn` 追加
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

### T3: `fav/src/frontend/lexer.rs` — `TokenKind::LinearArrow` 追加

- [x] `TokenKind` enum に `LinearArrow` を追加
- [x] `next_token` で `-` の後に `o` が続き、その後が単語境界（非英数字）の場合 `LinearArrow` を返す
  - `peek2()` を使って境界チェック（`self.chars` ではなく）
  - 例: `-o ` → `LinearArrow`、`-offset` → `Minus + Ident("offset")`

### T4: `fav/src/frontend/parser.rs` — `-o` 構文のパース

- [x] `parse_type_expr_inner` で `->` と同様に、型を読んだ後 `TokenKind::LinearArrow` を認識
- [x] `LinearArrow` トークンを consume して右辺の型を `parse_type_expr_inner` で読む
- [x] `TypeExpr::LinearArrow(Box<lhs>, Box<rhs>, span)` を返す

### T5: `fav/src/middle/checker.rs` — 線形型チェック実装

- [x] `Checker` struct に `linear_types: HashSet<String>` フィールドを追加
- [x] `Checker` struct に `linear_env: HashMap<String, LinearState>` フィールドを追加
- [x] `LinearState` enum を定義（`Available` / `Consumed`）
- [x] `Checker::new()` / `new_with_resolver()` で `linear_types` に `"Connection"` / `"Tx"` を登録
- [x] `check_fn_def` の開始時に `linear_env` を保存・初期化（saved_linear_env で入れ子対応）
- [x] `Stmt::Bind` チェック時: 右辺の型が `Type::Named(name, _)` かつ `linear_types` に含まれる場合、`linear_env` に `Available` で登録
- [x] `Expr::Ident` チェック時:
  - `linear_env.get(&name) == Some(Consumed)` なら **E0332** を emit
  - `linear_env.get(&name) == Some(Available)` なら `Consumed` に更新
- [x] 関数終了時（`check_fn_def` の終わり）:
  - `linear_env` に `Available` が残っていれば **E0333** を emit
- [x] 終了後 `saved_linear_env` を復元

### T6: `fav/src/driver.rs` — `v185000_tests` 追加

- [x] `v184000_tests::version_is_18_4_0` に `#[ignore]` を追加
- [x] `v185000_tests` モジュールを追加（5件）:
  - [x] `version_is_18_5_0` — Cargo.toml に "18.5.0" が含まれる
  - [x] `linear_arrow_lexes` — `-o` が `LinearArrow` トークンになる
  - [x] `linear_arrow_type_parses` — `Connection` パラメータを持つ関数がパースされる
  - [x] `linear_double_use_is_e0332` — `Connection` 変数を 2 回使うと E0332
  - [x] `linear_unused_is_e0333` — `Connection` 変数を使わずに関数終了すると E0333

### T7: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.4.0` → `18.5.0` に更新

### T8: `site/content/docs/language/linear-types.mdx` 作成

- [x] `T -o U` 線形関数型の構文説明
- [x] 組み込み線形型（`Connection` / `Tx`）の説明
- [x] E0332（二重消費）/ E0333（未消費）のエラー説明と例
- [x] `with_connection` パターンの使用例
- [x] 通常の `->` との違いの説明

---

## テスト結果

| テスト名 | 結果 |
|---|---|
| `version_is_18_5_0` | PASS |
| `linear_arrow_lexes` | PASS |
| `linear_arrow_type_parses` | PASS |
| `linear_double_use_is_e0332` | PASS |
| `linear_unused_is_e0333` | PASS |

**5/5 PASS。全体 1679 tests pass（リグレッションなし）。**

---

## 実装ノート

- **`LinearState` enum**: `Available` / `Consumed`。`check_fn_def` 呼び出しごとに保存・復元（入れ子 fn 対応）。
- **lexer の境界チェック**: `self.chars` は存在しない。`peek2()` を使う（`source[pos+1]`）。
- **`E0332`**: `Expr::Ident` の resolved type が返される前に `linear_env` をチェック。
- **`E0333`**: `check_fn_def` の末尾、`env.pop()` の前に `linear_env` の `Available` を列挙して emit。
- **compiler.rs の `LinearArrow`**: 実行時区別不要なので `Type::Arrow(...)` として扱う（codegen 変更なし）。
- **`Type::LinearFn`**: checker 内部の型表現。`display()` は `"{} -o {}"` 形式。
