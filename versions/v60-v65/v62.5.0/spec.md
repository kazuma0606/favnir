# v62.5.0 Spec — `fav bench --aot` コマンド（AOT vs VM 速度比較）

Version: 62.5.0
Status: 未着手
Base tests: 3390
Target tests: 3392

---

## 概要

`fav bench <file> --aot [--runs N]` を追加し、同一パイプラインを VM モードと AOT モードで
N 回コンパイル+実行し、Mean / P99 のレイテンシと speedup を比較表示する。
結果は `bench-results.json` にも書き出す。

**注意**: `Some("bench")` アームは v24.3.0 から既存。`cmd_bench(opts: &BenchOpts)` も既存。
新機能は `--aot` フラグで既存 bench ルーティングと区別し、
新関数 `cmd_bench_aot_vm(src: &str, runs: usize) -> String` に委譲する。
関数名の選定理由: `cmd_bench` シグネチャ競合回避を `cmd_bench_compare` との命名一貫性より優先した。

---

## 前提確認（T0 で実施）

- `Some("bench")` アームが `main.rs` に **存在する**（v24.3.0 実装済み）
- `cmd_bench` が `BenchOpts` シグネチャで **存在する** → 新関数名は `cmd_bench_aot_vm`
- `driver.rs` に `compile_program` が import 済み（L16）であることを確認
- `CraneliftBackend::lower_to_object_with_target_pub` が `cranelift_aot.rs` に存在することを確認
- `build_artifact(prog: &ast::Program) -> FvcArtifact` が `driver.rs` に存在することを確認（private fn）
- `serde_json` が `Cargo.toml` dependencies に存在することを確認
- `driver.rs` に `v62400_tests` が存在することを確認（挿入位置確認）
- `--aot` フラグが `Some("bench")` アーム内に **存在しない** ことを確認

---

## 実装スコープ

### 1. `driver.rs` — `cmd_bench_aot_vm` 追加

```rust
pub fn cmd_bench_aot_vm(src: &str, runs: usize, json_out: &str) -> String
```

`json_out` が空でない場合のみ `json_out` パスに JSON を書き出す（空文字列 = 非生成）。
テストは `json_out=""` で並列書き出し競合を回避する。CLI は `"bench-results.json"` を渡す。

処理フロー:
1. `Parser::parse_str(src, "<bench>")` — パースエラーは `"parse error: {e}"` で早期リターン
2. `let runs = runs.max(1);`
3. **VM タイミング**: `build_artifact(&program)` を N 回計測（μs 単位）
   - 各回: `let t = std::time::Instant::now(); let _ = build_artifact(&program); t.elapsed().as_micros() as f64`
4. **AOT タイミング**: `compile_program(&program)` + `CraneliftBackend::lower_to_object_with_target_pub(&ir, None)` を N 回計測
   - AOT でエラーが出た場合は `aot_mean_ms = 0.0`、`aot_p99_ms = 0.0` として続行
5. mean / P99 を計算（P99: ソートして index = runs * 99 / 100）
6. speedup = `vm_mean_ms / aot_mean_ms`（aot_mean_ms == 0.0 の場合は 1.0）
7. テーブル文字列を返す（下記フォーマット）
8. `bench-results.json` に JSON を書き出す（書き出しエラーは無視）

**出力フォーマット**:
```
Mode     | Mean (ms) | P99 (ms)
---------|-----------|----------
VM       |     X.XXX |    Y.YYY
AOT      |     X.XXX |    Y.YYY
Speedup  |    Z.ZZx  |
```

**bench-results.json フォーマット**:
```json
{"runs":N,"vm":{"mean_ms":X.XXX,"p99_ms":Y.YYY},"aot":{"mean_ms":X.XXX,"p99_ms":Y.YYY},"speedup":Z.ZZZ}
```

**ヘルパー private fn**（`cmd_bench_aot_vm` の直前に配置）:
- `fn mean_ms(timings_us: &[f64]) -> f64` — `sum / len / 1000.0`（μs → ms 変換）
- `fn p99_ms(timings_us: &mut Vec<f64>) -> f64` — ソートして `idx = len * 99 / 100` の値を ms に変換
  - runs=1 の場合は index=0 となり mean と同値になる（意図的）

### 2. `main.rs` — `Some("bench")` アームに `--aot` フラグ追加

`Some("bench")` アーム冒頭（`--baseline` チェックの前）に追加:
```rust
if args.iter().any(|a| a == "--aot") {
    let file = args.iter()
        .find(|a| !a.starts_with('-') && *a != "bench")
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            eprintln!("error: fav bench --aot requires a source file");
            process::exit(1);
        });
    let runs: usize = args.iter()
        .position(|a| a == "--runs")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let src = std::fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("error: cannot read {file}: {e}");
        process::exit(1);
    });
    let result = driver::cmd_bench_aot_vm(&src, runs);
    println!("{result}");
    return;
}
```

### 3. `driver.rs` — `v62500_tests` 追加

`v62400_tests` の直前（ファイル先頭方向）に挿入。

**`cmd_bench_runs_both_modes`**:
- ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
- `cmd_bench_aot_vm(src, 2)` を呼ぶ
- 結果が `"VM"` を含むことを確認
- 結果が `"AOT"` を含むことを確認
- 結果が `"Speedup"` を含むことを確認

**`bench_results_json_generated`**:
- 同ソースで `cmd_bench_aot_vm(src, 1)` を呼ぶ
- `"bench-results.json"` を `std::fs::read_to_string` で読む
- JSON 文字列が `"vm"` を含むことを確認
- JSON 文字列が `"aot"` を含むことを確認
- JSON 文字列が `"runs"` を含むことを確認
- `std::fs::remove_file("bench-results.json").ok();` でクリーンアップ

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62500` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3392 tests passed, 0 failed

---

## 非スコープ

- 実際のバイナリ実行（subprocess で AOT binary を起動）— `cc` 依存のため
- throughput (rows/s) 表示 — パイプライン行数の取得が必要なため（ロードマップとの意図的な乖離）
- `bench-results.json` の書き出し先指定 CLI フラグ — v62.6.0 以降で検討

## 設計注意事項

- **bench-results.json の並列実行競合**: `cmd_bench_aot_vm` は常に `bench-results.json`（cwd 相対）を書き出す。
  `cargo test -j 8` で両テストが並列実行されても、どちらの書き出しも有効な JSON を生成するため
  `bench_results_json_generated` が読んだ内容は常に有効な JSON になる。削除は最後に 1 件のみ行う。
- **bench-results.json の書き出し先**: `cargo test` の cwd は `fav/`。CI が別 cwd の場合 `let _ = std::fs::write(...)` でエラーを無視するため `bench_results_json_generated` が `unwrap_or_default("")` で空文字列を読む可能性がある。v62.6.0 の CLI フラグ追加時に対応する。
