# v33.2.0 — Spec: インクリメンタルコンパイル 確認・テスト補強

## 概要

v33.2.0 は **インクリメンタルコンパイル（Incremental Compilation）** の確認・テスト補強バージョン。

ロードマップ v33.2 のテーマ「変更ファイルのみ再コンパイルして開発サイクルを高速化する」は
v19.3.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `fav/src/incremental/fingerprint.rs` — `content_hash` / `file_hash`（SHA-256）| ✓ | v19.3.0 |
| `fav/src/incremental/cache.rs` — `IncrementalCache::new` / `write_artifact` / `read_artifact` / `is_hit` | ✓ | v19.3.0 |
| `fav/src/incremental/dep_graph.rs` — `DepGraph` / `build_dep_graph` / `affected_by` / `transitive_deps` | ✓ | v19.3.0 |
| `v193000_tests` — 4 件（`cache_creates_on_first_build` 等） | ✓ | v19.3.0 |

v33.2.0 では新規実装は行わず、`v332000_tests` で動作を確認・記録するにとどまる
（v32.1〜v33.1 と同じ「確認・記録」パターン）。

---

## インクリメンタルコンパイル 仕様確認

### キャッシュ方式

```
~/.fav/cache/<project-hash>/
  <content-hash>.artifact   # AST+型情報コンパイル済みアーティファクト
```

- コンテンツハッシュ（SHA-256）でキャッシュヒット判定
- 内容が変わると異なるハッシュ → 自動無効化

### 依存グラフ

```
use utils.format_date   →  pipeline が utils に依存
                        →  utils 変更時に pipeline も再コンパイル対象
```

- `use X.{ field }` （2セグメント以上）= ファイル間依存
- `use json` （1セグメント）= Rune import（スキップ）

---

## 追加するテスト（v332000_tests — 4 件）

`v332000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- `use crate::incremental::*` で必要なモジュールのみ明示 import

テスト名は v193000_tests（`cache_creates_on_first_build` / `cache_hits_on_second_build` /
`cache_invalidates_on_change` / `dep_graph_propagates`）と被らないよう
`incremental_` プレフィックスを使用。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_2_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.2.0"), "Cargo.toml must contain '33.2.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_2_0_exists() {
    let src = include_str!("../../benchmarks/v33.2.0.json");
    assert!(src.contains("33.2.0"), "benchmarks/v33.2.0.json must contain '33.2.0'");
}
```

### テスト 3: ハッシュ決定性確認

v193000_tests の `cache_invalidates_on_change`（異なるハッシュを比較）とは異なる視点で、
同一コンテンツから常に同じ SHA-256 が生成されることを確認する。

```rust
fn incremental_content_hash_deterministic() {
    // 同じコンテンツから同じ SHA-256 ハッシュが生成される（決定性）
    // v193000_tests::cache_invalidates_on_change とは異なる視点（ハッシュ自体の決定性確認）
    use crate::incremental::fingerprint;
    let bytes = b"fn main() -> Int { 42 }";
    let h1 = fingerprint::content_hash(bytes);
    let h2 = fingerprint::content_hash(bytes);
    assert_eq!(h1, h2, "content_hash must be deterministic");
    assert_eq!(h1.len(), 64, "SHA-256 hex should be 64 chars");
}
```

### テスト 4: 依存なしファイルは影響を受けない

v193000_tests の `dep_graph_propagates`（依存ありケース）の逆ケース:
`use` 宣言のないファイルは `affected_by` に含まれないことを確認する。

```rust
fn incremental_dep_graph_no_import_isolated() {
    // use 宣言のないファイルは affected_by に含まれない
    // v193000_tests::dep_graph_propagates の逆ケース（依存なし → 影響なし）
    use crate::incremental::dep_graph;
    use crate::frontend::parser::Parser;
    let src = "fn main() -> Int { 1 }"; // use 宣言なし
    let prog = Parser::parse_str(src, "isolated.fav").expect("parse");
    let graph = dep_graph::build_dep_graph(&prog, "isolated");
    let affected = graph.affected_by("utils");
    assert!(
        !affected.contains(&"isolated".to_string()),
        "file with no imports should not be affected by utils, got: {:?}",
        affected
    );
}
```

---

## テストモジュールの配置

`v332000_tests` は `v331000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.2.0"`
- `cargo_toml_version_is_33_1_0` が空スタブになっていること
- `cargo test --bin fav v332000` — 4/4 PASS
- `cargo test` — 全件 PASS（2504 件、0 failures）
- `CHANGELOG.md` に `[v33.2.0]` セクション
- `benchmarks/v33.2.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.2.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.2.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
