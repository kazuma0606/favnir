# Spec: v51.4.0 — `fav bench` 差分回帰検出

Date: 2026-07-19
Status: 計画中（spec-reviewer 指摘対応済み）

---

## 概要

`fav bench --compare <baseline.json>` + `--fail-on-regression` + `--threshold <pct>` フラグを追加。
ベンチマーク実行後に前回計測結果（`benchmarks/vX.Y.Z.json`）との差分を自動検出し、
閾値（デフォルト 10%）超過を警告する。`--fail-on-regression` フラグで CI 向け非ゼロ終了コードを返す。

```bash
$ fav bench --all --compare benchmarks/v51.3.0.json
... (bench 実行) ...
checker_ms:         12.0 → 18.0  (+50.0%)  [WARN: exceeds 10% threshold]
compiler_ms:         8.0 →  8.0  (+0.0%)   [OK]
REGRESSION: 1 metric(s) exceeded 10.0% threshold.

# CI での使い方
$ fav bench --all --compare benchmarks/v51.3.0.json --fail-on-regression
# 回帰あり → exit 1
```

---

## 前提確認

### 既存実装（変更しない）

- `cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` は
  `driver.rs` に実装済み（v24.3.0）。`{ "version": ..., "metrics": { key: f64, ... } }` 形式を受け付ける。
- `BenchStats` struct と `bench_stats_to_json` は `driver.rs` に実装済み。
- **`fav bench --baseline X --current Y`** 分岐は `main.rs` 行 1247〜1281 に**実装済み**。
  2 つの既存 JSON ファイルを比較するユースケース。このフラグは変更・削除しない。

### v51.4.0 で追加するもの

- `--compare <path>` — bench を実行し、結果を baseline と自動比較（`--baseline` は 2 ファイル必要。`--compare` は 1 ファイル）
- `--fail-on-regression` — 回帰検出時に exit 1
- `--threshold <pct>` — 回帰閾値（デフォルト 10.0）
- `BenchOpts` に上記 3 フィールド追加
- `bench_stats_to_compare_json` ヘルパー追加
- `cmd_bench` の戻り値を `bool` に変更
- `benchmarks/v51.3.0.json` 作成

### `--baseline` と `--compare` の共存方針

`main.rs` の `bench` アームの構造は変更後も以下のとおり:

```
Some("bench") => {
    if --baseline が存在 {
        // v24.3.0 の既存分岐: 2ファイル比較 → return;  (変更なし)
    }
    // ── original bench runner ──────────────────────────
    opts に --compare / --fail-on-regression / --threshold を追加
    let ok = cmd_bench(&opts);
    if !ok && opts.fail_on_regression { process::exit(1); }
}
```

`--baseline` は早期 `return;` するため、`--compare` との競合は発生しない。

---

## 実装詳細

### 1. `BenchOpts` 拡張（`driver.rs`）

```rust
pub struct BenchOpts {
    pub file: Option<String>,
    pub filter: Option<String>,
    pub runs: u64,
    pub warmup: u64,
    pub json: bool,
    pub stream: bool,
    // v51.4.0 追加
    pub compare: Option<String>,      // --compare <path>
    pub fail_on_regression: bool,     // --fail-on-regression
    pub threshold: f64,               // --threshold <pct>, default 10.0
}

impl Default for BenchOpts {
    fn default() -> Self {
        BenchOpts {
            file: None, filter: None, runs: 100, warmup: 5,
            json: false, stream: false,
            compare: None, fail_on_regression: false, threshold: 10.0,
        }
    }
}
```

### 2. `bench_stats_to_compare_json` ヘルパー（新規, `pub fn`）

`bench_stats_to_json` の直後に追加する。

```rust
/// BenchStats スライスを cmd_bench_compare が受け付ける
/// `{"version": ..., "metrics": { name: avg_ms_f64, ... }}` 形式の JSON に変換する。
/// avg_us (マイクロ秒) を / 1000.0 でミリ秒に変換し、f64 のまま出力する。
/// extract_bench_metrics は parse::<f64>() で読むため f64 出力で問題ない。
pub fn bench_stats_to_compare_json(version: &str, stats: &[BenchStats]) -> String {
    let mut metrics = serde_json::Map::new();
    for s in stats {
        // avg_us (μs) → ms 変換（f64 のまま出力して精度を保持）
        let avg_ms = s.avg_us / 1000.0;
        metrics.insert(s.name.clone(), serde_json::json!(avg_ms));
    }
    serde_json::json!({
        "version": version,
        "metrics": metrics,
    })
    .to_string()
}
```

### 3. `cmd_bench` シグネチャ変更 + compare ロジック追加（`driver.rs`）

シグネチャを `pub fn cmd_bench(opts: &BenchOpts) -> bool` に変更。

`all_stats` 収集後、`opts.compare` が `Some(path)` の場合:
1. `std::fs::read_to_string(path)` でベースライン JSON を読む
2. ファイル読み込みエラー → `eprintln!` + `return true`（回帰ではない）
3. `bench_stats_to_compare_json("51.4.0", &all_stats)` で current JSON 生成
4. `cmd_bench_compare(&baseline_json, &current_json, opts.threshold, false)` を呼ぶ
5. `println!("{report}")` で結果出力
6. `return ok`

compare なし → 最後に `true` を返す（既存挙動と互換）。

### 4. `main.rs` CLI 拡張

`bench` アームの `--stream` ハンドラの後に追加:

```rust
"--compare" => {
    opts.compare = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --compare requires a path argument");
        process::exit(1);
    }));
    i += 2;
}
"--fail-on-regression" => {
    opts.fail_on_regression = true;
    i += 1;
}
"--threshold" => {
    let raw = args.get(i + 1).unwrap_or_else(|| {
        eprintln!("error: --threshold requires a number");
        process::exit(1);
    });
    opts.threshold = raw.parse::<f64>().unwrap_or_else(|_| {
        eprintln!("error: --threshold must be a number");
        process::exit(1);
    });
    i += 2;
}
```

`cmd_bench` 呼び出しを変更:

```rust
// before: cmd_bench(&opts);
let ok = cmd_bench(&opts);
if !ok && opts.fail_on_regression {
    process::exit(1);
}
```

`use driver::{ ... }` に `bench_stats_to_compare_json` を追加する
（cmd_bench は既存インポート済み）。

### 5. `benchmarks/v51.3.0.json` 作成

NOTE: 実測値は `fav bench --json` の実行で取得するが、CI 環境によって変動するため
プレースホルダー値を記録し、「実際の回帰検出は --compare でのみ機能する」ことをコメントに明記する。

```json
{
  "version": "51.3.0",
  "date": "2026-07-19",
  "milestone": "Performance & Scale Sprint",
  "tests_passed": 3119,
  "tests_failed": 0,
  "metrics": {
    "checker_ms": 12,
    "compiler_ms": 8,
    "total_pipeline_ms": 25
  },
  "regression": false,
  "notes": "v51.3.0 ストリーミングバックプレッシャー実装後の計測。値はプレースホルダー（実環境で --json 実測値に更新すること）。"
}
```

### 6. `v51400_tests` — 2 件（`driver.rs`）

```rust
#[cfg(test)]
mod v51400_tests {
    use super::cmd_bench_compare;  // bench_stats_to_compare_json は使用しない（ハードコード JSON で十分）

    #[test]
    fn bench_regression_detected() {
        // baseline: checker_ms=12, compiler_ms=8
        let baseline = r#"{"version":"51.3.0","metrics":{"checker_ms":12,"compiler_ms":8}}"#;
        // current: checker_ms=18 (+50%) > 10% threshold → REGRESSION
        let current = r#"{"version":"51.4.0","metrics":{"checker_ms":18,"compiler_ms":8}}"#;
        let (ok, report) = cmd_bench_compare(baseline, current, 10.0, false);
        assert!(!ok, "regression should be detected: {report}");
        assert!(report.contains("REGRESSION"), "report should contain REGRESSION: {report}");
        assert!(report.contains("checker_ms"), "report should name regressed metric: {report}");
    }

    #[test]
    fn bench_no_regression_passes() {
        // baseline: checker_ms=12, compiler_ms=8
        let baseline = r#"{"version":"51.3.0","metrics":{"checker_ms":12,"compiler_ms":8}}"#;
        // current: checker_ms=13 (+8.3%) < 10% threshold → OK
        let current = r#"{"version":"51.4.0","metrics":{"checker_ms":13,"compiler_ms":8}}"#;
        let (ok, report) = cmd_bench_compare(baseline, current, 10.0, false);
        assert!(ok, "no regression expected: {report}");
        assert!(report.contains("OK"), "report should contain OK: {report}");
    }
}
```

---

## 完了条件

- `cargo test` 3121 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51400_tests` 2 件 pass: `bench_regression_detected`, `bench_no_regression_passes`
- `benchmarks/v51.3.0.json` 作成済み
- `fav bench --compare` / `--fail-on-regression` / `--threshold` フラグが動作する
- `--baseline` 既存分岐は変更なし（後方互換性維持）
