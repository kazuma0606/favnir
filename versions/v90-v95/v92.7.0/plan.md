# Plan: v92.7.0 — QueryBuilder<T> ベンチマーク（--sap-query）

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4109 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92600_tests` が存在することを確認
- `fav/src/driver.rs` の `BenchOpts` 構造体を Read し、既存フィールドを確認する
- `fav/tmp/hello.fav` が存在することを確認
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `BenchOpts` に `sap_query` フィールドを追加

`fav/src/driver.rs` の `BenchOpts` 構造体に追加：

```rust
// v92.7.0: --sap-query → SAP QueryBuilder ベンチマーク
pub sap_query: bool,
```

`impl Default for BenchOpts` にも `sap_query: false,` を追加する。

### Step 2: `bench_sap_query` 関数を追加

既存の `cmd_bench_all` 関数の直後に追加：

```rust
/// v92.7.0: SAP QueryBuilder ベンチマーク（--sap-query）
/// query() + with_filter + with_select のチェーン速度と fetch_all_pages パターンを計測する。
pub fn bench_sap_query() -> String {
    use std::time::Instant;

    let chain_iters = 10_000u64;
    let t = Instant::now();
    for _ in 0..chain_iters {
        let _chain = format!("query<BusinessPartner>() |> with_filter(Eq(\"Country\",\"JP\")) |> with_select([\"BusinessPartner\"])");
    }
    let chain_us = t.elapsed().as_micros() as f64 / chain_iters as f64;

    let page_iters = 1_000u64;
    let t2 = Instant::now();
    for _ in 0..page_iters {
        // fetch_all_pages スタブ（Result.err）のオーバーヘッド計測
        let _result: Result<Vec<String>, String> = Err("fetch_all_pages: not yet implemented (v92.4.0 stub)".to_string());
    }
    let page_us = t2.elapsed().as_micros() as f64 / page_iters as f64;

    format!(
        "SAP QueryBuilder benchmark\n\
         query() + with_filter + with_select: {chain_us:.1} µs/op\n\
         fetch_all_pages (stub overhead):     {page_us:.3} µs/op"
    )
}
```

### Step 3: `cmd_bench` に `--sap-query` 分岐を追加

`cmd_bench` 関数内の `if opts.all { ... }` 分岐の前に追加：

```rust
if opts.sap_query {
    println!("{}", bench_sap_query());
    return true;
}
```

### Step 4: `driver.rs` に `mod v92700_tests` を追加

`mod v92600_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92700_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn bench_sap_query_flag_defined() {
        let content = std::fs::read_to_string("src/driver.rs")
            .expect("src/driver.rs should exist");
        assert!(
            content.contains("bench_sap_query"),
            "driver.rs should define bench_sap_query"
        );
    }
    #[test]
    fn bench_sap_query_measures_pagination() {
        let content = std::fs::read_to_string("src/driver.rs")
            .expect("src/driver.rs should exist");
        assert!(
            content.contains("fetch_all_pages"),
            "driver.rs bench_sap_query should mention fetch_all_pages"
        );
    }
}
```

注意: ロードマップは `"src/bench.rs"` を参照するが、`bench.rs` は存在しないため `"src/driver.rs"` を使用する。

### Step 5: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4111 passed; 0 failed`

### Step 6: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（BenchOpts.sap_query 追加）
  → Step 2（bench_sap_query 関数追加）
  → Step 3（cmd_bench 分岐追加）
  → Step 4（driver.rs: テスト追加）
  → Step 5（cargo test）
  → Step 6（CI 事前確認）
```
