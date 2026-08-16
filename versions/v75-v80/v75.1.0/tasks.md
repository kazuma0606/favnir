# v75.1.0 タスクリスト — `FreshnessPolicy` 型基盤

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.0.0` であることを確認
- [x] `cargo test` が全 pass（3692 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型定義・関数追加

- [x] `fav/src/driver.rs` に `FreshnessStrategy` enum を追加する（Warn / Fail）
- [x] `FreshnessPolicy` 構造体を追加する（max_age_secs: u64, strategy: FreshnessStrategy）
- [x] `check_freshness(data_ts: i64, now: i64, policy: &FreshnessPolicy) -> bool` を追加する
- [x] `format_freshness_warning(policy: &FreshnessPolicy, age_secs: u64) -> String` を追加する
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.1.0 エントリを追加する
- [x] エントリに Added セクション（型定義・関数）と Tests セクション（2件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v751000_tests` モジュールを追加する
- [x] `freshness_policy_enforced` テストを実装する
  - TTL 内（299秒前）のデータで `check_freshness` が `true` を返すことを assert
- [x] `freshness_stale_detected` テストを実装する
  - TTL 超過（301秒前）のデータで `check_freshness` が `false` を返すことを assert
  - `format_freshness_warning` が "STALE" と "max_age=300s" を含むことを assert
- [x] `cargo test v751000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.0.0"` → `"75.1.0"` に変更する

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.1.0 に更新する
- [x] 「次に切る版」を v75.2.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3694 tests）
- [x] `cargo test v751000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.1.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.1.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `freshness_policy_enforced` が pass
- [x] `freshness_stale_detected` が pass
- [x] テスト総数: 3694（+2）
