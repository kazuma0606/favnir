# v19.7.0 仕様書 — 事前コンパイルキャッシュ

Date: 2026-06-17

## 概要

Lambda / ECS での高速コールドスタートを実現する。
現在: Lambda 起動ごとに `fav` がソースをコンパイルする（~200ms）。
目標: 事前コンパイルしたアーティファクト（`.favc`）で起動時間を ~5ms に短縮。

## 現状と目標

```
現状: fav run pipeline.fav
  → パース（~30ms）
  → 型チェック（~50ms）
  → コンパイル（~120ms）
  → VM 実行（~5ms）
  → 合計: ~205ms

目標: fav run --precompiled pipeline.favc
  → .favc ロード（~1ms）
  → バージョン検証（~0ms）
  → VM 実行（~5ms）
  → 合計: ~6ms
```

## `.favc` ファイル形式

既存の `artifact.rs` が実装する `FVC\x01` フォーマットを拡張し、
メタデータセクション（`META`）を追加する。

```
[ヘッダー]  （既存）
magic:        "FVC\x01"（4 bytes）
version:      u8 = 0x20 + 1 = 0x21（v19.7 で bump）
padding:      3 bytes

[カウンタ]  （既存）
str_count:    u32
fn_count:     u32
global_count: u32

[文字列テーブル]  （既存）
...

[型セクション]  （既存）
...

[グローバルセクション]  （既存）
...

[関数セクション]  （既存）
...

[METAセクション]  （v19.7 新規）
tag:          "META"（4 bytes）
source_hash:  32 bytes（SHA-256 of original source）
compiled_at:  u64（Unix timestamp, little-endian）
compiler_ver: length-prefixed UTF-8 string（例: "19.7.0"）
```

### バージョン互換性

- `VERSION` は `0x20` のまま（META セクションは additive で後付け追加、破壊的変更なし）
- セルフホスト compiler.fav が書き出す旧アーティファクト（META なし）も引き続き読み込める
- `ArtifactError::BadVersion` は 0xFF 等の未知バージョンに対して発動する
- エラーメッセージ例:
  ```
  error: unsupported artifact version: 0xFF (current: 0x20).
         Re-compile with: fav compile <source.fav>
  ```

## CLI

```bash
# ソースから事前コンパイル
fav compile src/pipeline.fav -o pipeline.favc

# 出力ファイル名を省略（.fav → .favc）
fav compile src/pipeline.fav
# → src/pipeline.favc

# 事前コンパイル済みアーティファクトを実行
fav run --precompiled pipeline.favc

# デプロイ時に自動的に .favc を生成
fav deploy --precompile
```

## 実装計画

### artifact.rs への変更

1. `VERSION: u8` を `0x20` → `0x21` に bump
2. `FavcMeta { source_hash: [u8; 32], compiled_at: u64, compiler_ver: String }` 追加
3. `FvcWriter.meta: Option<FavcMeta>` フィールド追加
4. `FvcArtifact.meta: Option<FavcMeta>` フィールド追加
5. `write_meta_section` / `read_meta_section` 実装（タグ `"META"`）
6. `ArtifactError::BadVersion` のエラーメッセージを拡充

### driver.rs への変更

1. `cmd_compile(src_path: &str, out_path: &str)` 実装:
   - ソース読み込み → パース → 型チェック → コンパイル → `FvcWriter.write_to(file)`
   - `FavcMeta` に `source_hash`（SHA-256）/ `compiled_at`（now）/ `compiler_ver` を設定
   - `META` セクションを書き込む

2. `fav run --precompiled <path>` 対応:
   - `.favc` 読み込み → `FvcArtifact::from_bytes` → `run_artifact(artifact)`
   - バージョン不一致時は `--precompiled` を付けた再コンパイル方法をエラーに明示

3. `fav deploy --precompile` 対応:
   - `cmd_compile` を呼び出してから既存の deploy ロジックを実行

### sha2 crate

`source_hash` の計算に `sha2` を使用する（既存依存に `sha2 = "0.10"` があり追加不要）。

## テスト（v197000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_7_0` | Cargo.toml に `"19.7.0"` が含まれる |
| `compile_produces_favc` | `cmd_compile` が `.favc` バイト列を生成し、FVC magic を持つ |
| `precompiled_runs` | `FvcArtifact::from_bytes` → `run_artifact` が成功する |
| `precompiled_same_output` | 通常実行と事前コンパイル実行の出力が一致する |
| `favc_version_check` | 旧バージョン（0x20）の `.favc` で `BadVersion` エラーが返る |

## 完了条件

- [ ] `fav compile src/pipeline.fav -o pipeline.favc` で `.favc` ファイルが生成される
- [ ] `fav run --precompiled pipeline.favc` が正しく実行される
- [ ] 通常の `fav run` と `--precompiled` の出力が一致する
- [ ] `.favc` のバージョンミスマッチで適切なエラーが出る
- [ ] `META` セクションに source_hash / compiled_at / compiler_ver が含まれる
- [ ] `cargo test v197000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
