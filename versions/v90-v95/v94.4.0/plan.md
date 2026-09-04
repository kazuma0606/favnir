# Plan: v94.4.0 — コールドスタートベンチマーク

## 実装ステップ

### Step 1: `scripts/bench_sap_coldstart.sh` を新規作成する

既存の `scripts/run-sap-demo.sh` などを参考にスタイルを統一する。

スクリプトの内容:
- シェルバン（`#!/usr/bin/env bash`）
- `set -euo pipefail` でエラー時即座に終了
- ベンチマーク計測ロジック（AWS CLI で Lambda を invoke して cold start 時間を取得、またはサンプル値でモック）
- 結果を `fav/tmp/sap_coldstart_bench.json` に書き出す
  - キー名に `sap_coldstart_bench` を含める（テスト `bench_sap_coldstart_output_path_defined` の要件）
- 標準出力に P50/P95/P99 の比較表を表示
- 実行権限（chmod +x）は作成後に確認

注意: `fav/tmp/` ディレクトリへの書き込みを想定。スクリプト内で `mkdir -p` を実行する。

### Step 2: `fav/src/driver.rs` に `mod v94400_tests` を追加する

`mod v94300_tests { ... }` の直後に追加。

テスト 2 件:
- `bench_sap_coldstart_script_exists`: `std::path::Path::new("../scripts/bench_sap_coldstart.sh").exists()` を assert
- `bench_sap_coldstart_output_path_defined`: `std::fs::read_to_string("../scripts/bench_sap_coldstart.sh")` で `sap_coldstart_bench` の存在を確認

### Step 3: `CHANGELOG.md` に v94.4.0 エントリを追記する

先頭に v94.4.0 エントリを追加する。

### Step 4: `cargo build` でコンパイル確認

driver.rs の変更がエラーなくコンパイルされることを確認する。

### Step 5: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,150 tests, 0 failures を確認する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
