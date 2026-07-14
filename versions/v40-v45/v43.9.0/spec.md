# v43.9.0 仕様書 — `fav check --show-inference`

## 概要

ロードマップ: "全式に推論された型を注釈表示。型推論のデバッグ支援。"

### v43.9.0 スコープ

`fav check --show-inference` フラグを追加し、型チェック通過後に**関数レベルの型注釈**を表示する。

あわせて `driver.rs` line 3948–3949 の**二重パース TODO を解消**する:

```rust
// TODO: collect_binding_types と collect_fn_inferred_return_types が同一ファイルを二重パースしている。
//       将来は parse 済み program を共有して統合する（v43.9.0 --show-inference で対応予定）。
```

### スコープ外（→ 将来バージョン）

- **式レベルの型注釈**（全 `bind` 式・全 `ECall` 式への型表示）: checker.fav が型を返さないため非対応
- **`--show-types` との統合**: 既存 `--show-types` の二重パース解消は collect_inference_annotations が担うが、`--show-types` 出力フォーマット自体は変更しない

---

## 実装設計

### `collect_inference_annotations(src: &str, filename: &str) -> Vec<String>`（driver.rs 新規追加）

**単一パース**で関数シグネチャを収集する（`collect_binding_types` と `collect_fn_inferred_return_types` 間の二重パース TODO を解消）。`cmd_check` 全体では依然として `check_single_file` による別パスが残る（完全統合は将来課題）:

1. `Parser::parse_str(src, filename)` — 一度だけパース
2. `run_checker_fav(lower_program(&program))` — 型チェック実行
3. チェック通過の場合のみ、各 `FnDef` の `(name, params, return_ty)` を収集
4. 型表示用に `display_ty_inline(ty: &TypeExpr) -> String` をローカル定義（`fmt_type_expr_simple` は private なため）
5. 出力例: `"fn filter_positive(xs: List<Int>) -> List<Int>"`

```rust
fn display_ty_inline(ty: &crate::ast::TypeExpr) -> String {
    use crate::ast::TypeExpr;
    match ty {
        TypeExpr::Named(name, args, _) if args.is_empty() => name.clone(),
        TypeExpr::Named(name, args, _) => {
            let a: Vec<String> = args.iter().map(display_ty_inline).collect();
            format!("{}<{}>", name, a.join(", "))
        }
        _ => "?".to_string(),
    }
}
```

### main.rs 変更

`Some("check")` アームに `show_inference` フラグを追加:

```rust
let mut show_inference = false;
// ...
"--show-inference" => { show_inference = true; i += 1; }
// ...
cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report, show_effects, refresh_schemas, show_inference);
```

### cmd_check 変更（driver.rs）

- シグネチャ末尾に `show_inference: bool` を追加
- `show_inference` が `true` のとき、`collect_inference_annotations` を呼び出して出力:

```
fn filter_positive(xs: List<Int>) -> List<Int>
fn transform(xs: List<Int>) -> List<Int>
```

- 出力後の TODO コメント（line 3948–3949）を削除

---

## 事前確認（T0）

- `cargo test` → 2925 passed; 0 failed
- `Cargo.toml` version = `43.8.0`
- `driver.rs` に `v43900_tests` モジュールが存在しないこと
- `driver.rs` line 3948 付近に `--show-inference で対応予定` TODO コメントが存在すること

---

## テスト設計

### v43.8.0 との差分

v43.8.0 はバリデーションリリース（checker.fav 変更なし）だった。
v43.9.0 は driver.rs / main.rs への実装追加を含む**実装リリース**。

### `v43900_tests`（2 件）

#### `cargo_toml_version_is_43_9_0`

バージョン確認テスト（次バージョン bump 時にスタブ化）。

#### `show_inference_collects_fn_annotations`

```rust
// v43900_tests は driver.rs の内部モジュールのため、collect_inference_annotations を直接呼び出す
// （use crate::driver::... は不要）
let src = r#"
fn add(a: Int, b: Int) -> Int { a + b }
fn identity(x: Int) -> Int { x }
"#;
let annotations = collect_inference_annotations(src, "v43900_test.fav");
assert!(!annotations.is_empty(), "should produce annotations");
assert!(annotations.iter().any(|s| s.contains("add")), "add missing: {:?}", annotations);
assert!(annotations.iter().any(|s| s.contains("identity")), "identity missing: {:?}", annotations);
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `collect_inference_annotations` 追加 / `cmd_check` シグネチャ更新 / TODO 解消 / `v43900_tests` 追加 |
| `fav/src/main.rs` | `--show-inference` フラグ追加 / `cmd_check` 呼び出し更新 |
| `fav/Cargo.toml` | version 43.8.0 → 43.9.0 |
| `CHANGELOG.md` | v43.9.0 エントリ追加 |
| `versions/current.md` | v43.9.0 最新安定版に更新 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.9.0 を COMPLETE に更新 |

**`fav/self/checker.fav` は変更不要**。

---

## 完了条件

- `cargo test` 2927 tests passed, 0 failed（2925 + 2）
- `v43900_tests` 2 件 pass
- `collect_inference_annotations` が `fn add` / `fn identity` を含む Vec を返す
- `fav check --show-inference src.fav` が関数レベル型注釈を表示する（手動確認）

---

## 影響範囲

- `cmd_check` シグネチャ変更（`show_inference: bool` 追加）→ 呼び出し元 main.rs も更新
- `cmd_check_all` は `show_inference` 非対応（単一ファイルモードのみ）
- **`file = None` 時**: `--show-inference` を指定しても何も出力しない（サイレント）。引数なし・`--all`/`--dir` モードでは `collect_inference_annotations` は呼ばれない
- **既知制限**: `display_ty_inline` は `Named` / `Named<args>` のみ対応。`Arrow`・`Optional` 等は `"?"` にフォールバック（将来拡張可）。フォールバック動作は今バージョンのテスト対象外（既知制限として記録）

---

## 前提条件

- v43.8.0 COMPLETE（2925 tests）
- `driver.rs` に `collect_binding_types` / `collect_fn_inferred_return_types` が存在（parse 一回ずつ）
- `fmt_type_expr_simple` は `fmt.rs` で private → `display_ty_inline` をインライン定義して回避
