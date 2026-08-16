# v75.6.0 タスクリスト — Stream freshness monitoring

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.5.0` であることを確認
- [x] `cargo test` が全 pass（3702 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.6.0: Stream freshness monitoring ---` コメントを追加する
- [x] `StreamFreshnessMonitor` 構造体を追加する（source: String, max_lag_secs: u64）
- [x] `StreamLagResult` 構造体を追加する（lag_secs: u64, exceeded: bool, source: String）
- [x] `check_stream_lag(last_event_ts: i64, now: i64, monitor: &StreamFreshnessMonitor) -> StreamLagResult` を追加する
  - `now.saturating_sub(last_event_ts).max(0) as u64` で lag_secs を計算
  - `lag_secs > monitor.max_lag_secs` で exceeded を判定（開区間）
- [x] `format_stream_lag_report(result: &StreamLagResult) -> String` を追加する
  - 正常時: `"[OK] source={source} lag={lag_secs}s"`
  - 超過時: `"[EXCEEDED] source={source} lag={lag_secs}s"`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.6.0 エントリを追加する
- [x] Added セクション（構造体 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v756000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `stream_lag_within_threshold` テストを実装する
  - `max_lag_secs=30` で lag=20 秒 → `exceeded=false`
  - ちょうど 30 秒 → `exceeded=false`（境界 exclusive）
  - `result.source == "kafka://orders-topic"` を `assert_eq!` で確認する
  - `format_stream_lag_report` の結果が `"[OK]"` を含む
- [x] `stream_lag_exceeded_detected` テストを実装する
  - lag=31 秒 → `exceeded=true`
  - `format_stream_lag_report` の結果が `"[EXCEEDED]"` を含む
  - 未来タイムスタンプ（`now < last_event_ts`）→ `lag_secs=0, exceeded=false`
- [x] `cargo test v756000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.5.0"` → `"75.6.0"` に変更する
- [x] `driver.rs` 内の `75.5.0` バージョン文字列アサーションを `75.6.0` に一括更新（replace_all）
- [x] `cargo test cargo_toml` で Cargo.toml バージョン検証テストが pass することを確認する

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.6.0 に更新する
- [x] 「次に切る版」を v75.7.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3704 tests）
- [x] `cargo test v756000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.6.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.6.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要（スキップ予定）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `stream_lag_within_threshold` が pass
- [x] `stream_lag_exceeded_detected` が pass
- [x] テスト総数: 3704（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | 内容 | 対応 |
|---|---|---|
| [MED] | `check_stream_lag` の doc に `saturating_sub` 選択理由が欠落 | doc コメントに「i64 オーバーフローを飽和演算で保護する」を追記 |
| [LOW] | `StreamFreshnessMonitor` / `StreamLagResult` に `PartialEq` がない | `#[derive(PartialEq)]` を追加 |
| [LOW] | format report テストで lag 数値の確認がない | `assert!(report.contains("lag=20s"))` を追加 |
| [LOW] | `now == last_event_ts`（lag=0）の明示テストがない | `stream_lag_within_threshold` に lag=0 アサーションを追加 |
