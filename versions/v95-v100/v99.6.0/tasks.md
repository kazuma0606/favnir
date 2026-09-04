# Tasks: v99.6.0 — SLA モニタリング + `fav sla-check`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.5.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.5.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99500_tests` が存在することを確認する（v99.5.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,267 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: driver.rs に SlaDefinition / SlaViolation / cmd_sla_check を追加

- [x] `pub struct SlaDefinition` を `driver.rs` に追加する（`endpoint: String` / `max_latency_ms: u32` / `availability: f64`）
- [x] `pub struct SlaViolation` を `driver.rs` に追加する（`sla: SlaDefinition` / `actual_ms: u32` / `timestamp: String`）
- [x] `pub fn cmd_sla_check(config: &str, from: &str, to: &str) -> String` を `driver.rs` に追加する
- [x] `cmd_sla_check` がスタブ実装（`format!` でレポートを返す）であることを確認する
- [x] `#[derive(Debug, Clone)]` が両構造体に付いていることを確認する
- [x] Rust ドキュメントコメント（`///`）が各定義に付いていることを確認する
- [x] 挿入位置が `mod v99400_tests` の直後・`mod v99500_tests` の直前であることを確認する

## T2: main.rs に sla-check サブコマンドルーティングを追加

- [x] `main.rs` の既存サブコマンド分岐に `"sla-check"` ケースを追加する
- [x] `cmd_sla_check(config, from, to)` を呼び出し、結果を `println!` で出力することを確認する

## T3: driver.rs に mod v99600_tests を追加

- [x] `mod v99500_tests` の直後に `mod v99600_tests`（2 テスト）を追加する:
  - `sla_check_struct_defined`: `SlaDefinition` / `SlaViolation` が `driver.rs` に含まれることを確認
  - `sla_check_cmd_defined`: `cmd_sla_check` が `driver.rs` に含まれることを確認
- [x] テストが `include_str!("driver.rs")` を使用していることを確認する
- [x] `mod v99600_tests` ブロック先頭に `// use super::* は不要（include_str! のみ使用）` という Rust コメントを 1 行追記する

## T4: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,269 tests, 0 failures であることを確認する

## T5: CHANGELOG.md に v99.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.6.0]` エントリを追加する

## T6: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.6.0` に更新する
- [x] 最新安定版を `v99.6.0` に更新する（テスト数 4,269）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- SlaDefinition / SlaViolation の runes/sap-odata/sla.fav 化は将来バージョンで対応予定 -->
<!-- 実際の SLA 測定・TOML 設定ファイル解析は将来バージョンで対応予定 -->

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED/BUG] | `v99600_tests` のアサート数が 3 で spec 要件（4）に不足 | `sla_check_cmd_defined` に `content.contains("pub fn cmd_sla_check")` アサートを追加 |
| 注意（後続実装時） | `config` をファイルパスとして使う際は `..` トラバーサル検証を追加すること | 現バージョンはスタブのため対応なし（後続バージョンで対応予定） |
