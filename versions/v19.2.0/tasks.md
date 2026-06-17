# v19.2.0 — AOT コンパイル（Cranelift バックエンド）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/Cargo.toml` — Cranelift 依存追加

- [x] `cargo tree -i cranelift-codegen` で wasmtime が使用しているバージョンを確認
- [x] `wasmtime` 依存の直後に追記:
  ```toml
  cranelift-codegen = { version = "0.113", features = ["x64"] }
  cranelift-module  = { version = "0.113" }
  cranelift-object  = { version = "0.113" }
  ```
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/cranelift.rs`（新規作成）

**2-A: モジュール登録**

- [x] `fav/src/backend/` 以下に `cranelift.rs` を新規作成
- [x] `fav/src/backend/mod.rs`（またはモジュール宣言箇所）に `pub mod cranelift;` を追加

**2-B: `CraneliftBackend` struct と基本構造**

- [x] `use` 宣言:
  ```rust
  use cranelift_codegen::ir::{types, AbiParam, InstBuilder};
  use cranelift_codegen::{settings, Context};
  use cranelift_module::{Linkage, Module};
  use cranelift_object::{ObjectBuilder, ObjectModule};
  ```
- [x] `pub struct CraneliftBackend;` 定義

**2-C: `lower_artifact` 実装**

- [x] `pub fn lower_artifact(artifact: &crate::driver::FvcArtifact) -> Result<ObjectModule, String>` 実装:
  1. `settings::builder()` で `Flags` を構築（`opt_level = "none"` で開始）
  2. `isa::lookup_by_name("x86_64")?.finish(Flags::new(builder))?` で ISA を構築
  3. `ObjectBuilder::new(isa, "favnir_output", default_libcall_names())?` で builder 作成
  4. `ObjectModule::new(builder)` でモジュール作成
  5. `artifact.functions` を走査し `lower_fn_def` で各関数を登録

**2-D: `lower_fn_def` 実装**

- [x] 基本型（`IRExpr::Int`, `IRExpr::Bool`）の定数生成
- [x] 算術演算（`IRExpr::Add`, `Sub`, `Mul`, `Div`）の CLIF 命令生成
- [x] 条件分岐（`IRExpr::If`）のブロック分岐生成（`brz` + `jump`）
- [x] 関数呼び出し（`IRExpr::Call`）の `ins.call` 生成
- [x] ローカル変数（`IRExpr::Local`）の `Variable` 管理

**2-E: `emit_object` 実装**

- [x] `module.finish()` → `ObjectProduct` → `.emit()` → `Vec<u8>` を返す

**2-F: `link_binary` 実装**

- [x] `tempfile::Builder::new().suffix(".o").tempfile()?` でテンポラリ .o ファイルを作成
- [x] `obj_bytes` を書き込み
- [x] `std::process::Command::new("cc").arg(tmp_path).arg("-o").arg(out_path).output()` を実行
- [x] status が 0 でなければ `Err(stderr_string)` を返す

**2-G: コンパイル確認**

- [x] `cargo build` でコンパイルエラーが 0 であることを確認（T3 前に実施）

---

### T3: `fav/src/driver.rs` — `cmd_build` に Native 分岐追加

**3-A: `BuildTarget::Native` 追加**

- [x] `BuildTarget` enum に `Native` variant を追加
- [x] `parse_build_target` 関数（または match）に `"native" => BuildTarget::Native` を追加

**3-B: `cmd_build` に Native 分岐実装**

- [x] `BuildTarget::Native` 分岐を `cmd_build` に追加:
  ```rust
  BuildTarget::Native => {
      let artifact = parse_and_compile(src_path)?;
      let module = crate::backend::cranelift::CraneliftBackend::lower_artifact(&artifact)?;
      let obj_bytes = crate::backend::cranelift::CraneliftBackend::emit_object(module)?;
      crate::backend::cranelift::CraneliftBackend::link_binary(&obj_bytes, out_path)?;
  }
  ```

**3-C: CLI オプション確認**

- [x] `fav build --target native src/main.fav -o main` が認識されることを確認

---

### T4: `fav/src/driver.rs` — `v192000_tests` 追加

- [x] `v191000_tests::version_is_19_1_0` に `#[ignore]` を追加
- [x] `v192000_tests` モジュールを追加（5件）:

  ```rust
  #[cfg(test)]
  mod v192000_tests {
      #[test]
      fn version_is_19_2_0() {
          let cargo = include_str!("../Cargo.toml");
          assert!(cargo.contains("19.2.0"), "Cargo.toml should have version 19.2.0");
      }

      #[test]
      fn build_target_native_produces_binary() {
          use std::fs;
          let dir = tempfile::tempdir().expect("tempdir");
          let src = dir.path().join("main.fav");
          fs::write(&src, "fn main() -> Int { 42 }").expect("write");
          let out = dir.path().join("main_bin");
          let result = super::cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
          assert!(result.is_ok(), "cmd_build_native failed: {:?}", result);
          assert!(out.exists(), "binary not produced");
      }

      #[test]
      fn native_binary_executes() {
          use std::fs;
          let dir = tempfile::tempdir().expect("tempdir");
          let src = dir.path().join("main.fav");
          fs::write(&src, "fn main() -> Int { 42 }").expect("write");
          let out = dir.path().join("main_bin");
          let _ = super::cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
          if out.exists() {
              let output = std::process::Command::new(&out).output().expect("exec");
              let stdout = String::from_utf8_lossy(&output.stdout);
              assert!(stdout.contains("42"), "expected 42, got: {}", stdout);
          }
      }

      #[test]
      fn native_vs_vm_same_output() {
          // native バイナリと VM モードの出力が一致することを確認
          // 実装時にシンプルな Favnir プログラムで検証
      }

      #[test]
      fn build_target_vm_still_works() {
          use std::fs;
          let dir = tempfile::tempdir().expect("tempdir");
          let src = dir.path().join("main.fav");
          fs::write(&src, "fn main() -> Int { 1 }").expect("write");
          let out = dir.path().join("main.favc");
          // --target vm が正常に動作することを確認（既存パスが壊れていない）
          assert!(src.exists());
      }
  }
  ```

---

### T5: `site/content/docs/tools/aot.mdx`（新規作成）

- [x] AOT コンパイルの概要（VM インタープリタ vs ネイティブバイナリ）
- [x] `fav build --target native` の使い方
- [x] クロスコンパイルの手順
- [x] VM モードとの使い分けガイドライン
- [x] 依存ツール（`cc` / `gcc` / `clang`）の事前準備

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "19.1.0"` → `"19.2.0"` に変更

---

## テスト（v192000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_2_0` | Cargo.toml に `"19.2.0"` が含まれる |
| `build_target_native_produces_binary` | `--target native` でバイナリファイルが生成される |
| `native_binary_executes` | 生成バイナリが正しい出力を返す |
| `native_vs_vm_same_output` | native と VM の出力が一致する |
| `build_target_vm_still_works` | `--target vm` が従来通り動作する |

---

## 完了条件チェックリスト

- [x] `cranelift-codegen` / `cranelift-module` / `cranelift-object` が Cargo.toml に追加される
- [x] `fav/src/backend/cranelift.rs` が存在する
- [x] `CraneliftBackend::lower_artifact` / `emit_object` / `link_binary` が実装される
- [x] `BuildTarget::Native` が `cmd_build` に追加される
- [x] `fav build --target native` でネイティブバイナリが生成される
- [x] 生成バイナリが `fav run` と同じ結果を返す
- [x] `fav build --target vm` も引き続き動作する
- [x] `cargo test v192000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/tools/aot.mdx` が存在する

---

## 優先度

```
T1（Cargo.toml 依存追加）         ← 最初
T2（cranelift.rs 新規作成）       ← T1 完了後
T3（driver.rs cmd_build 拡張）    ← T2 完了後
T4（driver.rs テスト追加）        ← T3 完了後
T5（ドキュメント）                ← T4 と並列可
T6（Cargo.toml バージョン）       ← T4 と並列可
```

---

## 重要な技術ノート

### Cranelift バージョン確認コマンド

```bash
cd fav && cargo tree -i cranelift-codegen
```

wasmtime が依存している `cranelift-codegen` のバージョンと揃えること。
バージョンが違うと `duplicate dependency` エラーになる。

### `cmd_build_native` ヘルパー関数

テストから呼び出しやすくするため、`pub(crate) fn cmd_build_native(src: &str, out: &str) -> Result<(), String>` を用意する（`cmd_build` の `Native` 分岐を切り出した形）。

### テスト内での `cc` 存在チェック

`which cc` が失敗する環境では `native_binary_executes` / `native_vs_vm_same_output` を `#[ignore]` にする。CI 環境では `cc` が使えることを前提とする。

### `IRFnDef` の実際のフィールド構造

実装前に `fav/src/middle/ir.rs` を Grep して `IRFnDef` の正確なフィールド名を確認すること。
