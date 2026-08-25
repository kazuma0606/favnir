# Tasks: v89.8.0 — パフォーマンス確認

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,033 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89700_tests` が存在することを確認する（v89.7.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する
- [x] `benchmarks/v80.0.0.json` が存在することを確認する（参照パターン）

## T1: `cargo test --release` で全テスト通過確認

- [x] `cargo test --release 2>&1 | grep "test result"` を実行し、4,033 tests, 0 failures を確認する

## T1.5: `fav bench --all` でベースライン乖離確認

- [x] `./target/release/fav bench --all` を実行し、既存ベースライン（`benchmarks/baseline.json`）との乖離がないことを確認する

## T2: `benchmarks/sap-odata-v89.8.0.json` を作成

- [x] `benchmarks/` ディレクトリに `sap-odata-v89.8.0.json` を新規作成する
- [x] 既存 JSON 形式（`version` / `milestone` / `date` / `tests_passed` / `tests_failed` / `duration_ms` / `notes`）を踏襲する
- [x] SAP 固有フィールド `lambda_cold_start_ms` / `pagination_1000_ms` を追加する
- [x] `"duration_ms"` フィールドが含まれていることを確認する（テスト `sap_perf_benchmark_has_duration_ms` の担保）

## T3: `mod v89800_tests` を `driver.rs` に追加

- [x] `mod v89700_tests { ... }` の直後に `#[cfg(test)] mod v89800_tests { ... }` を追加する
- [x] `sap_perf_benchmark_json_exists` テストを実装する（`"../benchmarks/sap-odata-v89.8.0.json"` の存在確認）
- [x] `sap_perf_benchmark_has_duration_ms` テストを実装する（`"duration_ms"` を含むことを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,035 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
