# v64.2.0 Spec — パフォーマンスリグレッションテスト自動化

Version: 64.2.0
Status: 未着手

---

## 概要

`driver.rs` に `cmd_bench_compare(ref_a: &str, ref_b: &str) -> String` を追加する。
`ref_a`（ベース）と `ref_b`（現在）は `cmd_bench_aot_vm` が出力する JSON 文字列。
AOT mean_ms を比較し、劣化率が `regression_threshold_pct`（デフォルト 10%）を超えた場合に
`"Regression detected: ..."` を返す。

また `fav.toml` の `[bench]` セクションを新設し、`toml.rs` に `BenchTomlConfig` を追加して
`regression_threshold_pct` をパースできるようにする。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3433 tests passed, 0 failed を確認
- `driver.rs` に `cmd_bench_suite` が存在することを確認（`cmd_bench_compare` の挿入位置参照）
- `driver.rs` に `cmd_bench_compare` が存在しないことを確認（新規追加）
- `driver.rs` に `v64100_tests` が存在することを確認（`v64200_tests` の挿入位置）
- `driver.rs` に `v64200_tests` が存在しないことを確認（新規追加）
- `toml.rs` に `BenchTomlConfig` が存在しないことを確認（新規追加）
- `FavToml` に `bench` フィールドが存在しないことを確認（新規追加）
- `toml.rs` に `parse_fav_toml_pub` が存在することを確認（テストから呼ぶため）

---

## 実装スコープ

### 1. `toml.rs` — `BenchTomlConfig` + `FavToml` フィールド追加

`BackpressureConfig` の直後に追加:

```rust
/// `[bench]` section of fav.toml (v64.2.0).
#[derive(Debug, Clone)]
pub struct BenchTomlConfig {
    /// リグレッション判定しきい値（%）。デフォルト: 10。
    pub regression_threshold_pct: Option<u32>,
}
```

`FavToml` の `backpressure` フィールドの直後に追加:

```rust
/// Optional bench configuration (v64.2.0).
pub bench: Option<BenchTomlConfig>,
```

`parse_fav_toml` に以下を追加:
- 変数初期化: `let mut bench_cfg: Option<BenchTomlConfig> = None;`
- セクション検出（`"[backpressure]"` の直後）:
  ```rust
  if trimmed == "[bench]" {
      section = "bench";
      continue;
  }
  ```
- `"bench"` アーム（`"backpressure"` アームの直後）:
  ```rust
  "bench" => {
      let mut current = bench_cfg.take().unwrap_or(BenchTomlConfig { regression_threshold_pct: None });
      if let Some((key, val)) = parse_kv(trimmed) {
          match key {
              "regression_threshold_pct" => {
                  current.regression_threshold_pct = val.parse::<u32>().ok();
              }
              _ => {}
          }
      }
      bench_cfg = Some(current);
  }
  ```
- `FavToml` 構造体リテラルに `bench: bench_cfg,` を追加

### 2. `driver.rs` — `cmd_bench_compare` + helper 追加

`cmd_bench_suite` の直後に追加:

```rust
/// JSON ベンチ結果から指定モード（"vm" / "aot"）の mean_ms を取り出す。
fn parse_bench_mean_ms(json: &str, mode: &str) -> Option<f64> {
    // {"runs":N,"vm":{"mean_ms":X,...},"aot":{"mean_ms":Y,...},"speedup":Z}
    let needle = format!("\"{}\":{{\"mean_ms\":", mode);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    rest[..end].parse().ok()
}

/// v64.2.0: ベンチ結果 JSON を比較し、AOT リグレッションを検出する。
/// `ref_a` = ベース、`ref_b` = 現在（いずれも `cmd_bench_aot_vm` の JSON 出力）。
/// デフォルトしきい値 10%。
pub fn cmd_bench_compare(ref_a: &str, ref_b: &str) -> String {
    let base_ms = match parse_bench_mean_ms(ref_a, "aot") {
        Some(v) => v,
        None => return "bench_compare: could not parse base bench result".to_string(),
    };
    let curr_ms = match parse_bench_mean_ms(ref_b, "aot") {
        Some(v) => v,
        None => return "bench_compare: could not parse current bench result".to_string(),
    };
    if base_ms <= 0.0 {
        return "bench_compare: base mean_ms must be > 0".to_string();
    }
    let pct = (curr_ms - base_ms) / base_ms * 100.0;
    let threshold = 10.0_f64; // デフォルト（toml で上書き可能、本バージョンでは CLI 統合非スコープ）
    if pct > threshold {
        format!(
            "Regression detected: AOT +{pct:.1}% slower (was {base_ms:.3}ms, now {curr_ms:.3}ms) — exceeds threshold {threshold:.0}%"
        )
    } else {
        format!(
            "No regression detected. AOT {pct:+.1}% (was {base_ms:.3}ms, now {curr_ms:.3}ms)"
        )
    }
}
```

### 3. `driver.rs` — `v64200_tests` 追加

`v64100_tests` の直前に挿入（関数本体は Step 2 で `cmd_bench_suite` の直後に追加済みの位置とは別）:

```rust
// -- v64200_tests (v64.2.0) -- パフォーマンスリグレッションテスト自動化 --
#[cfg(test)]
mod v64200_tests {
    #[test]
    fn bench_compare_detects_regression() {
        // base: AOT mean 10ms, current: AOT mean 15ms → +50% > threshold(10%)
        let base_json = r#"{"runs":1,"vm":{"mean_ms":20.000,"p99_ms":20.000},"aot":{"mean_ms":10.000,"p99_ms":10.000},"speedup":2.000}"#;
        let curr_json = r#"{"runs":1,"vm":{"mean_ms":20.000,"p99_ms":20.000},"aot":{"mean_ms":15.000,"p99_ms":15.000},"speedup":1.333}"#;
        let out = crate::driver::cmd_bench_compare(base_json, curr_json);
        assert!(out.contains("Regression"), "should detect regression: {}", out);
        assert!(out.contains("AOT"), "should mention AOT: {}", out);
        // 同一ベンチはリグレッションなし
        // 同一ベンチ（0%変化）は No regression
        let no_regr = crate::driver::cmd_bench_compare(base_json, base_json);
        assert!(no_regr.contains("No regression"), "identical bench should not regress: {}", no_regr);
        // 改善時（curr < base）も No regression（pct は負値、{pct:+.1} で "-X.X%" 表示）
        let improved = crate::driver::cmd_bench_compare(curr_json, base_json);
        assert!(improved.contains("No regression"), "improvement should not be regression: {}", improved);
    }

    #[test]
    fn bench_toml_threshold() {
        let toml_src = "[project]\nname = \"myproj\"\n\n[bench]\nregression_threshold_pct = 5\n";
        let toml = crate::toml::parse_fav_toml_pub(toml_src);
        let bench = toml.bench.expect("bench section should be parsed");
        assert_eq!(bench.regression_threshold_pct, Some(5), "threshold should be 5");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64200_tests` で 2 件 PASS
  - `bench_compare_detects_regression` PASS
  - `bench_toml_threshold` PASS
- `cargo test -j 8 -- --test-threads=8` で 3435 tests passed, 0 failed

---

## 非スコープ

- `main.rs` への `--compare` CLI フラグ統合（print_diag 経路切り替えも非スコープ）
- `fav.toml` の `regression_threshold_pct` を `cmd_bench_compare` に自動注入する処理
- 実際の git ref を読んで `bench-results.json` を取得する機能

---

## 技術ノート

### `parse_bench_mean_ms` の設計

`cmd_bench_aot_vm` が出力する JSON:
```
{"runs":N,"vm":{"mean_ms":X.XXX,"p99_ms":X.XXX},"aot":{"mean_ms":Y.YYY,"p99_ms":Y.YYY},"speedup":Z.ZZZ}
```
`"aot":{"mean_ms":` の後の数値部分（ASCII 数字 or `.`）をスライスで切り出して `parse::<f64>()` する。
サードパーティ JSON クレートは使わない（ゼロ依存・シンプル実装方針）。

**制約**: 負の `mean_ms` は非対応（終端検出クロージャが `-` を認識しないため `None` を返す）。
`cmd_bench_aot_vm` の正常出力は必ず非負値のため実運用上は問題ない。
破損 JSON や mock 値として負値を渡した場合は `"bench_compare: could not parse ..."` エラーになる。

### しきい値のデフォルト値

本バージョンでは CLI 統合は非スコープのため、`threshold = 10.0` をハードコードする。
`fav.toml` の `regression_threshold_pct` を読んで CLI 経路で注入する処理は後送り（v64.2 以降）。
`bench_toml_threshold` テストは `toml.rs` のパースが正しいことを検証する（driver との結合は非スコープ）。

### ベーステスト数

実際のベース: 3433（v64.1.0 完了後）
目標: 3433 + 2 = **3435**
