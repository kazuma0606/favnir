# v75.5.0 タスクリスト — `RetentionPolicy` 型

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.4.0` であることを確認
- [x] `cargo test` が全 pass（3700 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型定義追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.5.0: RetentionPolicy 型 ---` コメントを追加する
- [x] `RetentionAction` enum を追加する（Delete, Archive, Anonymize）— `#[derive(Debug, Clone, PartialEq)]`
- [x] `RetentionResult` enum を追加する（Keep, Delete, Archive, Anonymize）— `#[derive(Debug, Clone, PartialEq)]`
- [x] `RetentionPolicy` 構造体を追加する（max_age_days: u64, action: RetentionAction）
- [x] `apply_retention_check(row_ts: i64, now: i64, policy: &RetentionPolicy) -> RetentionResult` を追加する
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.5.0 エントリを追加する
- [x] Added セクション（型 3 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v755000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `retention_delete_old_rows` テストを実装する
  - `max_age_days=365, action=Delete` で 366 日後 → `RetentionResult::Delete`
  - ちょうど 365 日 → `RetentionResult::Keep`（boundary exclusive）
  - 100 日後 → `RetentionResult::Keep`
- [x] `retention_anonymize_action` テストを実装する
  - `max_age_days=90, action=Anonymize` で 91 日後 → `RetentionResult::Anonymize`
  - ちょうど 90 日 → `RetentionResult::Keep`
  - `now < row_ts`（未来行）→ `RetentionResult::Keep`
- [x] `cargo test v755000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.4.0"` → `"75.5.0"` に変更する
- [x] `driver.rs` 内の `75.4.0` バージョン文字列アサーションを `75.5.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.5.0 に更新する
- [x] 「次に切る版」を v75.6.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3702 tests）
- [x] `cargo test v755000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.5.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.5.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要（スキップ予定）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `retention_delete_old_rows` が pass
- [x] `retention_anonymize_action` が pass
- [x] テスト総数: 3702（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | 内容 | 対応 |
|---|---|---|
| [MED] | `RetentionAction` → `RetentionResult` の match が将来バリアント追加で漏れるリスク | `From<RetentionAction> for RetentionResult` を実装し apply_retention_check を簡潔化 |
| [MED] | `now - row_ts` 裸の減算でデバッグビルドパニックの可能性 | `now.saturating_sub(row_ts)` に変更 |
| [MED] | `max_age_days as i64` 理論的オーバーフロー | doc コメントに実用上限（i64::MAX / 86400）を明記 |
| [LOW] | Archive アクションのテストが欠落 | `retention_anonymize_action` 内に Archive ケースを追加 |
| [LOW] | `max_age_days=0` の doc コメントが不正確 | `row_ts < now である全行が対象` に修正、テストケース追加 |
