# v41.7.0 仕様書 — W030 lint: 冗長 refinement ガード検出

**フェーズ**: Type Precision（v41.x スプリント）
**前バージョン**: v41.6.0（Newtype 自動 impl、2865 tests）
**目標テスト数**: 2867（+2）

---

## 概要

`type PositiveInt = Int where |v| v >= 0` のような refinement type alias を持つパラメータに対し、
関数本体で `if x >= 0 { ... }` のような冗長ガードが書かれた場合に **W030** 警告を発する。

refinement 型の invariant はコンパイル時に保証されるため、同一条件を if ガードで再確認することは冗長（「二重チェック」）であり、コード品質上の問題である。

---

## 動機

```favnir
type PositiveInt = Int where |v| v >= 0

fn double(x: PositiveInt) -> Int {
    if x >= 0 {    // W030: x は PositiveInt — invariant が既に x >= 0 を保証している
        x * 2
    } else {
        0
    }
}
```

---

## スコープ

### v41.7.0 に含む

- **type alias refinement の冗長ガード検出**:
  - 対象: `TypeBody::Alias` で `invariants` が非空の TypeDef
  - invariant 形式: `|v| v op literal` または `|v| literal op v`（単純二項比較）
  - 検出パターン: FnDef の直接 if 条件が `param op literal` と構造的に一致する場合
  - 検出スコープ: 関数本体の直接 `Stmt::If` および Block トップレベルの if 文

- **適用演算子**: `>=`, `>`, `<=`, `<`, `==`, `!=`（`BinOp::GtEq` 等）

### v41.7.0 スコープ外

- ネストした if の中の二重ガード検出（v42.0+ へ）
- `&&` / `||` で組み合わされた複合条件の分解マッチ（v42.0+ へ）
- record refinement（`invariants` in `TypeBody::Record`）（v42.0+ へ）
- Wrapper type（`type Kg(Float)`）の refinement ガード（v42.0+ へ）

---

## 実装方針

### 1. lint.rs — `collect_refinement_aliases` ヘルパー

```rust
/// type alias の refinement 情報: (closure_param, BinOp, Box<Expr>, Box<Expr>)
fn collect_refinement_aliases(program: &Program) -> HashMap<String, (String, BinOp, Box<Expr>, Box<Expr>)>
```

処理手順:
1. `program.items` を走査し `Item::TypeDef(td)` を選択
2. `td.body` が `TypeBody::Alias(_)` かつ `td.invariants` が非空のものを対象とする
3. `td.invariants[0]` が `Expr::Closure(params, body_expr, _)` で `params.len() == 1` の場合に限り処理
4. `body_expr` が `Expr::BinOp(op, lhs, rhs, _)` の形であれば `(td.name) → (param_name, op, lhs, rhs)` としてマップに登録

### 2. lint.rs — `check_w030_redundant_refinement_guard`

```rust
pub fn check_w030_redundant_refinement_guard(program: &Program, errors: &mut Vec<LintError>) {
    let refinements = collect_refinement_aliases(program);
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            check_w030_fn(fd, &refinements, errors);
        }
    }
}
```

`check_w030_fn` の処理:
1. `fd.params` から `{param_name → refinement_info}` マップを構築（TypeExpr::Named(type_name) が refinement map にある場合のみ）
2. `fd.body.stmts` を走査し `Stmt::Expr(Expr::If(cond, _, _, span))` を検出
3. `cond` が `Expr::BinOp(if_op, lhs, rhs, _)` の場合:
   - `lhs` が `Expr::Ident(param_name)` → 右辺リテラルで比較
   - `rhs` が `Expr::Ident(param_name)` → 左辺リテラルで比較
4. `if_op` と invariant の `op` が一致し、リテラル値も一致する場合 → W030 を push

**注意**: `Stmt::Expr(if_expr)` だけでなく `Stmt::Bind` の rhs が if の場合も対象とするが、v41.7.0 では Stmt ループのみで十分。

### 3. lint.rs — `run_lint` への組み込み

```rust
// v41.7.0: W030
check_w030_redundant_refinement_guard(program, &mut errors);
```

を `check_w025_schema_mismatch` の呼び出しの後に追加。

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `fav/src/lint.rs` | ① `collect_refinement_aliases` 追加<br>② `check_w030_redundant_refinement_guard` 追加<br>③ `run_lint` に呼び出し追加 | 中（約 60 行） |
| `fav/src/driver.rs` | `v41600_tests::cargo_toml_version_is_41_6_0` スタブ化 + `v41700_tests` 追加（2 件） | 小 |
| `fav/Cargo.toml` | version: `41.6.0` → `41.7.0` | 1 行 |
| `CHANGELOG.md` | `[v41.7.0]` エントリ追加 | 数行 |

`checker.fav` は変更不要。

---

## テスト計画

### Rust テスト（driver.rs）— 2 件

```rust
mod v41700_tests {
    #[test]
    fn cargo_toml_version_is_41_7_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.7.0"), "Cargo.toml must contain version 41.7.0");
    }

    #[test]
    fn lint_w030_redundant_guard_detected() {
        use crate::frontend::parser::Parser;
        use crate::lint::run_lint;
        let src = r#"
type PositiveInt = Int where |v| v >= 0
fn double(x: PositiveInt) -> Int {
    if x >= 0 { x * 2 } else { 0 }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let errs = run_lint(&prog);
        assert!(
            errs.iter().any(|e| e.code == "W030"),
            "W030 should be reported for redundant refinement guard"
        );
    }
}
```

---

## 完了条件

- `cargo test` 全通過（2867 tests passed, 0 failed）
- `v41700_tests` 2 件すべて pass
- `lint_w030_redundant_guard_detected` が W030 を正しく検出する
- 既存テストが壊れていない
