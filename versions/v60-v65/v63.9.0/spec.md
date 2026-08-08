# v63.9.0 Spec — 安定化・Scale チェックリスト

Version: 63.9.0
Status: 未着手

---

## 概要

v63.1〜v63.8 の全機能が統合されていることを確認する安定化スプリント。

新規の公開関数追加はない。`driver.rs` に `v63900_tests` を追加し、以下の統合動作を確認する:

1. **インクリメンタルキャッシュ + 多段パイプライン**: `cmd_run_with_cache` + `cmd_parallel_stats` が協調動作すること
2. **DAG 最適化 + par 対応パイプライン**: `cmd_opt_stats` が dead stage 除去と pure stage fusion を正しく報告すること

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3425 tests passed, 0 failed を確認
- `driver.rs` に `cmd_run_with_cache` が存在することを確認（v63.2.0 実装済み）
- `driver.rs` に `cmd_parallel_stats` が存在することを確認（v63.4.0 実装済み）
- `driver.rs` に `cmd_opt_stats` が存在することを確認（v63.7.0 実装済み）
- `driver.rs` に `v63800_tests` が存在することを確認（`v63900_tests` の挿入位置確認）
- `driver.rs` に `v63900_tests` が存在しないことを確認（新規追加）

**ロードマップとの差異（重要）**:
- ロードマップ原案のテスト数（base=3422）は実際より古い値。v63.6.0 で code-reviewer 対応により +3 件追加（+2 → +5）されたため、v63.8.0 時点の実際の累積テスト数は 3425。ロードマップ推移表（行 289）は 3427 を正しく記録しているが、v63.9.0 セクション本文（行 241）の base=3422 は誤りであり実装後に修正する。
- W040/W041 の mode-gating 確認: `lint_program_with_config`（lint.rs 行 62-74）の `if config.perf || config.strict { check_w041... }` ガードにより、構造的に normal モードでは W041 が発火しないことが保証されている。v63600_tests の `lint_w041_large_collect`（perf=true で発火確認）と `lint_w041_no_false_positive_with_filter`（filter あり発火なし確認）で perf モード動作は検証済み。「normal モード非発火」の明示的なテストは本バージョンの非スコープとする（コード構造で保証）。
- E0428 の確認は後送り（v64.0 以降）。

---

## 実装スコープ

### `driver.rs` — `v63900_tests` 追加

`v63800_tests` の直前に挿入:

```rust
// -- v63900_tests (v63.9.0) -- 安定化・Scale チェックリスト --
#[cfg(test)]
mod v63900_tests {
    #[test]
    fn scale_e2e_incremental_par() {
        // 多段パイプライン（par ワークロードを想定した 3 ステージ）
        let src = concat!(
            "public stage LoadCsv: Int -> Int = |x| { x }\n",
            "public stage Transform: Int -> Int = |x| { x + 1 }\n",
            "public stage Insert: Int -> Int = |x| { x * 2 }\n",
            "pipeline Etl {\n",
            "    step \"load\" = seq LoadCsv\n",
            "    step \"transform\" = seq Transform after \"load\"\n",
            "    step \"insert\" = seq Insert after \"transform\"\n",
            "}"
        );
        let dir = tempfile::tempdir().expect("tempdir");
        let cache_dir = dir.path().to_str().unwrap();

        // 初回: キャッシュミス → パース成功 ("ok")
        let r1 = crate::driver::cmd_run_with_cache(src, cache_dir);
        assert_eq!(r1, "ok", "first run should be cache miss + parse ok: {}", r1);

        // 2 回目: キャッシュヒット
        let r2 = crate::driver::cmd_run_with_cache(src, cache_dir);
        assert!(r2.contains("cache hit"), "second run should be cache hit: {}", r2);

        // parallel_stats: 有効スレッド数 >= 1
        let pstats = crate::driver::cmd_parallel_stats("");
        assert!(pstats.contains("parallel stats:"), "expected stats header: {}", pstats);
        assert!(pstats.contains("effective="), "expected effective thread count: {}", pstats);
    }

    #[test]
    fn scale_dag_opt_dead_and_fused() {
        // dead stage + 連続 pure stage（fusion 対象）を含む par 対応パイプライン
        let src = concat!(
            "public stage Live: Int -> Int = |x| { x }\n",
            "public stage Dead: Int -> Int = |x| { x + 99 }\n",
            "public stage Pure1: Int -> Int = |x| { x * 2 }\n",
            "public stage Pure2: Int -> Int = |x| { x + 1 }\n",
            "pipeline P {\n",
            "    step \"live\" = seq Live\n",
            "    step \"p1\" = seq Pure1 after \"live\"\n",
            "    step \"p2\" = seq Pure2 after \"p1\"\n",
            "}"
        );
        let out = crate::driver::cmd_opt_stats(src);
        // Dead は pipeline 未参照 → eliminated
        assert!(out.contains("Dead"), "dead stage Dead should be reported: {}", out);
        assert!(out.contains("eliminated"), "output should mention eliminated: {}", out);
        // Pure1 -> Pure2 は連続 pure → fused
        assert!(out.contains("fused"), "consecutive pure stages should be fused: {}", out);
        assert!(out.contains("Pure1"), "fused output should mention Pure1: {}", out);
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v63900_tests` で 2 件 PASS
  - `scale_e2e_incremental_par` PASS
  - `scale_dag_opt_dead_and_fused` PASS
- `cargo test -j 8 -- --test-threads=8` で 3427 tests passed, 0 failed

---

## 非スコープ

- W040/W041 normal モード非発火の明示的テスト（`lint_program_with_config` のガード構造で保証済み）
- E0428 の表示・消去確認（v64.0 以降に後送り）
- `fav/benchmarks/` への `.fav` ファイル追加（v64.0 以降に後送り）
- `fav bench --suite` CLI フラグ（v64.0 以降に後送り）

---

## 技術ノート

### `cmd_run_with_cache` の挙動

- 同一 `src` に対して `stage_hash` が同一となるため 2 回目は必ずキャッシュヒット
- `tempfile::tempdir()` を使って一時ディレクトリを確保（テスト間の隔離）

### `cmd_opt_stats` の出力フォーマット

- `[optimizer] stage \`Dead\` has no downstream consumers — eliminated`
- `[optimizer] stages \`Pure1 -> Pure2\` fused (all pure) — 1 stage emitted`
- `optimizer: N eliminated, N fused`

### `opt_is_pure_stage` の判定ロジック

`opt_block_has_effect_call` で AST を走査し、`Io` / `Http` / `Db` / `Kafka` / `S3` / `Sqs` / `Slack` / `Email` / `Llm` / `Snowflake` / `Postgres` いずれかの名前空間への FieldAccess が存在しないステージを pure と判定する。テスト用ステージ `Pure1: Int -> Int = |x| { x * 2 }` はエフェクト呼び出しを含まないため pure 扱いになる。

### ベーステスト数の差異

ロードマップ記載: base=3422、target=3424。
実際: v63.6.0 code-reviewer 対応 +3 件等により base=3425、target=3427。
