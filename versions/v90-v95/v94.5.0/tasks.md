# Tasks: v94.5.0 — `fav bench --sap`（SAP 総合ベンチマーク）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,150 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94400_tests` が存在することを確認する（v94.4.0 完了済みの証拠）
- [x] `scripts/bench_sap_coldstart.sh` が存在することを確認する（v94.4.0 完了済みの証拠）

## T1: `fav/src/bench.rs` を新規作成する

- [x] `fav/src/bench.rs` を新規作成する
- [x] `pub fn bench_sap_all() -> String` を定義する（SAP Advanced Benchmark Suite レポートを返す）

## T2: `fav/src/lib.rs` に `pub mod bench;` を追加する

- [x] `lib.rs` に `pub mod bench;` を追加する

## T3: `fav/self/cli.fav` に `--sap` フラグ参照を追加する

- [x] `cli.fav` に bench コンテキストの `--sap` フラグをドキュメントコメントで追記する
- [x] `cli.fav` に `"--sap"` が含まれることを確認する（テスト要件）

## T4: `driver.rs` に `mod v94500_tests` を追加する

- [x] `mod v94400_tests { ... }` の直後に `#[cfg(test)] mod v94500_tests { ... }` を追加する（2 テスト）
- [x] `bench_sap_all_function_defined`: `src/bench.rs` に `bench_sap_all` が含まれることを確認する
- [x] `cli_fav_has_bench_sap_flag`: `self/cli.fav` に `--sap` が含まれることを確認する

## T5: `CHANGELOG.md` に v94.5.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.5.0 エントリを追加する

## T6: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T7: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,152 tests, 0 failures であることを確認する

## T8: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T7 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
  - 修正: `bench.rs` の `filter.replace("eq", "eq")` を除去（no_effect_replace lint）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 実装メモ

- `cli.fav` は `//` スタイルコメントのみ対応（`--` は parse エラーになる）
- `cli.fav` コメントの全角文字（`（）`）はパーサーエラーを引き起こす → ASCII のみ使用する
- `lib.rs` への `bench` モジュール登録は `#[cfg(not(target_arch = "wasm32"))]` でガード
- `bench.rs` は `lib.rs` に加えて `main.rs` にも `mod bench;` が必要（binary crate と library crate は別）
- `driver.rs` から `crate::bench::bench_sap_all()` を呼ぶためには main.rs に `mod bench;` が必須

## code-reviewer 指摘と対応

- [HIGH] `BenchOpts::sap` フィールド・`cmd_bench` early-return・main.rs `--sap` アームが欠落 → 追加
- [HIGH] `bench_parse_edmx` / `bench_entity_type_to_favnir` の計測単位が ms（実態はµs）→ `* 1_000_000.0` + `µs/op` 表記に修正
- [MED] bench 関数に `black_box` なし → 全ループで `std::hint::black_box(...)` 追加
- [LOW] "Total: 4 benchmarks" → "Total: 6 benchmarks" に修正
