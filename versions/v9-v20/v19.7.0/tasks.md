# v19.7.0 — 事前コンパイルキャッシュ タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/backend/artifact.rs` — FavcMeta + META セクション

- [x] `FavcMeta` struct を追加（`source_hash: [u8; 32]` / `compiled_at: u64` / `compiler_ver: String`）
- [x] `FvcWriter` に `meta: Option<FavcMeta>` フィールドを追加（`Default` で `None`）
- [x] `write_to` の末尾（EXPL/TMET の後）に META セクションを書き込む
- [x] `FvcArtifact` に `meta: Option<FavcMeta>` フィールドを追加（`#[serde(skip)]`）
- [x] `read_from` の section loop に `b"META"` タグを追加して読み込む
- [x] `write_meta_section` / `read_meta_section` ヘルパーを追加
- [x] `ArtifactError::BadVersion` のエラーメッセージを改善
- [x] `codegen.rs` の `FvcArtifact { ... }` に `meta: None` を追加
- [x] `write_artifact_to_path` に `meta: artifact.meta.clone()` を追加
- [x] 既存テストが引き続き PASS することを確認

**注記:** VERSION は `0x20` のまま（META はadditive、セルフホスト compiler.fav との後方互換を維持）。

---

### T2: `fav/src/driver.rs` — `cmd_compile_to_bytes` ヘルパー追加

- [x] `cmd_compile_to_bytes(src, filename) -> Result<Vec<u8>, String>` を実装
  - SHA-256（`sha2::Sha256::digest`）/ `chrono::Utc::now()` / `env!("CARGO_PKG_VERSION")`
  - `build_artifact(&program)` → `FvcWriter { ..., meta: Some(FavcMeta {...}) }.write_to()`

---

### T3: `fav/src/driver.rs` — `cmd_run_precompiled_bytes` ヘルパー追加

- [x] `cmd_run_precompiled_bytes(bytes) -> Result<(), String>` を実装
  - `FvcArtifact::from_bytes` → `exec_artifact_main` を呼ぶ

---

### T4: `fav/src/driver.rs` + `main.rs` — CLI コマンド追加

- [x] `cmd_compile(src_path, out_path: Option<&str>)` を実装（`.fav` → `.favc` 自動変換）
- [x] `cmd_run_precompiled(path)` を実装
- [x] `main.rs` に `Some("compile")` ブランチ追加（`-o` フラグ対応）
- [x] `main.rs` の `Some("run")` に `--precompiled` 早期リターン追加

---

### T5: `fav/src/driver.rs` — `v197000_tests` 追加（5件）

- [x] `v197000_tests` モジュール追加
- [x] `v196000_tests::version_is_19_6_0` に `#[ignore]` 追加
- [x] `cargo test v197000` — 5/5 PASS 確認

---

### T6: `fav/Cargo.toml` — バージョン更新

- [x] `version = "19.6.0"` → `"19.7.0"` に変更

---

### T7: `site/content/docs/tools/precompiled.mdx`（新規）

- [x] 事前コンパイルの概要（コールドスタート比較表）
- [x] `fav compile` / `fav run --precompiled` の使い方
- [x] Lambda bootstrap スクリプト変更例
- [x] `.favc` フォーマット説明
- [x] バージョン互換性と再コンパイルが必要なケース
- [x] CI/CD 統合例

---

## テスト（v197000_tests、5件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_19_7_0` | Cargo.toml に `"19.7.0"` が含まれる | PASS |
| `compile_produces_favc` | `cmd_compile_to_bytes` が `FVC\x01` magic + `0x20` version バイト列を返す | PASS |
| `precompiled_runs` | `.favc` バイト列を `cmd_run_precompiled_bytes` で実行できる | PASS |
| `precompiled_same_output` | 事前コンパイル実行がエラーなく完了する | PASS |
| `favc_version_check` | バージョン `0xFF` の `.favc` で `BadVersion(0xFF)` エラーが返る | PASS |

---

## 完了条件チェックリスト

- [x] `FavcMeta { source_hash, compiled_at, compiler_ver }` が定義されている
- [x] `FvcWriter` / `FvcArtifact` に `meta: Option<FavcMeta>` が追加されている
- [x] META セクションの読み書きが実装されている
- [x] VERSION は `0x20` のまま（後方互換）
- [x] `cmd_compile_to_bytes` が実装されている
- [x] `cmd_run_precompiled_bytes` が実装されている
- [x] `fav compile <src>` CLI が動作する
- [x] `fav run --precompiled <path>` CLI が動作する
- [x] `cargo test v197000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（1730 passed, 0 failed）
- [x] `site/content/docs/tools/precompiled.mdx` が存在する
