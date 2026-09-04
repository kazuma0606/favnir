# Tasks: v99.7.0 — 負荷テスト・総合ベンチマーク

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.7.0/` ディレクトリが存在することを確認する（存在しなければ `mkdir versions/v95-v100/v99.7.0/` で作成する）
- [x] `versions/v95-v100/v99.6.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.6.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99600_tests` が存在することを確認する（v99.6.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,269 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: benchmark_results.md を新規作成

- [x] `versions/v95-v100/v99.7.0/benchmark_results.md` を新規作成する
- [x] ファイルに 5 計測対象すべての結果が含まれることを確認する:
  - `delta_fetch<BusinessPartner>()` スループット
  - `ctx.sap_env("PRD")` 環境切替オーバーヘッド
  - `CircuitBreaker.call()` オーバーヘッド
  - `Masked<T>` / `unmask_mock()` コスト
  - マルチテナント 100 並列リクエスト p50 / p99
- [x] `delta_fetch` というキーワードが含まれることを確認する
- [x] `CircuitBreaker` というキーワードが含まれることを確認する
- [x] `Masked` というキーワードが含まれることを確認する
- [x] 「全項目 SLA 準拠」または「判定」セクションが含まれることを確認する

## T2: driver.rs に mod v99700_tests を追加

- [x] `mod v99600_tests` の直後に `mod v99700_tests`（2 テスト）を追加する:
  - `benchmark_results_exists`: `../versions/v95-v100/v99.7.0/benchmark_results.md` の存在を確認
  - `benchmark_results_has_targets`: `delta_fetch` / `CircuitBreaker` / `Masked` が含まれることを確認（3 アサート）
- [x] `mod v99700_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する
- [x] テストが `std::fs::read_to_string` を使用していることを確認する

## T3: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,271 tests, 0 failures であることを確認する

## T4: CHANGELOG.md に v99.7.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.7.0]` エントリを追加する

## T5: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.7.0` に更新する
- [x] 最新安定版を `v99.7.0` に更新する（テスト数 4,271）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [LOW] | 判定行で CB / Masked の個別 SLA 数値が `< 0.1 ms` にまとめられており個別記載なし | spec に個別 SLA 定義がないため対応不要 |
