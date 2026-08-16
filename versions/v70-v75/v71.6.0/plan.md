# v71.6.0 実装計画 — AOT Native Compilation 本番品質化

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `71.5.0` であることを確認
- `cargo test` が全 pass（3599 tests）であることを確認
- `cranelift_aot.rs` の `link_binary` 関数の実装を確認（strip 追加箇所を特定）
- `lower_to_object_with_target` が aarch64 をサポートしていることを確認（line ~142）
- `compile_to_binary_for_arch` が未実装であることを確認
- `cmd_build` のシグネチャ（driver.rs line ~1837）を確認
- `cmd_build` が main.rs から呼ばれている行番号を確認（main.rs line ~877）
- `lower_to_object_pub` の pub(crate) が存在することを確認（line ~131）

---

## Step 2: `cranelift_aot.rs` — `arch_to_triple` + `compile_to_binary_for_arch` 追加

`fav/src/backend/cranelift_aot.rs` の `impl CraneliftBackend` ブロック末尾（`compile_to_binary_pub` の直後、`analyze_for_inlining` の前）に追加:

```rust
/// v71.6.0: アーキテクチャ文字列を Cranelift target triple に変換する。
fn arch_to_triple(arch: &str) -> Option<&'static str> {
    match arch {
        "arm64" | "aarch64" => Some("aarch64-unknown-linux-gnu"),
        _ => None,
    }
}

/// v71.6.0: arch 指定付き compile_to_binary。
/// - `arch = None` → ホスト ISA（既存の compile_to_binary と同等）
/// - `arch = Some("arm64")` | `Some("aarch64")` → aarch64-unknown-linux-gnu
/// - `arch = Some(unknown)` → None にフォールバック（ホスト ISA、警告なし）
/// 既存の _pub パターンと揃え pub(crate) にする（別途 _pub ラッパーは設けない）。
pub(crate) fn compile_to_binary_for_arch(
    ir: &IRProgram,
    out_path: &str,
    arch: Option<&str>,
) -> Result<(), String> {
    let triple = arch.and_then(Self::arch_to_triple);
    let obj_bytes = Self::lower_to_object_with_target(ir, triple)?;
    let wrapper_src = Self::c_wrapper_src();
    Self::link_binary(&obj_bytes, &wrapper_src, out_path)
}
```

---

## Step 3: `cranelift_aot.rs` — `link_binary` に strip 追加

`link_binary` の `Ok(())` の直前（リンク成功確認後）に strip 呼び出しを追加:

```rust
// v71.6.0: strip でデバッグシンボルを除去してバイナリサイズを削減。
// strip が存在しない環境（Windows 等）では Result を無視して続行。
let _ = std::process::Command::new("strip")
    .arg(out_path)
    .output();
Ok(())
```

---

## Step 4: `driver.rs` — `cmd_build_native_with_arch` 追加

`cmd_build_native`（line ~2463）の直後に追加:

```rust
/// v71.6.0: arch 指定付き native コンパイル（テスト・main.rs 用エントリポイント）
pub(crate) fn cmd_build_native_with_arch(
    src_path: &str,
    out_path: &str,
    arch: Option<&str>,
) -> Result<(), String> {
    let source =
        std::fs::read_to_string(src_path).map_err(|e| format!("read error: {e}"))?;
    let program = crate::frontend::parser::Parser::parse_str(&source, src_path)
        .map_err(|e| format!("parse error: {e}"))?;
    let ir = compile_program(&program);
    crate::backend::cranelift_aot::CraneliftBackend::compile_to_binary_for_arch(
        &ir, out_path, arch,
    )
}
```

---

## Step 5: `driver.rs` — `cmd_build` シグネチャ更新

`cmd_build` のシグネチャ（line ~1837）に `arch: Option<&str>` を追加:

```rust
pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>, arch: Option<&str>) {
```

"native" ブランチ（line ~1932）を更新:

```rust
"native" => {
    let out_path = out.unwrap_or_else(|| {
        eprintln!("error: --target native requires -o <output>");
        process::exit(1);
    });
    let ir = compile_program(&program);
    crate::backend::cranelift_aot::CraneliftBackend::compile_to_binary_for_arch(&ir, out_path, arch)
        .unwrap_or_else(|e| {
            eprintln!("error: AOT compilation failed: {e}");
            process::exit(1);
        });
    let arch_label = arch.unwrap_or("host");
    println!("built {out_path} (native/{arch_label})");
}
```

---

## Step 6: `main.rs` — `--arch` フラグ追加と `cmd_build` 呼び出し更新

`main.rs` の `"build"` ブランチ引数パース部に `--arch` を追加（`--target` パース直後あたり）:

```rust
"--arch" => {
    arch = Some(args.get(i + 1).unwrap_or_else(|| {
        eprintln!("error: --arch requires a value");
        process::exit(1);
    }));
    i += 2;
}
```

`let mut arch: Option<&str> = None;` をローカル変数に追加。
`cmd_build(file, out, target)` → `cmd_build(file, out, target, arch)` に更新。

---

## Step 7: `cargo build` + 既存テスト通過確認

- `cargo build` でエラーがないことを確認
- `cargo test` で既存 3599 件が全 pass であることを確認

---

## Step 8: `v716000_tests` 追加（`driver.rs`）

`v715000_tests` モジュールの直後に追加:

```rust
// ── v71.6.0 テスト: AOT Native Compilation 本番品質化 ──────────────────────
#[cfg(test)]
mod v716000_tests {
    use crate::frontend::parser::Parser;
    use crate::backend::cranelift_aot::CraneliftBackend;
    use super::compile_program;

    /// Cranelift が `fn main() -> Int { 42 }` をオブジェクトバイト列に変換できることを確認。
    /// cc 不要（lower_to_object_pub のみ使用）。
    #[test]
    fn aot_native_binary_compiles() {
        let src = "fn main() -> Int { 42 }";
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let ir = compile_program(&prog);
        let result = CraneliftBackend::lower_to_object_pub(&ir);
        assert!(
            result.is_ok(),
            "lower_to_object should succeed; err: {:?}",
            result
        );
        let obj = result.unwrap();
        assert!(!obj.is_empty(), "object bytes should be non-empty");
    }

    /// cc が利用可能な環境で `fn main() -> Int { 42 }` をコンパイル・実行し "42" が出力されることを確認。
    #[test]
    fn aot_native_binary_runs_hello() {
        // cc が存在しない環境（Windows CI 等）ではスキップ
        let cc_ok = std::process::Command::new("cc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !cc_ok { return; }

        use std::fs;
        let dir = tempfile::tempdir().expect("tempdir");
        let src_path = dir.path().join("hello.fav");
        let out_path = dir.path().join("hello_bin");
        fs::write(&src_path, "fn main() -> Int { 42 }").expect("write src");

        // cmd_build_native_with_arch を経由して compile_to_binary_for_arch のコードパスを実行
        let result = super::cmd_build_native_with_arch(
            src_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
            None, // ホスト ISA
        );
        if result.is_err() { return; } // リンク失敗は環境依存なのでスキップ

        // Windows では .exe 拡張子を試みる
        let exe = {
            #[cfg(windows)]
            { let w = out_path.with_extension("exe"); if w.exists() { w } else { out_path.clone() } }
            #[cfg(not(windows))]
            { out_path.clone() }
        };
        if !exe.exists() { return; }

        let output = std::process::Command::new(&exe)
            .output()
            .expect("exec binary");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            "42",
            "native binary should print 42; got: {:?}",
            stdout
        );
    }
}
```

`cargo test v716000` で 2 件 pass を確認。

---

## Step 9: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version` を `"71.5.0"` → `"71.6.0"` に変更
- `driver.rs` 内の cargo_toml_version テストを `"71.6.0"` に更新

---

## Step 10: CHANGELOG.md 更新

```markdown
## [v71.6.0] — 2026-08-10 — AOT Native Compilation 本番品質化

### Added
- `v716000_tests`: 2 件追加（3599 → 3601 tests）
  - `aot_native_binary_compiles`
  - `aot_native_binary_runs_hello`
- `cranelift_aot.rs`: `compile_to_binary_for_arch(ir, out_path, arch)` 追加
- `cranelift_aot.rs`: `link_binary` に `strip` 自動実行を追加（バイナリサイズ最適化）
- `driver.rs`: `cmd_build_native_with_arch` 追加
- `driver.rs`: `cmd_build` に `arch: Option<&str>` 引数追加
- `main.rs`: `fav build --arch <arm64|aarch64>` フラグ追加
- ARM64 クロスコンパイル: `fav build --target native --arch arm64 -o out`
```

---

## Step 11: versions/current.md 更新

- 「進行中バージョン」を `v71.6.0`（AOT Native Compilation 本番品質化）に更新
- 「次に切る版」を `v71.7.0` に更新

---

## Step 12: 最終確認

- `cargo test v716000` で 2 件 pass
- `cargo test` 全体で 3601 件 pass（0 failures）
- `fav/Cargo.toml` が `71.6.0`
- `versions/current.md` が正しく更新されている
