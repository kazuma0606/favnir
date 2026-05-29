# Favnir v8.11.0 実装計画

Date: 2026-05-30

---

## Phase A: `compile_src_str_to_bytes` を抽出（compiler_fav_runner.rs）

`compile_file_to_bytes_rune` のコンパイル部分を共通ヘルパーに切り出す:

```rust
/// Compile a pre-merged source string via compiler.fav.
/// Returns raw FVC bytecode.
pub fn compile_src_str_to_bytes(merged: &str) -> Result<Vec<u8>, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact
        .fn_idx_by_name("compile_bytes_from_src")
        .ok_or_else(|| "compile_bytes_from_src not found in compiler.fav".to_string())?;
    let result = VM::run(&artifact, fn_idx, vec![Value::Str(merged.to_string())])
        .map_err(|e| format!("compiler_fav VM error: {e:?}"))?;
    // Result<List<Int>, String> → Vec<u8>（compile_file_to_bytes_rune と同じ変換）
    extract_bytes_from_result(result)
}
```

`compile_file_to_bytes_rune` は内部で `compile_src_str_to_bytes` を使うようリファクタリング。

---

## Phase B: `collect_project_sources` 追加（compiler_fav_runner.rs）

`collect_merged_sources`（rune only）を拡張したプロジェクト用版:

```rust
fn collect_project_sources(
    path: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
    visited: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    // canonicalize で重複防止
    // parse してインポートを解決
    // import "name" → src/<name>.fav（collect_project_sources 再帰）
    // import rune "name" → rune_modules/<name>/（collect_merged_sources 委譲）
    // strip して out.push
}
```

## Phase C: `compile_project_to_bytes` 追加（compiler_fav_runner.rs）

```rust
pub fn compile_project_to_bytes(
    entry: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
) -> Result<Vec<u8>, String> {
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    collect_project_sources(entry, root, toml, &mut visited, &mut sources)?;
    compile_src_str_to_bytes(&sources.join("\n"))
}
```

---

## Phase D: driver.rs 更新

### D-1: `run_fvc_bytes` を切り出し

`run_with_favnir_pipeline` のバイトコード実行部分（FvcArtifact::from_bytes → VM::run → 出力）を
`run_fvc_bytes(bytes: &[u8], db_url: Option<&str>)` として抽出。
これにより `run_with_favnir_pipeline` と `run_with_favnir_pipeline_project` が共有できる。

### D-2: `check_source_str` ヘルパー追加

```rust
fn check_source_str(src: &str) -> Vec<crate::checker_fav_runner::TypeErrorMsg> {
    let prog = Parser::parse_str(src, "project.fav").unwrap_or_else(|e| {
        eprintln!("error: {e:?}"); process::exit(1);
    });
    let prog_vm = lower_program(&prog);
    match run_checker_fav(prog_vm) {
        Ok(()) => vec![],
        Err(msgs) => msgs_to_type_errors(msgs),
    }
}
```

### D-3: `run_with_favnir_pipeline_project` 追加

```rust
fn run_with_favnir_pipeline_project(
    source_path: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
    db_url: Option<&str>,
) {
    // 1. compile_project_to_bytes でソース収集 + コンパイル
    //    (型チェックは compile_project_to_bytes 前に check_source_str で実施)
    let merged = collect_and_merge_project(source_path, root, toml);  // 共通化
    let errors = check_source_str(&merged);
    if !errors.is_empty() { print_errors_and_exit(&errors); }
    let bytes = compile_src_str_to_bytes(&merged).unwrap_or_else(...);
    run_fvc_bytes(&bytes, db_url);
}
```

### D-4: `cmd_run` の dispatch 変更

```rust
// Before:
let use_favnir = !legacy && proj.is_none();

// After:
let use_favnir = !legacy;

if use_favnir {
    match proj {
        Some((ref toml, ref root)) =>
            run_with_favnir_pipeline_project(&source_path, root, toml, db_url),
        None =>
            run_with_favnir_pipeline(&source_path, db_url),
    }
} else {
    // Rust pipeline (--legacy)
    ...
}
```

---

## Phase E: テスト追加（driver.rs）

### `dispatch_project_uses_favnir_pipeline`

```rust
#[test]
fn dispatch_project_uses_favnir_pipeline() {
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let root = dir.path();

    // fav.toml
    std::fs::write(root.join("fav.toml"), "[package]\nname = \"testproj\"\n").unwrap();

    // src/
    std::fs::create_dir(root.join("src")).unwrap();

    // src/utils.fav
    std::fs::write(
        root.join("src").join("utils.fav"),
        "fn add(a: Int, b: Int) -> Int { a + b }\n",
    ).unwrap();

    // src/main.fav
    std::fs::write(
        root.join("src").join("main.fav"),
        "import \"utils\"\npublic fn main() -> Int { add(40, 2) }\n",
    ).unwrap();

    let entry = root.join("src").join("main.fav").to_string_lossy().to_string();
    let toml = crate::toml::FavToml::load(root).unwrap();
    let bytes = crate::compiler_fav_runner::compile_project_to_bytes(&entry, root, &toml)
        .expect("compile_project_to_bytes failed");

    // VM 実行して結果が 42 か確認
    let artifact = crate::backend::artifact::FvcArtifact::from_bytes(&bytes).unwrap();
    let fn_idx = artifact.fn_idx_by_name("main").unwrap();
    let result = crate::backend::vm::VM::run(&artifact, fn_idx, vec![]).unwrap();
    assert_eq!(result, crate::value::Value::Int(42));
}
```

---

## Phase F: 確認・ドキュメント

```
cargo test dispatch_project
cargo test checker_fav         ← self-check 通過確認
cargo test                     ← 全件通ること（目標 1135+ tests）
```

tasks.md 完了・MEMORY.md 更新・commit。

---

## 実装上の注意

### `collect_merged_sources` との使い分け

| 関数 | 用途 | `import "name"` | `import rune` |
|---|---|---|---|
| `collect_merged_sources` | rune 専用（既存） | 非対応 | ✅ |
| `collect_project_sources` | プロジェクト用（新規） | ✅ `src/<name>.fav` | ✅ 委譲 |

### `check_single_file` vs `check_source_str`

| 関数 | 入力 | 用途 |
|---|---|---|
| `check_single_file(path, false)` | ファイルパス | 単一ファイル / rune import |
| `check_source_str(src)` | ソース文字列 | プロジェクト結合ソース |

どちらも checker.fav を使うので結果の意味は同じ。

### `FavToml::load` の引数

`FavToml::load` はプロジェクトルートを取る。テストでは `tempdir` を root として使う。

### `tempfile` クレートの確認

`compile_file_to_bytes_rune` のテスト (`dispatch_rune_import_uses_favnir_pipeline`) で
既に `tempdir` を使っているため、依存追加不要。
