# Tasks — v58.3.0 — スキーママイグレーション / バージョニング

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.3.0 セクションを確認
- [x] ベーステスト数 3283（v58.2.0 完了時点の実績値）を確認 — `cargo test 2>&1 | grep 'tests passed'` で 3283 であることを数値確認する
- [x] `fav/Cargo.toml` が `58.2.0` であることを確認（更新前）
- [x] `v58300_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v58200_tests` が `driver.rs` に存在することを確認（`v58300_tests` の挿入位置として使用）
- [x] `cmd_schema_diff` が `driver.rs` に存在することを確認（`cmd_schema_migrate` の挿入位置として使用）
- [x] `Some("schema")` arm に `"migrate"` サブコマンドが存在しないことを確認（main.rs）
- [x] `apply_migration_transform` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] rolling チェック 5 件（v56300 / v56900 / v57000 / v57900 / v58000）が `"58.2.0"` を期待していることを確認
- [x] `driver.rs` で `serde_json` が使用済みであることを確認（追加 import 不要のはず）

---

## 実装タスク（順序厳守）

- [x] T1: `fav/Cargo.toml` version を `58.3.0` に更新
- [x] T2: `fav/src/driver.rs` — `apply_migration_transform` 追加
  - [x] 関数シグネチャ: `pub fn apply_migration_transform(mut record: serde_json::Value, defaults: &[(&str, serde_json::Value)]) -> serde_json::Value`
  - [x] `record.as_object_mut()` で既存フィールドを保持しつつ不足フィールドを `or_insert_with` で補完
  - [x] `cmd_schema_diff` の直後に追加
- [x] T3: `fav/src/driver.rs` — `cmd_schema_migrate` 追加
  - [x] 関数シグネチャ: `pub fn cmd_schema_migrate(from: &str, to: &str, data_file: &str) -> i32`
  - [x] `println!("Schema migration: {} → {}", from, to)` 出力
  - [x] `println!("  Input : {}", data_file)` 出力
  - [x] `println!("  Status: OK (dry-run mode)")` 出力
  - [x] `0` を返す
  - [x] `apply_migration_transform` の直後に追加
- [x] T4: `fav/src/main.rs` — `Some("schema")` arm に `Some("migrate")` 追加
  - [x] `Some("diff")` アームの直後に `Some("migrate")` アームを追加
  - [x] `--from` / `--to` / `--data` フラグを `args.windows(2)` でパース（デフォルト: `"v1"` / `"v2"` / `"data.jsonl"`）
  - [x] `std::process::exit(cmd_schema_migrate(from, to, data))` で終了
  - [x] `cmd_schema_migrate` を main.rs の use インポートに追加
- [x] T5: `fav/src/driver.rs` — `v58300_tests` モジュールを `v58200_tests` の直前に追加
  - [x] `use super::{apply_migration_transform, cmd_schema_migrate};` をモジュール内に追加
  - [x] `schema_migration_transforms`: `apply_migration_transform` で `currency: "JPY"` を補完し、id/amount/currency を assert
  - [x] `cmd_schema_migrate_test`: `cmd_schema_migrate("v1", "v2", "orders.jsonl")` が 0 を返すことを assert
  - [x] テスト関数名は `cmd_schema_migrate_test`（関数名との衝突を避けるため）
- [x] T6: rolling バージョンチェック 5 件を `"58.2.0"` → `"58.3.0"` に更新
  - [x] v56300_tests
  - [x] v56900_tests
  - [x] v57000_tests
  - [x] v57900_tests
  - [x] v58000_tests

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3286 tests passed, 0 failed**）（code-review 対応で +1）
  - [x] `v58300_tests::schema_migration_transforms` ok
  - [x] `v58300_tests::schema_migration_no_overwrite` ok（code-review 対応で追加）
  - [x] `v58300_tests::cmd_schema_migrate_test` ok
  - [x] `v58200_tests` 全件引き続き通過
  - [x] `v58100_tests` 全件引き続き通過
  - [x] 既存 3283 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v58.3.0 エントリを追加（形式: `## [v58.3.0] — 2026-07-28 — スキーママイグレーション / バージョニング`）
  - [x] 日付は実装完了日に合わせること（ドキュメント作成日ではなく実装完了日を記入）
- [x] T11: `versions/current.md` を v58.3.0 / 実績テスト数 に更新
  - [x] code-review 対応でテスト数が増加した場合は実績値を反映すること（ベース 3285 が変動する可能性あり）
- [x] T12: `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.3.0 実績を COMPLETE に更新
  - [x] `3283 + 2 = 3285 tests passed, 0 failed（2026-07-28）` を追記
  - [x] code-review でテスト数が増加した場合は v58.4.0 以降のベース値も修正
- [x] T13: `versions/v55-v60/v58.3.0/tasks.md` を COMPLETE に更新

---

## 完了確認

- [x] `schema_migration_transforms` pass
- [x] `cmd_schema_migrate_test` pass
- [x] **3286 tests passed, 0 failed**（ベース 3283 + 3、code-review 対応で +1）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/main.rs` の `Some("schema")` arm に `Some("migrate")` アームが追加されている
- [x] rolling チェック 5 件が `"58.3.0"` になっている
- [x] `CHANGELOG.md` に `[v58.3.0]` エントリが追加されている
- [x] `versions/current.md` が v58.3.0 / 3285 tests を反映

---

## 実装メモ

- `apply_migration_transform` は `serde_json::Value` を `mut` で受け取り、`as_object_mut()` で直接補完する（clone しない）
- 挿入順: `cmd_schema_diff` → `apply_migration_transform` → `cmd_schema_migrate`（T2 → T3 の順）
- `cmd_schema_migrate` は I/O スタブ（実際のファイル読み込みは行わない）
- テスト名 `cmd_schema_migrate_test` は関数名 `cmd_schema_migrate` との衝突を避けるための命名
- rolling チェックは全バージョンで 5 件全件更新が必要（v56300 / v56900 / v57000 / v57900 / v58000）
- `serde_json` は Cargo.toml に `serde_json = "1"` として登録済み、追加不要
- `fav schema` のエラーメッセージ（usage 文）は現時点で `diff` のみ案内している。`migrate` の案内追加は将来バージョン（v58.4 以降）のタスク
- code-review 対応でテスト数が 3285 より増加した場合: CHANGELOG・current.md・roadmap の実績値と T13 完了確認欄を実績値に修正すること（v58.1.0 で +1、v58.2.0 で +2 増加した実績あり）
