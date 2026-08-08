# v62.4.0 Spec — AOT エフェクトディスパッチ最適化（Pure ステージのインライン化）

Version: 62.4.0
Status: 未着手
Base tests: 3388
Target tests: 3390

---

## 概要

ロードマップの「`!Pure` ステージのインライン化」を、現在の IR 構造に合わせた形で実装する。

`IRFnDef` に `effects` フィールドは存在せず（v35.4.0 で削除済み）、Effect enum も廃止されている。
そのため「pure」の判定基準を **「AOT コンパイル可能な IR のみからなる関数」**として定義する。
判定は `is_aot_pure(expr: &IRExpr) -> bool` により行う。

ロードマップ記載の「削減バイト数」表示は関数サイズ推定が必要なため v62.9.0 に後送りとする
（ロードマップとの意図的な乖離 — 実績欄に理由を記載する）。

---

## 前提確認（T0 で実施）

- `IRFnDef` に `effects` フィールドが **存在しない**（`ir.rs` L41-48）
- Effect enum は v35.4.0 で削除済み（`EffectDef` はスタブのみ）
- `IRExpr` の variant: Lit / Local / Global / TrfRef / CallTrfLocal / Call / Block / If / Match /
  FieldAccess / BinOp / Closure / Collect / Emit / RecordConstruct / RecordSpread / Par / AssertSchema
- `cranelift_aot.rs` の `lower_lit` が `Lit::Str` を非サポートとして Err を返すことを確認
- `driver.rs` に `compile_program` が存在することを確認（L16 で import 済み）
- `cranelift_aot.rs` に `AotStats` 構造体が **存在しない**
- `driver.rs` に `cmd_build_aot_stats` が **存在しない**
- `main.rs` の `Some("build")` アームに `--aot-stats` が **存在しない**

---

## 実装スコープ

### 1. `cranelift_aot.rs` — `AotStats` 構造体、`is_aot_pure` 補助関数、`analyze_for_inlining` 追加

**`AotStats` 構造体**（`impl` ブロック外、ファイル先頭側に追加）:
```rust
pub struct AotStats {
    pub inlined: Vec<String>,
    pub dispatched: Vec<String>,
}
```

**`is_aot_pure` 補助関数**（モジュールレベル private 関数）:
```rust
fn is_aot_pure(expr: &IRExpr) -> bool
```

判定ロジック（exhaustive — それ以外はすべて `false`）:
- `IRExpr::Lit(Lit::Str(_), _)` → `false`（`lower_lit` が Err を返すため）
- `IRExpr::Lit(_, _)` → `true`
- `IRExpr::Local(_, _)` → `true`
- `IRExpr::BinOp(_, lhs, rhs, _)` → `is_aot_pure(lhs) && is_aot_pure(rhs)`
- `IRExpr::If(cond, then_e, else_e, _)` → 3 つすべて pure
- `IRExpr::Block(stmts, final_expr, _)` → stmts の各 IRStmt が pure かつ final_expr が pure
  - `IRStmt::Bind(_, e)` / `LegacyBind(_, e)` / `Expr(e)` → `is_aot_pure(e)`
  - その他の IRStmt → `false`
- `_ =>` **`false`**（Global / TrfRef / CallTrfLocal / Call / Match / FieldAccess / Closure /
  Collect / Emit / RecordConstruct / RecordSpread / Par / AssertSchema は非 pure）

**`analyze_for_inlining` メソッド**（`impl CraneliftBackend` 内）:
```rust
pub(crate) fn analyze_for_inlining(ir: &IRProgram) -> AotStats
```
- 各 `IRFnDef` の body を `is_aot_pure` で判定
- pure → `inlined`、非 pure → `dispatched`

### 2. `driver.rs` — `cmd_build_aot_stats` 追加

```rust
pub fn cmd_build_aot_stats(src: &str) -> String
```
- parse → compile → `analyze_for_inlining`
- 成功時: `format!("AOT stats: {} inlined, {} dispatched", stats.inlined.len(), stats.dispatched.len())`
- エラー時: `format!("parse error: {e}")`

### 3. `main.rs` — `--aot-stats` フラグ追加

`Some("build")` アームのループ内に `"--aot-stats"` アームを追加。
`if aot_stats { ... } else if link { ... }` の順で分岐（`--aot-stats` を先にチェック）。

### 4. `driver.rs` — `v62400_tests` 追加

**`aot_pure_stage_inlined`**（純粋な算術関数がインライン候補に分類される）:
- ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
- `analyze_for_inlining` の `stats.inlined` に `"add"` が含まれることを確認
- `stats.inlined` が空でないことを確認

**`aot_effectful_stage_not_inlined`**（AOT 非サポート IR を持つ関数が dispatch 対象に分類される）:
- ソース: `"fn greeting() -> String { \"hello\" }\nfn main() -> Bool { 1 + 2 == 3 }"`
  - `fn greeting` は `Lit::Str` を含む → `is_aot_pure` が `false` → `dispatched`
- `stats.dispatched` に `"greeting"` が含まれることを確認
- `stats.inlined` に `"greeting"` が含まれないことを確認

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62400` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3390 tests passed, 0 failed

---

## 非スコープ

- 実際の cranelift IR へのインライン展開（IR レベルの `call` → `inline` 変換）
- `--aot-stats` のバイト削減量表示（ロードマップ記載あり、関数サイズ推定が必要なため v62.9.0 に後送り）
- `site/content/docs/runtime/aot.mdx` — v62.9.0 スコープのため作成しない
