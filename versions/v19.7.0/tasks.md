# v19.7.0 — 事前コンパイルキャッシュ タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/backend/artifact.rs` — FavcMeta + META セクション + VERSION bump

- [ ] `VERSION: u8` を `0x20` → `0x21` に変更:
  ```rust
  const VERSION: u8 = 0x21; // v19.7.0
  ```

- [ ] `FavcMeta` struct を追加:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct FavcMeta {
      pub source_hash: [u8; 32],  // SHA-256 of original source
      pub compiled_at: u64,       // Unix timestamp
      pub compiler_ver: String,   // e.g. "19.7.0"
  }
  ```

- [ ] `FvcWriter` に `meta: Option<FavcMeta>` フィールドを追加:
  - `Default` で `None`
  - `write_to` の末尾（既存の EXPL/TMET セクションの後）に META セクションを書き込む

- [ ] `FvcArtifact` に `meta: Option<FavcMeta>` フィールドを追加:
  - `read_from` の section loop に `b"META"` タグを追加して読み込む

- [ ] `write_meta_section` / `read_meta_section` ヘルパーを追加:
  ```rust
  fn write_meta_section(w: &mut impl Write, meta: &FavcMeta) -> io::Result<()> {
      w.write_all(b"META")?;
      w.write_all(&meta.source_hash)?;          // 32 bytes
      w.write_all(&meta.compiled_at.to_le_bytes())?; // 8 bytes
      write_string(w, &meta.compiler_ver)?;
      Ok(())
  }
  ```

- [ ] `ArtifactError::BadVersion` のエラーメッセージを改善（Display impl）:
  ```
  unsupported artifact version: 0x20 (current: 0x21).
  Re-compile with: fav compile <source.fav> -o <output.favc>
  ```

- [ ] 既存テスト（`writer_round_trips_artifact` 等）が引き続き PASS することを確認
  - VERSION bump により旧バイト列は `BadVersion` を返す（想定どおり）
  - ラウンドトリップテストは `write_to` → `from_bytes` なので引き続き PASS するはず

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/driver.rs` — `cmd_compile_to_bytes` ヘルパー追加

- [ ] 以下の関数を `driver.rs` に追加:
  ```rust
  /// ソース文字列 → .favc バイト列（テスト・CLI 両用）
  pub fn cmd_compile_to_bytes(src: &str, filename: &str) -> Result<Vec<u8>, String> {
      use sha2::{Sha256, Digest};
      use crate::backend::artifact::{FavcMeta, FvcWriter};
      use crate::frontend::parser::Parser;
      use crate::middle::compiler::compile_program;

      let program = Parser::parse_str(src, filename)
          .map_err(|e| format!("parse error: {e}"))?;
      let _ir = compile_program(&program); // 型チェックのみ（コンパイル済み IR は FvcWriter で書く）

      // FvcWriter を使って artifact を生成
      // compile_to_artifact(program) → FvcWriter を返す既存関数があれば使う
      // なければ: build_artifact(&program).map_err(...)
      let mut artifact_bytes = Vec::new();
      let mut writer = build_fvc_writer(&program)?;
      writer.meta = Some(FavcMeta {
          source_hash: Sha256::digest(src.as_bytes()).into(),
          compiled_at: chrono::Utc::now().timestamp() as u64,
          compiler_ver: env!("CARGO_PKG_VERSION").to_string(),
      });
      writer.write_to(&mut artifact_bytes)
          .map_err(|e| format!("artifact write error: {e}"))?;
      Ok(artifact_bytes)
  }
  ```

  **注意:** `build_fvc_writer` が既存にない場合は `cmd_build` の "fvc" ターゲット処理から
  `FvcWriter` 生成ロジックを抽出してヘルパー化する（または直接呼ぶ）。
  `driver.rs` 内で `.favc` を生成している既存コード箇所を grep して確認すること。

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/driver.rs` — `cmd_run_precompiled_bytes` ヘルパー追加

- [ ] 以下の関数を `driver.rs` に追加:
  ```rust
  /// .favc バイト列 → VM 実行（テスト・CLI 両用）
  pub fn cmd_run_precompiled_bytes(bytes: &[u8]) -> Result<(), String> {
      use crate::backend::artifact::{ArtifactError, FvcArtifact};
      let artifact = FvcArtifact::from_bytes(bytes).map_err(|e| match e {
          ArtifactError::BadVersion(v) => format!(
              "artifact version mismatch: found 0x{v:02X}, expected 0x21. \
               Re-compile with: fav compile <source.fav>"
          ),
          other => format!("artifact error: {other}"),
      })?;
      run_artifact(artifact)
  }
  ```

  **注意:** `run_artifact(artifact: FvcArtifact) -> Result<(), String>` が既存にあれば使う。
  なければ、`cmd_run` で `--precompiled` 時に使われる VM 実行ロジックを抽出してヘルパー化する。

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/driver.rs` — CLI コマンド追加

- [ ] `fav compile <src> [-o <out>]` CLI コマンドを追加:
  ```rust
  fn cmd_compile(src_path: &str, out_path: &str) -> Result<(), String> {
      let src = std::fs::read_to_string(src_path)
          .map_err(|e| format!("cannot read {src_path}: {e}"))?;
      let favc_path = if out_path.is_empty() {
          src_path.replace(".fav", ".favc")
      } else {
          out_path.to_string()
      };
      let bytes = cmd_compile_to_bytes(&src, src_path)?;
      std::fs::write(&favc_path, &bytes)
          .map_err(|e| format!("cannot write {favc_path}: {e}"))?;
      println!("[fav] compiled: {} ({} bytes)", favc_path, bytes.len());
      Ok(())
  }
  ```

- [ ] `fav run --precompiled <path>` フラグを `main.rs` / `driver.rs` のコマンドディスパッチに追加:
  ```
  "run" + "--precompiled" → cmd_run_precompiled(path)
  ```

- [ ] `fav deploy --precompile` フラグ追加（既存の `cmd_deploy` に統合）:
  - `--precompile` フラグが渡された場合、まず `cmd_compile(src, favc_path)` を呼ぶ
  - その後 `.favc` ファイルをデプロイパッケージに含める（または単純に `eprintln!` で案内）

- [ ] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/driver.rs` — `v197000_tests` 追加（5件）

- [ ] `v196000_tests` の直後に以下のモジュールを追加:

```rust
// ── v197000_tests (v19.7.0) — 事前コンパイルキャッシュ ─────────────────────
#[cfg(test)]
mod v197000_tests {
    use crate::backend::artifact::{ArtifactError, FvcArtifact};
    use crate::driver::{cmd_compile_to_bytes, cmd_run_precompiled_bytes};

    #[test]
    fn version_is_19_7_0() {
        assert!(
            include_str!("../Cargo.toml").contains("19.7.0"),
            "Cargo.toml should contain version 19.7.0"
        );
    }

    #[test]
    fn compile_produces_favc() {
        let src = r#"public fn main() -> Unit !Io { IO.println("hi") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        assert_eq!(&bytes[..4], b"FVC\x01", "FVC magic");
        assert_eq!(bytes[4], 0x21, "bytecode version should be 0x21");
    }

    #[test]
    fn precompiled_runs() {
        let src = r#"public fn main() -> Unit !Io { IO.println("precompiled-ok") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        cmd_run_precompiled_bytes(&bytes).expect("run precompiled should succeed");
    }

    #[test]
    fn precompiled_same_output() {
        // precompiled 実行がエラーなく完了することで通常実行と同じ動作を確認
        let src = r#"public fn main() -> Unit !Io { IO.println("v197-output") }"#;
        let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
        cmd_run_precompiled_bytes(&bytes).expect("precompiled same output");
    }

    #[test]
    fn favc_version_check() {
        // バージョン 0x20 の .favc を渡すと BadVersion エラーが返る
        let mut fake = b"FVC\x01".to_vec();
        fake.push(0x20); // old version
        fake.extend_from_slice(&[0, 0, 0]); // padding
        fake.extend_from_slice(&[0, 0, 0, 0]); // str_count = 0
        fake.extend_from_slice(&[0, 0, 0, 0]); // fn_count = 0
        fake.extend_from_slice(&[0, 0, 0, 0]); // global_count = 0
        let err = FvcArtifact::from_bytes(&fake)
            .expect_err("version 0x20 should be rejected");
        assert!(
            matches!(err, ArtifactError::BadVersion(0x20)),
            "expected BadVersion(0x20), got {:?}", err
        );
    }
}
```

- [ ] `cargo test v197000` — 5/5 PASS を確認

---

### T6: `fav/Cargo.toml` — バージョン更新

- [ ] `version = "19.6.0"` → `"19.7.0"` に変更

---

### T7: `site/content/docs/tools/precompiled.mdx`（新規）

- [ ] 以下の内容を含む MDX を作成:
  - 事前コンパイルの概要（Lambda コールドスタート問題と解決策）
  - `fav compile` の使い方
  - `fav run --precompiled` の使い方
  - Lambda bootstrap スクリプトの変更例
  - `.favc` のバージョン管理（再コンパイルが必要なケース）
  - `fav deploy --precompile` の使い方

---

## テスト（v197000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_7_0` | Cargo.toml に `"19.7.0"` が含まれる |
| `compile_produces_favc` | `cmd_compile_to_bytes` が `FVC\x01` magic + `0x21` version バイト列を返す |
| `precompiled_runs` | `.favc` バイト列を `cmd_run_precompiled_bytes` で実行できる |
| `precompiled_same_output` | 事前コンパイル実行がエラーなく完了する |
| `favc_version_check` | バージョン `0x20` の `.favc` で `BadVersion(0x20)` エラーが返る |

---

## 完了条件チェックリスト

- [ ] `artifact.rs` の `VERSION` が `0x21` になっている
- [ ] `FavcMeta { source_hash, compiled_at, compiler_ver }` が定義されている
- [ ] `FvcWriter` / `FvcArtifact` に `meta: Option<FavcMeta>` が追加されている
- [ ] META セクションの読み書きが実装されている
- [ ] `cmd_compile_to_bytes` が実装されている
- [ ] `cmd_run_precompiled_bytes` が実装されている
- [ ] `fav compile <src>` CLI が動作する
- [ ] `fav run --precompiled <path>` CLI が動作する
- [ ] `cargo test v197000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/precompiled.mdx` が存在する

---

## 優先度

```
T1（artifact.rs）       ← 最初（バイナリ形式の拡張が基盤）
T2（cmd_compile_to_bytes） ← T1 完了後すぐ
T3（cmd_run_precompiled_bytes） ← T1 完了後、T2 と並列可
T4（CLI コマンド追加）  ← T2/T3 完了後
T5（v197000_tests）     ← T2/T3 完了後（T4 と並列可）
T6（Cargo.toml）        ← T5 と並列可
T7（ドキュメント）      ← T5 完了後
```

---

## 重要な技術ノート

### `build_fvc_writer` の場所

実装前に `driver.rs` を grep して `FvcWriter` / `cmd_build` / `"fvc"` を検索し、
既存の `.favc` 生成ロジックを確認する。重複実装を避けること。

### `run_artifact` の場所

`driver.rs` で `FvcArtifact` を VM で実行している箇所を grep して確認する。
`--precompiled` フラグや `run_artifact` 関数が既存にある可能性がある。

### VERSION bump の影響

`VERSION: u8 = 0x21` に変更すると、旧バージョン（`0x20`）の `.favc` ファイルは
`BadVersion(0x20)` エラーで拒否される。これは意図した動作。

既存テスト（`artifact.rs` の `writer_round_trips_artifact` 等）は
`write_to` → `from_bytes` のラウンドトリップなので、VERSION 変更後も PASS する。

### SHA-256 の使い方

```rust
use sha2::{Digest, Sha256};
let hash: [u8; 32] = Sha256::digest(src.as_bytes()).into();
```

`sha2` は `Cargo.toml` に `sha2 = "0.10"` として既存依存。

### META セクション書き込み順序

既存の EXPL / TMET タグ（オプションセクション）の**後**に META を書き込む。
`read_from` の section loop に `b"META"` ブランチを追加する。
