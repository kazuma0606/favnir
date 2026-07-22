# Tasks: v52.5.0 — SLA 監視 Rune

Status: COMPLETE
Date: 2026-07-21

---

## T0 — 事前確認

- [x] `cargo test` 3144 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `runes/sla/` ディレクトリが**存在しない**ことを確認（`ls runes/ | grep sla` → `slack/` のみ）
- [x] `v52400_tests` に `cargo_toml_version_is_52_4_0` が**存在しない**ことを確認（削除対象なし）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("../../runes/sla/sla.fav")` → `fav/src/` から 2 階層上 = `favnir/runes/sla/sla.fav` ✓
  - [x] 他の rune の参照例: `include_str!("../../runes/prometheus/prometheus.fav")` と同じパターン ✓

## T1 — `runes/sla/sla.fav` 新規作成

- [x] `runes/sla/` ディレクトリを作成
- [x] `runes/sla/sla.fav` を作成（`runes/prometheus/prometheus.fav` パターン参照）
- [x] ファイル先頭に説明コメントを追加（rune 名・バージョン・使い方・!Observe 言及）
- [x] `check_freshness` 関数追加:
  - [x] `public fn check_freshness(timestamp: Int, max_age_seconds: Int) -> Result<Unit, String>`
  - [x] body: `Sla.check_freshness_raw(timestamp, max_age_seconds)`
  - [x] 関数の直前にコメントを追加（鮮度チェックの説明）
- [x] `check_latency` 関数追加:
  - [x] `public fn check_latency(stage: String, threshold_ms: Int) -> Result<Unit, String>`
  - [x] body: `Sla.check_latency_raw(stage, threshold_ms)`
  - [x] 関数の直前にコメントを追加（レイテンシチェックの説明）
- [x] `alert` 関数追加:
  - [x] `public fn alert(message: String) -> Result<Unit, String>`
  - [x] body: `Sla.alert_raw(message)`
  - [x] 関数の直前にコメントを追加（アラート送信の説明）
- [x] `cargo build` → コンパイルエラーなし確認
  - [x] `include_str!("../../runes/sla/sla.fav")` がテストで使われるとき解決できること

## T2 — `driver.rs` にテスト追加 + バージョン更新

- [x] `rg -n "v52400_tests" fav/src/driver.rs` で挿入位置を確認
- [x] `v52500_tests` モジュールを `v52400_tests` の直前に追加（2 件）:
  - [x] `sla_rune_latency_check`:
    - [x] `include_str!("../../runes/sla/sla.fav")` でソースを取得
    - [x] `src.contains("fn check_latency(")` を assert
    - [x] `src.contains("threshold_ms")` を assert
  - [x] `sla_rune_freshness_check`:
    - [x] `include_str!("../../runes/sla/sla.fav")` でソースを取得
    - [x] `src.contains("fn check_freshness(")` を assert
    - [x] `src.contains("max_age_seconds")` を assert
- [x] `v52400_tests` に version テストなし → 削除対象なし（確認済み）
- [x] `fav/Cargo.toml` version → `"52.5.0"`
- [x] `cargo test` 実行 → 3146 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T3 — 後処理

- [x] `CHANGELOG.md` に v52.5.0 エントリ追加
- [x] `versions/current.md` を v52.5.0（3146 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.5.0 実績欄を更新（2 点注意）:
  - [x] 推定テスト数 3145 → 実績 3146 に修正（v52.4.0 が 1 件多かったため）
  - [x] `!Observe` エフェクトはコメントのみのスタブ実装である旨を注釈追加
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
