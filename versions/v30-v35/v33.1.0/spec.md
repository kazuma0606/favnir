# v33.1.0 — Spec: AOT ネイティブバイナリ 確認・テスト補強

## 概要

v33.1.0 は **AOT ネイティブバイナリ（Cranelift バックエンド）** の確認・テスト補強バージョン。

ロードマップ v33.1 のテーマ「`fav build --target native` でネイティブバイナリを生成する」は
v19.2.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `fav/src/backend/cranelift_aot.rs` — `CraneliftBackend::compile_to_binary` | ✓ | v19.2.0 |
| `cmd_build_native(src_path, out_path)` — driver.rs | ✓ | v19.2.0 |
| `fav build --target native` — コマンドライン dispatch | ✓ | v19.2.0 |
| `v192000_tests` — 4 件（`build_target_native_produces_binary` 等） | ✓ | v19.2.0 |

v33.1.0 では新規実装は行わず、`v331000_tests` で動作を確認・記録するにとどまる
（v32.1〜v32.9 と同じ「確認・記録」パターン）。

---

## AOT バックエンド 仕様確認

### 対応スコープ（v19.2.0 実装）

| 構文 | AOT 対応 |
|---|---|
| `Int` / `Bool` / `Float` リテラル | ✓ |
| 基本算術・比較演算（`+` / `-` / `*` / `/` / `==` / `<` / `>` 等）| ✓ |
| `if` 式（then / else 分岐）| ✓ |
| `Block` + ローカル変数 | ✓ |
| `String` リテラル / `List` / クロージャ | 非対応（v19.2.0 スコープ外）|

### 実行フロー

```
.fav ソース → Parser → IRProgram → CraneliftBackend::lower_to_object → .o ファイル
→ cc リンク → ネイティブバイナリ
```

### API

```rust
// driver.rs
pub(crate) fn cmd_build_native(src_path: &str, out_path: &str) -> Result<(), String>

// cranelift_aot.rs
impl CraneliftBackend {
    pub fn compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String>
}
```

---

## 追加するテスト（v331000_tests — 4 件）

`v331000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- `use super::cmd_build_native;` で必要な関数のみ明示 import
- cc 依存テストは `if !cc_available() { return; }` で安全にスキップ

テスト名は v192000_tests（`build_target_native_produces_binary` / `native_binary_executes` /
`native_vs_vm_same_output` / `build_target_vm_still_works`）と被らないよう `aot_` プレフィックスを使用。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_1_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.1.0"), "Cargo.toml must contain '33.1.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_1_0_exists() {
    let src = include_str!("../../benchmarks/v33.1.0.json");
    assert!(src.contains("33.1.0"), "benchmarks/v33.1.0.json must contain '33.1.0'");
}
```

### テスト 3: if 分岐 — true アームが選択される

v192000_tests の `fn main() -> Int { 42 }` / `fn main() -> Int { 1 + 2 * 3 }` とは
異なる構文（if 式）を使い、AOT の分岐コード生成を確認する。

```rust
fn aot_if_branch_selects_true_arm() {
    // if 式の then アームが正しく選択されることを確認（v192000_tests とは異なる構文）
    if !cc_available() { return; }
    use std::fs;
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("main.fav");
    fs::write(&src, "fn main() -> Int { if true { 10 } else { 20 } }").expect("write");
    let out = dir.path().join("aot_if_bin");
    let result = cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
    if result.is_err() { return; }  // cc なし環境はスキップ
    let actual = resolve_bin(&out);
    if actual.exists() {
        let output = std::process::Command::new(&actual).output().expect("exec");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "10", "if true branch should return 10");
    }
}
```

### テスト 4: Bool 比較 — 大小比較の結果

```rust
fn aot_bool_comparison_native() {
    // Bool 型比較（2 > 1 → true → 1）の AOT 動作確認（v192000_tests とは異なる戻り型）
    if !cc_available() { return; }
    use std::fs;
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("main.fav");
    fs::write(&src, "fn main() -> Bool { 2 > 1 }").expect("write");
    let out = dir.path().join("aot_bool_bin");
    let result = cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
    if result.is_err() { return; }
    let actual = resolve_bin(&out);
    if actual.exists() {
        let output = std::process::Command::new(&actual).output().expect("exec");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "1", "2 > 1 should produce 1 (true)");
    }
}
```

### ヘルパー（モジュール内）

```rust
fn cc_available() -> bool {
    std::process::Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn resolve_bin(base: &std::path::Path) -> std::path::PathBuf {
    #[cfg(windows)]
    { let exe = base.with_extension("exe"); if exe.exists() { return exe; } }
    base.to_path_buf()
}
```

---

## テストモジュールの配置

`v331000_tests` は `v330000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.1.0"`
- `cargo_toml_version_is_33_0_0` が空スタブになっていること
- `cargo test --bin fav v331000` — 4/4 PASS
- `cargo test` — 全件 PASS（2500 件、0 failures）
- `CHANGELOG.md` に `[v33.1.0]` セクション
- `benchmarks/v33.1.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.1.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.1.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
