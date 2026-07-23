# Roadmap v51.1.0 〜 v52.0.0 — Performance & Scale

Date: 2026-07-18
Status: 計画中（v51.0 完了後に開始）

---

## 前提

- 直前完了: v51.0.0「Developer Experience 3.0」（tests ≥ 3113）
- マスターロードマップ: `roadmap-v50.1-v55.0.md`
- 本文書はマスターの v52.0 スプリント部分の詳細版
- **既存機能の扱い**: `par [A, B]` の AST（`FlwStep::Par`）は既存実装済み。
  v51.1 は「新構文追加」ではなく「Tokio 実行基盤への置換・streaming 対応」。
  詳細はマスターロードマップ冒頭「既存機能との位置づけ」テーブルを参照。

---

## 目標

v51.0 で診断・エディタ統合を確立した。このスプリントでは
**「`par` 並列実行の Tokio 化・バックプレッシャー・ベンチマーク回帰検出」**
を実装して Favnir v52.0 を宣言する。

---

## バージョン計画

### v51.1.0 — `par` stage Tokio 並列実行基盤への置換 Phase 1

`ast.rs` の `FlwStep::Par` はすでに存在するが、VM の実行は逐次またはスタブ。
`ir.rs` に `Par` opcode を追加し、`compiler.rs` で `par [...]` → `Par` opcode を emit。
VM で `tokio::spawn` を使い `Par` の各要素を並列実行・join。エラーは fail-fast。

```favnir
pipeline IngestPipeline {
  stage Process: Order -> EnrichedOrder = |order| {
    par [Enrich(order), Validate(order)] |> Merge
  }
}
```

**完了条件**: Rust テスト 2 件（実績推定 3115 tests passed, 0 failed）

**実績（2026-07-19 COMPLETE）**: 3115 tests passed, 0 failed — `IRExpr::Par` + `Opcode::ParStages` 追加、VM 直接実行（std::thread::spawn）、fail-fast on Result.err
- `par_stage_runs_parallel`
- `par_stage_error_propagation`

---

### v51.2.0 — `par` Phase 2（Merge.ordered / Merge.any + streaming 対応）

`Merge.ordered`（`tokio::join_all` 相当）と `Merge.any`（`FuturesUnordered` 相当）を実装。
`ast.rs` に `MergeMode` enum を追加。`Stream<T>` を返す stage にも `par` が適用できることを確認。

```favnir
// 全完了後に順序通りに結合
par [StageA, StageB, StageC] |> Merge.ordered

// 完了した順に結合（streaming フレンドリー）
par [StageA, StageB, StageC] |> Merge.any
```

**完了条件**: Rust テスト 2 件（実績推定 3117 tests passed, 0 failed）
- `par_stage_merge_ordered`
- `par_stage_merge_unordered`

**実績（2026-07-19 COMPLETE）**: 3117 tests passed, 0 failed — `MergeMode { Ordered, Any }` enum 追加、`FlwStep::Merge(MergeMode)` 追加、`IO.merge_ordered_raw` / `IO.merge_any_raw` VM ハンドラ実装
- `par_stage_merge_ordered`
- `par_stage_merge_unordered`
- Stream<T> 対応はスコープ外（v51.3.0 以降）

---

### v51.3.0 — ストリーミングバックプレッシャー制御

`fav.toml` の `[stream]` セクションを解析し `buffer_size` を VM のストリームバッファに適用。
Tokio の `mpsc::channel` で bounded channel を実装し、producer をブロック制御する。

```toml
# fav.toml
[stream]
buffer_size = 1000   # producer が 1000 件溜まったらブロック
```

```favnir
stage Consume: Stream<RawOrder> -> Stream<Order> = |raw| {
  bind order <- kafka.consume("orders")
  Ok(order)  // バッファ満杯時は自動的にブロック
}
```

**完了条件**: Rust テスト 2 件（実績推定 3119 tests passed, 0 failed）
- `stream_backpressure_blocks`
- `stream_buffer_size_config`

**実績（2026-07-19 COMPLETE）**: 3119 tests passed, 0 failed — `StreamConfig.buffer_size: Option<usize>` 追加、`VM::run_with_stream_buffer_size` 追加、`__streaming_pipeline` バックプレッシャー対応（`compiled_chunk_size.min(buf)`）
- `stream_buffer_size_config`
- `stream_backpressure_blocks`

---

### v51.4.0 — `fav bench` 差分回帰検出

`fav bench --compare <baseline.json>` フラグを追加。`benchmarks/` ディレクトリの前回結果と比較し、
閾値（デフォルト 10%）超過を警告。`--fail-on-regression` フラグで CI 向け非ゼロ終了コード。

```bash
$ fav bench --all --compare benchmarks/v51.3.0.json
checker_run_time:  1.2ms → 1.8ms  (+50%)  [WARN: exceeds 10% threshold]
compiler_run_time: 0.8ms → 0.9ms  (+12%)  [WARN]
vm_run_time:       2.1ms → 2.0ms  (-5%)   [OK]
```

**完了条件**: Rust テスト 2 件（実績推定 3121 tests passed, 0 failed）
- `bench_regression_detected`
- `bench_no_regression_passes`

**実績（2026-07-19 COMPLETE）**: 3121 tests passed, 0 failed — `BenchOpts.compare` / `fail_on_regression` / `threshold` 追加、`bench_stats_to_compare_json` 追加、`cmd_bench` → `bool` 戻り値変更、`--compare` / `--fail-on-regression` / `--threshold` CLI フラグ追加、`benchmarks/v51.3.0.json` 作成
- `bench_regression_detected`
- `bench_no_regression_passes`

---

### v51.5.0 — インクリメンタルコンパイル依存グラフ

ファイル間の import 依存グラフを構築し、変更ファイルの推移的依存ファイルのみ再コンパイル。
フィンガープリント（SHA-256）と依存グラフを `.fav-cache/dep-graph.json` に保存
（v49.3 のインクリメンタル型チェックをコンパイルフェーズにも拡張）。

```bash
# a.fav を変更した場合
$ fav build
[skip]    b.fav — unchanged
[skip]    c.fav — unchanged
[rebuild] a.fav — changed
[rebuild] d.fav — depends on a.fav
```

**完了条件**: Rust テスト 2 件（実績推定 3123 tests passed, 0 failed）
- `incremental_dep_graph_rebuilt`
- `incremental_transitive_invalidation`

**実績（2026-07-19 COMPLETE）**: 3124 tests passed, 0 failed — `DepGraph` に `Serialize`/`Deserialize` 追加、`transitive_affected_by` 追加（HashSet 重複管理）、`save_dep_graph_json` / `load_dep_graph_json` 追加（`#[cfg(not(wasm32))]`）、`incremental_files_to_rebuild` 追加、`dep_graph_json_roundtrip` テスト追加
- `incremental_dep_graph_rebuilt`
- `incremental_transitive_invalidation`

---

### v51.6.0 — checker / compiler ホットパス最適化

`fav profile --build` で checker と compiler の処理時間を段階別に計測。
型代入（`Subst`）のクローン頻度を削減する `SubstRef` 参照共有を導入。
`compiler.rs` の `collect_merged_sources` の重複読み込みをキャッシュで排除。
`benchmarks/v51.6.0.json` に計測結果を保存。

**完了条件**: Rust テスト 2 件（実績推定 3126 tests passed, 0 failed）
- `checker_perf_hot_path_improved`
- `compiler_perf_baseline_recorded`

**実績（2026-07-19 COMPLETE）**: 3126 tests passed, 0 failed — `SubstRef = Rc<Subst>` + `Subst::into_ref()` 追加（checker.rs）、`SourceCache` + `get_or_load` 追加（compiler_fav_runner.rs）、`fav profile --build` 追加（ProfileBuildResult / profile_build_file / cmd_profile_build）、`benchmarks/v51.6.0.json` 追加
- `checker_perf_hot_path_improved`
- `compiler_perf_baseline_recorded`

---

### v51.7.0 — WASM ビルドサイズ最適化

`wasm_dce.rs` の DCE（Dead Code Elimination）を強化し、未参照の export と内部関数を除去。
`wasm-opt -Os` 呼び出しを `build --target wasm` に統合。

```bash
$ fav build --target wasm
before: 412 KB
after:  287 KB  (-30%)
```

**完了条件**: Rust テスト 4 件（実績推定 3129 tests passed, 0 failed）
- `cargo_toml_version_is_51_7_0`
- `wasm_bundle_size_reduced`
- `wasm_dce_removes_unused_fns`
- `benchmark_json_exists`

**実績（2026-07-20 COMPLETE）**: 3130 tests passed, 0 failed — `WasmOptLevel::Os` 追加（`wasm_opt_pass.rs`）、`dce_from_exports` 追加（`wasm_dce.rs`、複数エントリ BFS union）、`cmd_build "wasm"` → `build_wasm_artifact_with_config(dce=true, Os)` 強化、`benchmarks/v51.7.0.json` 追加、code-review 対応: `dce_from_exports` 呼び出し修正・`os_flag_is_minus_os` テスト追加
- `wasm_dce_removes_unused_fns`
- `wasm_bundle_size_reduced`
- `benchmark_json_exists`
- `cargo_toml_version_is_51_7_0`

---

### v51.8.0 — ドキュメントサイト Performance 記事

`site/content/docs/runtime/parallel.mdx` — `par` stage の並列実行・マージモード・バックプレッシャー。
`site/content/docs/tools/bench-regression.mdx` — `fav bench --compare` による回帰検出の使い方。

**完了条件**: Rust テスト 2 件（実績推定 3131 tests passed, 0 failed）
- `docs_parallel_page_exists`
- `docs_bench_regression_page_exists`

**実績（2026-07-20 COMPLETE）**: 3131 tests passed, 0 failed — `site/content/docs/runtime/parallel.mdx` 作成（par / Merge.ordered / Merge.any / buffer_size）、`site/content/docs/tools/bench-regression.mdx` 作成（--compare / --fail-on-regression / --threshold）
- `docs_parallel_page_exists`
- `docs_bench_regression_page_exists`

---

### v51.9.0 — 安定化・コードフリーズ（Performance & Scale 前調整）

全 lint / clippy クリーン確認。`site/content/docs/performance-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（実績推定 3133 tests passed, 0 failed）
- `cargo_toml_version_is_51_9_0`
- `perf_overview_doc_exists`

**実績（2026-07-20 COMPLETE）**: 3133 tests passed, 0 failed — `site/content/docs/performance-overview.mdx` 作成（par / fav bench / Performance & Scale 言及）、全 clippy クリーン確認
- `cargo_toml_version_is_51_9_0`
- `perf_overview_doc_exists`

---

### v52.0.0 — Performance & Scale 宣言 ★クリーンアップ

**宣言文**:

> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

**完了条件**:
- v51.1〜v51.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3135**、実態推定 3136: 3133 - 1 + 4）
- `v52000_tests` 4 件 pass:
  - `cargo_toml_version_is_52_0_0`
  - `changelog_has_v52_0_0`
  - `milestone_has_performance_scale`
  - `readme_mentions_performance_scale`
- `MILESTONE.md` に `"Performance & Scale"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績（2026-07-20 COMPLETE）**: 3135 tests passed, 0 failed — `MILESTONE.md` に v52.0.0「Performance & Scale」エントリ追加、`README.md` に Performance & Scale 言及追加、`CHANGELOG.md` に v52.0.0 エントリ追加、`cargo clean` 完了、`code_freeze_v51_0_0` 番兵テスト削除（v51 スプリント終了）
- `cargo_toml_version_is_52_0_0`
- `changelog_has_v52_0_0`
- `milestone_has_performance_scale`
- `readme_mentions_performance_scale`

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v50.1-v51.0.md`
- 達成宣言: `MILESTONE.md`
