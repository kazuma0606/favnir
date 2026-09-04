# Tasks: v94.4.0 — コールドスタートベンチマーク

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,148 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94300_tests` が存在することを確認する（v94.3.0 完了済みの証拠）
- [x] `infra/lambda/sap-sync/main.tf` に `snap_start` が含まれることを確認する（v94.3.0 完了済みの証拠）

## T1: `scripts/bench_sap_coldstart.sh` を新規作成する

- [x] `scripts/bench_sap_coldstart.sh` を新規作成する
- [x] シェルバン（`#!/usr/bin/env bash`）と `set -euo pipefail` を含める
- [x] ベンチマーク結果を `fav/tmp/sap_coldstart_bench.json` に記録するロジックを追加する
- [x] スクリプト内に `sap_coldstart_bench` キー名が含まれていることを確認する（テスト要件）
- [x] P50/P95/P99 の比較表を標準出力に表示するロジックを含める

## T2: `driver.rs` に `mod v94400_tests` を追加する

- [x] `mod v94300_tests { ... }` の直後に `#[cfg(test)] mod v94400_tests { ... }` を追加する（2 テスト）
- [x] `bench_sap_coldstart_script_exists`: `../scripts/bench_sap_coldstart.sh` が存在することを確認する
- [x] `bench_sap_coldstart_output_path_defined`: スクリプトに `sap_coldstart_bench` が含まれることを確認する

## T3: `CHANGELOG.md` に v94.4.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.4.0 エントリを追加する

## T4: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,150 tests, 0 failures であることを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
