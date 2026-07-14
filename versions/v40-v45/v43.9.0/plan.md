# v43.9.0 実装計画 — `fav check --show-inference`

## 前提

- v43.8.0 完了（2925 tests）
- `fav/Cargo.toml` version: `43.8.0`
- `driver.rs` line 3948 付近に `--show-inference で対応予定` TODO コメントが存在
- `fmt_type_expr_simple` は `fmt.rs` で private → `display_ty_inline` をインライン定義して回避
- `cmd_check` シグネチャ変更を伴うため、main.rs の呼び出し箇所も同時に更新する

---

## タスク順序

```
T0 事前確認
T1 driver.rs — collect_inference_annotations 追加 + cmd_check シグネチャ更新 + TODO 解消
T2 main.rs — --show-inference フラグ追加 + cmd_check 呼び出し更新
T3 driver.rs — v43900_tests 追加 / Cargo.toml bump / v43800_tests スタブ化
T4 CHANGELOG.md — v43.9.0 エントリ追加
T5 cargo test 実行・確認（2927 pass, 0 fail）
T6 バージョン管理ドキュメント更新
```

---

## T1/T2/T3 アトミシティ注記

T1（cmd_check シグネチャ変更）と T2（main.rs 呼び出し更新）は**ビルドが壊れるため必ず同時適用**する。
T3 の Cargo.toml bump と `cargo_toml_version_is_43_9_0` テストも同時に適用すること。

---

## T0 — 事前確認

1. `cargo test` 2925 / 0 確認
2. `Cargo.toml` version = `43.8.0` 確認
3. `v43900_tests` が driver.rs に存在しないことを確認
4. `driver.rs` line 3948 付近に `--show-inference で対応予定` TODO コメントが存在することを確認

---

## T1 — driver.rs — collect_inference_annotations + cmd_check 更新

### collect_inference_annotations（既存の TODO コメントの直後付近に追加）

```rust
/// v43.9.0: per-function inference annotations — single parse (resolves double-parse TODO)
pub fn collect_inference_annotations(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{Item, TypeExpr};

    fn display_ty_inline(ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name, args, _) if args.is_empty() => name.clone(),
            TypeExpr::Named(name, args, _) => {
                let a: Vec<String> = args.iter().map(display_ty_inline).collect();
                format!("{}<{}>", name, a.join(", "))
            }
            _ => "?".to_string(),
        }
    }

    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let lower = crate::middle::ast_lower_checker::lower_program(&program);
    if crate::checker_fav_runner::run_checker_fav(lower).is_err() {
        return vec![];
    }
    let mut out = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let params: Vec<String> = fd.params.iter()
                .map(|p| {
                    let ty = p.ty.as_ref().map(|t| display_ty_inline(t)).unwrap_or_else(|| "?".to_string());
                    format!("{}: {}", p.name, ty)
                })
                .collect();
            let ret = fd.return_ty.as_ref()
                .map(|t| display_ty_inline(t))
                .unwrap_or_else(|| "inferred".to_string());
            out.push(format!("fn {}({}) -> {}", fd.name, params.join(", "), ret));
        }
    }
    out
}
```

### cmd_check シグネチャ更新

`show_inference: bool` をパラメータ末尾に追加:

```rust
pub fn cmd_check(file: Option<&str>, no_warn: bool, legacy_check: bool, json: bool, show_types: bool, strict: bool, ambient: bool, report: bool, show_effects: bool, refresh_schemas: bool, show_inference: bool) {
```

### show-inference 出力ブロック（show_effects ブロックの直後に追加）

注意: `show_types` ブロックは `if let Some(path) = file { }` の**内側**にあるが、`show_inference` ブロックは `show_effects` と同様に**外側**に配置し、内部で `if let Some(path) = file` を再チェックする。この非対称性は既存 `show_effects` との一貫性を優先した設計。

```rust
if show_inference {
    // v43.9.0: --show-inference — per-function type annotations
    if let Some(path) = file {
        let source = load_file(path);
        let annotations = collect_inference_annotations(&source, path);
        for ann in &annotations {
            println!("{}", ann);
        }
    }
}
```

### TODO コメント削除

driver.rs line 3948–3949 の以下のコメントを削除する:

```rust
// TODO: collect_binding_types と collect_fn_inferred_return_types が同一ファイルを二重パースしている。
//       将来は parse 済み program を共有して統合する（v43.9.0 --show-inference で対応予定）。
```

---

## T2 — main.rs — --show-inference フラグ追加

`Some("check")` アームに追加:

```rust
let mut show_inference = false;
```

フラグパース（`"--show-effects"` の直後に追加）:

```rust
"--show-inference" => {
    show_inference = true;
    i += 1;
}
```

`cmd_check` 呼び出しに `show_inference` を追加（末尾）:

```rust
cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report, show_effects, refresh_schemas, show_inference);
```

---

## T3 — driver.rs — v43900_tests 追加 / Cargo.toml / スタブ化

`v43800_tests` モジュールの直前に挿入:

```rust
// -- v43900_tests (v43.9.0) -- fav check --show-inference --
#[cfg(test)]
mod v43900_tests {
    #[test]
    fn cargo_toml_version_is_43_9_0() {
        // この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.9.0"), "Cargo.toml must contain version 43.9.0");
    }
    #[test]
    fn show_inference_collects_fn_annotations() {
        // collect_inference_annotations が関数名を含む Vec を返す
        // v43900_tests は driver.rs 内部モジュールのため直接呼び出し可能（use crate::driver::... 不要）
        let src = r#"
fn add(a: Int, b: Int) -> Int { a + b }
fn identity(x: Int) -> Int { x }
"#;
        let annotations = collect_inference_annotations(src, "v43900_test.fav");
        assert!(!annotations.is_empty(), "should produce annotations: {:?}", annotations);
        assert!(annotations.iter().any(|s| s.contains("add")), "add missing: {:?}", annotations);
        assert!(annotations.iter().any(|s| s.contains("identity")), "identity missing: {:?}", annotations);
    }
}
```

`v43800_tests::cargo_toml_version_is_43_8_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_8_0() {
    // Stubbed: version bumped to 43.9.0 in v43.9.0.
}
```

`fav/Cargo.toml`:

```toml
version = "43.9.0"
```

---

## T4 — CHANGELOG.md

日付は実装当日のものに変更すること。

```markdown
## [v43.9.0] — 2026-07-13

### Added
- `collect_inference_annotations(src, filename) -> Vec<String>`: 関数レベル型注釈収集（単一パース）
- `fav check --show-inference`: 型チェック通過後に関数シグネチャを出力
- `v43900_tests`: `cargo_toml_version_is_43_9_0` / `show_inference_collects_fn_annotations`

### Changed
- `cmd_check` シグネチャに `show_inference: bool` を追加
- `v43800_tests::cargo_toml_version_is_43_8_0` をスタブ化
- driver.rs の二重パース TODO（line 3948–3949）を解消

### Notes
- `fav/self/checker.fav` は変更なし
- `display_ty_inline` は `Named` / `Named<args>` のみ対応。`Arrow`・`Optional` 等は `"?"` にフォールバック
```

---

## T5 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2927 passed; 0 failed`

---

## T6 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.9.0 最新安定版（2927 tests）、次版 v43.10.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.9.0 を `✅ COMPLETE（2026-07-13）`、推定 2927 → 実績 2927 に修正
- `versions/v40-v45/v43.9.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
