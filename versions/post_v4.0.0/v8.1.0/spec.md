# Favnir v8.1.0 仕様書

Date: 2026-05-28
Theme: fav check 配線 — checker.fav を fav check パイプラインに接続

---

## 概要

v8.1.0 では `fav check foo.fav` が Rust 製 `checker.rs` ではなく
Favnir 製 `checker.fav` 経由で型チェックを行うように差し替える。

```
Before: fav check foo.fav → [Rust parser] → checker.rs → Vec<TypeError>
After:  fav check foo.fav → [Rust parser] → ast_lower_checker → checker.fav → Vec<TypeError>
```

v8.1.0 の完了をもって「**型チェッカーのセルフホスト完成**」とする。
Rust に残るのは VM core（バイトコード実行）+ primitive I/O + primitive 演算のみ。

---

## アーキテクチャ

### 処理フロー

```
fav check foo.fav
  │
  ├─ [Rust] load_file("foo.fav")
  ├─ [Rust] Parser::parse_str(source) → ast::Program
  ├─ [Rust] ast_lower_checker::lower_program(&prog) → VMValue  ← 新規
  ├─ [Rust] get_checker_fav_artifact()                         ← 新規（OnceLock キャッシュ）
  ├─ [VM]   VM::run(checker_artifact, "check", [prog_vm])      ← 新規
  └─ [Rust] parse_checker_errors(result) → Vec<TypeError>      ← 新規
```

### 新規コンポーネント

| ファイル | 役割 |
|---------|------|
| `src/middle/ast_lower_checker.rs` | Rust AST → VMValue 変換（checker.fav の型定義に対応） |
| `src/middle/checker_fav_runner.rs` | checker.fav アーティファクトのロード・実行・結果変換 |

### 既存コンポーネントの変更

| ファイル | 変更内容 |
|---------|---------|
| `src/driver.rs` | `check_single_file` を checker.fav 経由に差し替え |
| `src/backend/vm.rs` | `Compiler.check_raw` を checker.fav ベースに更新 |

---

## Phase A: AST Lowering（`ast_lower_checker.rs`）

Rust の `ast::Program` を checker.fav の型定義に対応した `VMValue` ツリーに変換する。

### VMValue の変換規則

checker.fav の variant 型は VM 内で以下のように表現される：
- 0 引数: `VMValue::Variant("TagName", None)`
- 1 引数: `VMValue::Variant("TagName", Some(Box::new(arg_vm)))`
- N 引数: `VMValue::Variant("TagName", Some(Box::new(VMValue::Record({"_0": v0, "_1": v1, ...}))))`

### 型対応表

#### Lit

| Rust `ast::Lit` | checker.fav `Lit` |
|-----------------|-------------------|
| `Lit::Int(n)` | `LInt(n)` |
| `Lit::Float(f)` | `LFloat(f)` |
| `Lit::Str(s)` | `LStr(s)` |
| `Lit::Bool(b)` | `LBool(b)` |
| `Lit::Unit` | `LUnit` |

#### Op（BinOp）

| Rust `ast::BinOp` | checker.fav `Op` |
|--------------------|-----------------|
| `Add` | `OpAdd` |
| `Sub` | `OpSub` |
| `Mul` | `OpMul` |
| `Div` | `OpDiv` |
| `Mod` | `OpMod` |
| `Eq` | `OpEq` |
| `NotEq` | `OpNeq` |
| `Lt` | `OpLt` |
| `Gt` | `OpGt` |
| `LtEq` | `OpLtEq` |
| `GtEq` | `OpGtEq` |
| `And` | `OpAnd` |
| `Or` | `OpOr` |

#### Pat（Pattern）

| Rust `ast::Pattern` | checker.fav `Pat` |
|---------------------|------------------|
| `Wildcard(_)` | `PWild` |
| `Bind(name, _)` | `PVar(name)` |
| `Lit(Lit::Int(n), _)` | `PInt(n)` |
| `Lit(Lit::Float(f), _)` | `PFloat(f)` |
| `Lit(Lit::Str(s), _)` | `PStr(s)` |
| `Lit(Lit::Bool(b), _)` | `PBool(b)` |
| `Lit(Lit::Unit, _)` | `PUnit` |
| `Variant(name, None, _)` | `PVariant(name)` |
| `Variant(name, Some(inner), _)` | `PVariantP(name, lower_pat(inner))` |
| その他 | `PWild`（フォールバック） |

#### Expr

| Rust `ast::Expr` | checker.fav `Expr` |
|------------------|-------------------|
| `Lit(lit, _)` | `ELit(lower_lit(lit))` |
| `Ident(name, _)` | `EVar(name)` |
| `BinOp(op, l, r, _)` | `EBinOp(lower_op(op), lower_expr(l), lower_expr(r))` |
| `If(cond, then, else_opt, _)` | `EIf(lower_expr(cond), lower_block(then), lower_block_or_unit(else_opt))` |
| `Closure(params, body, _)` | `ELambda(first_param_or_"_", lower_expr(body))` |
| `FieldAccess(expr, field, _)` | `EAccess(lower_expr(expr), field)` |
| `RecordConstruct(name, fields, _)` | `ERecordLit(name, lower_fields(fields))` |
| `Match(scrutinee, arms, _)` | `EMatch(lower_expr(scrutinee), lower_arms(arms))` |
| `Block(block)` | `lower_block(block)` |
| `Apply(func, args, _)` | `lower_apply(func, args)` ← 詳細は後述 |
| `Pipeline(steps, _)` | パイプラインを ECall チェーンに展開 |
| その他（TypeApply, FString 等）| `EVar("_unsupported_")` |

#### Apply の分解

`Apply(func, args)` を `ECall(ns, fname, earglist)` に変換する：

```
func = Ident("fname")         → ECall("", "fname", lower_args(args))
func = FieldAccess(Ident(ns), fname) → ECall(ns, fname, lower_args(args))
func = Ident("ns.fname")      → ECall("ns", "fname", lower_args(args))  ← "." で分割
その他                         → ECall("", lower_expr_as_name(func), lower_args(args))
```

引数リスト `args: Vec<Expr>` → 末尾から折り畳んで `EArgList`/`EArgNil` チェーンに変換。

#### Block の展開

`Block { stmts: Vec<Stmt>, tail: Expr }` →

```
stmts が空            → lower_expr(tail)
stmts[0] = Bind(b)    → EBind(b.pattern_name, lower_expr(b.expr), lower_rest)
stmts[0] = Expr(e)    → EBlock(lower_expr(e), lower_rest)
stmts[0] = Chain(c)   → EBind(c.name, lower_expr(c.expr), lower_rest)
stmts[0] = その他     → lower_rest
```

`b.pattern_name`: パターンが `Pattern::Bind(name, _)` なら `name`、それ以外は `"_"`.

#### TypeExpr

| Rust `ast::TypeExpr` | checker.fav `TypeExpr` |
|----------------------|------------------------|
| `Named("List", [t], _)` | `TeList(lower_te(t))` |
| `Named("Option", [t], _)` | `TeOption(lower_te(t))` |
| `Optional(t, _)` | `TeOption(lower_te(t))` |
| `Named("Result", [ok, err], _)` | `TeResult(lower_te(ok), lower_te(err))` |
| `Fallible(t, _)` | `TeResult(lower_te(t), TeSimple("String"))` |
| `Named("Map", [k, v], _)` | `TeMap(lower_te(k), lower_te(v))` |
| `Arrow(a, b, _)` | `TeFn(lower_te(a), lower_te(b))` |
| `Named(name, [], _)` | `TeSimple(name)` |
| その他 | `TeSimple("Unknown")` |

#### FnDef

```
ast::FnDef {
    visibility, name, effects, params, return_ty, body, ...
}
→ checker.fav FnDef {
    is_public: visibility == Public
    name: name
    effects: effects.iter().map(effect_to_str).collect()
    params: params.iter().map(lower_param).collect()
    ret: return_ty.map(lower_te).unwrap_or(TeSimple("Unit"))
    body: lower_block(&body)
}
```

#### Item

| Rust `ast::Item` | checker.fav `Item` |
|------------------|-------------------|
| `FnDef(fd)` | `IFn(lower_fn_def(fd))` |
| `TypeDef(td)` | `IType(lower_type_def(td))` |
| `TestDef(td)` | `ITest(lower_test_def(td))` |
| その他 | スキップ（Program.items に含めない） |

---

## Phase B: checker.fav ランナー（`checker_fav_runner.rs`）

### アーティファクトのキャッシュ

```rust
static CHECKER_FAV_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

fn get_checker_fav_artifact() -> Arc<FvcArtifact> {
    CHECKER_FAV_ARTIFACT.get_or_init(|| {
        let src = include_str!("../../self/checker.fav");
        // または: std::fs::read_to_string("self/checker.fav")
        let prog = Parser::parse_str(src, "checker.fav").expect("checker.fav parse");
        let ir   = compile_program(&prog);
        Arc::new(codegen_program(&ir))
    }).clone()
}
```

**注意**: `include_str!` を使うとバイナリに埋め込まれるため、`fav check` の起動が高速になる。
ただし checker.fav を変更するたびに再コンパイルが必要。v8.1.0 ではファイルパスから読む方式で開始し、後で embed に変更。

### checker.fav の呼び出し

```rust
pub fn run_checker_fav(prog_vm: VMValue) -> Result<(), Vec<String>> {
    let artifact = get_checker_fav_artifact();
    let check_idx = artifact.fn_idx_by_name("check")
        .expect("checker.fav must export `check`");
    let result = VM::run(&artifact, check_idx, vec![prog_vm])
        .map_err(|e| vec![e.to_string()])?;

    match result {
        VMValue::Variant(ref tag, _) if tag == "Ok" => Ok(()),
        VMValue::Variant(ref tag, Some(ref msg)) if tag == "Err" => {
            // msg は "E0xxx: message\nE0xxx: ..." の文字列
            let text = vm_extract_str(msg);
            Err(text.lines().map(|l| l.to_string()).collect())
        }
        _ => Err(vec!["unexpected checker.fav result".to_string()]),
    }
}
```

### エラー文字列 → TypeError 変換

checker.fav のエラーは `"E0xxx: message"` 形式の文字列。
v8.1.0 ではスパン情報なしで `TypeError` を生成する：

```rust
fn checker_fav_errors_to_type_errors(msgs: Vec<String>) -> Vec<TypeError> {
    msgs.into_iter().map(|msg| TypeError {
        message: msg,
        span: Span::default(),
    }).collect()
}
```

スパン情報は v8.2.0 以降で checker.fav 側に追加予定。

---

## Phase C: cmd_check への配線

`src/driver.rs` の `check_single_file` を更新：

```rust
fn check_single_file(path: &str) -> (String, Vec<TypeError>, Vec<FavWarning>) {
    let source = load_file(path);
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // checker.fav 経由でチェック
    let prog_vm = ast_lower_checker::lower_program(&program);
    match checker_fav_runner::run_checker_fav(prog_vm) {
        Ok(()) => (source, vec![], vec![]),
        Err(msgs) => {
            let errors = checker_fav_runner::checker_fav_errors_to_type_errors(msgs);
            (source, errors, vec![])
        }
    }
}
```

`--legacy-check` フラグ（オプション）で旧 checker.rs にフォールバックできるようにする。

---

## Phase D: Compiler.check_raw の更新

`vm.rs` の `"Compiler.check_raw"` primitive を checker.fav ベースに切り替え：

```rust
"Compiler.check_raw" => {
    let path = /* ... */;
    let src = std::fs::read_to_string(&path)?;
    let program = Parser::parse_str(&src, &path)?;
    let prog_vm = ast_lower_checker::lower_program(&program);
    match checker_fav_runner::run_checker_fav(prog_vm) {
        Ok(()) => Ok(ok_vm(VMValue::Str("compiled".to_string()))),
        Err(msgs) => Ok(err_vm(VMValue::Str(msgs.join("\n")))),
    }
}
```

---

## Phase E: テスト

### driver.rs 統合テスト（3 件追加）

| ID | テスト名 | 検証内容 |
|----|----------|----------|
| F-1 | `checker_fav_wire_valid_fn` | 型エラーなし Favnir ソースを checker.fav が `Ok` と判定する |
| F-2 | `checker_fav_wire_type_error` | 型エラーありソースを checker.fav が `Err(E0xxx)` と判定する |
| F-3 | `checker_fav_wire_generic_fn` | ジェネリクス関数を含むソースが checker.fav で正しく通る |

---

## 完了条件

- `fav check fav/self/checker.fav` が checker.fav 自身経由で通る（完全ブートストラップ）
- `fav check examples/basic/users.fav` 等の既存ファイルが正しくチェックされる
- `cargo test` — 1103+ tests passing（+3 新規）
- checker.rs への依存は `--legacy-check` フォールバックのみ

---

## 既知の制約（v8.1.0 スコープ外）

- **スパン情報なし**: エラーに行番号・列番号が付かない（"foo.fav: E0001: ..." 形式）
  → v8.2.0 以降で checker.fav にスパン追跡を追加
- **Pipeline / FString / Collect 等**: `EVar("_unsupported_")` にフォールバック
  → 型エラーが検出されない可能性あり（偽陰性）
- **複雑なパターン（Record パターン等）**: `PWild` にフォールバック
- **include_str! 埋め込み**: v8.1.0 ではファイルパスから読む方式（デプロイ時に self/ ディレクトリが必要）
