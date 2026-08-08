# Tasks — v58.0.0 — Enterprise Security 宣言 ★クリーンアップ

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v57.1-v58.0.md` の v58.0.0 セクションを確認
- [x] `versions/roadmap/roadmap-v55.1-v60.0.md` の v58.0.0 欄が存在することを確認（T14 の更新対象）
- [x] ベーステスト数 3272（v57.9.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `57.9.0` であることを確認（更新前）
- [x] `v58000_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v57900_tests` が `driver.rs` に存在することを確認（`v58000_tests` の挿入位置として使用）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"57.9.0"` を期待していることを確認（更新対象）
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` が `"57.9.0"` を期待していることを確認（更新対象）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` が `"57.9.0"` を期待していることを確認（更新対象・rolling）
- [x] `v57900_tests::cargo_toml_version_is_57_9_0` が `"57.9.0"` を期待していることを確認（更新対象・rolling）
- [x] `v57100_tests` 〜 `v57800_tests` に `cargo_toml_version_is_*` が存在しないことを確認（rolling 更新対象外 — v57.1〜v57.8 は宣言バージョンではないため rolling チェックが存在しない）
- [x] `MILESTONE.md` に `v58.0.0` エントリが存在しないことを確認（新規追加対象）
- [x] `README.md` の現状確認（v58.0.0 / Enterprise Security の記述有無）

---

## 実装タスク（順序厳守）

- [x] T1: `fav/Cargo.toml` version を `58.0.0` に更新
- [x] T2: `MILESTONE.md` — Enterprise Security エントリを先頭に追加
  - [x] `"Enterprise Security"` キーワードを含む（テスト検証対象）
  - [x] 宣言文（v57.1〜v57.9 達成内容）を記載
- [x] T3: `README.md` — v58.0.0 / Enterprise Security 達成を追記
  - [x] `"Enterprise Security"` キーワードを含む（テスト検証対象、すでに存在するため最小限の更新でも可）
- [x] T4: `CHANGELOG.md` — v58.0.0 エントリを追加（**T5 より前に必須**）
  - [x] `"v58.0.0"` キーワードを含む（テスト検証対象）
- [x] T5: `fav/src/driver.rs` — `v58000_tests` モジュールを `v57900_tests` の直前に追加
  - [x] `cargo_toml_version_is_58_0_0`: rolling 形式でバージョンを検証
  - [x] `changelog_has_v58_0_0`: CHANGELOG.md に `v58.0.0` が含まれることを検証
  - [x] `milestone_has_enterprise_security`: MILESTONE.md に `Enterprise Security` が含まれることを検証
  - [x] `readme_mentions_enterprise_security`: README.md に `Enterprise Security` が含まれることを検証
- [x] T6: `fav/src/driver.rs` — バージョンチェックテスト更新（rolling 4 件）
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` を `"57.9.0"` → `"58.0.0"` に更新（メッセージも）
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` を `"57.9.0"` → `"58.0.0"` に更新
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` を `"57.9.0"` → `"58.0.0"` に更新
  - [x] `v57900_tests::cargo_toml_version_is_57_9_0` を `"57.9.0"` → `"58.0.0"` に更新
  - [x] モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3276 tests passed, 0 failed**）
  - [x] `v58000_tests::cargo_toml_version_is_58_0_0` ok
  - [x] `v58000_tests::changelog_has_v58_0_0` ok
  - [x] `v58000_tests::milestone_has_enterprise_security` ok
  - [x] `v58000_tests::readme_mentions_enterprise_security` ok
  - [x] 既存 3272 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ★クリーンアップ

- [x] T10: `fav/tmp/hello.fav` が存在することを確認（`cargo clean` 後も残る）
- [x] T11: `cargo clean` 実行（`target/` ディレクトリを削除）— 31.6 GiB 削除

---

## ポスト処理

- [x] T12: `versions/current.md` を v58.0.0 / 3276 tests に更新
- [x] T13: `versions/roadmap/roadmap-v57.1-v58.0.md` の v58.0.0 実績を COMPLETE に更新
  - [x] `3272 + 4 = 3276 tests passed, 0 failed（2026-07-28）` を追記
- [x] T14: `versions/roadmap/roadmap-v55.1-v60.0.md` の v58.0.0 実績欄も COMPLETE に更新
  - [x] テスト数推移テーブルの v58.0.0 行: `~3276` → `3276`（実績値）に書き換え、実績コメント `実績値（2026-07-28 COMPLETE）` を追記
- [x] T15: `versions/v55-v60/v58.0.0/tasks.md` を COMPLETE に更新

---

## 完了確認

- [x] `cargo_toml_version_is_58_0_0` pass
- [x] `changelog_has_v58_0_0` pass
- [x] `milestone_has_enterprise_security` pass
- [x] `readme_mentions_enterprise_security` pass
- [x] **3276 tests passed, 0 failed**（ベース 3272 + 4）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `MILESTONE.md` に `Enterprise Security` 宣言文エントリが追加されている
- [x] `CHANGELOG.md` に `## [v58.0.0]` エントリが追加されている
- [x] rolling 更新 4 件（v56300 / v56900 / v57000 / v57900）が `"58.0.0"` になっている
- [x] `versions/current.md` が v58.0.0 / 3276 tests を反映
- [x] `cargo clean` 完了（★クリーンアップ）
- [x] T13 / T14 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- **実装順序**: T1 → T2 → T3 → T4（CHANGELOG） → T5（v58000_tests） → T6（rolling） → T7〜T9（検証） → T10〜T11（cargo clean）
- `changelog_has_v58_0_0` は `include_str!(\"../../CHANGELOG.md\")` でコンパイル時評価 → T4（CHANGELOG 更新）が T5 より先であること
- rolling 更新対象は **4 件**（v56300 / v56900 / v57000 / v57900）— v57100〜v57800 は対象外
- `cargo clean` は `fav/tmp/hello.fav` を削除しない（`target/` のみ）— 実施前に hello.fav 存在確認済み
- `v57100_tests` 〜 `v57800_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象外
