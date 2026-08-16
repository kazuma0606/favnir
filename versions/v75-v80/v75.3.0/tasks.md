# v75.3.0 タスクリスト — SCD Type 1 / Type 2 ネイティブ型

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.2.0` であることを確認
- [x] `cargo test` が全 pass（3696 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型定義追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.3.0: SCD Type 1 / Type 2 ネイティブ型 ---` コメントを追加する
- [x] `ScdType` enum を追加する（Type1, Type2）
- [x] `ScdRow` 構造体を追加する（valid_from: i64, valid_to: Option<i64>, is_current: bool, data: String）
- [x] `apply_scd2_update(existing: &[ScdRow], new_data: &str, new_ts: i64) -> Vec<ScdRow>` を追加する
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.3.0 エントリを追加する
- [x] Added セクション（型・関数 3 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v753000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `scd2_creates_history_row` テストを実装する
  - `existing` に 1 件 `is_current=true, data="旧"` のレコード
  - `apply_scd2_update` の結果が 2 件（旧 + 新）
  - 新レコードの `is_current=true`, `valid_from=new_ts`, `valid_to=None` を assert
- [x] `scd2_marks_previous_expired` テストを実装する
  - 旧レコードの `is_current=false`, `valid_to=Some(new_ts - 1)` を assert
  - no-op ケース（data 同一）で新レコードが追加されないことを assert
- [x] `cargo test v753000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.2.0"` → `"75.3.0"` に変更する
- [x] `driver.rs` 内の `75.2.0` バージョン文字列アサーションを `75.3.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.3.0 に更新する
- [x] 「次に切る版」を v75.4.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3698 tests）
- [x] `cargo test v753000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.3.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.3.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要（スキップ確認済み）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `scd2_creates_history_row` が pass
- [x] `scd2_marks_previous_expired` が pass
- [x] テスト総数: 3698（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | ID | 概要 | 対応 |
|---|---|---|---|
| [HIGH] | BUG-1 | `new_ts=0` で `valid_to=Some(-1)` になる | `new_ts <= 0` バリデーション追加、`Result` 返却に変更済み |
| [HIGH] | BUG-2 | 複数 `is_current=true` でデータ汚染 | `current_count > 1` バリデーション追加済み |
| [HIGH] | DESIGN | `ScdType` が Dead Code | `apply_scd1_update` を追加して `ScdType` を活用 |
| [MED] | BUG-3 | 全行 expired 時に no-op 誤判定 | `!has_current` チェック追加済み |
| [MED] | STYLE | `pub` スコープ | API として `pub` は適切と判断、現状維持 |
| [LOW] | TEST | テスト境界値カバレッジ不足 | 既存テスト内に `new_ts=0`・複数 current・全 expired ケースを追加 |
| [LOW] | DOC | `new_ts` 前提条件がドキュメント未記載 | `# Errors` セクションをドキュメントコメントに追記済み |
