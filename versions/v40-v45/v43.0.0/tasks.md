# v43.0.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2900（前バージョン 2896 + 4）
**実績テスト数**: 2900 passed, 0 failed（2026-07-12）

---

## T0 — 事前確認

- [x] `cargo test` が 2896 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.9.0` であることを確認
- [x] `v42900_tests::cargo_toml_version_is_42_9_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録（line 44658）
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること（確認済み・欠落なし）
- [x] `v42900_tests` の閉じ `}` の行番号を確認し記録（line 44671）
- [x] `MILESTONE.md` に `Real-Time Power` が含まれないことを確認
- [x] `README.md` に `Real-Time Power` が含まれないことを確認
- [x] `driver.rs` に `v43000_tests` モジュールが存在しないことを確認

---

## T1 — `MILESTONE.md` 更新

- [x] v43.0.0 エントリを v42.0.0 エントリの直前（先頭）に追加
  - 宣言文を含める
  - `Real-Time Power` という文字列を含めること
  - 達成コンポーネント表（v42.1〜v42.9 の 9 件）
  - 宣言日: 2026-07-12
- [x] `src.contains("Real-Time Power")` を満たすことを確認

---

## T2 — `README.md` 更新

- [x] `Real-Time Power`（v43.0）の記述を v42.0 記述の直後に追加
- [x] `src.contains("Real-Time Power")` を満たすことを確認

---

## T3 — `fav/Cargo.toml` バージョン bump

- [x] `version = "42.9.0"` → `"43.0.0"`

---

## T4 — `CHANGELOG.md` 更新

- [x] `[v43.0.0]` エントリを `[v42.9.0]` の直前に追加
- [x] Real-Time Power 宣言・cargo clean の旨を記載

---

## T5 — `driver.rs` テストモジュール更新

- [x] `v42900_tests::cargo_toml_version_is_42_9_0` をスタブ化（`assert!(true)` + "Stubbed: version bumped to 43.0.0" コメント）
- [x] `v43000_tests` モジュール（4 テスト）を `v42900_tests` の直前に追加:
  - `cargo_toml_version_is_43_0_0`（NOTE コメント付き）
  - `changelog_has_v43_0_0`
  - `milestone_has_real_time_power`
  - `readme_mentions_real_time_power`

---

## T6 — テスト実行・確認（クリーンアップ前）

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2900 を確認（2896 + 4 件）
- [x] `v43000_tests` 4 件すべて pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T7 — ★cargo clean + hello.fav 確認 + cargo test 再実行

- [x] `cargo clean` を実行（29.9 GiB 削除）
- [x] `fav/tmp/hello.fav` の存在を確認（cargo clean 後も保持されていた）
- [x] `cargo test` を再実行し 2900 passed / 0 failed を確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v43.0.0（最新安定版）・v43.1.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v43.0.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] `versions/v40-v45/v43.0.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [MED-1]: tasks.md T7 の `hello.fav` 復元手順に復元コード内容が記載されていなかった → 追加
- [MED-2]: spec.md 影響テーブルに `versions/current.md` と `versions/roadmap/roadmap-v42.1-v43.0.md` が欠落していた → 追加
- [MED-3]: spec.md 完了条件に `versions/current.md` 更新が含まれていなかった → 追加
- [LOW-1]: spec.md 宣言文で `@max_inflight` と `#[max_inflight]` が混在 → `#[max_inflight]` に統一（replace_all）
- [LOW-2]: tasks.md spec-reviewer 指摘記録欄が形式的に未記入 → 上記対応完了をここに記録することで解消

## code-reviewer 指摘・対応記録

- [MED]: v43000_tests の挿入位置が v41500_tests（行 44633）の直後に見えることを指摘。実際は v42900_tests の直前（行 44654）が正しいプロジェクト慣習（最新版を直前バージョンの前に挿入）に従っており対応不要。v41500_tests が v42.x ブロック全体の直上にある構造的問題はこのバージョン以前から既存であり、今回のスコープ外と判断
- [LOW-1]: v42900_tests スタブの `assert!(true)` が既存パターン（空ボディ）と不一致 → `assert!(true)` を削除し既存パターンに統一
- [LOW-2]: CHANGELOG.md の `cargo clean` 表記が `\`cargo clean\` ★クリーンアップ実施` で既存 x.0.0 パターン（`★ \`cargo clean\`（x.0.0 クリーンアップ）実施`）と不一致 → 修正
