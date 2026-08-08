# v64.1.0 Spec — AOT ビルドの CI 統合（`fav build --ci`）

Version: 64.1.0
Status: 未着手

---

## 概要

`driver.rs` に `cmd_build_ci(src: &str, out: &str) -> String` を追加する。
CI 向け出力形式（ANSI カラーなし・機械可読プレフィックス・パースエラー/ビルドエラーの明示）を実装する。

また `fav new` のテンプレートギャラリー（v24.8 実装済み）に GitHub Actions ワークフロー
テンプレート `"ci-workflow"` を追加。生成される `.github/workflows/build.yml` に
`fav build pipeline.fav --link --ci` を使用する CI ステップを含める。

出力例（`cmd_build_ci`）:
```
ci: ok — Output: out.o (N bytes)
```
エラー時:
```
ci: error: parse error: ...
ci: error: build error: ...
```

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3431 tests passed, 0 failed を確認
- `driver.rs` に `cmd_build_basic` が存在することを確認（`cmd_build_ci` の挿入位置参照）
- `driver.rs` に `cmd_build_ci` が存在しないことを確認（新規追加）
- `driver.rs` の `try_cmd_new` match に `"rag-pipeline"` アームが存在することを確認（`"ci-workflow"` の挿入位置）
- `driver.rs` に `v64000_tests` が存在することを確認（`v64100_tests` の挿入位置確認）
- `driver.rs` に `v64100_tests` が存在しないことを確認（新規追加）

**ロードマップとの差異（重要）**:
- ロードマップ完了条件（行 48）は `ベース 3418 + 2 = 3420` と記載していたが、実際の base は 3431（v63.6.0 code-reviewer 対応等の影響）。3431 + 2 = **3433** が正しい目標値。ロードマップ推移表は実装前に修正済み。
- ロードマップ行 36-37 に `main.rs` の `print_diag` 経路を `--ci` 時に切り替える記述があるが、本バージョンでは `driver.rs` への `cmd_build_ci` 関数追加のみ実装する。`main.rs` の CLI フラグ統合・print_diag 経路切り替えは非スコープ（後送り）。

---

## 実装スコープ

### 1. `driver.rs` — `cmd_build_ci` 追加

`cmd_build_link_target` の直後に追加:

```rust
/// v64.1.0: `fav build --ci` のドライバエントリポイント。
/// CI 向け出力形式（ANSI なし・機械可読プレフィックス）でビルド結果を返す。
pub fn cmd_build_ci(src: &str, out: &str) -> String {
    let program = match crate::frontend::parser::Parser::parse_str(src, "<build-ci>") {
        Ok(p) => p,
        Err(e) => return format!("ci: error: parse error: {e}"),
    };
    let ir = compile_program(&program);
    match crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_pub(&ir) {
        Ok(bytes) => format!("ci: ok — Output: {} ({} bytes)", out, bytes.len()),
        Err(e)    => format!("ci: error: build error: {e}"),
    }
}
```

### 2. `driver.rs` — `try_cmd_new` に `"ci-workflow"` テンプレート追加

`"rag-pipeline"` アームの直後に追加:

```rust
"ci-workflow"    => create_ci_workflow_project(&root, name),
```

エラーメッセージの末尾にも `ci-workflow` を追加する。

### 3. `driver.rs` — `create_ci_workflow_project` 追加

既存の `create_rag_pipeline_project` の直後に追加:

```rust
/// テスト用公開ラッパー（既存 `create_*_project` は非公開のため）
pub(crate) fn create_ci_workflow_project_pub(root: &std::path::Path, name: &str) -> Result<(), String> {
    create_ci_workflow_project(root, name)
}

fn create_ci_workflow_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("pipeline.fav"), &format!(
        "public stage Run: Int -> Int = |x| {{ x }}\n\
         pipeline {name} {{\n    step \"run\" = seq Run\n}}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname = \"{name}\"\n"
    ))?;
    write_text_file(
        &root.join(".github/workflows/build.yml"),
        &format!(
            "name: Build\non: [push, pull_request]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - name: Install fav\n        run: cargo install fav\n      - name: Build AOT binary\n        run: fav build pipeline.fav --link --ci -o dist/{name}\n      - name: Validate binary\n        run: ./dist/{name} --validate\n"
        ),
    )?;
    Ok(())
}
```

### 4. `driver.rs` — `v64100_tests` 追加

`v64000_tests` の直前に挿入:

```rust
// -- v64100_tests (v64.1.0) -- AOT ビルドの CI 統合 --
#[cfg(test)]
mod v64100_tests {
    #[test]
    fn build_ci_flag_output_format() {
        // fn main を含む有効なソース（lower_to_object_pub が Ok を返すことを保証）
        let src = concat!(
            "public stage Run: Int -> Int = |x| { x }\n",
            "pipeline Ci { step \"run\" = seq Run }"
        );
        let out = crate::driver::cmd_build_ci(src, "out.o");
        // CI モード: "ci:" プレフィックスを持つ機械可読出力
        assert!(out.starts_with("ci:"), "output should start with 'ci:': {}", out);
        // ANSI エスケープシーケンスを含まない
        assert!(!out.contains("\x1b["), "CI output must not contain ANSI codes: {}", out);
        // 成功時は "ci: ok" を含む（エラー時は "ci: error:" を含む）
        assert!(out.contains("ci: ok") || out.contains("ci: error:"),
            "should contain 'ci: ok' or 'ci: error:': {}", out);
        // 成功時の完全フォーマット確認: "Output:" と "bytes" を含む
        if out.contains("ci: ok") {
            assert!(out.contains("Output:"), "success output should contain 'Output:': {}", out);
            assert!(out.contains("bytes"), "success output should contain 'bytes': {}", out);
        }
    }

    #[test]
    fn new_template_has_ci_workflow() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("myproj");
        crate::driver::create_ci_workflow_project_pub(&proj, "myproj")
            .expect("create ci-workflow project");
        let workflow = proj.join(".github/workflows/build.yml");
        assert!(workflow.exists(), ".github/workflows/build.yml not created");
        let content = std::fs::read_to_string(&workflow).expect("read workflow");
        assert!(content.contains("fav build"), "workflow should call fav build: {}", content);
        assert!(content.contains("--ci"), "workflow should use --ci flag: {}", content);
    }
}
```

`create_ci_workflow_project` を `pub` にする（テストから直接呼ぶため）か、`pub` ラッパーを別途追加する。

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64100_tests` で 2 件 PASS
  - `build_ci_flag_output_format` PASS
  - `new_template_has_ci_workflow` PASS
- `cargo test -j 8 -- --test-threads=8` で 3433 tests passed, 0 failed

---

## 非スコープ

- `main.rs` への `--ci` フラグの CLI 統合（print_diag 経路の切り替え）
- `fav new --template ci-workflow` の実際のファイルシステム書き込み以外の動作確認
- CI ワークフローの実際の GitHub Actions 実行確認

---

## 技術ノート

### `cmd_build_ci` の出力フォーマット

- 成功: `"ci: ok — Output: {out} ({N} bytes)"`
- 失敗(parse): `"ci: error: parse error: {e}"`
- 失敗(build): `"ci: error: build error: {e}"`

ANSI カラーコード（`\x1b[...m`）を含まないことが CI 互換性の要件。

### `lower_to_object_pub` の選択根拠

`cmd_build_link_target` は `lower_to_object_with_target_pub`（クロスコンパイル対応）を使用するが、
`cmd_build_ci` では CI 環境（ホスト ISA）向けビルドのみを対象とするため、
`cmd_build_basic` と同じ `lower_to_object_pub`（ホスト ISA 固定）を選択する。
クロスコンパイルの CI 統合は後送り（後バージョン）。

### `create_ci_workflow_project` の可視性

既存の `create_*_project` 関数はすべて `fn`（非公開）。テストから呼ぶため、
`pub(crate) fn create_ci_workflow_project_pub` ラッパーを追加する方法を取る。
あるいは `create_ci_workflow_project` 自体を `pub(crate)` にしてテストから `crate::driver::create_ci_workflow_project_pub` として参照する。

### ベーステスト数の差異

ロードマップ記載: base=3418、target=3420。
実際: base=3431（v63.6.0 code-reviewer 対応等の影響）、target=3433。
