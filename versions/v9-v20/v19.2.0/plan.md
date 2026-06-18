# v19.2.0 実装計画 — AOT コンパイル（Cranelift バックエンド）

## 実装順序

```
T1（Cargo.toml 依存追加）         ← 最初（T2 のコンパイルが依存）
T2（cranelift.rs 新規作成）       ← T1 完了後
T3（driver.rs cmd_build 拡張）    ← T2 完了後
T4（driver.rs テスト追加）        ← T3 完了後
T5（ドキュメント）                ← T4 と並列可
T6（Cargo.toml バージョン）       ← T4 と並列可
```

---

## T1: Cargo.toml — Cranelift 依存追加

`wasmtime` の後に追記:

```toml
cranelift-codegen = { version = "0.113", features = ["x64"] }
cranelift-module  = { version = "0.113" }
cranelift-object  = { version = "0.113" }
```

`cargo build` でコンパイルエラーが 0 であることを確認。

---

## T2: `fav/src/backend/cranelift.rs`（新規）

### モジュール宣言

`fav/src/backend/mod.rs`（または `fav/src/lib.rs`）に:
```rust
pub mod cranelift;
```

### `CraneliftBackend` 実装

```rust
use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, Signature};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context;
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

pub struct CraneliftBackend;

impl CraneliftBackend {
    /// Favnir FvcArtifact → Cranelift ObjectModule
    pub fn lower_artifact(artifact: &crate::driver::FvcArtifact) -> Result<ObjectModule, String> {
        // ...
    }

    /// ObjectModule → オブジェクトファイルのバイト列
    pub fn emit_object(module: ObjectModule) -> Result<Vec<u8>, String> {
        // module.finish() → ObjectProduct → .emit()
    }

    /// オブジェクトファイル → ネイティブバイナリ（cc 経由）
    pub fn link_binary(obj_bytes: &[u8], out_path: &str) -> Result<(), String> {
        // tempfile::NamedTempFile に obj_bytes を書き込み
        // std::process::Command::new("cc") を呼び出し
    }
}
```

### IR → CLIF 変換の実装方針

`lower_artifact` は `artifact.functions` を走査し各 `IRFnDef` を:

1. `cranelift_module` で関数シンボルを宣言
2. `cranelift_codegen::Function` を構築
3. `IRExpr` を再帰的に CLIF 命令列に変換
4. `module.define_function(id, ctx)` でモジュールに登録

対象 `IRExpr` variant:
- `IRExpr::Int(n)` → `ins.iconst(types::I64, n)`
- `IRExpr::Bool(b)` → `ins.iconst(types::I8, b as i64)`
- `IRExpr::Add(a, b)` / `Sub` / `Mul` → `ins.iadd` / `isub` / `imul`
- `IRExpr::If(cond, then, else_)` → ブロック分岐（`brz` + `jump`）
- `IRExpr::Call(fn_idx, args)` → `ins.call(fn_ref, &arg_vals)`
- `IRExpr::Local(slot)` → `Variable` 参照
- `IRExpr::Global(idx)` → `FuncRef` 参照

---

## T3: `fav/src/driver.rs` — `cmd_build` 拡張

### `BuildTarget` 変更

既存の `BuildTarget` enum に `Native` を追加:

```rust
pub enum BuildTarget {
    Fvc,
    Wasm,
    Native,  // 新規
}
```

`parse_build_target(s: &str) -> BuildTarget` に `"native"` → `BuildTarget::Native` を追加。

### `cmd_build` に Native 分岐

```rust
BuildTarget::Native => {
    let artifact = parse_and_compile(src_path)?;
    let module = CraneliftBackend::lower_artifact(&artifact)
        .map_err(|e| format!("cranelift error: {e}"))?;
    let obj_bytes = CraneliftBackend::emit_object(module)
        .map_err(|e| format!("emit error: {e}"))?;
    CraneliftBackend::link_binary(&obj_bytes, out_path)
        .map_err(|e| format!("link error: {e}"))?;
}
```

---

## T4: `fav/src/driver.rs` — `v192000_tests`

### `#[ignore]` 追加

```rust
#[test]
#[ignore]
fn version_is_19_1_0() { ... }
```

### `v192000_tests` モジュール

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
        // tempdir に .fav を書き込み、cmd_build --target native を呼び出す
        // 出力バイナリが存在することを確認
    }

    #[test]
    fn native_binary_executes() {
        // 生成バイナリを実行し、stdout が期待値と一致することを確認
    }

    #[test]
    fn native_vs_vm_same_output() {
        // native バイナリと fav run の出力が一致することを確認
    }

    #[test]
    fn build_target_vm_still_works() {
        // --target vm で .favc が生成されることを確認
    }
}
```

---

## 注意点

### Cranelift バージョンの一致

`wasmtime = "30"` は `cranelift-codegen 0.113` を内部使用する。
`cranelift-*` crate も `0.113` に固定しないと依存解決でコンフリクトが発生する。
`cargo tree -i cranelift-codegen` で確認してから依存を追記すること。

### `cc` コマンドの可用性

テスト環境に `cc`（または `gcc`/`clang`）がインストールされていること前提。
`which cc` で確認し、存在しない場合はテストを `#[ignore]` にする。

### v19.2.0 の範囲制限

- 対応する `IRExpr` は基本型（Int/Bool）の算術・条件分岐・関数呼び出しのみ
- `List` / `Stream` / `Closure` 等は v19.3.0 以降でサポート
- `native_binary_executes` / `native_vs_vm_same_output` テストは Int を返す簡単な fn を対象

### `IRFnDef` の構造確認

`IRFnDef` の実際のフィールド（`params`, `body`, `locals` 等）を Grep で確認してから実装すること。
