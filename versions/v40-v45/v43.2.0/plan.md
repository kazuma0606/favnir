# v43.2.0 実装計画 — 戻り値���推論: `fav check` 統合・E0410 系

## 前提

- v43.1.0 完了（2903 tests）
- `fav/Cargo.toml` version: `43.1.0`
- checker.fav `check_body_ty` �� `TeSimple("")` パス実装済み

---

## タスク順序

```
T0 事前確認
T1 error_catalog.rs — E0410/E0411 追加
T2 checker.fav — check_body_ty E0410 パス追加
T3 driver.rs — FnReturnInfo + collect_fn_inferred_return_types + show_types 拡張
T4 driver.rs — v43200_tests 追加（v43100_tests の直前）
T5 Cargo.toml — version 43.1.0 → 43.2.0
T6 CHANGELOG.md — v43.2.0 エントリ追加
T7 cargo test 実行・確認（2906 pass, 0 fail）
T8 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` で 2903 / 0 を確認
2. `Cargo.toml` version が `43.1.0` ��あることを確認
3. `error_catalog.rs` に E0410/E0411 が存在しないことを確認

---

## T1 — error_catalog.rs

**挿��位置**: E0406 エントリの `},` の直後、`\ ── E042x: CEP パターン` コメントの直前。

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
        description: "Return type was omitted, but the type inferred from the body does not match the explicitly declared type in the usage context. (v43.3.0+ で検出開始)",
        example: "fn f() { 42 }  // inferred Int; annotation elsewhere declares String → E0411",
        fix: "Add an explicit `-> RetType` annotation to the function, or fix the usage context.",
    },
```

---

## T2 — checker.fav

`check_body_ty` 関数の `TeSimple("")` 分岐を拡張:

```favnir
fn check_body_ty(fname: String, ret: TypeExpr, r: InfResult) -> Result<String, String> {
    \ v43.1.0: ret == TeSimple("") means return type was omitted ��� infer from body
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

---

## T3 — driver.rs

### 3-a: `FnReturnInfo` struct を `BindingInfo` の直後に追加

```rust
/// v43.2.0: inferred return type info for --show-types
#[derive(Debug)]
struct FnReturnInfo {
    name: String,
    file: String,
    line: u32,
}
```

### 3-b: `collect_fn_inferred_return_types` 関数を `collect_binding_types` の直後に追加

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

### 3-c: `cmd_check` の `show_types` ブロックを拡張

既存の `show_types` ブロック（行 3906 付近）を以下に置き換え:

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

---

## T4 — driver.rs v43200_tests

`v43100_tests` の直前に挿入（モジュール順は降順: v43200 → v43100 → v43000...）:

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

## T5 — Cargo.toml

```toml
version = "43.2.0"
```

合わせて `v43100_tests` の `cargo_toml_version_is_43_1_0` テストをスタブ化（空ボディ）:

```rust
fn cargo_toml_version_is_43_1_0() {
    // Stubbed: version bumped to 43.2.0 -- assertion intentionally removed
}
```

---

## T6 — CHANGELOG.md

```markdown
## [v43.2.0] — 2026-07-12

### Added
- `fav/src/error_catalog.rs`: E0410（ambiguous return type）/ E0411（inferred return type mismatch）追加
- `fav/self/checker.fav`: `check_body_ty` — `TeSimple("")` かつ body Unknown 時に E0410 を返すパス追加
- `fav/src/driver.rs`: `FnReturnInfo` + `collect_fn_inferred_return_types` — omitted return type 関数を収集
- `fav/src/driver.rs`: `fav check --show-types` に fn inferred return type 行を追加
- `v43200_tests`: `cargo_toml_version_is_43_2_0` / `e0410_e0411_in_error_catalog` / `checker_fav_check_body_ty_has_e0410`

### Notes
- E0411 は本バージョンで catalog に追加のみ（checker.fav での検出は v43.3.0 以降）
- v43100_tests::cargo_toml_version_is_43_1_0 をスタブ化
```

---

## T7 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2906 passed; 0 failed`

---

## T8 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.2.0 最新安定版（2906 tests）、次版 v43.3.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.2.0 を `✅ COMPLETE（2026-07-12）`、推定 2898 → 実績 2906 に修正
- `versions/v40-v45/v43.2.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
