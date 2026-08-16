# v75.7.0 タスクリスト — Temporal contracts

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.6.0` であることを確認
- [x] `cargo test` が全 pass（3704 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.7.0: Temporal contracts ---` コメントを追加する
- [x] `TemporalContract` 構造体を追加する（name: String, freshness: Option<FreshnessPolicy>, retention: Option<RetentionPolicy>）
- [x] `validate_temporal_contract(contract: &TemporalContract, data_ts: i64, now: i64) -> Result<(), String>` を追加する
  - 鮮度チェック: age > max_age_secs → `Err("freshness violation: ...")`
  - 保持チェック: age > max_age_days * 86400 → `Err("retention exceeded: ...")`
  - `age = now.saturating_sub(data_ts).max(0) as u64`（未来 data_ts は age=0）
  - 境界値（age == max）は Ok（開区間）
- [x] `format_temporal_contract_report(contract: &TemporalContract, result: &Result<(), String>) -> String` を追加する
  - Ok 時: `"[OK] contract={name}"`
  - Err 時: `"[VIOLATION] contract={name} reason={msg}"`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.7.0 エントリを追加する
- [x] Added セクション（型 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v757000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `temporal_contract_freshness_violation` テストを実装する
  - `max_age_secs=300` で age=400 → `Err`
  - age=300（境界）→ `Ok(())`（開区間）
  - `format_temporal_contract_report` の Err ケースが `"[VIOLATION]"` と contract name を含む
  - Ok ケースが `"[OK]"` を含む
- [x] `temporal_contract_retention_exceeded` テストを実装する
  - `max_age_days=7` で 8 日後 → `Err`
  - 7 日（境界）→ `Ok(())`（開区間）
  - `format_temporal_contract_report` が `"[VIOLATION]"` / `"[OK]"` を正しく返す
- [x] `cargo test v757000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.6.0"` → `"75.7.0"` に変更する
- [x] `driver.rs` 内の `75.6.0` バージョン文字列アサーションを `75.7.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.7.0 に更新する
- [x] 「次に切る版」を v75.8.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3706 tests）
- [x] `cargo test v757000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.7.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.7.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要（スキップ予定）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `temporal_contract_freshness_violation` が pass
- [x] `temporal_contract_retention_exceeded` が pass
- [x] テスト総数: 3706（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | `FreshnessStrategy::Warn` のとき Err を返していた（コメントと矛盾） | `match fp.strategy` で Warn は Ok(()) を返すよう実装修正 |
| [HIGH] | age 計算の型（u64）と v75.5.0 apply_retention_check（i64）の不統一 | doc コメントに「v75.7.0 は saturating_mul と統一するため u64 を使用」と明記 |
| [MED] | `FreshnessStrategy::Warn` のテストが欠落 | `temporal_contract_retention_exceeded` 内に Warn テストを追加 |
| [MED] | 両フィールド None のテストが欠落 | `empty` コントラクトで常に Ok となることを確認するテストを追加 |
| [LOW] | `format_temporal_contract_report` の引数型スタイル | 現状維持（実害なし・Clippy 警告なし） |
