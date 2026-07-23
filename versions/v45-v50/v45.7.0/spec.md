# Spec: v45.7.0 — エラーメッセージ改善 Phase 2 + 数値リテラル `_`

Date: 2026-07-16
Status: TODO

---

## 概要

v45.6.0 で E0101〜E0200 に `suggestion` テキストを付与した。
本バージョンでは E0201〜E0413 の残りエントリに `suggestion` テキストを追加し、
あわせて数値リテラル中の `_` セパレータをレキサーでサポートする。

---

## 変更対象

### §1 — `error_catalog.rs`: E0201〜E0413 への `suggestion` 追加

`ErrorEntry.suggestion: Option<&'static str>` フィールドは v45.6.0 で追加済み。
現状 `suggestion: None` になっているエントリに対して意味ある提案テキストを設定する。

実在するエントリの対象コードと提案テキスト:

| コード | エラー名 | suggestion テキスト |
|---|---|---|
| E0213 | private field access | `"Add a dedicated accessor function in the type definition."` |
| E0219 | field access on non-record | `"Check that the value is a record type before accessing its fields."` |
| E0220 | undefined interface | `"Define the interface with \`interface Name { ... }\` or check for typos."` |
| E0221 | interface not implemented | `"Add \`impl InterfaceName for TypeName { ... }\` to implement the interface."` |
| E0222 | undefined function or variable | `"Define the variable with \`bind\`, or declare the function before using it."` |
| E0223 | match arm type mismatch | `"Make all match arms return the same type."` |
| E0224 | non-exhaustive match | `"Add the missing variant cases or add a wildcard arm \`_ => ...\`."` |
| E0225 | invalid binary operand types | `"Ensure both operands have the correct type for the operator."` |
| E0226 | if branch type mismatch | `"Make both branches of the if expression return the same type."` |
| E0227 | invariant type error | `"Invariant expressions must evaluate to Bool. Use a comparison like \`value > 0\`."` |
| E0241〜E0245 | record field errors | `"Check the field name and type against the type definition."` |
| E0251, E0253, E0254 | generic/type errors | `"Check the type parameters and constraints."` |
| E0274 | pipeline type mismatch | `"Check that all pipeline stages have compatible input/output types."` |
| E0310 | record spread missing field | `"Add the missing field to the record spread."` |
| E0311 | record spread extra field | `"Remove the extra field from the record spread."` |
| E0312〜E0315 | record/module errors | `"Check the record structure or module path."` |
| E0319〜E0324 | collection type errors | `"Check the element types and collection operations."` |
| E0365, E0368, E0369, E0373 | runtime errors | `"Check the operation and input types."` |
| E0374 | removed !Effect syntax | `"The \`!Effect\` syntax has been removed. Use \`ctx\` or \`bind\` instead."` |
| E0380〜E0384 | misc errors | `"Check the surrounding context and types."` |
| E0401 | missing method impl | `"Add the required method implementation for this interface."` |
| E0402 | method signature mismatch | `"Check the method signature against the interface definition."` |
| E0403 | undefined interface in impl | `"Verify the interface name is correct and defined."` |
| E0404〜E0406 | interface/impl errors | `"Check the interface and implementation definitions."` |
| E0410 | opaque type coercion | `"Check the opaque type definition and use a constructor function."` |
| E0411 | opaque inner type mismatch | `"Provide a value of the declared inner type."` |
| E0412 | undefined variant | `"Use the correct variant name from the type definition."` |
| E0413 | opaque type coerce | `"Use a dedicated constructor function to create this opaque type value."` |

注意:
- **E0230 は error_catalog.rs に存在しない**（エントリなし）— 対象外
- **E0414 は error_catalog.rs に存在しない**（予約のみ）— 対象外
- E0241〜E0245、E0251、E0253〜E0254 の各エントリ個別に適切な suggestion を設定する（上表は参考）

### §2 — `lexer.rs`: 数値リテラル `_` セパレータ

`lex_number`（`fav/src/frontend/lexer.rs`、現行 line ~573〜621）の digit スキャンループを修正し、
`_` を合法な桁区切りとして受け入れる。

**整数スキャン（変更前）**:
```rust
while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
    s.push(self.advance());
}
```

**整数スキャン（変更後）**:
```rust
while self.peek().map(|c| c.is_ascii_digit() || c == '_').unwrap_or(false) {
    let ch = self.advance();
    if ch != '_' { s.push(ch); }
}
```

小数部スキャンも同様に修正する（`.` 後の digit ループ）。
指数部スキャンは `lex_number` に存在しないため修正対象外。

`_` は advance するが `s` には push しない。これにより既存の `s.parse::<i64>()` / `s.parse::<f64>()` がそのまま動作する。

**許可するリテラル形式**:
- `1_000_000` → トークン値 `1000000`（Int）
- `0.000_15` → トークン値 `0.00015`（Float）
- `1_23_4` → トークン値 `1234`（任意のグルーピング）

### §3 — テスト（`driver.rs`）

`v457000_tests` モジュールを `v456000_tests` の直後に追加（3 件）。

`run_inline` ヘルパーは既存の各テストモジュールと同パターン（`Value` を返す）:
```rust
use crate::frontend::parser::Parser;
use crate::middle::checker::Checker;
use crate::backend::vm::VM;
use crate::middle::compiler::compile_program;
use crate::backend::codegen::codegen_program;
use crate::value::Value;

fn run_inline(src: &str) -> Value {
    let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
    let (errors, _) = Checker::check_program(&prog);
    assert!(errors.is_empty(), "type errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let fn_idx = artifact.fn_idx_by_name("main").expect("main not found");
    VM::run(&artifact, fn_idx, vec![]).expect("run failed")
}
```

テスト 3 件:
```rust
#[test]
fn e0410_suggestion() {
    use crate::error_catalog::ERROR_CATALOG;
    let entry = ERROR_CATALOG.iter().find(|e| e.code == "E0410").expect("E0410 not found");
    assert!(entry.suggestion.is_some(), "E0410 should have a suggestion");
    let s = entry.suggestion.unwrap();
    assert!(!s.is_empty(), "E0410 suggestion should not be empty");
}

#[test]
fn numeric_literal_underscore_int() {
    let src = r#"public fn main() -> Int { 1_000_000 }"#;
    let result = run_inline(src);
    assert_eq!(result, Value::Int(1_000_000));
}

#[test]
fn numeric_literal_underscore_float() {
    let src = r#"public fn main() -> Float { 0.000_15 }"#;
    let result = run_inline(src);
    assert_eq!(result, Value::Float(0.000_15));
}
```

---

## 変更しないファイル

- `ast.rs`（AST ノード変更なし）
- `parser.rs`（lexer のみ変更）
- `checker.rs`（型チェック変更なし）
- `compiler.rs` / `vm.rs`（数値リテラルはすでに正しく処理される）
- `site/`（ドキュメント更新は本バージョンのスコープ外）
- `emit_python.rs`

---

## 完了条件

- `cargo test` 全通過（2985 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v45.7.0 エントリ追加
- `versions/current.md` を v45.7.0（2985 tests）に更新
- `fav/Cargo.toml` version → `45.7.0`
