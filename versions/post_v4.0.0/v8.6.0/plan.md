# Favnir v8.6.0 実装計画

Date: 2026-05-30

---

## Phase A: compiler.fav に `compile_bytes_from_src` 追加

**変更ファイル**: `fav/self/compiler.fav`

既存の `compile_bytes` 関数の直後に追加:

```fav
// Compile from a pre-merged source string (no file I/O).
// Used by the rune-import-aware Rust runner to pass concatenated sources.
public fn compile_bytes_from_src(src: String) -> Result<List<Int>, String> {
    Result.and_then(lex(src), |toks|
    Result.and_then(parse_tokens(toks), |prog|
    Result.and_then(compile(prog), |artifact|
    Result.ok(serialize_artifact(artifact)))))
}
```

`compile_file_quiet` との違い: `IO.read_file_raw` を呼ばず `src` を直接 lex に渡す。

---

## Phase B: compiler_fav_runner.rs に `compile_file_to_bytes_rune` 追加

**変更ファイル**: `fav/src/compiler_fav_runner.rs`

### B-1: `collect_merged_sources` ヘルパー

```rust
fn collect_merged_sources(
    path: &str,
    visited: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let canon = std::path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("cannot resolve path `{}`: {}", path, e))?;
    let canon_str = canon.to_string_lossy().to_string();
    if visited.contains(&canon_str) {
        return Ok(());
    }
    visited.insert(canon_str);

    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read `{}`: {}", path, e))?;

    // Parse to discover rune imports
    let program = crate::frontend::parser::Parser::parse_str(&src, path)
        .map_err(|e| e.to_string())?;

    let source_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    // Recurse: process rune deps first (deps-before-user order)
    for item in &program.items {
        if let crate::frontend::ast::Item::ImportDecl {
            path: rune_name,
            is_rune: true,
            ..
        } = item
        {
            let rune_dir = source_dir.join("rune_modules").join(rune_name.as_str());
            if rune_dir.is_dir() {
                let entry = crate::toml::rune_entry_file(&rune_dir, rune_name);
                let entry_str = entry.to_string_lossy().to_string();
                collect_merged_sources(&entry_str, visited, out)?;
            }
        }
    }

    // Strip `import rune` and `namespace` lines from this file's source
    let stripped: String = src
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("import rune") && !t.starts_with("namespace ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    out.push(stripped);
    Ok(())
}
```

### B-2: `compile_file_to_bytes_rune`

```rust
/// Rune-import-aware variant of compile_file_to_bytes.
/// Collects all rune dependency sources, merges them, then calls
/// compiler.fav's compile_bytes_from_src.
pub fn compile_file_to_bytes_rune(path: &str) -> Result<Vec<u8>, String> {
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    collect_merged_sources(path, &mut visited, &mut sources)?;
    let merged = sources.join("\n");

    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("compile_bytes_from_src")
        .ok_or_else(|| "compile_bytes_from_src not found in compiler.fav".to_string())?;

    let result = crate::backend::vm::VM::run(artifact, fn_idx, vec![crate::value::Value::Str(merged)])
        .map_err(|e| format!("compiler.fav VM error: {}", e))?;

    match result {
        crate::value::Value::Variant(ref tag, Some(ref payload)) if tag == "ok" => {
            // List<Int> → Vec<u8>（既存の compile_file_to_bytes と同パターン）
            ...
        }
        crate::value::Value::Variant(ref tag, ref payload) if tag == "err" => {
            let msg = match payload.as_deref() {
                Some(crate::value::Value::Str(s)) => s.clone(),
                _ => "unknown compiler.fav error".to_string(),
            };
            Err(msg)
        }
        other => Err(format!("unexpected compiler.fav return: {:?}", other)),
    }
}
```

---

## Phase C: driver.rs の dispatch 条件変更

**変更ファイル**: `fav/src/driver.rs`

### C-1: `run_with_favnir_pipeline` を `compile_file_to_bytes_rune` に切替

```rust
fn run_with_favnir_pipeline(source_path: &str, db_url: Option<&str>) {
    let (source, errors, _) = check_single_file(source_path, false);
    if !errors.is_empty() {
        for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
        process::exit(1);
    }
    // v8.6.0: rune import 対応版
    let bytes = crate::compiler_fav_runner::compile_file_to_bytes_rune(source_path)
        .unwrap_or_else(|e| { eprintln!("compiler.fav error: {}", e); process::exit(1); });
    let artifact = crate::backend::artifact::FvcArtifact::from_bytes(&bytes)
        .unwrap_or_else(|e| { eprintln!("artifact error: {:?}", e); process::exit(1); });
    exec_artifact_main_with_source(&artifact, db_url, Some(source_path))
        .unwrap_or_else(|msg| { eprintln!("{msg}"); process::exit(1); });
}
```

### C-2: dispatch 条件から `!has_rune_imports` 除去

```rust
// Before:
let use_favnir = !legacy && proj.is_none() && !has_rune_imports(&program);

// After:
let use_favnir = !legacy && proj.is_none();
```

`has_rune_imports` 関数と v8.5.0 の `dispatch_rune_import_uses_rust_fallback` テストは
このバージョンで削除・更新する。

---

## Phase D: 統合テスト

**変更ファイル**: `fav/src/driver.rs`

### D-1: `dispatch_rune_import_uses_favnir_pipeline`

rune import を含むファイルを Favnir pipeline でコンパイル・実行できることを確認。

```rust
#[test]
fn dispatch_rune_import_uses_favnir_pipeline() {
    // Create a simple rune in a temp rune_modules/ directory
    let dir = tempfile::tempdir().unwrap();
    // rune_modules/mymath/mymath.fav
    let rune_dir = dir.path().join("rune_modules").join("mymath");
    std::fs::create_dir_all(&rune_dir).unwrap();
    std::fs::write(rune_dir.join("mymath.fav"),
        "public fn double(x: Int) -> Int { x * 2 }").unwrap();
    // main file
    let main_path = dir.path().join("main.fav");
    std::fs::write(&main_path,
        "import rune \"mymath\"\npublic fn main() -> Int { double(21) }").unwrap();

    let bytes = crate::compiler_fav_runner::compile_file_to_bytes_rune(
        main_path.to_str().unwrap()
    ).expect("should compile with rune import");
    let artifact = crate::backend::artifact::FvcArtifact::from_bytes(&bytes).unwrap();
    let fn_idx = artifact.fn_idx_by_name("main").unwrap();
    let result = crate::backend::vm::VM::run(&artifact, fn_idx, vec![]).unwrap();
    assert_eq!(result, crate::value::Value::Int(42));
}
```

### D-2: 既存テスト更新

- `dispatch_rune_import_uses_rust_fallback` → `dispatch_rune_import_detected` に改名
  （`has_rune_imports` が true を返すことのテストとして残すか、削除する）
- 既存 `run_self_hosted_tests` (7件) と `run_dispatch_tests` が引き続き通ること

---

## Phase E: 最終確認

- `cargo build` — コンパイルエラーなし
- `cargo test` — 1123 tests passing (+ 1 新規)
- tasks.md 完了・commit

---

## 実装ノート

### import rune 行の除去方法
行頭が `import rune` で始まる行を strip するだけで十分。
エイリアス付き (`import rune "sql" as Sql`) も同じルールで除去される。

### namespace 行の除去
複数ファイルを結合すると `namespace X` 宣言が複数出現し得る。
compiler.fav のパーサーは現在 namespace 宣言をどう扱うか確認が必要。
問題があれば除去する。

### checker.fav の rune import 検証
`check_single_file(path, false)` は現状 rune import を含むファイルも処理する
（checker.fav は import 宣言を無視する）。
rune の関数シグネチャが未知のため型エラーが出る可能性がある。
v8.6.0 ではチェックエラーが出た場合は Rust pipeline にフォールバックする
（`check_single_file` がエラーを返したら `run_with_favnir_pipeline` ではなく
Rust パスに切り替える）ことも検討。
→ まずシンプルに実装してテストし、型エラー問題があれば対処する。

### トランジティブ依存
v8.6.0 では 1 段の rune import のみ対応（rune が別の rune を import するケースは未対応）。
`collect_merged_sources` は再帰実装なので自然に多段も処理できる見込みだが、
テストは 1 段に限定する。
