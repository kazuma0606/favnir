# Plan: v49.2.0 — パフォーマンス計測 + ボトルネック修正

## 作業順序

### Step 1: `benchmarks/v49.2.0.json` 作成

`favnir/benchmarks/` 直下に作成（既存フラット命名慣例: `v20.0.0.json`〜`v35.5.0.json` と同形式）。

既存スキーマ（`v35.5.0.json` の `metrics` / `test_count` フィールド）に準拠すること。
`"regression": false` のスペースは Rust テストの assert 文字列 `"\"regression\": false"` と一致させること。

### Step 2: `driver.rs` に `v492000_tests` 追加

`v491000_tests` の直前に挿入（2テスト）:
- `bench_all_result_recorded`: `benchmarks/v49.2.0.json` が `"checker_ms"` と `"49.2.0"` を含むことを確認
- `checker_perf_regression_none`: `benchmarks/v49.2.0.json` が `"regression": false` を含むことを確認

**注記**: `include_str!("../../benchmarks/v49.2.0.json")` はコンパイル時解決。Step 1 のファイル作成と同一作業セッションで追加すること。

### Step 3: `Cargo.toml` version 更新

`"49.1.0"` → `"49.2.0"`

### Step 4: 完了処理

- `cargo test` 3073 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.2.0 エントリ追加（`metrics.checker_ms` / `compiler_ms` 記録を明記）
- `versions/current.md` 更新（v49.2.0・3073 tests・進行中 v49.3.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.2.0 実績を記入（推定値 3066 → 実績 3073）
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `benchmarks/v49.2.0.json` | 新規作成 |
| `fav/src/driver.rs` | `v492000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.2.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.2.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/middle/checker.rs` | 計測記録のみ・ホットパス改善は v49.3.0 以降 |
| `fav/src/middle/compiler.rs` | 計測記録のみ・ホットパス改善は v49.3.0 以降 |
