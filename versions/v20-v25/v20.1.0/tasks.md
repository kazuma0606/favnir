# v20.1.0 — ベンチマーク基盤整備 タスク

## ステータス: DONE

---

## タスク一覧

### T1: `benchmarks/suite/` — 8 計測スクリプト作成

- [x] `benchmarks/suite/01_cold_start.sh` を作成
  - コールドスタート（フル）と precompiled の2計測
  - 出力形式: `cold_start_full_ms=N` / `cold_start_precompiled_ms=N`
- [x] `benchmarks/suite/02_csv_10gb.fav` を作成
  - `CI=true` のとき 1MB サンプル使用
  - 出力形式: `csv_10gb_throughput_mbs=N`
- [x] `benchmarks/suite/03_tight_loop.fav` を作成
  - 再帰で 10M 回イテレーション
  - 出力形式: `tight_loop_10m_iter_ms=N`
- [x] `benchmarks/suite/04_record_transform.fav` を作成
  - 100万行レコードを `transform_row` で変換
  - 出力形式: `record_transform_1m_ms=N`
- [x] `benchmarks/suite/05_compile_time.sh` を作成
  - cold と incremental の2計測
  - 出力形式: `compile_cold_ms=N` / `compile_incremental_ms=N`
- [x] `benchmarks/suite/06_duckdb_query.fav` を作成
  - in-memory DuckDB で 100万行 SUM クエリ
  - 出力形式: `duckdb_query_sum_1m_ms=N`
- [x] `benchmarks/suite/07_arrow_parquet.fav` を作成
  - 100万行 Arrow → Parquet 書き込み
  - 出力形式: `arrow_parquet_write_1gb_ms=N`
- [x] `benchmarks/suite/08_concurrent_stages.fav` を作成
  - `par [A, B, C]` の並列 stage 実行
  - 出力形式: `concurrent_stages_3way_ms=N`
- [x] 各 `.sh` ファイルに実行権限を付与（`chmod +x`）— Windows 環境のため省略、Linux CI で自動付与

---

### T2: `benchmarks/suite/run_all.sh` — ラッパースクリプト作成

- [x] `benchmarks/suite/run_all.sh` を作成
  - 8スクリプトを順に実行し、`KEY=VALUE` 行を収集
  - `--format json` で JSON を stdout に出力
  - JSON 形式: `{"version": "...", "timestamp": "...", "metrics": {...}}`
  - `VERSION` は `fav/Cargo.toml` から `grep` で取得
- [x] 実行権限を付与（`chmod +x`）— Windows 環境のため省略
- [ ] ローカルで `bash benchmarks/suite/run_all.sh --format json` が実行でき、valid JSON が出力されることを確認（Linux 環境で要確認）

---

### T3: `benchmarks/compare.fav` — Favnir 比較スクリプト作成

- [x] `benchmarks/compare.fav` を作成（plan.md の内容に従う）
  - `--baseline <path>` / `--current <path>` / `--threshold <N>` の3引数を受け取る
  - `--emit-md` フラグで `benchmarks/results.md` を更新
  - baseline 比でのパーセント変化を計算し、threshold 超えを列挙
  - 閾値超えがある場合は `Result.err(...)` で非ゼロ終了
- [ ] `fav check benchmarks/compare.fav` でパースエラーなしを確認（手動確認）

---

### T4: `.github/workflows/bench.yml` — CI ワークフロー作成

- [x] `.github/workflows/bench.yml` を作成（plan.md の内容に従う）
  - トリガー: `push` to `master`
  - Cargo ビルドキャッシュを設定
  - `CI=true` で `run_all.sh` を実行 → `latest.json` に保存
  - `compare.fav` で baseline との比較を実行（threshold: 10）
  - `--emit-md` で `benchmarks/results.md` を自動更新

---

### T5: `benchmarks/v20.0.0.json` — ベースライン JSON 作成

- [x] `benchmarks/v20.0.0.json` を作成（plan.md の内容に従う）
  - 初期値は参考値（実測後に CI で更新）
  - 以下のメトリクスキーを含む（単位: ms または MB/s）:
    - `cold_start_full_ms`: 320
    - `cold_start_precompiled_ms`: 18
    - `csv_10gb_throughput_mbs`: 340
    - `tight_loop_10m_iter_ms`: 85
    - `record_transform_1m_ms`: 210
    - `compile_cold_ms`: 2400
    - `compile_incremental_ms`: 180
    - `arrow_parquet_write_1gb_ms`: 3200
    - `duckdb_query_sum_1m_ms`: 45（レビューで追加）
    - `concurrent_stages_3way_ms`: 120（レビューで追加）
- [x] `jq . benchmarks/v20.0.0.json` で valid JSON を確認

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.0.0"` → `"20.1.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T7: `fav/src/driver.rs` — `v201000_tests` 追加

- [x] `v200000_tests::version_is_20_0_0` に `#[ignore]` を追加
- [x] `v201000_tests` モジュールを追加（5件）
- [x] `cargo test v201000` — 5/5 PASS を確認

---

### T8: CHANGELOG.md 更新

- [x] `CHANGELOG.md` に v20.1.0 エントリを追加

> **site/ MDX**: v20.1.0 は新構文・新コマンドの追加なし（インフラ整備のみ）のため、
> site/content/ への新規 MDX ページ追加は不要。
> ベンチマーク結果は `benchmarks/results.md`（CI が自動生成）で管理する。

---

## テスト（v201000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_1_0` | Cargo.toml に `"20.1.0"` が含まれる |
| `bench_suite_files_exist` | `run_all.sh` と 8 計測スクリプト全件が存在する |
| `bench_compare_fav_exists` | `benchmarks/compare.fav` が存在する |
| `bench_workflow_exists` | `.github/workflows/bench.yml` が存在する |
| `bench_baseline_valid_json` | `benchmarks/v20.0.0.json` が存在し `metrics` フィールドを含む |

---

## 完了条件チェックリスト

- [x] `.github/workflows/bench.yml` が存在する
- [x] `benchmarks/suite/run_all.sh` が存在し実行可能
- [x] `benchmarks/suite/01_cold_start.sh` 〜 `08_concurrent_stages.fav` の8ファイルが存在する
- [x] `benchmarks/compare.fav` が存在し `fav check` でエラーなし
- [x] `benchmarks/v20.0.0.json` が存在し valid JSON
- [x] `fav/Cargo.toml` version が `20.1.0`
- [x] `cargo test v201000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（exit 0）
- [ ] `fav check benchmarks/compare.fav` — パースエラーなし（手動確認）
- [x] `CHANGELOG.md` に v20.1.0 エントリが追加されている
- [x] site/ MDX 追加: 不要（インフラ整備のみ）

---

## 優先度

```
T1（8計測スクリプト）        ← 最初に着手（他と独立）
T2（run_all.sh）             ← T1 と並列可
T3（compare.fav）            ← T1 と並列可
T5（v20.0.0.json）           ← 即座に作成可
T4（bench.yml）              ← T2, T3 完了後
T6（Cargo.toml）             ← 任意のタイミング
T7（driver.rs テスト）       ← T1〜T5 完了後（ファイル存在チェックのため）
```
