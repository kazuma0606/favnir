# v28.4.0 Spec — `fav profile` 強化（`--compare` フラグ追加）

## 概要

既存の `fav profile`（v9.9.0 で追加、profiler/ モジュール）に
`--compare <version>` フラグを追加し、`benchmarks/vX.Y.Z.json` に記録された
ベースライン stage 別実行時間と現在の計測値を比較して、
劣化 stage を `[SLOWER]` でハイライトした比較レポートを出力する。

> **既実装確認**: `--format flamegraph`（SVG 生成、inferno 統合）/ `--format table`（テーブル出力）/
> `--format text` / `--format json` は v24.3.0〜v28.3.0 時点で実装済み。
> ロードマップ v28.4 記載の `--format flamegraph`（inferno 統合）は v24.3.0 以前の実装で完了済み。
> v28.4.0 の追加スコープは `--compare` フラグのみ。

> **スタブ制約**: 現行の `benchmarks/*.json` は `test_count` / `version` / `timestamp` のみ保持し
> stage 別 ms データを持たない。そのため `--compare` 実行時は `extract_profile_stages` が
> 空マップを返し、全 stage が `[NEW]` マーカーで出力される（設計上の制約）。
> stage 別ベースラインデータは v28.x 以降で `benchmarks/*.json` フォーマット拡張時に対応予定。

> **`cmd_bench_compare` との設計判断**: `cmd_profile_compare` は `cmd_bench_compare`（v24.3.0）と異なり
> stdout を直接出力する（CLI 直結性を優先。テスト可能性よりも使い勝手を重視した設計）。

## ロードマップ参照

`versions/roadmap/roadmap-v28.1-v29.0.md` — v28.4 セクション

## 実装内容

### T1 — Cargo.toml バージョン bump
`28.3.0` → `28.4.0`

### T2 — driver.rs: `cmd_profile_compare` 追加

`fav/src/driver.rs` に `pub fn cmd_profile_compare` を追加。
`cmd_bench_compare`（v24.3.0）の実装パターンを参照。

```rust
/// fav profile --compare <baseline_version> <path>
/// benchmarks/{baseline_version}.json の stage 別 ms データと
/// 現在のプロファイル計測値を比較してレポートを出力する。
pub fn cmd_profile_compare(baseline_version: &str, path: &str) {
    // 1. benchmarks/{baseline_version}.json を読み込む
    // 2. path を compile_profiled_str でコンパイル & 実行
    // 3. 各 stage の ms を比較し [SLOWER] / [FASTER] / [NEW] を付ける
    // 4. 比較レポートを stdout に出力
}
```

比較レポートの出力形式:
```
fav profile --compare v28.3.0 src/etl.fav

Stage           Baseline (ms)   Current (ms)   Diff      Mark
──────────────────────────────────────────────────────────────
ExtractOrders       12             18           +50.0%   [SLOWER]
TransformOrders      8              7           -12.5%   [FASTER]
LoadWarehouse       25             25            +0.0%
NewStage             -             10              -     [NEW]
```

### T3 — main.rs: `--compare` フラグ追加

既存 `Some("profile")` アームに `--compare` フラグを追加。
`--compare` がある場合は `cmd_profile_compare(version, path)` にディスパッチ。

```rust
} else if let Some(v) = arg.strip_prefix("--compare=") {
    compare = Some(v.to_string()); i += 1;
} else if arg == "--compare" {
    compare = args.get(i + 1).cloned(); i += 2;
}
// ...dispatch:
if let Some(ref v) = compare {
    cmd_profile_compare(v, &path);
} else {
    cmd_profile(&path, &format, runs, stage_filter.as_deref(), out.as_deref());
}
```

### T4 — テストフィクスチャ: `fav/tests/fixtures/etl.fav` 新規作成

ロードマップ参照: `fav profile --format flamegraph tests/fixtures/etl.fav`

```favnir
// fav/tests/fixtures/etl.fav — profile テスト用 ETL フィクスチャ (v28.4.0)
stage ExtractOrders: Unit -> Unit = |_| { unit }
stage TransformOrders: Unit -> Unit = |_| { unit }
stage LoadWarehouse: Unit -> Unit = |_| { unit }

seq EtlPipeline = ExtractOrders |> TransformOrders |> LoadWarehouse
```

### T5 — ドキュメント更新: `profiling.mdx`

`site/content/docs/performance/profiling.mdx` に `--compare` セクションを追加。

### T6 — CHANGELOG 更新
`CHANGELOG.md` に `[v28.4.0]` セクション追加。

### T7 — ベンチマーク
`benchmarks/v28.4.0.json` 新規作成（test_count: 2262）。

### T8 — driver.rs テスト（Phase 9b）
`v284000_tests` モジュール（9 件）を `driver.rs` に追加。

### T9 — テスト全通過確認
`cargo test --bin fav` で 2262 tests PASS。

## エラー処理

| ケース | 対応 |
|---|---|
| `benchmarks/{version}.json` が存在しない | `eprintln!("error: benchmark file not found: benchmarks/{version}.json"); process::exit(1)` |
| JSON parse 失敗 | `eprintln!("error: cannot parse benchmarks/{version}.json"); process::exit(1)` |
| `.fav` ファイルが存在しない | `cmd_profile` 既存のエラー処理に委譲 |
| stage 名が一致しない | `[NEW]` マーカーで出力（エラーではない） |
| `baseline_version` に不正文字（`/`, `\n`, `\r` 等）が含まれる | サニタイズして使用（`replace('\n', "").replace('\r', "").replace('/', "")` 等） |

## テスト数

- v28.3.0: 2253 tests
- v28.4.0: **2262 tests**（+9）

## 完了条件

- [ ] `Cargo.toml` version = "28.4.0"
- [ ] `fav/src/driver.rs` に `pub fn cmd_profile_compare` あり
- [ ] `fav/src/driver.rs` の `cmd_profile_compare` が `"benchmarks/"` を参照
- [ ] `fav/src/driver.rs` の `cmd_profile_compare` が `[SLOWER]` マーカーを含む
- [ ] `fav/src/driver.rs` の `cmd_profile_compare` が `[FASTER]` マーカーを含む
- [ ] `fav/src/driver.rs` の `cmd_profile_compare` が `[NEW]` マーカーを含む
- [ ] `fav/src/main.rs` に `--compare` フラグ処理あり
- [ ] `fav/tests/fixtures/etl.fav` 存在（`EtlPipeline` seq 含む）
- [ ] `site/content/docs/performance/profiling.mdx` に `--compare` の記述あり
- [ ] `CHANGELOG.md` に `[v28.4.0]` セクションあり
- [ ] `benchmarks/v28.4.0.json` 存在（test_count: 2262）
- [ ] `cargo test --bin fav v284000` — 9/9 PASS
- [ ] `cargo test --bin fav` — 2262 tests PASS
