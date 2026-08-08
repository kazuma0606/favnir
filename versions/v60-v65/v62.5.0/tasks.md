# v62.5.0 タスクリスト

Status: COMPLETE
Version: 62.5.0
Base tests: 3390
Target tests: 3392

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3390 tests passed, 0 failed を確認
- [x] `main.rs` の `Some("bench")` アームが **存在する** ことを確認（v24.3.0 実装済み）
- [x] `driver.rs` の `pub fn cmd_bench` が `BenchOpts` シグネチャであることを確認（新関数名に `cmd_bench_aot_vm` を使う理由）
- [x] `driver.rs` L16 に `use crate::middle::compiler::compile_program;` が存在することを確認
- [x] `cranelift_aot.rs` に `lower_to_object_with_target_pub` が存在することを確認
- [x] `driver.rs` に `build_artifact` が存在することを確認（private fn）
- [x] `Cargo.toml` に `serde_json` が存在することを確認
- [x] `driver.rs` に `v62400_tests` が存在することを確認（挿入位置確認）
- [x] `Some("bench")` アーム内に `--aot` が **存在しない** ことを確認
- [x] `driver.rs` に `cmd_bench_aot_vm` が **存在しない** ことを確認
- [x] ロードマップの `Throughput` 列（rows/s）が非スコープであることを spec.md 非スコープ節で確認（パイプライン行数取得困難のため意図的な乖離）

---

## T1: `driver.rs` — `mean_ms` / `p99_ms` + `cmd_bench_aot_vm` 追加

- [x] `cmd_build_aot_stats` の直後に以下を追加:
  ```rust
  fn mean_ms(timings_us: &[f64]) -> f64 {
      if timings_us.is_empty() { return 0.0; }
      timings_us.iter().sum::<f64>() / timings_us.len() as f64 / 1000.0
  }

  fn p99_ms(timings_us: &mut Vec<f64>) -> f64 {
      if timings_us.is_empty() { return 0.0; }
      timings_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
      let idx = (timings_us.len() * 99 / 100).min(timings_us.len() - 1);
      timings_us[idx] / 1000.0
  }

  /// v62.5.0: VM vs AOT コンパイル時間を N 回計測して比較表を返す。
  /// bench-results.json にも書き出す。
  pub fn cmd_bench_aot_vm(src: &str, runs: usize) -> String {
      let program = match crate::frontend::parser::Parser::parse_str(src, "<bench>") {
          Ok(p) => p,
          Err(e) => return format!("parse error: {e}"),
      };
      let runs = runs.max(1);

      // VM タイミング: build_artifact を N 回計測
      let mut vm_us: Vec<f64> = (0..runs).map(|_| {
          let t = std::time::Instant::now();
          let _ = build_artifact(&program);
          t.elapsed().as_micros() as f64
      }).collect();

      // AOT タイミング: compile_program + lower_to_object N 回計測
      let mut aot_us: Vec<f64> = (0..runs).map(|_| {
          let t = std::time::Instant::now();
          let ir = compile_program(&program);
          let _ = crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_with_target_pub(&ir, None);
          t.elapsed().as_micros() as f64
      }).collect();

      let vm_mean  = mean_ms(&vm_us);
      let vm_p99   = p99_ms(&mut vm_us);
      let aot_mean = mean_ms(&aot_us);
      let aot_p99  = p99_ms(&mut aot_us);
      let speedup  = if aot_mean > 0.0 { vm_mean / aot_mean } else { 1.0 };

      let table = format!(
          "Mode     | Mean (ms) | P99 (ms)\n---------|-----------|----------\nVM       | {:9.3} | {:8.3}\nAOT      | {:9.3} | {:8.3}\nSpeedup  | {:7.2}x |",
          vm_mean, vm_p99, aot_mean, aot_p99, speedup
      );

      let json = format!(
          r#"{{"runs":{runs},"vm":{{"mean_ms":{vm_mean:.3},"p99_ms":{vm_p99:.3}}},"aot":{{"mean_ms":{aot_mean:.3},"p99_ms":{aot_p99:.3}}},"speedup":{speedup:.3}}}"#
      );
      let _ = std::fs::write("bench-results.json", &json);

      table
  }
  ```
- [x] `cargo build` でエラーなし

---

## T2: `main.rs` — `Some("bench")` アームに `--aot` 分岐追加

- [x] `Some("bench")` アーム冒頭（`--baseline` チェックの前）に以下を追加:
  ```rust
  if args.iter().any(|a| a == "--aot") {
      let file = args.iter()
          .find(|a| !a.starts_with('-') && a.as_str() != "bench")
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
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v62500_tests` 追加

- [x] `v62400_tests` の直前（ファイル先頭方向）に `v62500_tests` モジュールを挿入
- [x] `use super::*;` を先頭に追加
- [x] `cmd_bench_runs_both_modes` テスト追加:
  - ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
  - `cmd_bench_aot_vm(src, 2)` を呼ぶ
  - 結果が `"VM"` を含むことを確認
  - 結果が `"AOT"` を含むことを確認
  - 結果が `"Speedup"` を含むことを確認
- [x] `bench_results_json_generated` テスト追加:
  - 同ソースで `cmd_bench_aot_vm(src, 1)` を呼ぶ
  - `std::fs::read_to_string("bench-results.json")` で読む（`unwrap_or_default`）
  - JSON が `"vm"` を含むことを確認
  - JSON が `"aot"` を含むことを確認
  - JSON が `"runs"` を含むことを確認
  - `std::fs::remove_file("bench-results.json").ok();` でクリーンアップ
- [x] `cargo test v62500` で 2 件 PASS

---

## T4: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62500` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3392 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.5.0 セクションに実績を追記
  - 関数名が `cmd_bench_aot_vm`（`cmd_bench` シグネチャ競合のため）になった経緯を記載
  - 計測対象がコンパイル時間（実行時間ではなく）であることを記載
  - throughput 表示は非スコープ（パイプライン行数取得が困難）を記載
- [x] `versions/current.md` の「進行中」を v62.5.0（3392 tests）に更新、「次」を v62.6.0 に
- [x] `CHANGELOG.md` に v62.5.0 エントリを追加
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

（実装中に発覚した問題）
- **並列テスト競合**: `cmd_bench_aot_vm` が常に `bench-results.json` を書き出すと並列テストで競合。`json_out: &str` 引数を追加し、空文字列 = 非生成として `cmd_bench_runs_both_modes` は `""` を渡すよう修正。`bench_results_json_generated` は `temp_dir` のユニークパスを使用。修正済み。

（code-reviewer 指摘対応）
- **[MED] `--aot` の `file` 抽出が args[0]="fav" を拾う** — `args.iter().skip(2).find(|a| !a.starts_with('-'))` に修正。修正済み。
- **[MED] `json_out` ハードコード** — v62.6.0 で `--json-out` フラグを追加予定。現バージョンでは仕様通り `"bench-results.json"` をデフォルトとして維持。受け入れ。
- **[LOW] `p99_ms` NaN ソート** — `unwrap_or(Ordering::Equal)` → `unwrap_or(Ordering::Less)` に変更（NaN を最小扱い）。修正済み。
- **[LOW] サイレント write 失敗** — `let _ =` → `if let Err(e) = ... { eprintln!(...) }` に変更。修正済み。
- **[LOW] PERF 計測範囲の違い** — `aot_us` 計測ループにコメントを追加して「Speedup は参考値」と明記。修正済み。

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3392 passed, 0 failed（ベース 3390 + 2）
- 関数名: `cmd_bench_aot_vm`（`cmd_bench` シグネチャ競合のため）
- 計測対象: コンパイル時間（実行時間ではない）
- 完了日: 2026-08-01
