# Spec: v94.5.0 — `fav bench --sap`（SAP 総合ベンチマーク）

## Background

v94.4.0 でコールドスタートベンチマークスクリプトを追加した。
v94.5.0 では SAP 関連の全ベンチマーク（QueryBuilder・$batch・Metadata Infer）を
一括実行する `fav bench --sap` コマンドを実装する。

ベンチマーク関数は新規ファイル `fav/src/bench.rs` に集約し、
既存の `fav/src/driver.rs` から切り出した形で実装する。

### 既存の実装との関係

- `fav bench --all`: `driver::cmd_bench_all()` — 汎用 intrinsic ベンチ（v70.3.0）
- `fav bench --sap-query`: `driver::bench_sap_query()` — QueryBuilder ベンチ（v92.7.0）
- `fav bench --sap`（本バージョン）: `bench::bench_sap_all()` — SAP 全ベンチ一括

`bench.rs` の `bench_sap_all()` は既存の `bench_sap_query()` を組み込み、
`$batch` と `Metadata Infer` のベンチ結果も含めた総合レポートを返す。

## Goals

1. `fav/src/bench.rs` を新規作成し `bench_sap_all()` 関数を追加する
2. `fav/self/cli.fav` に `--sap` フラグ（bench コンテキスト）の参照コメントを追加する
   - `fav bench` コマンドは main.rs 側の Rust コードで処理される（CLI ルーティング）
   - cli.fav は Favnir 自己記述 CLI として機能し、`--sap` フラグを
     ドキュメントコメント（`--` 形式）で self-documentation として記載する
   - ロードマップの「呼び出す」は「cli.fav が --sap フラグを文書化する」意味で解釈する

## Syntax/API Examples

```
$ fav bench --sap

SAP Advanced Benchmark Suite
=============================
QueryBuilder:
  query() + 3 chains:              0.9 µs/op
  filter_to_odata_string (complex): 1.1 µs/op

BatchRequest:
  batch_request (100 ops):         12 µs/op
  change_set serialization:         8 µs/op

Metadata Infer:
  parse_edmx (A_BusinessPartner): 2.3 ms
  entity_type_to_favnir:          0.4 ms

Total: 4 benchmarks, all PASS
```

```rust
// fav/src/bench.rs（抜粋）
pub fn bench_sap_all() -> String {
    // QueryBuilder / BatchRequest / Metadata Infer の各ベンチを実行し
    // 総合レポートを返す
}
```

## Success Criteria

- `fav/src/bench.rs` が存在し、`bench_sap_all` が含まれる
- `fav/self/cli.fav` に `--sap` が含まれる（bench コンテキストのフラグ参照）
- `driver.rs` の `mod v94500_tests` が pass する
  - `bench_sap_all_function_defined`: `fav/src/bench.rs` に `bench_sap_all` が含まれる
  - `cli_fav_has_bench_sap_flag`: `fav/self/cli.fav` に `--sap` が含まれる
- `cargo test 2>&1 | grep "test result"` が 4,152 tests, 0 failures を示す（着手前: 4,150）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/bench.rs` | **新規作成** | `pub fn bench_sap_all() -> String` |
| `fav/src/lib.rs` | **追記** | `pub mod bench;` を追加 |
| `fav/self/cli.fav` | **追記** | `--sap` フラグ参照（bench コンテキストのドキュメントコメント） |
| `fav/src/driver.rs` | **追加** | `mod v94500_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.5.0 エントリ |
