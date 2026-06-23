# v24.3.0 — 継続的パフォーマンス回帰検知

Date: 2026-06-23

## 目標

v20.1 で整備したベンチマーク基盤（`compare.fav` / `suite/run_all.sh` / `bench.yml`）を本番稼働させる。
master への merge ごとに自動でパフォーマンス比較を実行し、5% 以上の劣化で CI を fail させる。

```bash
# CI（bench.yml）で自動実行されるフロー
bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json
fav run benchmarks/compare.fav \
  -- --baseline benchmarks/v24.2.0.json \
     --current  benchmarks/latest.json \
     --threshold 5 \
     --emit-md
```

---

## 既存インフラの問題点と対応

| 問題 | 対応 |
|---|---|
| `bench.yml` baseline が `v20.0.0.json`（古い） | `v24.2.0.json` に更新 |
| `bench.yml` threshold が `10`（ロードマップは 5） | `5` に更新 |
| `bench.yml` に回帰時の CI fail ステップがない | `|| exit 1` を追加 |
| `v24.2.0.json` の `metrics` に非数値（`stage4_deferred: true`）が混在 | `metrics` を数値のみに修正（compare.fav の `Map<String, Float>` 前提） |
| Rust ユニットテスト用の比較関数がない | `pub fn cmd_bench_compare` を driver.rs に追加 |

---

## ロードマップとの対応

| ロードマップ | v24.3.0 での対応 |
|---|---|
| `--threshold 5`（5% 以上の劣化で CI fail） | `bench.yml` に反映 ✓ |
| `--baseline benchmarks/v24.0.0.json` | `v24.2.0.json`（最新安定版）に変更 ✓（理由: v24.0.0.json は metrics 非数値フィールド問題が未修正であり、v24.2.0 が最新安定版のため） |
| `--emit-md` で `results.md` 自動更新 | `bench.yml` ステップに追加 ✓（`results.md` の自動 commit は v24.7 ドキュメントサイト v2 と同時対応予定） |
| `benchmarks/latest.json` | テンプレートを新規作成 ✓ |
| `https://bench.favnir.dev` グラフ公開 | スコープ外（v24.7 ドキュメントサイト v2 と同時対応） |

---

## スコープ

### Rust（driver.rs + main.rs）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 公開関数追加 | `driver.rs` | `pub fn cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` |
| サブコマンド追加 | `main.rs` | `fav bench --baseline <path> --current <path> [--threshold N] [--emit-md]` |

### ファイル更新

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 更新 | `.github/workflows/bench.yml` | baseline → v24.2.0.json、threshold → 5、fail ステップ追加 |
| 更新 | `benchmarks/v24.2.0.json` | `metrics` から非数値フィールドを削除し数値のみに修正 |
| 新規作成 | `benchmarks/latest.json` | CI 出力テンプレート（空 metrics） |
| 新規作成 | `benchmarks/v24.3.0.json` | test_count: 1944 |
| 新規作成 | `site/content/docs/performance/benchmark-regression.mdx` | 回帰検知の使い方 |

---

## `cmd_bench_compare` 関数定義

```rust
/// benchmark JSON 文字列 2 件を受け取り、metrics の pct_change を計算する。
/// pct_change = (current - baseline) / baseline * 100
/// いずれかの metric が threshold% を超えたら (false, report) を返す。
/// emit_md = true の場合、report が Markdown テーブル形式になる。
pub fn cmd_bench_compare(
    baseline_json: &str,
    current_json: &str,
    threshold: f64,
    emit_md: bool,
) -> (bool, String)
```

**実装方針:**
- `metrics` セクション（`"metrics": { ... }`）から数値 key-value を文字列スキャンで抽出（**フラット数値のみ対応**。ネストオブジェクトは非対応）
- `version` フィールドも抽出してレポートに含める
- 各 metric について `pct_change > threshold` なら regression として記録（**減少は検出しない**。`test_count` が減っても regression 扱いにならない）
- `emit_md = false`: テキスト形式レポート（`OK: ...` / `REGRESSION: ...`）
- `emit_md = true`: Markdown テーブル形式レポート

> **threshold の型**: `cmd_bench_compare` の `threshold` パラメータは `f64`（CLI から小数指定可能）。`compare.fav` 経由の場合は `IO.args()` の文字列を `Int` として扱うため、整数のみ。

**入力 JSON 形式（`metrics` は数値のみ）:**
```json
{
  "version": "24.2.0",
  "metrics": {
    "duration_ms": 16600,
    "test_count": 1940
  }
}
```

**レポート例（テキスト形式）:**
```
OK: all metrics within 5.0% of baseline (v24.2.0 → v24.3.0).
```

```
REGRESSION: 1 metric(s) exceeded 5.0% threshold:
  duration_ms: +22.3% (baseline=16600, current=20300)
```

---

## `bench.yml` 更新内容

```yaml
      - name: Compare with baseline
        env:
          FAV: ./fav/target/release/fav
        run: |
          $FAV run benchmarks/compare.fav \
            -- --baseline benchmarks/v24.2.0.json \
               --current  benchmarks/latest.json \
               --threshold 5 \
               --emit-md || exit 1
```

変更点:
- `v20.0.0.json` → `v24.2.0.json`
- `10` → `5`
- `|| exit 1` 追加（回帰検知時に CI を fail させる）

---

## `benchmarks/v24.2.0.json` 修正

現在の `metrics` フィールドに `"stage4_deferred": true`（非数値）が含まれており、
compare.fav の `Json.get_object` → `Map<String, Float>` 変換で型エラーになる。

修正後:
```json
{
  "version": "24.2.0",
  "date": "2026-06-23",
  "test_count": 1940,
  "feature": "4-Stage Bootstrap 検証",
  "metrics": {
    "test_count": 1940,
    "duration_ms": 16600
  }
}
```

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_3_0` | Cargo.toml に `version = "24.3.0"` | — |
| `bench_compare_no_regression` | threshold 5%、metrics 変化なし | `(true, "OK: ...")` |
| `bench_compare_regression_detected` | duration_ms +22% → threshold 5% 超 | `(false, "REGRESSION: ...")` |
| `bench_compare_emit_md_has_header` | emit_md=true の出力に Markdown ヘッダ | `"# Benchmark"` を含む |
| `changelog_has_v24_3_0` | CHANGELOG.md に `[v24.3.0]` | — |

---

## 完了条件

- [ ] `pub fn cmd_bench_compare` 実装済み
- [ ] `fav bench` CLI サブコマンド追加済み
- [ ] `bench.yml` — baseline: v24.2.0.json、threshold: 5、`|| exit 1` 追加済み
- [ ] `benchmarks/v24.2.0.json` — metrics 数値のみに修正済み
- [ ] `benchmarks/latest.json` テンプレート作成済み
- [ ] `cargo test v243000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1944 件合格）
- [ ] `CHANGELOG.md` に v24.3.0 エントリ
- [ ] `benchmarks/v24.3.0.json` 作成済み（test_count: 1944）
- [ ] `site/content/docs/performance/benchmark-regression.mdx` 作成済み
