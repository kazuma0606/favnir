# Spec: v51.8.0 — ドキュメントサイト Performance 記事

Date: 2026-07-20
Status: 設計中

---

## 目的

v51.1〜v51.7 で実装した Performance & Scale 機能群（`par` 並列実行・バックプレッシャー・`fav bench --compare`）に対応するドキュメント記事を追加する。
ユーザーが機能を使いこなせるよう、具体的なコード例・設定例・CLI 出力例を含む実用的な記事とする。

---

## 対象ファイル

| ファイル | 内容 |
|---|---|
| `site/content/docs/runtime/parallel.mdx` | `par` stage 並列実行・`Merge.ordered`/`Merge.any`・バックプレッシャー |
| `site/content/docs/tools/bench-regression.mdx` | `fav bench --compare` による差分回帰検出の使い方 |

**注意**: `site/content/docs/runtime/` ディレクトリは未存在のため新規作成が必要。

---

## 各記事の概要

### `runtime/parallel.mdx`

- `par [A, B] |> Merge` 構文の基本説明
- `Merge.ordered`（全完了後に順序通り結合）vs `Merge.any`（完了順に結合）の使い分け
- バックプレッシャー設定（`fav.toml` の `[stream] buffer_size`）
- 実用的な Favnir コード例
- 内部実装（`std::thread::spawn` ベース）の補足説明

### `tools/bench-regression.mdx`

- `fav bench --all` の出力形式の説明
- `--compare <baseline.json>` による差分比較フロー
- `--fail-on-regression` / `--threshold` フラグの CI 活用方法
- `benchmarks/` ディレクトリの管理ポリシー
- CLI 出力例（+50% WARN / -5% OK 等）

---

## テスト仕様

### `docs_parallel_page_exists`

```rust
let src = include_str!("../../site/content/docs/runtime/parallel.mdx");
assert!(src.contains("par"), "parallel.mdx must mention par");
assert!(src.contains("Merge"), "parallel.mdx must mention Merge");
assert!(src.contains("buffer_size"), "parallel.mdx must mention buffer_size");
```

`include_str!` のパス: `fav/src/driver.rs` から `../../site/content/docs/runtime/parallel.mdx`

### `docs_bench_regression_page_exists`

```rust
let src = include_str!("../../site/content/docs/tools/bench-regression.mdx");
assert!(src.contains("--compare"), "bench-regression.mdx must mention --compare");
assert!(src.contains("--fail-on-regression"), "bench-regression.mdx must mention --fail-on-regression");
```

`include_str!` のパス: `fav/src/driver.rs` から `../../site/content/docs/tools/bench-regression.mdx`

---

## テスト数

- ベース: 3130（v51.7.0 完了時点）
- `cargo_toml_version_is_51_7_0` 削除: -1
- 新規追加: +2（`docs_parallel_page_exists` + `docs_bench_regression_page_exists`）
- **完了後合計: 3131 tests passed, 0 failed**

---

## 完了条件

- `site/content/docs/runtime/parallel.mdx` が作成され、`par` / `Merge` / `buffer_size` を含む
- `site/content/docs/tools/bench-regression.mdx` が作成され、`--compare` / `--fail-on-regression` を含む
- `fav/Cargo.toml` version → `"51.8.0"`
- `cargo test` 3131 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v51.8.0 エントリ追加
- `versions/current.md` を v51.8.0（3131 tests）に更新
- `roadmap-v51.1-v52.0.md` の v51.8.0 実績欄を更新
