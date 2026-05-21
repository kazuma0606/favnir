# Favnir v6.0.0 実装計画 — セルフホスト

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `fav/src/backend/vm.rs` | `IO.argv` / `List.take_while` / `List.drop_while` 追加 |
| `fav/src/middle/checker.rs` | 上記 3 関数の型シグネチャ追加 |
| `fav/self/lexer.fav` | 新規作成 |
| `fav/self/parser.fav` | 新規作成 |
| `fav/self/checker.fav` | 新規作成 |
| `fav/self/codegen.fav` | 新規作成 |
| `fav/self/compiler.fav` | 新規作成 |
| `fav/src/backend/vm_stdlib_tests.rs` | 新規 vm テスト追加 |
| `fav/src/driver/self_tests.rs` | 新規 end-to-end テスト追加 |

---

## Phase A: VM Primitive 追加

### A-1: `IO.argv` (vm.rs)

`fav run main.fav -- arg1 arg2` で渡した引数を Favnir コードから取得できるようにする。
VM の `run_builtin` に追加。`args` フィールドを `VmContext` に持たせるか、
またはグローバルな静的変数として保持する。

```rust
// vm.rs: run_builtin に追加
"IO.argv" => {
    let argv: Vec<VMValue> = std::env::args()
        .skip_while(|a| a != "--")
        .skip(1) // "--" 自体をスキップ
        .map(|a| VMValue::Str(a))
        .collect();
    Ok(VMValue::List(argv))
}
```

checker.rs に型シグネチャを追加:

```rust
("IO", "argv") => Some(Type::List(Box::new(Type::String))),
```

### A-2: `List.take_while` / `List.drop_while` (vm.rs)

```rust
"List.take_while" => {
    let mut it = args.into_iter();
    let list = it.next().ok_or_else(|| "List.take_while requires 2 arguments".to_string())?;
    let pred = it.next().ok_or_else(|| "List.take_while requires 2 arguments".to_string())?;
    match list {
        VMValue::List(xs) => {
            let mut result = Vec::new();
            for x in xs {
                match call_closure(&pred, vec![x.clone()], ctx)? {
                    VMValue::Bool(true) => result.push(x),
                    _ => break,
                }
            }
            Ok(VMValue::List(result))
        }
        _ => Err("List.take_while requires (List, Fn)".to_string()),
    }
}
"List.drop_while" => {
    let mut it = args.into_iter();
    let list = it.next().ok_or_else(|| "List.drop_while requires 2 arguments".to_string())?;
    let pred = it.next().ok_or_else(|| "List.drop_while requires 2 arguments".to_string())?;
    match list {
        VMValue::List(xs) => {
            let mut iter = xs.into_iter();
            loop {
                match iter.as_slice().first() {
                    None => break,
                    Some(x) => match call_closure(&pred, vec![x.clone()], ctx)? {
                        VMValue::Bool(true) => { iter.next(); }
                        _ => break,
                    }
                }
            }
            Ok(VMValue::List(iter.collect()))
        }
        _ => Err("List.drop_while requires (List, Fn)".to_string()),
    }
}
```

checker.rs 型シグネチャ（`List.take` の直後に追加）:

```rust
("List", "take_while") | ("List", "drop_while") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

---

## Phase B: Lexer (`fav/self/lexer.fav`)

核となる実装パターン:

```favnir
fn is_digit(c: String) -> Bool {
    String.contains("0123456789", c)
}

fn is_alpha(c: String) -> Bool {
    String.contains("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_", c)
}

fn scan_number(chars: List<String>) -> {token: Token, rest: List<String>} {
    bind digits <- List.take_while(chars, |c| is_digit(c))
    bind rest <- List.drop_while(chars, |c| is_digit(c))
    bind s <- String.from_chars(digits)
    match Int.parse(s) {
        | Ok(n) -> { token: TkInt(n), rest: rest }
        | Err(_) -> { token: TkInt(0), rest: rest }
    }
}

fn scan_ident(chars: List<String>) -> {token: Token, rest: List<String>} {
    bind word_chars <- List.take_while(chars, |c| String.or(is_alpha(c), is_digit(c)))
    bind rest <- List.drop_while(chars, |c| String.or(is_alpha(c), is_digit(c)))
    bind s <- String.from_chars(word_chars)
    bind tok <- match s {
        | "fn" -> TkFn
        | "type" -> TkType
        | "if" -> TkIf
        | "else" -> TkElse
        | "match" -> TkMatch
        | "bind" -> TkBind
        | "import" -> TkImport
        | "public" -> TkPublic
        | "true" -> TkTrue
        | "false" -> TkFalse
        | _ -> TkIdent(s)
    }
    { token: tok, rest: rest }
}

fn lex(src: String) -> Result<List<Token>, String> {
    bind chars <- String.chars(src)
    scan(chars, [])
}

fn scan(chars: List<String>, acc: List<Token>) -> Result<List<Token>, String> {
    bind trimmed <- List.drop_while(chars, |c| String.contains(" \t\r\n", c))
    match List.first(trimmed) {
        | None -> Result.ok(List.reverse(List.concat([TkEof], acc)))
        | Some(c) ->
            bind rest <- List.drop(trimmed, 1)
            match c {
                | "+" -> scan(rest, List.concat([TkPlus], acc))
                | "-" -> ...
                | _ ->
                    if is_digit(c)
                    then
                        bind r <- scan_number(trimmed)
                        scan(r.rest, List.concat([r.token], acc))
                    else if is_alpha(c)
                    then
                        bind r <- scan_ident(trimmed)
                        scan(r.rest, List.concat([r.token], acc))
                    else
                        Result.err(String.concat("unknown char: ", c))
            }
    }
}
```

**実装上の注意**:
- `String.or` は非存在 → `Bool` の `||` 演算子を使う: `is_alpha(c) || is_digit(c)`
- `Int.parse` が必要 → vm.rs に追加が必要か確認（`String.to_int` を使う）
- リストへの先頭追加は `List.concat([new_tok], acc)` で模倣
- 再帰深度に注意（大きなファイルでスタックオーバーフロー可能性あり）

---

## Phase C: Parser (`fav/self/parser.fav`)

パーサー状態の管理:

```favnir
type ParseState = { tokens: List<Token>, pos: Int }

fn peek(state: ParseState) -> Token {
    Option.unwrap_or(List.get(state.tokens, state.pos), TkEof)
}

fn advance(state: ParseState) -> ParseState {
    { tokens: state.tokens, pos: state.pos + 1 }
}

fn expect(state: ParseState, expected: Token) -> Result<ParseState, String> {
    if peek(state) == expected
    then Result.ok(advance(state))
    else Result.err(String.concat("expected token, got something else"))
}

fn parse_expr(state: ParseState) -> Result<{expr: Expr, state: ParseState}, String> {
    parse_equality(state)
}

fn parse_equality(state: ParseState) -> Result<{expr: Expr, state: ParseState}, String> {
    Result.and_then(parse_comparison(state), |left_r|
        parse_equality_rhs(left_r.expr, left_r.state)
    )
}
```

**実装上の注意**:
- `ParseState` を `Result` 内で引き回すパターン（モナディックスタイル）
- 左再帰を避けるため `parse_equality_rhs` のような tail 関数を用意
- `List.get(tokens, pos)` の O(n) アクセスが多くなる → v6.1.0 でスライスベースに改善

---

## Phase D: 型チェッカー (`fav/self/checker.fav`)

型環境の実装:

```favnir
type CheckState = {
    env: Map<String, String>,    // 変数名 → 型の文字列表現
    errors: List<String>
}

fn type_to_str(ty: Type) -> String {
    match ty {
        | TInt -> "Int"
        | TFloat -> "Float"
        | TBool -> "Bool"
        | TStr -> "String"
        | TUnit -> "Unit"
        | TList(inner) -> String.concat("List<", String.concat(type_to_str(inner), ">"))
        | TOption(inner) -> String.concat("Option<", String.concat(type_to_str(inner), ">"))
        | TNamed(n) -> n
        | TUnknown -> "_"
        | _ -> "?"
    }
}

fn infer_expr(state: CheckState, expr: Expr) -> {ty: Type, state: CheckState} {
    match expr {
        | ELit(LInt(_)) -> { ty: TInt, state: state }
        | ELit(LFloat(_)) -> { ty: TFloat, state: state }
        | ELit(LBool(_)) -> { ty: TBool, state: state }
        | ELit(LStr(_)) -> { ty: TStr, state: state }
        | ELit(LUnit) -> { ty: TUnit, state: state }
        | EVar(name) ->
            bind ty_str <- Option.unwrap_or(Map.get(state.env, name), "_")
            { ty: str_to_type(ty_str), state: state }
        | EBinOp(BAdd, _, _) -> { ty: TInt, state: state }
        | EBinOp(BEq, _, _) -> { ty: TBool, state: state }
        | _ -> { ty: TUnknown, state: state }
    }
}
```

---

## Phase E: バイトコードコンパイラ (`fav/self/codegen.fav`)

オペコード → バイト列変換:

```favnir
fn encode_opcode(op: Opcode) -> List<Int> {
    match op {
        | OpConst(idx) ->
            [0x01, Int.band(Int.shr(idx, 8), 0xFF), Int.band(idx, 0xFF)]
        | OpConstUnit -> [0x02]
        | OpConstTrue -> [0x03]
        | OpConstFalse -> [0x04]
        | OpLoadLocal(idx) -> [0x10, Int.band(idx, 0xFF)]
        | OpStoreLocal(idx) -> [0x11, Int.band(idx, 0xFF)]
        | OpReturn -> [0x16]
        | OpAdd -> [0x20]
        | OpSub -> [0x21]
        | OpMul -> [0x22]
        | OpDiv -> [0x23]
        | OpEq -> [0x24]
        | OpJump(offset) ->
            [0x30, Int.band(Int.shr(offset, 8), 0xFF), Int.band(offset, 0xFF)]
        | OpJumpIfFalse(offset) ->
            [0x31, Int.band(Int.shr(offset, 8), 0xFF), Int.band(offset, 0xFF)]
        | _ -> []
    }
}

fn compile_expr(ctx: CodegenCtx, expr: Expr) -> Result<{ctx: CodegenCtx, ops: List<Opcode>}, String> {
    match expr {
        | ELit(LInt(n)) ->
            bind idx <- add_const(ctx, CLInt(n))
            Result.ok({ ctx: idx.ctx, ops: [OpConst(idx.idx)] })
        | EBinOp(BAdd, left, right) ->
            Result.and_then(compile_expr(ctx, left), |l|
                Result.and_then(compile_expr(l.ctx, right), |r|
                    Result.ok({ ctx: r.ctx, ops: List.concat(List.concat(l.ops, r.ops), [OpAdd]) })
                )
            )
        | _ -> Result.err("unsupported expression in codegen")
    }
}
```

---

## Phase F: 統合 (`fav/self/compiler.fav`)

各フェーズを `Result.and_then` でパイプライン化:

```favnir
fn compile_file(path: String) -> Result<List<Int>, String> !Io {
    Result.and_then(IO.read_file_raw(path), |src|
        Result.and_then(lex(src), |tokens|
            Result.and_then(parse(tokens), |program|
                Result.and_then(check(program), |_|
                    compile(program)
                )
            )
        )
    )
}

public fn main() -> Unit !Io {
    bind args <- IO.argv()
    match List.first(args) {
        | None -> IO.println("usage: compiler.fav <source.fav>")
        | Some(path) ->
            match compile_file(path) {
                | Err(e) -> IO.println(String.concat("error: ", e))
                | Ok(bytes) ->
                    bind out_path <- String.replace(path, ".fav", ".fvc")
                    IO.write_bytes_raw(out_path, bytes)
                    IO.println(String.concat("compiled: ", out_path))
            }
    }
}
```

---

## Phase G: ブートストラップ検証

```bash
# Stage 1: Rust コンパイラで Favnir 製コンパイラをコンパイル
fav build fav/self/compiler.fav -o /tmp/compiler_s1.fvc

# Stage 2: Stage 1 で自身を再コンパイル
fav run /tmp/compiler_s1.fvc -- fav/self/compiler.fav

# Stage 3: Stage 2 のバイトコードで再度コンパイル
fav run /tmp/compiler_s1.fvc -- fav/self/compiler.fav  # 同じ入力で同じ出力
diff /tmp/compiler_s1.fvc /tmp/compiler_s2.fvc         # 一致を確認
```

Rust 側のブートストラップテスト（`fav/src/driver/self_tests.rs`）:

```rust
#[test]
fn self_hosted_lexer_matches_rust_lexer() {
    // fav/self/lexer.fav を fav run で実行し、
    // 出力トークン列が Rust レキサーのトークン列と一致することを確認
}

#[test]
fn self_hosted_compiler_type_checks() {
    // fav check fav/self/compiler.fav がエラーなしで通ることを確認
}
```

---

## 実装順序

1. Phase A-1: IO.argv を vm.rs + checker.rs に追加
2. Phase A-2: List.take_while / List.drop_while を vm.rs + checker.rs に追加
3. Phase B: lexer.fav を実装し `fav check` を通す
4. Phase B テスト: Rust レキサーとの出力一致テスト
5. Phase C: parser.fav を実装し `fav check` を通す
6. Phase C テスト: 簡単なプログラムのパーステスト
7. Phase D: checker.fav を実装し `fav check` を通す
8. Phase E: codegen.fav を実装し `fav check` を通す
9. Phase F: compiler.fav を実装し `fav check` を通す
10. Phase G: ブートストラップ検証

---

## 注意点

### `call_closure` の使い方

`List.take_while` / `List.drop_while` の実装で closure を呼び出す際は、
vm.rs の既存 `List.filter` / `List.map` の実装パターンを参照する。

### `Int.parse` vs `String.to_int`

`String.to_int` が `Option<Int>` を返すので使用可能。
`Option.unwrap_or(String.to_int(s), 0)` で安全に変換。

### 再帰深度

Favnir VM はスタックベースのインタプリタ。大きなファイルを処理する際は
再帰関数の深度に注意。レキサー・パーサーは末尾再帰に近い形で実装する。

### ブートストラップ完了の定義

v6.0.0 では「`fav check fav/self/compiler.fav` が通る + 対象サブセットのプログラムを
正しくコンパイルできる」を完了条件とする。
バイトコード完全一致（Stage 1 == Stage 2）は Phase G の追加検証として行う。
