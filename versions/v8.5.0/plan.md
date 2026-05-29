# Favnir v8.5.0 実装計画

Date: 2026-05-30

---

## Phase A: `cmd_run` の dispatch 化

**変更ファイル**: `src/driver.rs`

### A-1: `run_with_favnir_pipeline` 内部関数を追加

`cmd_run_self_hosted` のコアロジック（設定ロード除く）を抽出した純粋な実行関数:

```rust
fn run_with_favnir_pipeline(source_path: &str, db_url: Option<&str>) {
    let (source, errors, _) = check_single_file(source_path, false);
    if !errors.is_empty() {
        for e in &errors { eprintln!("{}", format_diagnostic(&source, e)); }
        process::exit(1);
    }
    let bytes = crate::compiler_fav_runner::compile_file_to_bytes(source_path)
        .unwrap_or_else(|e| { eprintln!("compiler.fav error: {}", e); process::exit(1); });
    let artifact = crate::backend::artifact::FvcArtifact::from_bytes(&bytes)
        .unwrap_or_else(|e| { eprintln!("artifact error: {:?}", e); process::exit(1); });
    exec_artifact_main_with_source(&artifact, db_url, Some(source_path))
        .unwrap_or_else(|msg| { eprintln!("{msg}"); process::exit(1); });
}
```

### A-2: `cmd_run` に `legacy: bool` 引数を追加し dispatch を実装

```rust
pub fn cmd_run(file: Option<&str>, db_url: Option<&str>, legacy: bool) {
    // ... 設定ロード（変更なし）...

    // ファイルをパースして Favnir pipeline が使えるか判定
    let (source_path, proj) = find_entry(file);
    let source = load_file(&source_path);
    let program = Parser::parse_str(&source, &source_path).unwrap_or_else(...);

    let use_favnir = !legacy && proj.is_none() && !has_rune_imports(&program);

    if use_favnir {
        run_with_favnir_pipeline(&source_path, db_url);
    } else {
        // 既存 Rust pipeline（load_and_check_program → build_artifact → exec）
        // ただし parse 済みなので二度パースしないよう整理
        run_with_rust_pipeline_parsed(program, source, &source_path, proj, db_url);
    }
}
```

注意: `load_and_check_program` は内部でも `find_entry` + parse を行う。二重パース回避のため、
Rust pipeline パスは parse 済みの `program` を受け取る内部関数 `run_with_rust_pipeline_parsed` に分離する。

### A-3: `cmd_run_self_hosted` を `cmd_run(file, db_url, legacy=false)` の thin wrapper に変更

設定ロード部分は `cmd_run` に統合されるため、`cmd_run_self_hosted` は不要になる。
後方互換のために `pub fn cmd_run_self_hosted` は残すが、内部は `cmd_run(file, db_url, false)` を呼ぶだけ。

---

## Phase B: main.rs のフラグ処理更新

**変更ファイル**: `src/main.rs`

- `cmd_run` のシグネチャ変更に合わせて import を更新
- `Some("run")` ブランチに `--legacy` フラグを追加
- `--self-host` はそのまま残す（legacy=false として扱う）
- dispatch:
  ```rust
  cmd_run(file, db_path.as_deref(), legacy);
  ```

---

## Phase C: 統合テスト

**変更ファイル**: `src/driver.rs`

- `run_default_favnir_path` — フラグなし `fav run` 相当（legacy=false）が Favnir pipeline を通る
  - `compile_file_to_bytes` を直接呼んで結果を検証する既存テストで充足
- `run_legacy_rust_path` — `legacy=true` が Rust pipeline を通る（既存の `exec_artifact_main` で検証）
- rune import 検出テスト — `has_rune_imports` が true のファイルが Rust pipeline にフォールバックする

---

## Phase D: 最終確認

1. `cargo test` — 全テスト通過
2. `fav run fav/tmp/hello.fav` 等で手動確認（任意）
3. tasks.md 完了・commit

---

## 考慮事項

### 二重パース回避

`cmd_run` は dispatch 判定のために一度 parse する。Favnir pipeline の場合は
`check_single_file` 内部でも parse される（二度 parse）。
パフォーマンス上の問題があれば `check_single_file_parsed(program, source, path)` を
将来作ることができるが、v8.5.0 では許容する。

### `ensure_no_partial_flw` の扱い

現在の `cmd_run` は `ensure_no_partial_flw(&run_program)` を呼んでいる。
Favnir pipeline パスではこのチェックをスキップする（compiler.fav は partial flow を考慮しない）。
これは既存の `cmd_run_self_hosted` でも同様。

### `cmd_run` の引数変更と既存テスト

`cmd_run(file, db_url)` → `cmd_run(file, db_url, legacy)` のシグネチャ変更により、
`cmd_run` を直接呼んでいる既存テストのコンパイルエラーが出る可能性がある。
`cargo build` → `cargo test` で確認し、必要な呼び出し側を `legacy=false` に更新する。
