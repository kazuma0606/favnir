# Tasks: v99.3.0 — Rate Limiting / Circuit Breaker

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.2.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.2.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99200_tests` が存在することを確認する（v99.2.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,261 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: resilience.fav を新規作成

- [x] `runes/sap-odata/resilience.fav` を新規作成する
- [x] ファイル先頭コメントに `-- runes/sap-odata/resilience.fav` が含まれることを確認する
- [x] `CircuitState` 列挙型（`Closed` / `Open` / `HalfOpen`）が定義されていることを確認する
- [x] `CircuitBreaker<T>` 型（`state` / `failure_count` / `threshold` / `reset_timeout_ms` / `tag`）が定義されていることを確認する
- [x] `circuit_breaker_default<T>(tag: String) -> CircuitBreaker<T>` 関数が実装されていることを確認する
- [x] `circuit_breaker_call_mock<T>(cb: CircuitBreaker<T>, value: T) -> Result<T, String>` 関数が実装されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: sap_odata.fav に use と re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の `use` 宣言ブロックに `use sap_odata.resilience` を追加する
- [x] `sap_odata.fav` 末尾に `-- Circuit Breaker 型 re-export（v99.3.0〜）` セクションを追加する
- [x] `CircuitState` / `CircuitBreaker` / `circuit_breaker_call_mock` の 3 シンボルが re-export されていることを確認する
- [x] `circuit_breaker_default` も re-export されていることを確認する（ヘルパー関数）

## T3: driver.rs に mod v99300_tests を追加

- [x] `mod v99200_tests` の直後に `mod v99300_tests`（2 テスト）を追加する:
  - `resilience_fav_exists`: `runes/sap-odata/resilience.fav` の存在を確認
  - `resilience_fav_has_circuit_breaker`: `CircuitState` / `CircuitBreaker` / `circuit_breaker_default` / `circuit_breaker_call_mock` が含まれることを確認
- [x] `mod v99300_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T4: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,263 tests, 0 failures であることを確認する

## T5: CHANGELOG.md に v99.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.3.0]` エントリを追加する

## T6: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.3.0` に更新する
- [x] 最新安定版を `v99.3.0` に更新する（テスト数 4,263）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- ctx.sap_circuit の AppCtx への組み込みは将来バージョンで対応予定 -->

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `CircuitBreaker<T>` のファントム型パラメータの意図が未説明 | `resilience.fav` に `-- T は Circuit Breaker が保護する値の型（ファントム型パラメータ）` コメントを追加 |
| [MED] | `sap_odata.fav` の `circuit_breaker_default` 戻り型が `resilience.CircuitBreaker<T>`（完全修飾）で既存パターンと不一致 | re-export 済みエイリアス `CircuitBreaker<T>` に統一 |
