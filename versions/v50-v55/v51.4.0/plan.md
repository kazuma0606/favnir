# Plan: v51.4.0 — `fav bench` 差分回帰検出

Date: 2026-07-19（spec-reviewer 指摘対応済み）

---

## 実装順序

### Step 1 — `BenchOpts` 拡張（`driver.rs`）

`BenchOpts` struct に 3 フィールド追加:
- `compare: Option<String>`
- `fail_on_regression: bool`
- `threshold: f64`

`Default` 実装を更新（`compare: None, fail_on_regression: false, threshold: 10.0`）。

**影響範囲**: `driver.rs` のみ。
`BenchOpts::default()` 呼び出しは自動的に補完される。
フルリテラル構築（`BenchOpts { file: None, ... }` の形式）がテストコード内にある場合は
コンパイルエラーで検出されるため、`..Default::default()` を追加して対処する。

---

### Step 2 — `bench_stats_to_compare_json` ヘルパー追加（`driver.rs`）

`bench_stats_to_json` の直後（行 5626 付近）に追加。

```rust
pub fn bench_stats_to_compare_json(version: &str, stats: &[BenchStats]) -> String {
    let mut metrics = serde_json::Map::new();
    for s in stats {
        let avg_ms = s.avg_us / 1000.0;  // μs → ms 変換（f64 で精度保持）
        metrics.insert(s.name.clone(), serde_json::json!(avg_ms));
    }
    serde_json::json!({ "version": version, "metrics": metrics }).to_string()
}
```

`extract_bench_metrics` は `parse::<f64>()` で値を読むため、f64 出力で問題ない。

---

### Step 3 — `cmd_bench` 更新（`driver.rs`）

シグネチャを `pub fn cmd_bench(opts: &BenchOpts) -> bool` に変更。

`all_stats` 収集完了後（既存の `if opts.json { ... }` ブロックの後）に以下を追加:

```rust
if let Some(compare_path) = opts.compare.as_deref() {
    let baseline_json = match std::fs::read_to_string(compare_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read baseline {compare_path}: {e}");
            return true; // ファイル読み込み失敗は回帰ではない
        }
    };
    let current_json = bench_stats_to_compare_json("51.4.0", &all_stats);
    let (ok, report) = cmd_bench_compare(&baseline_json, &current_json, opts.threshold, false);
    println!("{report}");
    return ok;
}
true  // compare なし → 常に ok
```

compare なしのパスで関数末尾が `true` を返すことを確認する。

---

### Step 4 — `main.rs` CLI 拡張

**重要**: `main.rs` 行 1247〜1281 には `--baseline` / `--current` 分岐（v24.3.0）が既存実装されている。
この分岐は `return;` で早期終了するため、`--compare` との競合は発生しない。既存分岐は変更しない。

`bench` アームの `--stream` ハンドラの直後に追加（`other =>` アームより前）:

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

`cmd_bench(&opts);` を以下に変更:
```rust
let ok = cmd_bench(&opts);
if !ok && opts.fail_on_regression {
    process::exit(1);
}
```

`bench_stats_to_compare_json` は `cmd_bench` 内部から呼ぶため、`main.rs` の `use driver::` への追加は不要。
`cmd_bench` はすでにインポート済みのため `use driver::` の変更は不要。

---

### Step 5 — `benchmarks/v51.3.0.json` 作成

既存の `v49.2.0.json` と同一フォーマット。
NOTE: metrics 値はプレースホルダー。実環境での `fav bench --json` 実測値に更新することが推奨される。

---

### Step 6 — `v51400_tests` 追加 + バージョン更新

`v51400_tests` モジュールを `v51300_tests` の直前に追加（2 件）。
`use super::cmd_bench_compare;` のみ（`bench_stats_to_compare_json` / `BenchStats` は使用しない）。

`fav/Cargo.toml` version → `"51.4.0"`。
`CHANGELOG.md` に `[v51.4.0]` エントリ追加。

---

## 注意点

- `cmd_bench` 戻り値の変更（`()` → `bool`）により `main.rs` の `cmd_bench(&opts);` が
  「戻り値未使用」警告になる → `let ok = cmd_bench(&opts);` に変更し `ok` を使うことで解消。
- `bench_stats_to_compare_json` が生成する current JSON と `benchmarks/v51.3.0.json` の metrics は
  単位（ms）を合わせているが、bench case 名（keys）がベースラインの手動作成キー名と一致しない場合は
  比較がスキップされる（`extract_bench_metrics` は current にある key のみを処理するため）。
  実用上はベースライン JSON を `fav bench --json` 出力から生成することが前提。
