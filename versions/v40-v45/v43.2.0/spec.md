# v43.2.0 仕様書 — 戻り値型推論: `fav check` 統合・E0410 系

## 概要

v43.1.0 で「戻り値型省略」を実装した。本バージョンでは推論が失敗する場合のエラーコード（E0410/E0411）を追加し、`fav check --show-types` に推論戻り値型の表示を追加する。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0410/E0411 エントリ追加 |
| `fav/self/checker.fav` | `check_body_ty` — E0410 パス追加 |
| `fav/src/driver.rs` | `collect_fn_inferred_return_types` + `--show-types` 拡張 + `v43200_tests` |
| `fav/Cargo.toml` | version 43.1.0 → 43.2.0 |
| `CHANGELOG.md` | v43.2.0 エントリ追加 |

---

## T1 — `fav/src/error_catalog.rs`

E0406 エントリの直後、E042x セクションコメントの直前に以下を挿入する:

```rust
    // ── E041x: 戻り値型推論 (v43.2.0) ──────────────────────────────────────────
    ErrorEntry {
        code: "E0410",
        title: "ambiguous return type",
        category: "types",
        description: "Return type was omitted but cannot be inferred from the function body (body expression yields `Unknown`).",
        example: "fn f() { undefined_fn() }  // E0410: body type Unknown",
        fix: "Add an explicit return type annotation `-> RetType`, or ensure the body has a deterministic type.",
    },
    ErrorEntry {
        code: "E0411",
        title: "inferred return type mismatch",
        category: "types",
        // ロードマップ定義: 「省略型と明示型の不一致」
        // = 戻り値型を省略した関数の推論結果が、明示的に宣言された型（別の関数のシグネチャ等）と合わない場合
        description: "Return type was omitted, but the type inferred from the body does not match the explicitly declared type in the usage context. (v43.3.0+ で検出開始)",
        example: "fn f() { 42 }  // inferred Int; annotation elsewhere declares String → E0411",
        fix: "Add an explicit `-> RetType` annotation to the function, or fix the usage context.",
    },
```

**挿入位置**: `E0406` エントリの `},` の直後、`\ ── E042x: CEP パターン` の直前。

---

## T2 — `fav/self/checker.fav`

### `check_body_ty` 変更

**変更前:**
```favnir
fn check_body_ty(fname: String, ret: TypeExpr, r: InfResult) -> Result<String, String> {
    \ v43.1.0: ret == TeSimple("") means return type was omitted — infer from body (always OK)
    if type_expr_to_str(ret) == "" {
        Result.ok(fname)
    } else {
        ...
    }
}
```

**変更後:**
```favnir
fn check_body_ty(fname: String, ret: TypeExpr, r: InfResult) -> Result<String, String> {
    \ v43.1.0: ret == TeSimple("") means return type was omitted �� infer from body
    if type_expr_to_str(ret) == "" {
        \ v43.2.0: if body infers Unknown, E0410 (ambiguous return type)
        if apply_subst(r.subst, r.ty) == "Unknown" {
            Result.err(fmt_err("E0410", String.concat(fname, ": ambiguous return type — body type cannot be inferred")))
        } else {
            Result.ok(fname)
        }
    } else {
        if types_compatible(apply_subst(r.subst, r.ty), type_expr_to_str(ret)) {
            Result.ok(fname)
        } else {
            Result.err(fmt_err("E0009", String.concat(fname, String.concat(": declared return ", String.concat(type_expr_to_str(ret), String.concat(" but body infers ", apply_subst(r.subst, r.ty)))))))
        }
    }
}
```

**注意**: checker.fav のコメント行は `\` で始まる（`//` ではない）。

---

## T3 — `fav/src/driver.rs`

### `collect_fn_inferred_return_types` 関数追加

`collect_binding_types` 関数の直後に追加:

```rust
/// Collect fn definitions with omitted return type for --show-types (v43.2.0).
fn collect_fn_inferred_return_types(file: &str) -> Vec<FnReturnInfo> {
    use crate::ast::Item;

    let source = load_file(file);
    let program = match crate::frontend::parser::Parser::parse_str(&source, file) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let mut result = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            if fd.return_ty.is_none() {
                result.push(FnReturnInfo {
                    name: fd.name.clone(),
                    file: file.to_string(),
                    line: fd.span.line,
                });
            }
        }
    }
    result
}
```

### `FnReturnInfo` struct 追加

`BindingInfo` struct の直後に追加:

```rust
/// v43.2.0: inferred return type info for --show-types
#[derive(Debug)]
struct FnReturnInfo {
    name: String,
    file: String,
    line: u32,
}
```

### `cmd_check` の `show_types` ブロック拡張

既存の `show_types` ブロック（`collect_binding_types` 呼び出し後）に追記:

```rust
if show_types {
    // 既存: bind/chain types
    let bindings = collect_binding_types(path);
    for b in &bindings {
        let location = format!("{}:{}", b.file, b.line);
        let w006_mark = if b.warning.as_deref() == Some("W006") { "  \u{2190} W006" } else { "" };
        println!("{:<30} bind {:10} : {}{}", location, b.name, b.ty, w006_mark);
    }
    // v43.2.0: fn inferred return types
    let fn_returns = collect_fn_inferred_return_types(path);
    for f in &fn_returns {
        let location = format!("{}:{}", f.file, f.line);
        println!("{:<30} fn   {:<10} : (return type inferred from body)", location, f.name);
    }
}
```

### `v43200_tests` 追加

`v43100_tests` の直前に挿入（降順）:

```rust
// -- v43200_tests (v43.2.0) -- 戻り値型推論: fav check 統合・E0410 系 --
#[cfg(test)]
mod v43200_tests {
    #[test]
    fn cargo_toml_version_is_43_2_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.2.0"), "Cargo.toml must contain version 43.2.0");
    }
    #[test]
    fn e0410_e0411_in_error_catalog() {
        let catalog = include_str!("../src/error_catalog.rs");
        assert!(catalog.contains("E0410"), "error_catalog.rs must contain E0410");
        assert!(catalog.contains("E0411"), "error_catalog.rs must contain E0411");
    }
    #[test]
    fn checker_fav_check_body_ty_has_e0410() {
        let checker = include_str!("../self/checker.fav");
        assert!(checker.contains("E0410"), "checker.fav must contain E0410 in check_body_ty");
    }
}
```

---

## 完了条件

- `cargo test` 2906 tests passed, 0 failed（2903 + 3）
- `v43200_tests` 3 件 pass
- `fav check --show-types` に fn inferred return type 行が出力される
- `error_catalog.rs` に E0410/E0411 エントリが存在する
- `checker.fav` の `check_body_ty` に E0410 パスが存在する

---

## 影響範囲

- **既存テスト**: `check_body_ty` の変更は「Unknown 型の body」にのみ影響。正常なコードに影響なし。
- **E0009 回帰**: `TeSimple("")` + 非 Unknown body → `Result.ok(fname)` は v43.1.0 と同一。
- **E0411**: 本バージョンでは catalog に追加するが checker.fav での検出は v43.3.0 以降。
- **FnReturnInfo の JSON 出力**: `FnReturnInfo` は `#[derive(Debug)]` のみ（`serde::Serialize` なし）。テキスト専用（`show_types` ブロックの非 json パス）で使用する。`--show-types --json` との統合は v43.9.0 `fav check --show-inference` 実装時に追加する。
- **`--json` フラグとの併用**: `json` が `true` の場合は既存の JSON 出力パス（行 3882〜3903）を経由するため、`collect_fn_inferred_return_types` は呼ばれない（テキスト専用パスのみ拡張）。
