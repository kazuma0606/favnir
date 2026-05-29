# Favnir v8.5.0 Spec

Date: 2026-05-30
Theme: `fav run` のデフォルト Favnir 化

---

## 背景

v8.4.0 で `fav run --self-host` が型チェック・コンパイル共に完全 Favnir になった。
v8.5.0 ではこれを **フラグなしの `fav run` のデフォルト動作** にする。

```
現状 (v8.4.0):
  fav run <file>             → Rust pipeline（変更なし）
  fav run --self-host <file> → checker.fav + compiler.fav

目標 (v8.5.0):
  fav run <file>             → checker.fav + compiler.fav  ← NEW DEFAULT
  fav run --legacy <file>    → Rust pipeline（明示的退避）
  fav run --self-host <file> → 後方互換 alias（= --legacy なし = デフォルト）
```

---

## 制約条件

### Rune import / プロジェクトモード

`compiler.fav` は単一ファイルを読んでコンパイルする。
`rune import` を含むファイルは `load_all_items` による rune_modules/ の結合が必要であり、
compiler.fav はこれを行わない。

**自動検出・フォールバック**:
- `has_rune_imports(program)` が true → Rust pipeline（既存動作）
- `fav.toml` プロジェクトモード（`find_entry` が Some を返す）→ Rust pipeline
- それ以外（単一ファイル）→ Favnir pipeline（デフォルト）

ユーザーから見ると:
- 単純な `.fav` ファイルは自動で Favnir pipeline
- rune import やプロジェクト構成では Rust pipeline に自動フォールバック（透過的）
- `--legacy` で常に Rust pipeline を強制できる

---

## 設計

### `cmd_run` の dispatch ロジック

```rust
pub fn cmd_run(file: Option<&str>, db_url: Option<&str>, legacy: bool) {
    // ... 設定ロード（schemas, auth, log, env, aws）...

    let (source_path, proj) = find_entry(file);
    let source = load_file(&source_path);
    let program = Parser::parse_str(&source, &source_path)?;

    let use_favnir = !legacy
        && proj.is_none()
        && !has_rune_imports(&program);

    if use_favnir {
        // Favnir pipeline: checker.fav + compiler.fav
        run_with_favnir_pipeline(&source_path, &source, db_url)
    } else {
        // Rust pipeline: 既存の load_and_check_program + build_artifact
        run_with_rust_pipeline(file, db_url)
    }
}
```

### `run_with_favnir_pipeline`（内部関数）

```rust
fn run_with_favnir_pipeline(source_path: &str, source: &str, db_url: Option<&str>) {
    // 1. Type-check: checker.fav
    let (_, errors, _) = check_single_file(source_path, false);
    if !errors.is_empty() { /* print errors + exit */ }

    // 2. Compile: compiler.fav
    let bytes = compiler_fav_runner::compile_file_to_bytes(source_path)?;
    let artifact = FvcArtifact::from_bytes(&bytes)?;

    // 3. Execute: Rust VM
    exec_artifact_main_with_source(&artifact, db_url, Some(source_path))
}
```

### `cmd_run_self_hosted` の扱い

`cmd_run_self_hosted` は引き続き存在するが、`cmd_run(file, db_url, legacy=false)` の
単一ファイルパスと同等になる。`--self-host` フラグは後方互換のために残すが、
内部では `legacy=false` を指定した `cmd_run` と同じ動作をする。

---

## スコープ外

- compiler.fav の rune import 対応: v9.0.0 以降
- fav.toml プロジェクトモードの Favnir 化: 同上
- checker.fav の E0001/E0002 未検出問題の修正: v9.0.0 候補（infer_arg_tys 逆順バグ）
- パフォーマンス最適化: compiler.fav は Rust pipeline より遅い可能性がある。
  v8.5.0 では正確性を優先し、パフォーマンス計測は別途。
