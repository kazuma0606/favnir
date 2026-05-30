# Favnir v8.6.0 Spec

Date: 2026-05-30
Theme: `fav run` の rune import 対応（Favnir pipeline 制限解除）

---

## 背景

v8.5.0 で `fav run <file>` のデフォルトが Favnir pipeline になったが、
`import rune "..."` を含むファイルは Rust pipeline に自動フォールバックする制限が残った。

v8.6.0 では rune import を含む単一ファイルも Favnir pipeline でコンパイル・実行できるようにする。

```
現状 (v8.5.0):
  fav run single.fav          → Favnir pipeline (checker.fav + compiler.fav)
  fav run file_with_rune.fav  → Rust pipeline フォールバック ← 制限

目標 (v8.6.0):
  fav run single.fav          → Favnir pipeline (変更なし)
  fav run file_with_rune.fav  → Favnir pipeline ← NEW
  fav run --legacy <file>     → Rust pipeline（明示的退避、変更なし）
```

---

## 設計

### アーキテクチャ方針

rune import 解決（ファイルパス探索・読み込み・結合）は Rust 側で行い、
結合済みソース文字列を compiler.fav に渡してコンパイルする。

理由:
- `load_all_items` のパス解決ロジック（rune.toml 解析、rune_modules/ 探索）がすでに Rust に存在
- v8.6.0 の目標は「rune import ありでも Favnir pipeline が動く」であり、
  ファイル解決のFavnir実装はスコープ外（v9.0.0 以降）

### 変更概要

```
Rust (compiler_fav_runner.rs)           compiler.fav
────────────────────────────────        ─────────────────────────────
compile_file_to_bytes_rune(path)   →    compile_bytes_from_src(src)
  1. 依存ファイルを DFS で探索           lex(src)
  2. 各ソースを結合                   → parse_tokens
     (import rune 行は除去)           → compile
  3. compile_bytes_from_src を呼ぶ    → serialize_artifact
                                      → Result<List<Int>, String>
```

### (A) compiler.fav: `compile_bytes_from_src` 追加

```fav
// ソース文字列から直接コンパイルする（ファイル読み込みなし）。
// rune 結合済みソースを受け取る想定。
public fn compile_bytes_from_src(src: String) -> Result<List<Int>, String> {
    Result.and_then(lex(src), |toks|
    Result.and_then(parse_tokens(toks), |prog|
    Result.and_then(compile(prog), |artifact|
    Result.ok(serialize_artifact(artifact)))))
}
```

既存の `compile_file_quiet` との違い: `IO.read_file_raw` を呼ばず、
引数の `src` をそのまま使う。

### (B) compiler_fav_runner.rs: `compile_file_to_bytes_rune` 追加

```rust
/// compile_file_to_bytes の rune import 対応版。
/// エントリファイルと依存 rune ソースを結合してから compiler.fav でコンパイルする。
pub fn compile_file_to_bytes_rune(path: &str) -> Result<Vec<u8>, String> {
    // 1. 全依存ファイルを DFS 探索（standalone モード: rune_modules/<name>/）
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    collect_merged_sources(path, &mut visited, &mut sources)?;

    // 2. ソースを結合（import rune 行を除去して連結）
    let merged = sources.join("\n");

    // 3. compile_bytes_from_src を呼ぶ
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact.fn_idx_by_name("compile_bytes_from_src")?;
    let result = VM::run(&artifact, fn_idx, vec![Value::Str(merged)])?;
    // ... List<Int> → Vec<u8> 変換（既存パターン）
}

fn collect_merged_sources(
    path: &str,
    visited: &mut HashSet<String>,
    sources: &mut Vec<String>,
) -> Result<(), String> {
    // DFS: ImportDecl(is_rune=true) → rune_modules/<name>/<name>.fav を先に処理
    // import rune 行は除去してから sources に push
}
```

### (C) driver.rs: dispatch 条件の変更

```rust
// Before (v8.5.0):
let use_favnir = !legacy && proj.is_none() && !has_rune_imports(&program);

// After (v8.6.0):
let use_favnir = !legacy && proj.is_none();
// → rune import があっても Favnir pipeline を使う
// → fav.toml プロジェクトモードのみ Rust pipeline フォールバック（変更なし）
```

また `run_with_favnir_pipeline` 内で `compile_file_to_bytes_rune` に切り替える:

```rust
fn run_with_favnir_pipeline(source_path: &str, db_url: Option<&str>) {
    let (source, errors, _) = check_single_file(source_path, false);
    if !errors.is_empty() { /* print + exit */ }

    // rune import 対応版コンパイル（v8.6.0）
    let bytes = crate::compiler_fav_runner::compile_file_to_bytes_rune(source_path)
        .unwrap_or_else(...);
    let artifact = FvcArtifact::from_bytes(&bytes)...;
    exec_artifact_main_with_source(&artifact, db_url, Some(source_path))...;
}
```

---

## ソース結合ルール

`collect_merged_sources` は `load_all_items` のパス解決ロジックを simple に再実装:

1. Standalone モード（fav.toml なし）のみ対応
2. 対象: `import rune "name"` 宣言
3. 探索順: エントリファイルのディレクトリから `rune_modules/<name>/` を探す
4. エントリファイル: `rune.toml` の `entry` フィールド → なければ `<name>.fav`
5. 同一ファイルの重複ロードは visited セットで防ぐ
6. 結合ソースから除去する行: `import rune "..."` と `namespace ...`（namespace 重複を避けるため）

---

## スコープ外

- fav.toml プロジェクトモードの Favnir 化: v9.0.0 以降
- compiler.fav 自身による rune import 解決（IO primitives 使用）: 将来版
- rune import のトランジティブ解決（rune が別の rune を import）: v8.6.0 では 1 段のみ対応
