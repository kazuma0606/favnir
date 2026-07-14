# v42.0.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2874（前バージョン 2870 + 4）
**実績テスト数**: 2874 passed, 0 failed（2026-07-12）

---

## T0 — 事前確認

- [x] `cargo test` が 2870 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.9.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v42.0.0 を確認
- [x] `v41900_tests::cargo_toml_version_is_41_9_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 44650
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v41900_tests` の閉じ `}` の行番号を確認し記録: 44663
- [x] `MILESTONE.md` に `Type Precision` が含まれないことを確認
- [x] `README.md` に `Type Precision` が含まれないことを確認
- [x] `driver.rs` に `v42000_tests` モジュールが存在しないことを確認

---

## T1 — `MILESTONE.md` 更新

- [x] v42.0.0 エントリを v41.0.0 エントリの直前（先頭）に追加
  - 宣言文を含める
  - `Type Precision` という文字列を含めること
  - 達成コンポーネント表（v41.1〜v41.9 の 9 件）
  - 宣言日: 2026-07-12
- [x] `src.contains("Type Precision")` を満たすことを確認

---

## T2 — `README.md` 更新

- [x] `Type Precision`（v42.0）の記述を v41.0 記述の直後に追加
- [x] `src.contains("Type Precision")` を満たすことを確認

---

## T3 — Cargo.toml バージョン bump

- [x] `version = "41.9.0"` → `"42.0.0"`

---

## T4 — CHANGELOG.md 更新

- [x] `[v42.0.0]` エントリを `[v41.9.0]` の直前に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v41900_tests::cargo_toml_version_is_41_9_0` をスタブ化（"Stubbed: version bumped to 42.0.0"）
- [x] `v42000_tests` モジュール（4 テスト）を `v41900_tests` の直前に追加:
  - `cargo_toml_version_is_42_0_0`（NOTE コメント付き）
  - `changelog_has_v42_0_0`
  - `milestone_has_type_precision`
  - `readme_mentions_type_precision`

---

## T6 — テスト実行・確認（クリーンアップ前）

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2874 を確認（2870 + 4 件）
- [x] `v42000_tests` 4 件すべて pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T7 — ★cargo clean + hello.fav 確認 + cargo test 再実行

- [x] `cargo clean` を実行（28.5 GiB 削除）
- [x] `fav/tmp/hello.fav` の存在を確認（cargo clean 後も保持されていた）
- [x] `cargo test` を再実行し 2874 passed / 0 failed を確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.0.0（最新安定版）・v42.1.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v42.0.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）` を追記）
- [x] `versions/v40-v45/v42.0.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: T1 で実施済み

---

## 最終ステータス

- [x] 全タスク完了
