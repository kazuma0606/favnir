# v19.2.0 Spec — AOT コンパイル（Cranelift バックエンド）

## 概要

バイトコード VM から脱却し、ネイティブバイナリを生成する。
Cranelift を AOT バックエンドとして採用し、実行速度を大幅に改善する。

**テーマ**: Production Performance シリーズ第2弾

---

## 動機

```
現状: .fav → (コンパイル) → バイトコード → (VM インタープリタ) → 実行
目標: .fav → (AOT コンパイル) → ネイティブバイナリ → 実行
```

VM インタープリタは開発フローに適しているが、本番ワークロードでは
ネイティブバイナリの実行速度が必要になる。

---

## CLI

```bash
# ネイティブバイナリとしてビルド
fav build --target native src/pipeline.fav -o pipeline

# 実行（fav run 不要、直接実行）
./pipeline

# クロスコンパイル
fav build --target x86_64-unknown-linux-musl src/pipeline.fav -o pipeline-linux

# VM モード（従来通り）
fav build --target vm src/pipeline.fav -o pipeline.favc
fav run --precompiled pipeline.favc
```

---

## Cranelift の採用理由

| | Cranelift | LLVM |
|---|---|---|
| Rust との統合 | ネイティブ（wasmtime と共通） | FFI（libLLVM が大きい） |
| ビルド時間 | 速い（設計目標） | 遅い |
| 最適化品質 | 中程度（JIT 向け設計） | 高い |
| 依存サイズ | 小さい | 非常に大きい |

---

## IR → Cranelift IR の変換

```
Favnir IR                    Cranelift IR（CLIF）
--------                     -------------------
Opcode::Push(Int(n))    →    iconst.i64 n
Opcode::Add             →    iadd
Opcode::Call(fn_idx)    →    call fn_name(args...)
Opcode::Jump(target)    →    jump block_label
Opcode::JumpIf(target)  →    brnz val, block_label
```

---

## 実装内容

### T1: `fav/Cargo.toml` — Cranelift 依存追加

```toml
cranelift-codegen = { version = "0.113", features = ["x64"] }
cranelift-module  = { version = "0.113" }
cranelift-object  = { version = "0.113" }
```

### T2: `fav/src/backend/cranelift.rs`（新規）

- `CraneliftBackend` struct:
  - `lower_artifact(artifact: &FvcArtifact) -> Module` — Favnir IR → Cranelift IR 変換
  - `emit_object(module: Module) -> Vec<u8>` — オブジェクトファイルのバイト列を生成
  - `link_binary(obj_bytes: &[u8], out_path: &str) -> Result<(), String>` — `cc` 経由でリンク

- IR 変換の対象:
  - `IRExpr::Int` / `IRExpr::Bool` / `IRExpr::Str` → CLIF 定数
  - `IRExpr::Add` / `IRExpr::Sub` / `IRExpr::Mul` → CLIF 演算命令
  - `IRExpr::Call` → CLIF `call` 命令
  - `IRExpr::If` → CLIF ブロック分岐
  - `IRExpr::Local` / `IRExpr::Global` → CLIF 変数 / グローバルシンボル

### T3: `fav/src/driver.rs` — `cmd_build` 拡張

- `BuildTarget` enum に `Native` variant を追加:
  ```rust
  pub enum BuildTarget {
      Fvc,   // 既存: バイトコード .favc
      Wasm,  // 既存: WebAssembly
      Native, // 新規: ネイティブバイナリ
  }
  ```

- `cmd_build` に `Native` 分岐:
  1. `parse_and_compile(src)` → `FvcArtifact`
  2. `CraneliftBackend::lower_artifact(&artifact)` → Cranelift Module
  3. `CraneliftBackend::emit_object(module)` → `Vec<u8>` (オブジェクトファイル)
  4. `CraneliftBackend::link_binary(&obj_bytes, out)` → ネイティブバイナリ

- CLI `--target` オプションに `"native"` を追加

### T4: `fav/src/driver.rs` — `v192000_tests` 追加

- `v191000_tests::version_is_19_1_0` に `#[ignore]` を追加
- `v192000_tests` モジュール（5件）:
  - `version_is_19_2_0`
  - `build_target_native_produces_binary`
  - `native_binary_executes`
  - `native_vs_vm_same_output`
  - `build_target_vm_still_works`

### T5: `site/content/docs/tools/aot.mdx`（新規）

- AOT コンパイルの使い方ガイド
- `fav build --target native` の手順
- クロスコンパイルの方法
- VM モードとの使い分け

### T6: `fav/Cargo.toml` バージョン更新

- `version = "19.1.0"` → `"19.2.0"`

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

## 完了条件

- [ ] `cranelift-codegen` / `cranelift-module` / `cranelift-object` が Cargo.toml に追加される
- [ ] `fav/src/backend/cranelift.rs` が存在する
- [ ] `fav build --target native` でネイティブバイナリが生成される
- [ ] 生成バイナリが `fav run` と同じ結果を返す
- [ ] `fav build --target vm` も引き続き動作する
- [ ] `cargo test v192000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/aot.mdx` が存在する

---

## 技術ノート

### Cranelift のバージョン

`wasmtime = "30"` は内部で `cranelift-codegen 0.113` 系を使用している。
`cranelift-codegen`, `cranelift-module`, `cranelift-object` も同じ `0.113` で統一する。

### オブジェクトファイルのリンク

`cranelift-object` が生成する `.o` ファイルを `cc` コマンドでリンクする（`std::process::Command`）。
テスト時は `tempfile` crate でテンポラリディレクトリを確保して生成物を配置する。

### `FvcArtifact` の構造

既存フィールド:
- `str_table: Vec<String>`
- `globals: Vec<IRFnDef>`
- `functions: Vec<IRFnDef>`
- `type_metas: Vec<TypeMeta>`

Cranelift バックエンドはこれらを直接読んで CLIF を生成する。

### v19.2.0 の範囲制限

- `Int` / `Bool` / `Float` / `String` の基本型のみ対応
- `List`, `Map`, `Stream` 等のコレクション型は v19.3.0 以降
- ネイティブバイナリのランタイムライブラリは最小限（stdlib は動的リンク）
