# Favnir v9.8.0 Implementation Plan

Date: 2026-06-02
Theme: `fav doc` — `///` ドキュメントコメント + 型シグネチャ → Markdown 自動生成

---

## Phase A: compiler.fav — `TkDocComment` トークン追加

`scan_collect`（`fav/self/compiler.fav` 約 line 290）の `//` 処理分岐を拡張する。

現状:
```favnir
if c == "/" && next_char_is(tail, "/") { scan_collect(skip_line_comment(...)) }
```

変更後:
```favnir
if c == "/" && next_char_is(tail, "/") {
    if next_char_is(List.drop(tail, 1), "/") {
        // "///" — doc comment: capture text as TkDocComment
        bind text <- collect_line_text(List.drop(tail, 2), "")
        scan_collect_with(List.drop_while(...), acc + [TkDocComment(text)])
    } else {
        scan_collect(skip_line_comment(...))
    }
}
```

- `Token` type に `TkDocComment(String)` variant を追加（`TkLineComment` のような位置）
- `token_eq` / `token_to_string` に `TkDocComment` arm 追加
- `collect_line_text(chars, acc)` ヘルパー追加（改行まで文字を収集）

---

## Phase B: compiler.fav — AST 構造体に `doc: String` 追加

対象 5 構造体すべてに `doc: String` フィールドを追加する。

```favnir
type FnDef = {
    is_public: Bool
    name: String
    params: List<Param>
    ret: TypeExpr
    body: Expr
    doc: String          // 新規追加（空文字 = ドキュメントなし）
}

type TypeDef = {
    name: String
    is_record: Bool
    variants: List<VariantDef>
    fields: List<Param>
    doc: String
}

type StageDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    param_ty:    TypeExpr
    ret_ty:      TypeExpr
    effects:     List<String>
    body:        Option<Expr>
    doc:         String
}

type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<String>
    doc:         String
}

type WrapperDef = {
    name:       String
    inner:      String
    where_pred: Expr
    with_impls: List<String>
    doc:        String
}
```

**既存の構造体リテラルをすべて `doc: ""` で更新する**（`parse_fn_def` など各パース関数内）。

---

## Phase C: compiler.fav — パーサー `collect_doc` 追加

```favnir
// parse_items の先頭で TkDocComment を収集する
fn collect_doc(toks: List<Token>, acc: String) -> DocCollect {
    match List.first(toks) {
        Some(TkDocComment(text)) =>
            collect_doc(List.drop(toks, 1), if acc == "" { text } else { String.concat(acc, String.concat("\n", text)) })
        _ => DocCollect { doc: acc, rest: toks }
    }
}

type DocCollect = { doc: String, rest: List<Token> }
```

`parse_items` 内で各 item をパースする前に `collect_doc` を呼び出し、得られた `doc` 文字列を各定義の `doc` フィールドにセットする。

具体的には `parse_items(toks, acc)` の先頭:
```favnir
fn parse_items(toks: List<Token>, acc: List<Item>) -> ParseResult<Program> {
    bind dc <- collect_doc(toks, "")
    // dc.doc = 収集したドキュメント, dc.rest = 残りトークン
    match List.first(dc.rest) {
        Some(TkFn)     => ... parse_fn_def_with_doc(dc.rest, dc.doc) ...
        Some(TkType)   => ... parse_type_or_wrapper_with_doc(dc.rest, dc.doc) ...
        Some(TkStage)  => ... parse_stage_def_with_doc(dc.rest, dc.doc) ...
        Some(TkSeq)    => ... parse_seq_def_with_doc(dc.rest, dc.doc) ...
        ...
    }
}
```

---

## Phase D: compiler.fav — `doc_program` 関数追加

### ヘルパー関数

```favnir
fn doc_fn_def(fd: FnDef) -> String {
    if fd.is_public {
        bind sig <- pretty_fn_sig(fd)   // "fn name(params) -> ret" の 1 行
        bind body <- if fd.doc == "" { "" } else { String.concat("\n\n", fd.doc) }
        String.concat("### ", String.concat(fd.name, String.concat("\n\n```\n", String.concat(sig, String.concat("\n```", body)))))
    } else { "" }
}

fn doc_wrapper_def(wd: WrapperDef) -> String { ... }
fn doc_type_def(td: TypeDef) -> String { ... }
fn doc_stage_def(sd: StageDef) -> String { ... }
fn doc_seq_def(sd: SeqDef) -> String { ... }
```

### `doc_items`

```favnir
fn doc_items(items: List<Item>, fns: List<String>, types: List<String>, stages: List<String>) -> String {
    match List.first(items) {
        None => build_doc_sections(fns, types, stages)
        Some(IFn(fd)) => doc_items(List.drop(items, 1), List.append(fns, doc_fn_def(fd)), types, stages)
        Some(IType(td)) => doc_items(List.drop(items, 1), fns, List.append(types, doc_type_def(td)), stages)
        Some(IWrapper(wd)) => doc_items(List.drop(items, 1), fns, List.append(types, doc_wrapper_def(wd)), stages)
        Some(IStage(sd)) => doc_items(List.drop(items, 1), fns, types, List.append(stages, doc_stage_def(sd)))
        Some(ISeq(sd)) => doc_items(List.drop(items, 1), fns, types, List.append(stages, doc_seq_def(sd)))
        Some(ITest(_)) => doc_items(List.drop(items, 1), fns, types, stages)
    }
}
```

### Public エントリポイント

```favnir
public fn doc_source(src: String) -> Result<String, String> {
    match lex(src) {
        Err(e)   => Result.err(e)
        Ok(toks) => match parse_program(toks) {
            Err(e)   => Result.err(e)
            Ok(prog) => Result.ok(doc_items(prog.items, [], [], []))
        }
    }
}
```

---

## Phase E: `compiler_fav_runner.rs` — `doc_source_str` 追加

`fmt_source_str`（lines 228–254）と完全に同じパターンで追加:

```rust
pub fn doc_source_str(src: &str) -> Result<String, String> {
    let compiled = load_compiler_fav_bytes()?;
    let mut store = build_store();
    let result = run_fav_fn(
        &compiled,
        &mut store,
        "doc_source",
        vec![VMValue::String(src.to_string())],
    )?;
    match result {
        VMValue::Variant(tag, args) if tag == "Ok" => {
            // args[0] is String
            ...
        }
        VMValue::Variant(tag, args) if tag == "Err" => {
            ...
        }
        _ => Err("unexpected result from doc_source".to_string()),
    }
}
```

---

## Phase F: `vm.rs` — `Compiler.doc_source_raw` primitive 追加

`Compiler.fmt_source_raw`（line 6576 付近）の直後に追加:

```rust
"Compiler.doc_source_raw" => match args.as_slice() {
    [VMValue::String(src)] => {
        let result = compiler_fav_runner::doc_source_str(src)
            .map(VMValue::String)
            .map_err(VMValue::String);
        Ok(result_to_vm_variant(result))
    }
    _ => Err("Compiler.doc_source_raw: expected String".to_string()),
},
```

---

## Phase G: `driver.rs` — `cmd_doc` 追加

```rust
pub fn cmd_doc(path: &str, out_dir: &str) -> Result<(), String> {
    // ディレクトリなら **/*.fav を走査、ファイルなら単一ファイル処理
    let entries = collect_fav_files(path)?;
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let src = std::fs::read_to_string(&entry).map_err(|e| e.to_string())?;
        let md = compiler_fav_runner::doc_source_str(&src)?;
        let out_name = entry.file_stem().unwrap().to_string_lossy() + ".md";
        let out_path = std::path::Path::new(out_dir).join(out_name.as_ref());
        std::fs::write(out_path, md).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

---

## Phase H: `main.rs` + `cli.fav` — `fav doc` サブコマンド追加

### main.rs

```rust
["doc", rest @ ..] => {
    let path = rest.iter().find(|a| !a.starts_with("--")).copied().unwrap_or(".");
    let out = rest.windows(2)
        .find(|w| w[0] == "--out")
        .map(|w| w[1])
        .unwrap_or("docs");
    match cmd_doc(path, out) {
        Ok(_) => {}
        Err(e) => { eprintln!("error: {e}"); std::process::exit(1); }
    }
}
```

### cli.fav

```favnir
| CmdDoc(String, String)   // (path, out_dir)

// parse_named_cmd に追加:
else if cmd == "doc" { parse_doc_cmd(args) }

fn parse_doc_cmd(args: List<String>) -> CliCmd {
    bind rest <- List.drop(args, 1)
    bind out  <- find_flag_value(rest, "--out", "docs")
    match find_positional(rest) {
        None    => CmdUnknown("doc requires a path argument")
        Some(p) => CmdDoc(p, out)
    }
}

fn run_doc(path: String, out: String) -> Unit !IO {
    match IO.read_file_raw(path) {
        Err(e) => { bind _ <- IO.write_stderr_raw(...) IO.exit_raw(1) }
        Ok(src) => match Compiler.doc_source_raw(src) {
            Err(e) => { bind _ <- IO.write_stderr_raw(...) IO.exit_raw(1) }
            Ok(md) => match IO.write_file_raw(out, md) {
                Err(e) => { bind _ <- IO.write_stderr_raw(...) IO.exit_raw(1) }
                Ok(_)  => IO.println(String.concat("doc: ", path))
            }
        }
    }
}
```

---

## Phase I: 統合テスト + self-check + bootstrap + バージョン更新

### `driver.rs` の `v980_tests` モジュール

| テスト名 | 内容 |
|---|---|
| `doc_fn_no_comment` | `///` なし `public fn` → シグネチャのみ出力 |
| `doc_fn_with_comment` | `/// Desc\npublic fn f...` → コメント + シグネチャ |
| `doc_private_fn_excluded` | `fn` (非 public) → 出力に含まれない |
| `doc_type_def` | `/// Desc\ntype Foo = ...` → Types セクションに出力 |
| `doc_wrapper_def` | `/// Desc\ntype Pct(Float) where ...` → Types セクション |
| `doc_stage_def` | `/// Desc\npublic stage ...` → Stages セクション |
| `doc_multiline_comment` | `///` 複数行 → 改行で結合 |
| `doc_empty_file` | 空ソース → `""` または最小 Markdown |

### self-check / bootstrap

- `cargo test checker_fav_wire_self_check` — 通過
- `cargo test bootstrap` — bytecode 維持
- `cargo test` — 全件通過（目標: 1215 件以上）

---

## 注意事項

- `doc: String` フィールドの追加により、既存の構造体リテラル生成箇所（`parse_fn_def` 等）を**すべて** `doc: ""` で更新する必要がある。見落とすとコンパイルエラー。
- `collect_doc` を `parse_items` に追加すると `TkDocComment` が item パースに食い込むため、`parse_items` のトークン列先頭に `TkDocComment` が来ても無限ループにならないよう注意（`collect_doc` が消費するため OK）。
- Rust 側 `cmd_doc` は Favnir の `Compiler.doc_source_raw` 経由でドキュメント生成するため、ディレクトリ走査・ファイル書き込みのみ Rust が担当する。
