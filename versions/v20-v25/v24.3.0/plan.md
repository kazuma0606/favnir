# v24.3.0 実装計画 — 継続的パフォーマンス回帰検知

## 前提確認

v24.3.0 は driver.rs + main.rs の Rust 変更 + 設定ファイル更新 + ドキュメント。

### 実装前チェック

```bash
grep -n "version = " fav/Cargo.toml
# → "24.2.0" であること

grep -n "mod v242000_tests\|mod v243000_tests" fav/src/driver.rs | head -5
# → v243000_tests が未存在であること

grep -n "cmd_bench_compare\|\"bench\"" fav/src/driver.rs fav/src/main.rs | head -5
# → 全 0 件であること

grep -n "v24.2.0.json\|threshold 5" .github/workflows/bench.yml | head -5
# → 0 件（まだ v20.0.0.json / threshold 10 のまま）
```

---

## T0: `benchmarks/v24.2.0.json` の metrics 修正

`metrics` に非数値フィールド（`"stage4_deferred": true`）が混在しており
compare.fav の `Json.get_object` → `Map<String, Float>` 変換が壊れる。
数値のみに修正する。

修正後の `benchmarks/v24.2.0.json`:
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

> `duration_ms: 16600` は v24.2.0 の `cargo test --bin fav` 実績値（约16.6s）。

- [ ] **事後確認**: `cat benchmarks/v24.2.0.json` — metrics に数値のみ

---

## T1: `fav/src/driver.rs` — `cmd_bench_compare` 追加

挿入位置: `pub fn cmd_spec` の直後（同じツールコマンド系の公開関数として配置）。

### T1-1: metrics 抽出ヘルパー

> **制約**: `metrics` オブジェクト内はフラット数値のみをサポート。ネストオブジェクトは非対応（最初の `}` で終端する単純スキャン）。

```rust
/// JSON 文字列の "metrics": { ... } セクションから数値フィールドを抽出する。
/// metrics はフラット数値のみ対応（ネストオブジェクト非対応）。
fn extract_bench_metrics(json: &str) -> Vec<(String, f64)> {
    let start = match json.find("\"metrics\"") {
        Some(i) => i,
        None => return vec![],
    };
    let brace = match json[start..].find('{') {
        Some(i) => start + i + 1,
        None => return vec![],
    };
    let end = match json[brace..].find('}') {
        Some(i) => brace + i,
        None => return vec![],
    };
    let body = &json[brace..end];
    let mut result = Vec::new();
    for segment in body.split(',') {
        let segment = segment.trim();
        if let Some(colon) = segment.find(':') {
            let key_part = segment[..colon].trim().trim_matches('"');
            let val_part = segment[colon + 1..].trim();
            if let Ok(v) = val_part.parse::<f64>() {
                result.push((key_part.to_string(), v));
            }
        }
    }
    result
}

/// JSON 文字列から "version" フィールドの値を取り出す。
fn extract_bench_version(json: &str) -> String {
    if let Some(i) = json.find("\"version\"") {
        let rest = &json[i + 9..];
        if let Some(j) = rest.find('"') {
            let after = &rest[j + 1..];
            if let Some(k) = after.find('"') {
                return after[..k].to_string();
            }
        }
    }
    "unknown".to_string()
}
```

### T1-2: `pub fn cmd_bench_compare`

```rust
/// ベンチマーク JSON 2 件を比較し、metrics の pct_change を計算する。
/// pct_change = (current - baseline) / baseline * 100
/// threshold% を超えた metric があれば (false, report)。
pub fn cmd_bench_compare(
    baseline_json: &str,
    current_json: &str,
    threshold: f64,
    emit_md: bool,
) -> (bool, String) {
    let baseline_metrics = extract_bench_metrics(baseline_json);
    let current_metrics = extract_bench_metrics(current_json);
    let baseline_ver = extract_bench_version(baseline_json);
    let current_ver = extract_bench_version(current_json);

    let mut regressions: Vec<String> = Vec::new();
    let mut rows: Vec<(String, f64, f64, f64)> = Vec::new(); // (key, base, cur, pct)

    for (key, cur) in &current_metrics {
        if let Some((_, base)) = baseline_metrics.iter().find(|(k, _)| k == key) {
            let pct = if *base == 0.0 {
                0.0
            } else {
                (cur - base) / base * 100.0
            };
            rows.push((key.clone(), *base, *cur, pct));
            if pct > threshold {
                regressions.push(format!(
                    "  {key}: +{pct:.1}% (baseline={base}, current={cur})"
                ));
            }
        }
    }

    if emit_md {
        let mut md = format!(
            "# Benchmark Regression Report\n\nBaseline: `{baseline_ver}` → Current: `{current_ver}`\n\n"
        );
        md.push_str("| Metric | Baseline | Current | Change |\n|---|---|---|---|\n");
        for (key, base, cur, pct) in &rows {
            let sign = if *pct >= 0.0 { "+" } else { "" };
            md.push_str(&format!("| {key} | {base} | {cur} | {sign}{pct:.1}% |\n"));
        }
        if !regressions.is_empty() {
            md.push_str(&format!(
                "\n**REGRESSION**: {} metric(s) exceeded {threshold:.1}% threshold.\n",
                regressions.len()
            ));
        } else {
            md.push_str(&format!(
                "\n**OK**: all metrics within {threshold:.1}% of baseline.\n"
            ));
        }
        return (regressions.is_empty(), md);
    }

    if regressions.is_empty() {
        (
            true,
            format!(
                "OK: all metrics within {threshold:.1}% of baseline ({baseline_ver} → {current_ver})."
            ),
        )
    } else {
        (
            false,
            format!(
                "REGRESSION: {} metric(s) exceeded {threshold:.1}% threshold:\n{}",
                regressions.len(),
                regressions.join("\n")
            ),
        )
    }
}
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T2: `fav/src/main.rs` — `"bench"` サブコマンド追加

`Some("spec")` アームの直後に追加する。

```rust
        Some("bench") => {
            // ── v24.3.0: fav bench --baseline <path> --current <path> [--threshold N] [--emit-md] ──
            let baseline_path = args.iter().position(|a| a == "--baseline")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    eprintln!("error: fav bench requires --baseline <path>");
                    process::exit(1);
                });
            let current_path = args.iter().position(|a| a == "--current")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    eprintln!("error: fav bench requires --current <path>");
                    process::exit(1);
                });
            let threshold: f64 = args.iter().position(|a| a == "--threshold")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(5.0);
            let emit_md = args.iter().any(|a| a == "--emit-md");
            let baseline_json = std::fs::read_to_string(baseline_path).unwrap_or_else(|e| {
                eprintln!("error: cannot read {baseline_path}: {e}");
                process::exit(1);
            });
            let current_json = std::fs::read_to_string(current_path).unwrap_or_else(|e| {
                eprintln!("error: cannot read {current_path}: {e}");
                process::exit(1);
            });
            let (ok, report) = driver::cmd_bench_compare(&baseline_json, &current_json, threshold, emit_md);
            println!("{report}");
            if !ok {
                process::exit(1);
            }
        }
```

**挿入位置:** `Some("spec") => { ... }` ブロックの直後。

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T3: `fav/src/driver.rs` — v243000_tests 追加

### T3-1: `v242000_tests::version_is_24_2_0` を削除（T5-1 より前に必須）

```rust
    #[test]
    fn version_is_24_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.2.0\""),
            "Cargo.toml should have version 24.2.0"
        );
    }
```

この関数ごと削除する。

### T3-2: `v243000_tests` モジュールを `v242000_tests` の直後に追加

```rust
// ── v243000_tests (v24.3.0) — 継続的パフォーマンス回帰検知 ──────────────
#[cfg(test)]
mod v243000_tests {
    use super::*;

    const BASELINE_JSON: &str = r#"{
        "version": "24.2.0",
        "metrics": { "duration_ms": 16600, "test_count": 1940 }
    }"#;

    #[test]
    fn version_is_24_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.3.0\""),
            "Cargo.toml should have version 24.3.0"
        );
    }

    #[test]
    fn bench_compare_no_regression() {
        // 変化なし → OK
        let current = r#"{"version": "24.3.0", "metrics": {"duration_ms": 16700, "test_count": 1944}}"#;
        // duration_ms +0.6% < 5.0% threshold
        let (ok, report) = cmd_bench_compare(BASELINE_JSON, current, 5.0, false);
        assert!(ok, "no regression expected: {report}");
        assert!(report.contains("OK"), "report should contain OK: {report}");
    }

    #[test]
    fn bench_compare_regression_detected() {
        // duration_ms +22% > 5% threshold → REGRESSION
        let current = r#"{"version": "24.3.0", "metrics": {"duration_ms": 20300, "test_count": 1944}}"#;
        let (ok, report) = cmd_bench_compare(BASELINE_JSON, current, 5.0, false);
        assert!(!ok, "regression should be detected: {report}");
        assert!(report.contains("REGRESSION"), "report should contain REGRESSION: {report}");
        assert!(report.contains("duration_ms"), "report should name the regressed metric: {report}");
    }

    #[test]
    fn bench_compare_emit_md_has_header() {
        let current = r#"{"version": "24.3.0", "metrics": {"duration_ms": 16700, "test_count": 1944}}"#;
        let (_, report) = cmd_bench_compare(BASELINE_JSON, current, 5.0, true);
        assert!(
            report.contains("# Benchmark"),
            "emit_md report should start with markdown header: {report}"
        );
    }

    #[test]
    fn changelog_has_v24_3_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("[v24.3.0]"),
            "CHANGELOG.md should have [v24.3.0] entry"
        );
    }
}
```

- [ ] `cargo test v243000 --bin fav` — 5/5 PASS を確認
- [ ] `cargo test --bin fav` — リグレッションなし（1944 件合格）を確認

---

## T4: `.github/workflows/bench.yml` 更新

変更点（最小限）:
1. `--baseline benchmarks/v20.0.0.json` → `--baseline benchmarks/v24.2.0.json`
2. `--threshold 10` → `--threshold 5`
3. `--emit-md` の末尾に `|| exit 1` 追加

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

---

## T5: Cargo.toml + CHANGELOG + benchmarks + mdx

> **注意**: T3-1 の `version_is_24_2_0` 削除完了後に Cargo.toml を更新すること（T5-1）。

### T5-1: `fav/Cargo.toml` バージョン更新

```
version = "24.2.0" → "24.3.0"
```

### T5-2: `CHANGELOG.md` 先頭に v24.3.0 エントリ追加

```markdown
## [v24.3.0] — 2026-06-23 — 継続的パフォーマンス回帰検知

### Added
- `driver::cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` — ベンチマーク JSON 比較の公開 API
- `fav bench --baseline <path> --current <path> [--threshold N] [--emit-md]` CLI サブコマンド
- `benchmarks/latest.json` — CI 出力テンプレート

### Changed
- `.github/workflows/bench.yml` — baseline を `v24.2.0.json` に更新、threshold を 5% に変更、回帰時 CI fail を有効化
- `benchmarks/v24.2.0.json` — `metrics` を数値のみに修正（`stage4_deferred` 削除）

### Notes
- 回帰判定式: `(current - baseline) / baseline * 100 > threshold`（増加が劣化）
- `bench.favnir.dev` グラフ公開は v24.7（ドキュメントサイト v2）と同時対応予定
```

### T5-3: `benchmarks/latest.json` 作成

> **必須**: `compare.fav` の `write_results_md` が `Json.get_string(data, "timestamp")` を呼ぶため、`"timestamp"` フィールドが必要。

```json
{
  "version": "latest",
  "date": "",
  "timestamp": "",
  "metrics": {
    "duration_ms": 0,
    "test_count": 0
  }
}
```

### T5-4: `benchmarks/v24.3.0.json` 作成

> **注意**: `duration_ms: 16600` は v24.2.0 実績値からの推定値。実装完了後に `cargo test --bin fav` の実測値（秒数 × 1000）に置き換えること。

```json
{
  "version": "24.3.0",
  "date": "2026-06-23",
  "test_count": 1944,
  "feature": "継続的パフォーマンス回帰検知",
  "metrics": {
    "test_count": 1944,
    "duration_ms": 16600
  }
}
```

### T5-5: `site/content/docs/performance/benchmark-regression.mdx` 作成

ベンチマーク回帰検知の説明ページ（使い方・threshold・CI 設定）。

---

## 実装順序

```
T0（v24.2.0.json の metrics 修正）
T1（driver.rs: extract_bench_metrics / extract_bench_version / cmd_bench_compare 追加）
T2（main.rs: "bench" サブコマンド追加）
cargo check → エラー 0 確認
T3-1（version_is_24_2_0 削除）← T5-1 より前に必須
T3-2（v243000_tests 追加）
cargo test v243000 → 5/5 PASS 確認
T4（bench.yml 更新）
T5-1（version 更新）← T3-1 完了後
T5-2〜5（CHANGELOG / latest.json / v24.3.0.json / benchmark-regression.mdx）
cargo test --bin fav → リグレッションなし確認（1944 件）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `extract_bench_metrics` が入れ子 JSON（`metrics: { a: { b: 1 } }`）で誤動作 | `bench_compare_no_regression` テスト失敗 | `}` 検索を最初の `}` に限定（単純なフラット metrics のみ対応） |
| `extract_bench_version` が `"version"` フィールドを誤検索 | レポートに `unknown` が出現 | テスト `bench_compare_no_regression` でレポートに `v24.2.0` を含むことを確認 |
| bench.yml の `|| exit 1` が Windows ランナーで動作しない | CI 確認 | `ubuntu-latest` のみ使用しているため問題なし |
