# v33.5.0 — Spec: fav run --precompiled 確認・テスト補強

## 概要

v33.5.0 は **`fav run --precompiled`（事前コンパイル済みアーティファクト実行）** の確認・テスト補強バージョン。

ロードマップ v33.5 のテーマ「事前コンパイル済みアーティファクトで起動して Lambda コールドスタートを削減」は
v19.7.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `cmd_compile_to_bytes(src, filename)` — `.favc` バイト列生成 | ✓ | v19.7.0 |
| `cmd_compile_to_file(src_path, out_path)` — `.favc` ファイル書き出し | ✓ | v19.7.0 |
| `cmd_run_precompiled_bytes(bytes)` — バイト列から実行 | ✓ | v19.7.0 |
| `cmd_run_precompiled(path)` — `.favc` ファイルから実行 | ✓ | v19.7.0 |
| `FVC\x01` マジックバイト + バイトコードバージョン `0x20` | ✓ | v19.7.0 |
| `FavcMeta`（`source_hash` / `compiled_at` / `compiler_ver`）| ✓ | v19.7.0 |
| 不正バージョン拒否（`ArtifactError::BadVersion`）| ✓ | v19.7.0 |
| `v197000_tests` — 4 件（`compile_produces_favc` 等）| ✓ | v19.7.0 |

v33.5.0 では新規実装は行わず、`v335000_tests` で動作を確認・記録するにとどまる
（v33.1〜v33.4 と同じ「確認・記録」パターン）。

---

## fav run --precompiled 仕様確認

### ワークフロー

```bash
# ソースをコンパイルして .favc を生成
fav compile src/main.fav -o pipeline.favc

# .favc から直接実行（コンパイルをスキップ）
fav run --precompiled pipeline.favc
# 起動時間: ~5ms（通常の fav run: ~200ms）
```

### .favc フォーマット

```
[0..4]  マジック: b"FVC\x01"
[4]     バイトコードバージョン: 0x20
[5..7]  パディング: [0, 0, 0]
[8..12] str_count: u32 LE
[12..16] fn_count: u32 LE
[16..20] global_count: u32 LE
...     文字列テーブル / 関数テーブル / グローバル変数テーブル
[末尾]  META セクション（FavcMeta: source_hash 32B + compiled_at 8B + compiler_ver 4B）
```

### FavcMeta 構造体

```rust
pub struct FavcMeta {
    pub source_hash: [u8; 32],   // SHA-256 of source
    pub compiled_at: u64,        // Unix timestamp
    pub compiler_ver: u32,       // compiler version
}
```

`source_hash` は同一ソースから常に同一ハッシュを生成（決定性）。

---

## 追加するテスト（v335000_tests — 4 件）

`v335000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- 必要なものだけモジュール冒頭で明示 import

```rust
mod v335000_tests {
    use crate::driver::cmd_compile_to_bytes;
    use crate::backend::artifact::FvcArtifact;
    // ...
}
```

テスト名は v197000_tests（`compile_produces_favc` / `precompiled_runs` /
`precompiled_same_output` / `favc_version_check`）と被らないよう設計する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_5_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.5.0"), "Cargo.toml must contain '33.5.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_5_0_exists() {
    let src = include_str!("../../benchmarks/v33.5.0.json");
    assert!(src.contains("33.5.0"), "benchmarks/v33.5.0.json must contain '33.5.0'");
}
```

### テスト 3: META セクションに非ゼロ source_hash が含まれる

v197000_tests は FVC マジックバイトとバイトコードバージョンのみを確認する。
v33.5.0 では `FavcMeta.source_hash` が非ゼロ（SHA-256 が計算済み）であることを確認する。

```rust
fn favc_meta_source_hash_is_nonzero() {
    // FavcMeta.source_hash は SHA-256 なので非ゼロになる
    let src = "fn main() -> Int { 42 }";
    let bytes = cmd_compile_to_bytes(src, "test.fav").expect("compile");
    let artifact = FvcArtifact::from_bytes(&bytes).expect("parse artifact");
    assert!(artifact.meta.is_some(), "FavcMeta section should be present");
    let meta = artifact.meta.expect("FavcMeta should be present");
    assert_ne!(
        meta.source_hash,
        [0u8; 32],
        "source_hash should be non-zero (SHA-256 of source)"
    );
}
```

### テスト 4: 異なるソースは異なるバイト列を生成する

v197000_tests は `precompiled_same_output`（同一ソース）を確認する。
v33.5.0 では異なるソースが異なるバイト列になることを確認する（衝突なし保証）。

```rust
fn favc_different_sources_differ() {
    // 内容が異なるソースから生成した .favc は異なるバイト列になる
    use crate::driver::cmd_compile_to_bytes;
    let bytes_a = cmd_compile_to_bytes("fn main() -> Int { 1 }", "a.fav").expect("compile a");
    let bytes_b = cmd_compile_to_bytes("fn main() -> Int { 2 }", "b.fav").expect("compile b");
    assert_ne!(bytes_a, bytes_b, "different sources must produce different .favc bytes");
}
```

---

## テストモジュールの配置

`v335000_tests` は `v334000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.5.0"`
- `cargo_toml_version_is_33_4_0` が空スタブになっていること
- `cargo test --bin fav v335000` — 4/4 PASS
- `cargo test` — 全件 PASS（2516 件、0 failures）
- `CHANGELOG.md` に `[v33.5.0]` セクション
- `benchmarks/v33.5.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.5.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.5.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
