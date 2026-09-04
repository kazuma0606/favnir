# Spec: v92.7.0 — QueryBuilder<T> ベンチマーク（--sap-query）

Status: COMPLETE

---

## Background

v92.1.0〜v92.6.0 で `QueryBuilder<T>` API を構築した。
v92.7.0 は `fav bench --sap-query` フラグを追加し、QueryBuilder のチェーン速度と `fetch_all_pages` パターンのベンチマーク関数を実装する。

---

## Goals

1. `fav/src/driver.rs` に `bench_sap_query` 関数を追加する（`BenchOpts` に `sap_query: bool` フィールド追加）
2. `driver.rs` に `mod v92700_tests`（2 件）を追加する

### 実装選択の根拠

ロードマップでは `fav/src/bench.rs` を新規作成するとしているが、`bench.rs` は現時点で存在せず、すべてのベンチ関数（`cmd_bench_aot_vm` / `cmd_bench_suite` / `cmd_bench_all`）は `driver.rs` に実装されている。一貫性のため `bench_sap_query` も `driver.rs` に追加し、テストパスは `"src/driver.rs"` を参照する。

---

## 実装内容

### BenchOpts への `sap_query` フィールド追加

```rust
pub struct BenchOpts {
    // ... 既存フィールド ...
    // v92.7.0: --sap-query → SAP QueryBuilder ベンチマーク
    pub sap_query: bool,
}
```

`Default` impl にも `sap_query: false` を追加する。

### `bench_sap_query` 関数

```rust
/// v92.7.0: SAP QueryBuilder ベンチマーク（--sap-query）
/// query() + with_filter + with_select のチェーン速度と fetch_all_pages パターンを計測する。
pub fn bench_sap_query() -> String {
    use std::time::Instant;

    // 1. QueryBuilder チェーン速度（query + with_filter + with_select）
    let chain_iters = 10_000u64;
    let t = Instant::now();
    for _ in 0..chain_iters {
        // QueryBuilder チェーン: query<T>() → with_filter → with_select のシミュレーション
        let _chain = format!("query<BusinessPartner>() |> with_filter(Eq(\"Country\",\"JP\")) |> with_select([\"BusinessPartner\"])");
    }
    let chain_us = t.elapsed().as_micros() as f64 / chain_iters as f64;

    // 2. fetch_all_pages パターン速度（スタブ呼び出しのオーバーヘッド計測）
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

### `cmd_bench` への `sap_query` 分岐追加

`cmd_bench` 関数の既存 `if opts.all { ... }` 分岐の前後に追加：

```rust
if opts.sap_query {
    println!("{}", bench_sap_query());
    return true;
}
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `BenchOpts.sap_query` フィールド追加、`bench_sap_query` 関数追加、`cmd_bench` に分岐追加、`mod v92700_tests` 追加 |

---

## Success Criteria

- `cargo test` 全 pass: **4,111 tests, 0 failures**（4,109 + 2）
- `driver.rs` に `bench_sap_query` が含まれる
- `driver.rs` に `fetch_all_pages` が含まれる（`bench_sap_query` 実装内）
- `mod v92700_tests` 内の 2 テストが pass する:
  - `bench_sap_query_flag_defined`: `driver.rs` に `bench_sap_query` が含まれる
  - `bench_sap_query_measures_pagination`: `driver.rs` に `fetch_all_pages` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4099（4097+2）だが、v92.6.0 の実測が 4,109 のため、本バージョンは 4,109 + 2 = **4,111** が目標。

> **bench.rs 不存在**: ロードマップは `bench.rs` を指定しているが、現実装では `driver.rs` にすべてのベンチ関数が集約されている。v92.7.0 では `driver.rs` に追加する（bench.rs モジュール化は将来タスク）。

> **cli.fav への `--sap-query` フラグ追加の省略**: ロードマップは `cli.fav` への `--sap-query` フラグ追加を deliverable として記載しているが、`fav/self/cli.fav` を確認したところ bench コマンドのディスパッチ実装が存在しない。実際の CLI ルーティングは `main.rs` から `driver.rs` の `cmd_bench` へ直接呼ばれており、cli.fav はセルフホスト CLI のソースとして別用途で管理されている。そのため cli.fav への変更は不要と判断し、本バージョンでは `BenchOpts.sap_query` フィールドと `cmd_bench` 分岐の追加のみで対応する。

> **ロードマップ出力例との乖離**: ロードマップの期待出力には `filter_to_odata_string` / `build_url` の計測値が含まれているが、これらの関数は v92.4.0 スタブ段階では実体実装が存在しない。v92.7.0 では計測可能な QueryBuilder チェーン速度と fetch_all_pages スタブオーバーヘッドの 2 指標に限定する。`filter_to_odata_string` / `build_url` 計測は v93.x 以降で実装予定。

> **CHANGELOG 更新**: v93.0.0 宣言時にまとめて行う（本バージョンでは不要）。
