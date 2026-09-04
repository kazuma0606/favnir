# Plan: v94.5.0 — `fav bench --sap`（SAP 総合ベンチマーク）

## 実装ステップ

### Step 1: `fav/src/bench.rs` を新規作成する

`pub fn bench_sap_all() -> String` を定義する。

内容:
- SAP Advanced Benchmark Suite のヘッダ文字列を返す
- QueryBuilder / BatchRequest / Metadata Infer の各ベンチ結果をまとめた文字列を生成
- 既存の `bench_sap_query()` の出力を組み込む形で実装

注意:
- `bench.rs` は `driver.rs` への依存を避け、独立した関数として実装する
- `pub fn bench_sap_all()` は `String` を返す（Driver の他ベンチと同じ戻り型）

### Step 2: `fav/src/lib.rs` に `pub mod bench;` を追加する

`lib.rs` に `pub mod bench;` を追加して `bench.rs` モジュールを公開する。
既存の `pub mod` 宣言の末尾に追加する。

### Step 3: `fav/self/cli.fav` に `--sap` フラグ参照を追加する

cli.fav の末尾またはヘルプ出力生成箇所に、bench コンテキストの `--sap` フラグを
ドキュメントコメント（`--` 形式）で追記する。

テスト `cli_fav_has_bench_sap_flag` は `cli.fav` に `"--sap"` が含まれることを確認する。

### Step 4: `fav/src/driver.rs` に `mod v94500_tests` を追加する

`mod v94400_tests { ... }` の直後に追加。

テスト 2 件:
- `bench_sap_all_function_defined`: `std::fs::read_to_string("src/bench.rs")` で
  `bench_sap_all` の存在を確認（driver.rs は `fav/` をカレントとして実行される）
- `cli_fav_has_bench_sap_flag`: `std::fs::read_to_string("self/cli.fav")` で
  `--sap` の存在を確認

### Step 5: `CHANGELOG.md` に v94.5.0 エントリを追記する

### Step 6: `cargo build` でコンパイル確認

### Step 7: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,152 tests, 0 failures を確認する。

### Step 8: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
