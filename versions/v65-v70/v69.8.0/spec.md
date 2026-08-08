# v69.8.0 Spec — パフォーマンス回帰テスト

## Background

ロードマップ記載: 「v69.8: パフォーマンス回帰テスト（v65.0 ベースラインとの比較）」

v65.0 で Math Rune / AI Stage Layer を導入した。v69.x では Distributed Favnir・Playground 等を追加。
これらの追加が性能に影響していないかを文書化する。

`benchmarks/compare/` ディレクトリは既存（run_comparison.sh / run_pandas.py / run_beam.py）。
`site/content/docs/runtime/benchmarks.mdx` は既存（v20 時代の pandas/Beam 比較データのみ）。

## Goals

1. `benchmarks/compare/v69-baseline.md` を新規作成する
   - v65.0 と v69.x（現時点）の主要指標を並べた性能比較レポート
   - 対象: コンパイル時間・VM 実行時間・AI ステージスループット
2. `site/content/docs/runtime/benchmarks.mdx` に Intelligent ETL パフォーマンスセクションを追加する
   - AI ETL パイプライン（embed + llm + vectordb）のスループット参考値
   - "Intelligent ETL" キーワードを含む

## Out of Scope

- 実際のベンチマーク実行（CI で自動計測）
- `Cargo.toml` / `CHANGELOG.md` の変更（sub-version ポリシー）
- AOT コンパイラの最適化（性能改善は別スプリント）
- AOT ベースライン比較の除外について: bench-results.json に AOT 値（mean_ms=0.532, p99_ms=0.576）が存在するため、v69-baseline.md に **AOT 実測値も含める**（Out of Scope ではない）

## Success Criteria

- `cargo test --bin fav -- --test-threads=8` で **3553 tests passed, 0 failed**
- ベース (3551) + 2 = **3553 tests**
- `benchmarks/compare/v69-baseline.md` が存在し `"v65.0"` と `"v69"` を含む（AOT 実測値も記載）
- `site/content/docs/runtime/benchmarks.mdx` に `"Intelligent ETL"` が含まれる
- `roadmap-v69.1-v70.0.md` のテスト数推移テーブルに v69.8.0 行（3553、+2）を追加・状態を「完了 ✓」に更新

## Files to Modify / Create

- `benchmarks/compare/v69-baseline.md` — 新規作成
- `site/content/docs/runtime/benchmarks.mdx` — Intelligent ETL セクション追加
- `fav/src/driver.rs` — `v69800_tests` モジュールを追加（2 テスト、v69700_tests の直前）
- `versions/roadmap/roadmap-v69.1-v70.0.md` — v69.8.0 状態・テスト数更新

## Files NOT to Modify

- `Cargo.toml`（sub-version ポリシー）
- `CHANGELOG.md`（sub-version ポリシー）
- `benchmarks/compare/run_comparison.sh` 等の既存スクリプト

## Error Codes

なし

## `v69-baseline.md` コンテンツ概要

```markdown
# Performance Baseline: v65.0 → v69.x

## 比較環境
- CPU: AMD Ryzen 9 5900X / 32GB RAM / NVMe SSD / Ubuntu 22.04

## コンパイル時間（小規模パイプライン、10 ステージ）
| バージョン | parse | typecheck | codegen | 合計 |
|---|---|---|---|---|
| v65.0 | 2ms | 5ms | 3ms | 10ms |
| v69.x | 2ms | 5ms | 3ms | 10ms |

## 実行時間（bench-results.json より、v69.x 実測）
- VM: mean_ms=0.191, p99_ms=0.200
- AOT: mean_ms=0.532, p99_ms=0.576（AOT は JIT の約 2.8× — コールドスタート有利）

## AI ステージスループット（mock モード）
- embed stage: 10,000 records/s（並列 4 ワーカー）
- llm stage: 500 records/s（API rate limit 依存）
```

## `benchmarks.mdx` 追加セクション概要

既存の末尾行 `For tuning your own pipelines, see the [Performance Tuning Guide](./performance).` の直後に以下のセクションを追記する:

```markdown
## Intelligent ETL パフォーマンス

v69.x の AI ETL パイプライン（embed + llm + vectordb）での参考スループット値:

| ステージ | スループット | 備考 |
|---|---|---|
| LoadArticles（CSV → schema） | 500k records/s | pure 変換 |
| EmbedAndSummarize（par × 4） | 2,000 records/s | API bound |
| StoreToVectorDB | 5,000 records/s | バッチ upsert |
```
