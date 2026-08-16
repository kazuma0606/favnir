# v75.4.0 タスクリスト — Temporal join（時点結合）

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.3.0` であることを確認
- [x] `cargo test` が全 pass（3698 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型定義追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.4.0: Temporal join（時点結合） ---` コメントを追加する
- [x] `TemporalJoinConfig` 構造体を追加する（left_key, right_key, as_of_field: String）
- [x] `validate_temporal_join_config(config: &TemporalJoinConfig) -> Result<(), String>` を追加する
- [x] `format_temporal_join_sql(left_table, right_table, config) -> String` を追加する
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.4.0 エントリを追加する
- [x] Added セクション（型・関数 3 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v754000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `temporal_join_sql_generated` テストを実装する
  - `JOIN prices ON orders.product_id = prices.product_id` を含む
  - `prices.valid_from <= orders.order_date` を含む
  - `prices.valid_to IS NULL OR prices.valid_to > orders.order_date` を含む
- [x] `temporal_join_invalid_config_rejected` テストを実装する
  - `left_key=""` / `right_key=""` / `as_of_field=""` がそれぞれ `Err` を返す
  - `left_key="order-date"` など英数字アンダースコア以外を含む場合も `Err` を返す
  - 全フィールド正常値が `Ok(())` を返す
- [x] `cargo test v754000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.3.0"` → `"75.4.0"` に変更する
- [x] `driver.rs` 内の `75.3.0` バージョン文字列アサーションを `75.4.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.4.0 に更新する
- [x] 「次に切る版」を v75.5.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3700 tests）
- [x] `cargo test v754000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.4.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.4.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要（スキップ確認済み）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `temporal_join_sql_generated` が pass
- [x] `temporal_join_invalid_config_rejected` が pass
- [x] テスト総数: 3700（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | 内容 | 対応 |
|---|---|---|
| [MED] | テーブル名 SQL インジェクション検証が呼び出し側任せ | 設計上の判断（spec.md 責任境界に明記済み、doc コメントあり）。現状維持 |
| [MED] | `valid_to` 境界が exclusive か doc コメント未記載 | doc コメントに「valid_to は exclusive（開区間）」を明記 |
| [LOW] | `temporal_join_sql_generated` が `validate` を呼んでいない | `assert!(validate_temporal_join_config(&config).is_ok())` を追加 |
| [LOW] | `right_key`・`as_of_field` の不正文字ケースがない | `bad_chars_right`・`bad_chars_asof` ケースを追加 |
