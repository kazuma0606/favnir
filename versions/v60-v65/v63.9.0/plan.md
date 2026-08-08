# v63.9.0 Plan — 安定化・Scale チェックリスト

Version: 63.9.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v63900_tests` 追加のみ |

新規公開関数なし。既存の `cmd_run_with_cache` / `cmd_parallel_stats` / `cmd_opt_stats` を統合テストで呼び出す。

---

## 実装ステップ

### Step 1: `v63900_tests` 追加

`v63800_tests` の直前に挿入:

- `scale_e2e_incremental_par`
  - `tempfile::tempdir()` で一時キャッシュディレクトリを作成
  - `cmd_run_with_cache(src, cache_dir)` を 2 回呼び出し: "ok" → "cache hit:"
  - `cmd_parallel_stats("")` で "parallel stats:" と "effective=" を確認

- `scale_dag_opt_dead_and_fused`
  - dead stage（Dead）+ 連続 pure stage（Pure1, Pure2）を含む pipeline ソース
  - `cmd_opt_stats(src)` で "Dead", "eliminated", "fused", "Pure1" を確認

### Step 2: ビルド・テスト全件確認

- `cargo build` でコンパイルエラー 0
- `cargo test --bin fav v63900_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3427 tests passed, 0 failed

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `scale_e2e_incremental_par` | cache miss → cache hit + parallel stats 有効 |
| `scale_dag_opt_dead_and_fused` | dead stage 検出 + pure fusion 検出 |

ベース: 3425 → 目標: 3427（+2）
