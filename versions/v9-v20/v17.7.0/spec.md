# v17.7.0 — `forall` プロパティベーステスト Spec

Date: 2026-06-15

## 概要

`fav test` に「任意の型の入力を自動生成し、任意の性質（プロパティ）が成立するか検証する」機能を追加する。

`forall x: Type { body }` 構文で記述し、デフォルト 100 ケースを生成・実行する。
失敗時は「反例（counterexample）」となった入力値を報告する。

---

## 構文

```fav
// 基本: 単一変数の forall
test "trim は冪等" {
  forall s: String {
    bind trimmed <- String.trim(s)
    assert_eq(trimmed, String.trim(trimmed))
  }
}

// guard 付き（where で入力の前提条件を絞り込む）
test "非ゼロ整数は安全な除数" {
  forall n: Int where { n != 0 } {
    bind q <- 100 / n
    assert_true(q * n == 100 || (100 % n) != 0)
  }
}

// Int のプロパティ
test "絶対値は非負" {
  forall n: Int {
    assert_true(Math.abs(n) >= 0)
  }
}

// Bool の対称性
test "Bool は true か false のどちらか" {
  forall b: Bool {
    assert_true(b == true || b == false)
  }
}
```

---

## 対応型（v17.7.0）

| 型 | 生成される値 |
|---|---|
| `Int` | 0, 1, -1, 2^31-1, -2^31, + ランダム整数 |
| `Float` | 0.0, 1.0, -1.0, 0.5, -0.5, + ランダム浮動小数点（NaN/Inf 除外）|
| `String` | `""`, `" "`, `"a"`, ASCII ランダム文字列（0〜20 文字） |
| `Bool` | `true` / `false` 交互（計2パターン、N に関係なく最大2ケース）|

> 注: v17.7.0 では単一変数のみサポート。複数変数（`forall a: Int, b: Int`）は v17.8.0 以降。
> `List<T>` 型の生成は v17.8.0 以降。

---

## 失敗時の出力

```
FAIL  test "trim は冪等"
  [counterexample] s = "  hello  "
  assertion failed: assert_eq
```

失敗した最初の入力値を反例として報告する。
v17.7.0 では Shrinking（反例の縮小）は行わない（v17.8.0 以降）。

---

## `--cases N` オプション

```bash
fav test src/pipeline.test.fav --cases 200
```

`forall` ブロックの試行回数を N に変更する（デフォルト: 100）。
`forall b: Bool` のように生成できる値が少ない型は試行回数が上限になる。

---

## ガード条件（`where { ... }`）

```fav
forall n: Int where { n != 0 } {
  ...
}
```

- 生成した値がガード条件を満たさない場合はスキップ（該当ケースをカウントしない）
- スキップが多い場合、`--cases N` を大きくしてリトライ上限を増やすことを推奨
- スキップ上限は `N * 10`（試行回数 × 10）に達したら `too many filtered cases` で警告

---

## 実装スコープ（変更ファイル）

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/lexer.rs` | `TokenKind::Forall` 追加（`"forall"` キーワード） |
| `fav/src/ast.rs` | `Stmt::Forall(ForallStmt)` / `ForallStmt` / `ForallVar` 追加 |
| `fav/src/frontend/parser.rs` | `parse_forall_stmt` 追加 |
| `fav/src/middle/checker.rs` | `check_stmt` に `Stmt::Forall` 追加、型検査 |
| `fav/src/middle/compiler.rs` | `compile_stmt_into` に `Stmt::Forall` — ForIn ループへデシュガー |
| `fav/src/backend/vm.rs` | `__forall_gen_int` / `__forall_gen_str` / `__forall_gen_bool` / `__forall_gen_float` VM primitive 追加 |
| `fav/src/fmt.rs` | `Stmt::Forall` exhaustive match 追加 |
| `fav/src/emit_python.rs` | `Stmt::Forall` exhaustive match 追加 |
| `fav/src/lineage.rs` | `Stmt::Forall` exhaustive match 追加（4 関数） |
| `fav/src/lint.rs` | `Stmt::Forall` exhaustive match 追加（7 関数） |
| `fav/src/main.rs` | `fav test --cases N` オプション追加 |
| `fav/src/driver.rs` | `cmd_test` に `--cases` 対応、`v177000_tests` 追加 |

---

## AST 定義

```rust
// ast.rs に追加
pub enum Stmt {
    // ... 既存 ...
    Forall(ForallStmt),   // NEW: forall x: Type where { guard } { body }
}

pub struct ForallStmt {
    pub vars: Vec<ForallVar>,       // 変数リスト（v17.7.0 は1件のみ）
    pub guard: Option<Expr>,        // where { ... } ガード（省略可）
    pub body: Block,                // テスト本体
    pub span: Span,
}

pub struct ForallVar {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}
```

---

## コンパイル戦略（ForIn デシュガー）

`Stmt::Forall` をコンパイル時に `ForIn` ループへ展開する。
新 VM opcode は不要。

```
// forall x: Int { body }
// → コンパイル後（概念）:
bind __vals <- __forall_gen_int(CASES)   // List<Int>
for x in __vals {
  body
}

// forall x: Int where { x != 0 } { body }
// → コンパイル後:
bind __vals <- __forall_gen_int(CASES * 10)  // ガード考慮で多めに生成
bind __filtered <- [v | v <- __vals, v != 0] // 内包表記でフィルタ
bind __taken <- List.take(__filtered, CASES)  // CASES件に絞る
for x in __taken {
  body
}
```

`CASES` は `FORALL_CASES` 環境変数（デフォルト 100）から取得。

---

## VM Primitive 仕様

```rust
// vm.rs / compiler.rs に追加

// __forall_gen_int(n: Int) -> List<Int>
// 先頭: 0, 1, -1, i32::MAX, i32::MIN
// 残り: n-5 件の疑似乱数整数（シード固定）
fn forall_gen_int(n: i64) -> Value

// __forall_gen_str(n: Int) -> List<String>
// 先頭: "", " ", "a", "\n", "hello world"
// 残り: ランダム ASCII 文字列（長さ 0〜20）
fn forall_gen_str(n: i64) -> Value

// __forall_gen_bool(n: Int) -> List<Bool>
// [true, false, true, false, ...] の繰り返し（最大 n 件）
fn forall_gen_bool(n: i64) -> Value

// __forall_gen_float(n: Int) -> List<Float>
// 先頭: 0.0, 1.0, -1.0, 0.5, -0.5
// 残り: ランダム浮動小数点（NaN/Inf 除外）
fn forall_gen_float(n: i64) -> Value
```

---

## テスト（v177000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_7_0` | Cargo.toml に "17.7.0" が含まれる |
| `forall_int_parses` | `forall n: Int { ... }` が AST として解析される |
| `forall_string_idempotent` | `String.trim` の冪等性が forall で確認できる |
| `forall_finds_counterexample` | 偽のプロパティ（`n > 0` をすべての Int で主張）が失敗する |
| `forall_with_guard` | `where { n != 0 }` ガードで 0 を除外した Int のプロパティが成立する |

---

## 完了条件（PASS=5）

1. `forall n: Int { ... }` が AST として解析される
2. `forall s: String { assert_eq(String.trim(s), String.trim(String.trim(s))) }` が 100 ケースでパス
3. 偽のプロパティを持つ `forall` テストが失敗し、反例が報告される
4. `forall n: Int where { n != 0 } { ... }` がガード付きで動作する
5. `cargo test` 全件パス（リグレッションなし）

---

## 非対応（スコープ外）

- 複数変数 `forall a: Int, b: Int { ... }` — v17.8.0 以降
- `List<T>` / `Option<T>` / カスタム型の生成 — v17.8.0 以降
- Shrinking（反例の縮小） — v17.8.0 以降
- `--seed N` 固定シードによる再現性 — v17.8.0 以降
