# v13.7.0 Plan — seq pipeline + ctx 統合 実装計画

Date: 2026-06-10

---

## 実装アプローチの概要

最小変更でコアを実装する。`FlwDef` に `ctx_param: Option<String>` を追加し、
コンパイラで 2-param 関数にする。E0022 は Rust checker でのアリティチェックのみ。

**変更ファイル一覧**（推定）:

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `FlwDef.ctx_param: Option<String>` 追加 |
| `fav/src/frontend/parser.rs` | `seq Name(ctx) = ...` 構文追加 |
| `fav/src/middle/compiler.rs` | `compile_flw_def` を ctx-aware パスに対応 |
| `fav/src/middle/ir.rs` | 変更なし（IRFnDef.param_count=2 で既存フォーマット利用可能） |
| `fav/src/middle/checker.rs` | E0022 チェック追加 |
| `fav/src/error_catalog.rs` | E0022 エントリ追加 |
| `fav/src/middle/ast_lower_checker.rs` | FlwDef の ctx_param を checker IR に反映（要調査） |
| `fav/src/driver.rs` | v137000_tests モジュール追加 |
| `infra/e2e-demo/fav2py/src/pipeline.fav` | seq Pipeline(ctx) に書き換え |
| `infra/e2e-demo/airgap/src/analyze.fav` | seq AnalyzePipeline(ctx) に書き換え |
| `fav/Cargo.toml` | version = "13.7.0" |

---

## Phase A — AST 拡張

### A-1: `fav/src/ast.rs`

`FlwDef` 構造体に `ctx_param` フィールドを追加:

```rust
pub struct FlwDef {
    pub name: String,
    pub steps: Vec<FlwStep>,
    pub ctx_param: Option<String>,  // ← 追加
    pub span: Span,
}
```

影響箇所: `FlwDef` の構築（`FlwDef { name, steps, span }` → `ctx_param: None` を補完）
- 既存の `FlwDef` 生成箇所を全件 `ctx_param: None` で初期化
- grep: `FlwDef {` で全件確認

---

## Phase B — パーサー

### B-1: `fav/src/frontend/parser.rs`

`parse_flw_def` 関数（または相当箇所）で `(ident)` オプションを追加。

現行の seq パース（推定コード）:
```rust
// seq Name = Step1 |> Step2
expect(Token::Seq)?;
let name = parse_ident()?;
// ctx_param なし
expect(Token::Eq)?;
let steps = parse_flw_steps()?;
```

変更後:
```rust
expect(Token::Seq)?;
let name = parse_ident()?;
let ctx_param = if peek() == Token::LParen {
    consume(); // (
    let ident = parse_ident()?;
    expect(Token::RParen)?; // )
    Some(ident)
} else {
    None
};
expect(Token::Eq)?;
let steps = parse_flw_steps()?;
FlwDef { name, steps, ctx_param, span }
```

**注意**: 実際のパーサー構造を先に読んで適切な場所に挿入すること。
`seq` のパーサー箇所を `grep "Seq\|seq\|FlwDef" parser.rs` で特定。

---

## Phase C — コンパイラ

### C-1: `fav/src/middle/compiler.rs`

`compile_flw_def` を分岐させる。

```rust
fn compile_flw_def(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    if fd.ctx_param.is_some() {
        compile_flw_def_ctx_aware(fd, ctx)
    } else {
        compile_flw_def_plain(fd, ctx)  // 既存ロジックをリネーム
    }
}
```

`compile_flw_def_ctx_aware` の実装:

```rust
fn compile_flw_def_ctx_aware(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    // param_count = 2: slot 0 = $ctx, slot 1 = $input
    let ctx_slot = ctx.define_local("$ctx");    // slot 0
    let input_slot = ctx.define_local("$input"); // slot 1

    // 各ステージを Stage($ctx, intermediate) で呼び出す
    // SeqChain で fail-fast を維持
    ...
}
```

`build_step_call_ctx` (新規ヘルパー):

```rust
fn build_step_call_ctx(
    step: &FlwStep,
    ctx_slot: u16,
    input: IRExpr,
    ctx: &mut CompileCtx,
) -> IRExpr {
    match step {
        FlwStep::Stage(name) => {
            let callee = ...;
            let ctx_expr = IRExpr::Local(ctx_slot, Type::Unknown);
            IRExpr::Call(Box::new(callee), vec![ctx_expr, input], Type::Unknown)
        }
        FlwStep::Par(_) => {
            // v13.7.0 スコープ外: 実行時パニックまたはコンパイルエラー
            IRExpr::Global(u16::MAX, Type::Unknown) // E0007 相当
        }
    }
}
```

---

## Phase D — チェッカー

### D-1: `fav/src/error_catalog.rs`

E0022 エントリを追加:
```rust
("E0022", "ctx-aware pipeline called with wrong number of arguments"),
```

### D-2: `fav/src/middle/checker.rs`

ctx-aware FlwDef の call-site アリティチェック。

**実装方針**:

checker.rs の `check_program`（またはその呼び出し先）で:
1. プログラム内の全 FlwDef を収集し `ctx-aware` かどうかをマッピング
2. `Expr::Apply(FieldAccess(Ident(name), ...) | Ident(name), args, span)` を走査
3. `name` が ctx-aware FlwDef の場合:
   - `args.len() == 2` → OK
   - `args.len() != 2` → E0022

逆に ctx なし FlwDef が 2 引数で呼ばれた場合も E0022（arity mismatch）。

**FlwDef 収集ヘルパー**:
```rust
fn collect_flw_defs(program: &Program) -> HashMap<String, Option<String>> {
    // name → ctx_param の map
    program.items.iter().filter_map(|item| {
        if let Item::FlwDef(fd) = item {
            Some((fd.name.clone(), fd.ctx_param.clone()))
        } else { None }
    }).collect()
}
```

**注意**: checker.fav 経由の型推論は変更しない。Rust checker レベルのアリティチェックのみ実装。

### D-3: `fav/src/middle/ast_lower_checker.rs`

checker.fav 向けの IR lowering で FlwDef の ctx_param を反映する必要があるか調査。
- `lower_flw_def` または相当箇所を確認
- ctx_param がある場合、checker.fav には `param_count=2` の FnDef として見せる必要がある

---

## Phase E — テスト

### E-1: `fav/src/driver.rs` に `v137000_tests` モジュールを追加

テスト一覧:

```rust
mod v137000_tests {
    // バージョン確認
    fn version_is_13_7_0()

    // ctx-aware seq パース
    fn seq_ctx_param_parsed()       // seq Pipeline(ctx) = A |> B をパースできる

    // ctx-aware seq コンパイル
    fn seq_ctx_compiles_param_count_2()  // param_count = 2 でコンパイルされる
    fn seq_ctx_stage_gets_ctx_arg()      // 各ステージ呼び出しが ctx を第1引数で受ける

    // E0022 チェック
    fn e0022_ctx_pipeline_called_without_ctx()  // seq P(ctx)=... → P(data) → E0022
    fn e0022_plain_pipeline_called_with_ctx()   // seq P=... → P(ctx, data) → E0022（オプション）

    // E2E デモ コンパイル（型チェックのみ）
    fn e2e_fav2py_seq_ctx_compiles()   // pipeline.fav の seq Pipeline(ctx) が型チェックパス
    fn e2e_airgap_seq_ctx_compiles()   // analyze.fav の seq AnalyzePipeline(ctx) が型チェックパス

    // 後方互換
    fn seq_no_ctx_backward_compat()    // 既存 seq（ctx なし）が変わらず動作
}
```

### E-2: E2E デモ更新（型チェックのみ）

`infra/e2e-demo/fav2py/src/pipeline.fav`:
- `fn main` の chain 3 行 → `seq Pipeline(ctx) = ... + Pipeline(ctx, ...)` に書き換え
- 型チェックが通ること（実行は v14.0 以降）

`infra/e2e-demo/airgap/src/analyze.fav`:
- `fn analyze_pipeline` + main の手動 bind → `seq AnalyzePipeline(ctx) = ...` に書き換え
- `write_output` の戻り値型が `Unit`（非 Result）なので SeqStageCheck との相性を確認

---

## Phase F — バージョンバンプ + コミット

### F-1: `fav/Cargo.toml`

```toml
version = "13.7.0"
```

### F-2: self-check（オプション）

```bash
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
```

### F-3: cargo test

```bash
cargo test v137000
cargo test  # 全件（リグレッション確認）
```

---

## 実装上の注意点・リスク

### R-1: seq の既存テスト数
FlwDef に関連するテストが多い場合、`ctx_param: None` の補完漏れで一斉コンパイルエラーになる。
Phase A 完了後すぐに `cargo build` で確認すること。

### R-2: airgap の `write_output` 戻り値型
現在 `fn write_output(ctx: AppCtx, rows: List<TxnRow>) -> Unit`（非 Result）。
SeqChain が Result の unwrap を想定しているため、`write_output` を seq の最終ステージにする場合は問題なし（最終ステージは SeqStageCheck を通らない）。
ただし中間ステージには Result を返す関数のみ使用可能。

### R-3: `par` ステップの ctx 転送
v13.7.0 では par + ctx は未対応。
ctx-aware FlwDef に par ステップが含まれる場合のエラー処理を忘れずに実装すること。

### R-4: checker.fav の型推論
checker.fav は `ECall(ns, fname, args)` を型推論するが、FlwDef の直接呼び出しは
`ECall("", "Pipeline", args)` として infer_call_user に解決される。
param_count=2 の FlwDef が checker.fav に正しく見えるか確認が必要。

### R-5: AbstractFlwDef との整合性
`AbstractFlwDef`（型パラメータ付き FlwDef template）は v13.7.0 では変更しないが、
`FlwDef { ctx_param: None }` のデフォルト補完が必要な箇所がある可能性あり。

---

## 実装順序（推奨）

```
A（AST）→ cargo build で補完漏れ確認
→ B（Parser）→ 手動テスト: fav check で seq P(ctx) = ... がパース可能か確認
→ C（Compiler）→ cargo test (seq 関連テスト) で既存テスト維持確認
→ D（Checker）→ E0022 テスト追加
→ E-2（E2E デモ書き換え）→ E-1（テスト追加）
→ F（バージョンバンプ + cargo test 全件）
```
