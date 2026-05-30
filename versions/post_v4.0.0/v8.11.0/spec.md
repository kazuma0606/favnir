# Favnir v8.11.0 Spec

Date: 2026-05-30
Theme: fav.toml プロジェクトモードを Favnir pipeline 化（v9.0.0 への最終ステップ）

---

## 背景

v8.10.0 時点で `fav run` は「単一ファイル」「rune import あり」どちらも Favnir pipeline で動作するが、
**fav.toml プロジェクトモード**（`proj.is_some()`）のみ Rust pipeline にフォールバックする。

v8.11.0 でこれを Favnir pipeline 化し、v9.0.0「セルフホスト完成宣言」の前提条件を満たす。

---

## fav.toml プロジェクトの構造

```
my-project/
├── fav.toml          ← プロジェクト設定
├── src/
│   ├── main.fav      ← エントリポイント（public fn main）
│   └── utils.fav     ← ローカルモジュール（import "utils" で参照）
└── rune_modules/     ← rune 依存（import rune "..." で参照）
    └── mymath/
```

### import の種類

| 構文 | 解決先 |
|---|---|
| `import "utils"` | `src/utils.fav`（ローカルモジュール） |
| `import rune "mymath"` | `rune_modules/mymath/`（rune モジュール） |

---

## 設計

### 基本方針

v8.6.0 と同じ「ソース結合 → Favnir pipeline」方式を拡張:

1. **Rust 側**: `collect_project_sources` で全ソースを再帰収集・結合
   - `import "name"` → `src/<name>.fav` を再帰
   - `import rune "name"` → `rune_modules/<name>/` を `collect_merged_sources` に委譲
   - `import "..."` / `namespace ...` 行を strip してから結合
2. **Favnir 側**: `compile_bytes_from_src(merged)` を呼ぶ（変更なし）

### 型チェックの扱い

現状の `run_with_favnir_pipeline` は `check_single_file(path)` でファイルパスを使う。
プロジェクトモードでは複数ファイルを結合した文字列を検査する必要があるため、
`check_source_str(src: &str)` ヘルパーを追加してソース文字列から直接チェックする。

---

## 変更ファイル

### `fav/src/compiler_fav_runner.rs`

#### 1. `compile_src_str_to_bytes(merged: &str) -> Result<Vec<u8>, String>`（新規）

`compile_file_to_bytes_rune` の compile 部分を共通化したヘルパー:

```rust
pub fn compile_src_str_to_bytes(merged: &str) -> Result<Vec<u8>, String> {
    let artifact = get_compiler_fav_artifact();
    let fn_idx = artifact.fn_idx_by_name("compile_bytes_from_src")
        .ok_or("compile_bytes_from_src not found")?;
    let result = VM::run(&artifact, fn_idx, vec![Value::Str(merged.to_string())])
        .map_err(|e| format!("compiler_fav VM error: {e:?}"))?;
    // Result<List<Int>, String> → Vec<u8>（既存パターン）
    ...
}
```

`compile_file_to_bytes_rune` はこれを使うようリファクタリング。

#### 2. `collect_project_sources(...)` （新規）

```rust
fn collect_project_sources(
    path: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
    visited: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let canon = std::path::Path::new(path).canonicalize()
        .map_err(|e| format!("cannot canonicalize {path}: {e}"))?
        .to_string_lossy().to_string();
    if !visited.insert(canon) { return Ok(()); }

    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {path}: {e}"))?;
    let program = Parser::parse_str(&src, path)
        .map_err(|e| format!("parse error in {path}: {e:?}"))?;

    for item in &program.items {
        match item {
            // ローカルモジュール: import "name" → src/<name>.fav
            Item::ImportDecl { path: name, is_rune: false, .. } => {
                let dep = toml.src_dir(root).join(format!("{}.fav", name.as_str()));
                collect_project_sources(&dep.to_string_lossy(), root, toml, visited, out)?;
            }
            // rune モジュール: import rune "name" → rune_modules/<name>/
            Item::ImportDecl { path: name, is_rune: true, .. } => {
                let rune_dir = root.join("rune_modules").join(name.as_str());
                if rune_dir.is_dir() {
                    let entry = crate::toml::rune_entry_file(&rune_dir, name);
                    collect_merged_sources(&entry.to_string_lossy(), visited, out)?;
                }
            }
            _ => {}
        }
    }

    // import / namespace 行を strip して追加
    let stripped: String = src.lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("import ") && !t.starts_with("namespace ")
        })
        .collect::<Vec<_>>().join("\n");
    out.push(stripped);
    Ok(())
}
```

#### 3. `compile_project_to_bytes(entry, root, toml) -> Result<Vec<u8>, String>`（新規）

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

### `fav/src/driver.rs`

#### 1. `check_source_str(src: &str) -> Vec<TypeErrorMsg>`（新規内部ヘルパー）

```rust
fn check_source_str(src: &str) -> Vec<crate::checker_fav_runner::TypeErrorMsg> {
    use crate::{frontend::parser::Parser, middle::ast_lower_checker::lower_program,
                checker_fav_runner::{run_checker_fav, msgs_to_type_errors}};
    let prog = match Parser::parse_str(src, "project.fav") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: parse error: {:?}", e);
            process::exit(1);
        }
    };
    let prog_vm = lower_program(&prog);
    match run_checker_fav(prog_vm) {
        Ok(()) => vec![],
        Err(msgs) => msgs_to_type_errors(msgs),
    }
}
```

#### 2. `run_with_favnir_pipeline_project(source_path, root, toml, db_url)`（新規）

```rust
fn run_with_favnir_pipeline_project(
    source_path: &str,
    root: &std::path::Path,
    toml: &crate::toml::FavToml,
    db_url: Option<&str>,
) {
    // 1. ソース収集・結合
    let mut visited = std::collections::HashSet::new();
    let mut sources: Vec<String> = Vec::new();
    crate::compiler_fav_runner::collect_project_sources_pub(
        source_path, root, toml, &mut visited, &mut sources
    ).unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let merged = sources.join("\n");

    // 2. 型チェック
    let errors = check_source_str(&merged);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}", crate::checker_fav_runner::format_type_error(e));
        }
        process::exit(1);
    }

    // 3. コンパイル（merged source → FVC bytes）
    let bytes = crate::compiler_fav_runner::compile_src_str_to_bytes(&merged)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });

    // 4. VM 実行（既存パターンと同じ）
    run_fvc_bytes(&bytes, db_url);
}
```

#### 3. `cmd_run` の dispatch 変更

```rust
// Before (v8.10.0):
let use_favnir = !legacy && proj.is_none();

// After (v8.11.0):
let use_favnir = !legacy;

if use_favnir {
    if let Some((ref toml, ref root)) = proj {
        run_with_favnir_pipeline_project(&source_path, root, toml, db_url);
    } else {
        run_with_favnir_pipeline(&source_path, db_url);
    }
} else {
    // Rust pipeline (--legacy)
    ...
}
```

---

## テスト

### `dispatch_project_uses_favnir_pipeline`

```rust
#[test]
fn dispatch_project_uses_favnir_pipeline() {
    // tempdir に fav.toml + src/main.fav + src/utils.fav を作成
    // fav.toml:  [package]\nname = "test"\n
    // utils.fav: fn add(a: Int, b: Int) -> Int { a + b }
    // main.fav:  import "utils"\npublic fn main() -> Int { add(40, 2) }

    // compile_project_to_bytes → VM 実行 → 42
}
```

---

## 注意事項

### `run_fvc_bytes` の共通化

`run_with_favnir_pipeline` と `run_with_favnir_pipeline_project` は、
バイトコード実行部分（FvcArtifact::from_bytes → VM::run → 戻り値表示）を共有する。
この部分を `run_fvc_bytes(bytes, db_url)` として切り出す。

### `collect_project_sources` の公開範囲

`compiler_fav_runner.rs` 内の private 関数 `collect_project_sources` は、
`driver.rs` 側の `run_with_favnir_pipeline_project` から呼ぶ必要があるため、
`pub fn collect_project_sources_pub(...)` として公開するか、
あるいは `compile_project_to_bytes` を public にして `driver.rs` 側は
それだけ呼ぶ設計にする（後者が望ましい）。

### `check_source_str` と `check_single_file` の関係

`check_single_file(path, false)` はファイルパスから source を読んでチェックする。
`check_source_str(src)` は結合済みソース文字列から直接チェックする。
どちらも checker.fav を使うため、整合性は保たれる。

### 既存の `compile_file_to_bytes_rune` との関係

`compile_file_to_bytes_rune` は `collect_merged_sources`（rune only）を使う。
`compile_project_to_bytes` は `collect_project_sources`（rune + local import）を使う。
両者は compile 部分で共通の `compile_src_str_to_bytes` を使う。
