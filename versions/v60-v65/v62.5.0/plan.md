# v62.5.0 Plan — `fav bench --aot` コマンド（AOT vs VM 速度比較）

Version: 62.5.0
Status: 未着手

---

## 実装順序

### Step 1: `driver.rs` — `cmd_bench_aot_vm` 追加

`cmd_build_aot_stats` の直後（または `v62500_tests` 挿入位置の近く）に配置。
`mean_ms` / `p99_ms` helper を `cmd_bench_aot_vm` の直前に置く。
`cargo build` でエラーなし確認。

### Step 2: `main.rs` — `Some("bench")` アームに `--aot` 分岐追加

`Some("bench")` アーム冒頭（`--baseline` チェックの前）に `if args.iter().any(|a| a == "--aot")` ブロックを追加。
ファイルを読み込んで `cmd_bench_aot_vm` を呼び、結果を `println!` する。
`cargo build` でエラーなし確認。

### Step 3: `driver.rs` — `v62500_tests` 追加

`v62400_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62500` で 2 件 PASS 確認。

### Step 4: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3392 tests passed, 0 failed を確認。

### Step 5: ドキュメント更新

roadmap / current.md / CHANGELOG.md / tasks.md を更新。

---

## 設計メモ

### cmd_bench_aot_vm の計測対象

| モード | 計測内容 |
|---|---|
| VM | `build_artifact(&program)` N 回（バイトコードコンパイル時間） |
| AOT | `compile_program(&program)` + `lower_to_object_with_target_pub(&ir, None)` N 回 |

実際のバイナリ実行ではなくコンパイル時間を計測する（`cc` 不要）。
テストが環境非依存で動作し、CI でも安定する。

### P99 計算

```rust
fn p99_ms(timings_us: &mut Vec<f64>) -> f64 {
    if timings_us.is_empty() { return 0.0; }
    timings_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = (timings_us.len() * 99 / 100).min(timings_us.len() - 1);
    timings_us[idx] / 1000.0
}
```

### AOT エラー時のフォールバック

`lower_to_object_with_target_pub` が `Err` を返した場合（cranelift 非対応プラットフォーム等）、
`aot_mean_ms = 0.0`、`aot_p99_ms = 0.0` として表を出力し続ける（panic しない）。

### bench-results.json

テスト後のクリーンアップは `std::fs::remove_file("bench-results.json").ok()` で行う。
並列テスト実行での競合を避けるため、テストは `bench_results_json_generated` 1 件のみ JSON を生成する。

### ロードマップとの乖離

- 実際のバイナリ実行（throughput 含む）は非スコープ（spec に明記）
- 関数名は `cmd_bench_aot_vm`（`cmd_bench` シグネチャ競合のため）— tasks.md T5 実績欄に記載
