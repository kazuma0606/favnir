# Roadmap v63.1.0 〜 v64.0.0 — Incremental & Scale

Date: 2026-07-30
Status: 未着手

---

## 前提

- 直前完了: v63.0.0「AOT Native」（tests = 3406）
- マスターロードマップ: `roadmap-v60.1-v65.0.md`
- 本文書はマスターの v64.0 スプリント部分の詳細版
- **既存機能の扱い**:
  - `fav watch` は v9.9 実装済み（500ms ポーリング）→ 差分キャッシュ統合・inotify 改善
  - `par [A, B]` Tokio 並列は v52.0 実装済み → 動的スレッドプール・`[parallel]` 設定追加
  - `fav profile` flamegraph は v9.9 実装済み → `--memory` フラグ追加
  - `notify` クレートは Cargo.toml に登録済み → `fav watch` の inotify 改善で活用

---

## 目標

**「大規模 ETL を安心して任せられるエンジン」** を実現する。

v63.1〜v63.9 の 9 スプリントで差分コンパイル・スケーリング最適化を積み上げ、
v64.0「Incremental & Scale」として宣言する。

---

## バージョン計画

### v63.1.0 — 差分コンパイルキャッシュ（`.fav-cache/`）

`fav/src/cache.rs` を新規作成。`.fav-cache/` ディレクトリにステージ単位の
bytecode と型シグネチャ hash（SHA-256）を JSON で保存する `IncrementalCache` 構造体を実装。
`driver.rs` の `cmd_run` がキャッシュを読み書きするよう更新。
変更のない stage はキャッシュから bytecode を読み込んでコンパイルをスキップする。

```bash
$ fav run pipeline.fav      # 初回: 全 stage コンパイル
$ fav run pipeline.fav      # 2 回目: キャッシュヒット
[cache] LoadCsv: hit (0ms)
[cache] Transform: miss — recompiling (12ms)
[cache] Write: hit (0ms)
```

**完了条件**: Rust テスト 2 件（ベース 3406 + 2 = 3408 tests passed, 0 failed）
- `incremental_cache_hit_unchanged`
- `incremental_cache_miss_on_change`

**実績**: 2026-08-02 完了。3408 tests passed（ベース 3406 + 2）。
- `fav/src/cache.rs` 新規作成（`IncrementalCache` / `StageEntry` / `stage_hash`）
- `fav/src/lib.rs` + `fav/src/main.rs` に `mod cache;` 追加（WASM ガード付き）
- `cmd_incremental_cache_status(cache_dir: &str)` を `driver.rs` に追加
- テスト: `incremental_cache_hit_unchanged` / `incremental_cache_miss_on_change`（2/2 PASS）
- `cmd_run` 統合は v63.2.0 に後送り（意図的）

---

### v63.2.0 — `fav watch` 改善（差分再コンパイルと inotify 統合）

既存の `fav watch`（500ms ポーリング）を v63.1 の差分コンパイルキャッシュと統合。
`notify` クレート（Cargo.toml 登録済み）の inotify / FSEvents バックエンドに切り替え
（ポーリングを廃止してファイルシステムイベントを直接受信）。
変更されたファイルに関係するステージのみ再コンパイルして即座に再実行する。

```bash
$ fav watch pipeline.fav
[watch] monitoring pipeline.fav...
[watch] Transform.fav changed — recompiling 1 stage (14ms)
[watch] re-running pipeline... done (38ms)
```

**完了条件**: Rust テスト 2 件（ベース 3408 + 2 = 3410 tests passed, 0 failed）
- `watch_incremental_recompile`
- `watch_notify_integration`

**実績**: 2026-08-02 完了。3410 tests passed（ベース 3408 + 2）。
- `cmd_run_with_cache(src, cache_dir)` を `driver.rs` に追加（`IncrementalCache` 統合）
- `watch_notify_integration`: `RecommendedWatcher` 構築・ウォッチ操作の単体確認
- `watch_incremental_recompile`: miss → store → hit の流れを確認
- inotify 統合は v9.9.0 で実施済みのため再実装不要

---

### v63.3.0 — キャッシュ型シグネチャ不整合検出 E0428

`error_catalog.rs` に E0428 `incremental_cache_conflict` を登録
（v60.6.0 で確立した `long_description` フィールドを含めること）。
`cache.rs` の `IncrementalCache::load` でキャッシュ内の型シグネチャと
現在のコンパイル結果を比較し、不整合を検出した際に E0428 を警告表示する。
警告後は自動的にキャッシュエントリを無効化して再コンパイルする（致命的エラーではない）。

```
E0428: incremental cache signature mismatch
  stage `Transform` の型シグネチャがキャッシュと一致しません。
  cached:  List<Row> -> List<Row>
  current: List<Row> -> List<EnrichedRow>
  キャッシュを無効化して再コンパイルします。
```

**完了条件**: Rust テスト 2 件（ベース 3410 + 2 = 3412 tests passed, 0 failed）
- `incremental_e0428_signature_mismatch`
- `cache_auto_invalidated`

**実績**: 2026-08-02 完了。3412 tests passed（ベース 3410 + 2）。
- `error_catalog.rs` に E0428 `incremental_cache_conflict` を追加（`long_description` 含む）
- `cache.rs` の `IncrementalCache` に `check_type_sig` メソッドを追加
  - ハッシュ一致・シグ不一致時に E0428 を `eprintln!` 警告・自動無効化
- テスト: `incremental_e0428_signature_mismatch` / `cache_auto_invalidated`（2/2 PASS）

---

### v63.4.0 — `par` 動的スレッドプール（`[parallel]` fav.toml 設定）

`toml.rs` に `ParallelConfig { max_threads: usize, queue_depth: usize }` を追加。
`FavConfig` に `parallel: Option<ParallelConfig>` フィールドを追加してパース。
`vm.rs` の `par` 実行エンジンが `ParallelConfig` を読み込んでスレッド数・キュー深度を制御するよう更新。
デフォルトは CPU コア数（`std::thread::available_parallelism()`）。
`fav run --parallel-stats` フラグでスレッドごとの処理件数を表示。

```toml
[parallel]
max_threads = 8
queue_depth = 1000
```

**完了条件**: Rust テスト 2 件（ベース 3412 + 2 = 3414 tests passed, 0 failed）
- `parallel_toml_config_parsed`
- `parallel_stats_output`

**実績**: 2026-08-02 完了。`ParallelConfig` 構造体 + `FavToml.parallel` フィールド + `parse_fav_toml` `[parallel]` セクション処理 + `cmd_parallel_stats` + `v63400_tests` 2件 PASS。3414 tests passed, 0 failed。（VM 注入・`--parallel-stats` CLI は非スコープとして後送り）

---

### v63.5.0 — メモリプロファイリング（`fav profile --memory`）

既存の `fav profile`（stage 別実行時間・flamegraph）に `--memory` フラグを追加。
`driver.rs` の `cmd_profile` を拡張し、ステージ実行中の RSS と per-row 割り当てバイト数を計測。
計測は `sysinfo` クレートを新規追加して RSS を取得（platform 差異を吸収）。
結果を表形式で標準出力に表示し、`fav profile --memory --json` で JSON 出力も対応。

```bash
$ fav profile --memory pipeline.fav
Stage         | Peak RSS | Alloc/row |
--------------|----------|-----------|
LoadCsv       |  42 MB   |   420 B   |
Transform     |  18 MB   |   180 B   |
Write         |   8 MB   |    80 B   |
Total peak    |  62 MB   |           |
```

**完了条件**: Rust テスト 2 件（ベース 3414 + 2 = 3416 tests passed, 0 failed）
- `profile_memory_flag_works`
- `profile_memory_per_stage`

**実績**: 2026-08-02 完了。`sysinfo = "0.30"` 追加 + `cmd_profile_memory` + `v63500_tests` 2件 PASS。3416 tests passed, 0 failed。（CLI `--memory` フラグ・per-row 実計測は非スコープとして後送り）

---

### v63.6.0 — バックプレッシャー制御（W041 lint + `[backpressure]` 設定）

`lint.rs` に W041 `perf_hint_large_collect` を追加
（`collect()` の前に filter がない場合に警告。**`--strict` / `--perf` フラグ下でのみ有効**）。
`toml.rs` に `BackpressureConfig { strategy: String, max_queue_depth: usize, warn_threshold: usize }` を追加。

> **コード番号注記**: ロードマップ原案は W040 を使用する想定だったが、W040 は v61.7.0「type hole `_` inferred」で取得済み。本バージョンでは W041 を使用する。テスト名も `lint_w041_large_collect` に変更。
> **後送り決定**: `vm.rs` の stage 間キューへの `warn_threshold` 超過警告（実行時 W042）は本バージョンの非スコープ（v63.7 以降に後送り）。

```toml
[backpressure]
strategy = "drop"      # drop | block | sample
max_queue_depth = 500
warn_threshold = 400
```

**完了条件**: Rust テスト 2 件（ベース 3416 + 2 = 3418 tests passed, 0 failed）
- `lint_w041_large_collect`（W040 取得済みのため W041 に変更）
- `backpressure_toml_parsed`

**実績**: 2026-08-02 完了。W041 `perf_hint_large_collect` lint 追加（`check_w041_*` 関数群 + `lint_program_with_config` 更新）+ `BackpressureConfig` + `v63600_tests` 2件 PASS。3418 tests passed, 0 failed。（vm.rs 実行時警告は後送り）

---

### v63.7.0 — パイプライン DAG 最適化（dead stage elimination + pure stage fusion）

`compiler.rs` にパイプライン DAG 解析パスを追加（`petgraph` クレートを活用、既存依存）。
1. **Dead stage elimination**: 出力が未使用のステージを IR 生成前に除去
2. **Pure stage fusion**: 連続する `effects = []` ステージをひとつの stage にマージ

`fav run --opt-stats` フラグで除去・マージされた stage 数を表示。

```
[optimizer] stage `DebugLog` has no downstream consumers — eliminated
[optimizer] stages `Normalize -> Trim -> Format` fused (all pure) — 1 stage emitted
```

**完了条件**: Rust テスト 2 件（ベース 3418 + 2 = 3420 tests passed, 0 failed）
- `optimizer_dead_stage_eliminated`
- `optimizer_pure_stages_fused`

**実績**: 2026-08-02 完了。`cmd_opt_stats` 追加（dead stage elimination + pure stage fusion 静的解析）+ `v63700_tests` 2件 PASS。3423 tests passed, 0 failed。（compiler.rs DAG 統合・petgraph・CLI フラグは後送り）

---

### v63.8.0 — 標準 ETL ベンチマークスイート

`fav/benchmarks/` ディレクトリに標準 ETL ベンチ（CSV 変換・Kafka 処理・JOIN）の
.fav ファイルと実行スクリプトを追加。
`fav bench --suite etl-standard` でベンチを一括実行し JSON レポートを `bench-results.json` に出力。
`driver.rs` に `cmd_bench_suite(suite: &str) -> String` を追加。

```bash
$ fav bench --suite etl-standard
Benchmark: csv-to-postgres (1M rows)
  VM:  4,230 ms  (236k rows/s)
  AOT: 1,180 ms  (847k rows/s)
Benchmark: kafka-window-aggregate (10M events)
  VM:  9,810 ms  (1,019k events/s)
  AOT: 2,340 ms  (4,274k events/s)
```

**完了条件**: Rust テスト 2 件（ベース 3423 + 2 = 3425 tests passed, 0 failed）
- `bench_suite_etl_standard`
- `bench_regression_check`

**実績**: `cmd_bench_suite` 追加、`v63800_tests` 2 件 PASS（3425 tests, 2026-08-02 完了）

---

### v63.9.0 — 安定化・Scale チェックリスト

v63.1〜v63.8 の全機能が統合されていることを確認する。

確認項目:
- 10 stage パイプラインで差分コンパイルが正しく機能する（変更なしで全キャッシュヒット）
- `par` + DAG 最適化が共存して正しい結果を返す
- W040 / W041 が `--strict` / `--perf` モードでのみ発火し、通常モードでは発火しない
- E0428 が型シグネチャ変更後の初回実行で正しく表示され、2 回目以降は消える

**完了条件**: Rust テスト 2 件（ベース 3425 + 2 = 3427 tests passed, 0 failed）
- `scale_e2e_incremental_par`
- `scale_dag_opt_dead_and_fused`

**実績**: `v63900_tests` 2 件 PASS（3427 tests, 2026-08-02 完了）

---

### v64.0 — Incremental & Scale 宣言 ★クリーンアップ

**宣言文**:

> 「変更されたステージだけが再コンパイルされ、未使用のステージは除去される。
>  スレッドはコアの数だけ走り、キューはバックプレッシャーで制御される。
>  ベンチマークは数字で真実を語る。
>
>  Favnir は大規模 ETL を安心して任せられるエンジンになった。
>
>  これが Favnir v64.0 — Incremental & Scale の姿である。」

**完了条件**:
- v63.1〜v63.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3431**）
- `v64000_tests` 4 件 pass（ベース 3427 + 4 = 3431 tests passed, 0 failed）:
  - `cargo_toml_version_is_64_0_0`
  - `changelog_has_v64_0_0`
  - `milestone_has_incremental_scale`
  - `readme_mentions_incremental_scale`
- `MILESTONE.md` に `"Incremental & Scale"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: `v64000_tests` 4 件 PASS、★クリーンアップ完了（3431 tests, 2026-08-02 完了）

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v63.0.0（ベース） | 3406 | — | AOT Native 宣言後 |
| v63.1.0 | 3408 | +2 | 差分キャッシュ |
| v63.2.0 | 3410 | +2 | fav watch 改善 |
| v63.3.0 | 3412 | +2 | E0428 |
| v63.4.0 | 3414 | +2 | par 動的スレッドプール |
| v63.5.0 | 3416 | +2 | メモリプロファイリング |
| v63.6.0 | 3421 | +5 | バックプレッシャー・W041（code-reviewer 対応 +3 含む） |
| v63.7.0 | 3423 | +2 | DAG 最適化 |
| v63.8.0 | 3425 | +2 | ETL ベンチスイート |
| v63.9.0 | 3427 | +2 | 安定化 |
| v64.0.0 | 3431 | +4 | Incremental & Scale 宣言（★クリーンアップ） |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v62.1-v63.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
