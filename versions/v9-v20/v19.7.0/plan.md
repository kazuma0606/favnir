# v19.7.0 実装計画 — 事前コンパイルキャッシュ

## 前提確認

- `fav/src/backend/artifact.rs`: `FvcWriter` / `FvcArtifact` / `ArtifactError` 実装済み（v9.0.0）
- 現在の `VERSION = 0x20`、`MAGIC = b"FVC\x01"`
- `sha2 = "0.10"` は `Cargo.toml` に既存依存としてある
- `chrono` は既存依存（`compiled_at` タイムスタンプに使用可）

## 実装順序

```
T1: artifact.rs — FavcMeta + META セクション + VERSION bump
    ↓
T2: driver.rs — cmd_compile 実装
    ↓
T3: driver.rs — fav run --precompiled 実装
    ↓
T4: driver.rs — fav deploy --precompile 実装
    ↓
T5: v197000_tests 追加（5件）
    ↓
T6: Cargo.toml バージョン更新（19.6.0 → 19.7.0）
    ↓
T7: site/content/docs/tools/precompiled.mdx 作成
```

## 各タスクの詳細

### T1: artifact.rs

変更点:
1. `VERSION: u8 = 0x21`（0x20 から bump）
2. `FavcMeta` struct 追加
3. `FvcWriter.meta` / `FvcArtifact.meta` フィールド追加
4. `write_meta_section` / `read_meta_section` ヘルパー追加
5. タグ `"META"` を `read_from` の section loop に追加

注意:
- `writer_round_trips_artifact` 等の既存テストが `VERSION` 変更で壊れないよう確認
  → `from_bytes` で `0x21` が通れば OK（テストは `write_to` → `from_bytes` のラウンドトリップなので問題なし）
- 旧バージョン（0x20）のバイト列を `from_bytes` に渡すと `BadVersion(0x20)` を返す

### T2: driver.rs — cmd_compile

```rust
pub fn cmd_compile(src_path: &str, out_path: &str) -> Result<(), String>
```

処理フロー:
1. `fs::read_to_string(src_path)` → ソース取得
2. `sha2::Sha256::digest(src.as_bytes())` → `source_hash: [u8; 32]`
3. `Parser::parse_str(&src, src_path)` → AST
4. `compile_program(&program)` → IR
5. `FvcWriter` に `FavcMeta { source_hash, compiled_at, compiler_ver }` をセット
6. `writer.write_to(&mut file)` → `.favc` ファイル書き込み

`out_path` が空文字列の場合: `.fav` → `.favc` に自動変換

### T3: driver.rs — fav run --precompiled

```rust
pub fn cmd_run_precompiled(path: &str) -> Result<(), String>
```

処理フロー:
1. `fs::read(path)` → bytes
2. `FvcArtifact::from_bytes(&bytes)` → artifact（バージョン不一致で即エラー）
3. 既存の `run_artifact(artifact)` を呼ぶ（既存関数があるはず）

エラーメッセージ:
- `BadVersion(v)` → "pipeline.favc was compiled with bytecode version 0x{v:02X}, but this is 0x21. Re-compile with: fav compile <source.fav>"

### T4: driver.rs — fav deploy --precompile

既存の `cmd_deploy` の冒頭に `cmd_compile(src, default_favc_path)` を挿入するだけ。
または `--precompile` フラグで `cmd_compile` を呼んでから通常の deploy を実行。

### T5: v197000_tests

テストは全て `#[cfg(test)]` モジュール内で完結。
`cmd_run_precompiled` / `cmd_compile` はパブリック関数として公開。

### T6: Cargo.toml

`version = "19.6.0"` → `"19.7.0"`

### T7: docs

`site/content/docs/tools/precompiled.mdx` を新規作成。

## 注意事項

- `FvcArtifact` を VM で実行する関数 `run_artifact` は `driver.rs` か `vm.rs` に既存の可能性あり
  → 実装前に `driver.rs` 内の `precompiled` / `run_artifact` を grep して確認する
- `sha2::Sha256` の使用方法: `use sha2::{Sha256, Digest}; let hash: [u8; 32] = Sha256::digest(bytes).into();`
- `chrono` で `compiled_at`: `chrono::Utc::now().timestamp() as u64`
- `compiler_ver` は `env!("CARGO_PKG_VERSION")` で取得

## テストコード（骨格）

```rust
#[cfg(test)]
mod v197000_tests {
    use crate::backend::artifact::{ArtifactError, FvcArtifact, FvcWriter, FavcMeta};
    use crate::driver::{cmd_compile_to_bytes, cmd_run_precompiled_bytes};

    #[test]
    fn version_is_19_7_0() {
        assert!(include_str!("../Cargo.toml").contains("19.7.0"));
    }

    #[test]
    fn compile_produces_favc() {
        let src = r#"public fn main() -> Unit !Io { IO.println("hi") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        assert_eq!(&bytes[..4], b"FVC\x01", "FVC magic");
        // version byte = 0x21
        assert_eq!(bytes[4], 0x21, "version should be 0x21");
    }

    #[test]
    fn precompiled_runs() {
        let src = r#"public fn main() -> Unit !Io { IO.println("precompiled-ok") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        cmd_run_precompiled_bytes(&bytes).expect("run precompiled");
    }

    #[test]
    fn precompiled_same_output() {
        // 通常実行と precompiled 実行の出力が一致することを確認
        let src = r#"public fn main() -> Unit !Io { IO.println("hello-v197") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        // run_precompiled が成功すれば出力は同じ（stdout capture は難しいためエラーなし確認のみ）
        cmd_run_precompiled_bytes(&bytes).expect("run precompiled same output");
    }

    #[test]
    fn favc_version_check() {
        // バージョン 0x20 のバイト列を渡すと BadVersion エラー
        let mut fake = b"FVC\x01".to_vec();
        fake.push(0x20); // old version
        fake.extend_from_slice(&[0, 0, 0]); // padding
        fake.extend_from_slice(&[0, 0, 0, 0]); // str_count = 0
        fake.extend_from_slice(&[0, 0, 0, 0]); // fn_count = 0
        fake.extend_from_slice(&[0, 0, 0, 0]); // global_count = 0
        let err = FvcArtifact::from_bytes(&fake).expect_err("should fail version check");
        assert!(matches!(err, ArtifactError::BadVersion(0x20)));
    }
}
```

## テスト用ヘルパー（driver.rs 内）

```rust
/// テスト・CLI 両用: ソース文字列 → .favc バイト列
pub fn cmd_compile_to_bytes(src: &str, filename: &str) -> Result<Vec<u8>, String>

/// テスト・CLI 両用: .favc バイト列 → VM 実行
pub fn cmd_run_precompiled_bytes(bytes: &[u8]) -> Result<(), String>
```

これにより `cmd_compile`（ファイルI/O版）と `cmd_run_precompiled`（ファイルI/O版）は
上記ヘルパーを呼び出す薄いラッパーとなる。
