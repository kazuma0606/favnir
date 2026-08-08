# Tasks — v57.0.0 — Language Power 2.0 宣言 ★クリーンアップ

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v57.0.0 セクションを確認
- [x] ベーステスト数 3248（v56.9.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.9.0` であることを確認（更新前）
- [x] `MILESTONE.md` が `"Language Power 2.0"` を含まないことを確認（T0: `"Language Power"` はあるが `"Language Power 2.0"` はないこと）
- [x] `README.md` が `"Language Power 2.0"` を含まないことを確認
- [x] `CHANGELOG.md` が `"[v57.0.0]"` を含まないことを確認
- [x] `v56900_tests` が `driver.rs` に存在することを確認（`v57000_tests` の挿入位置として使用）
- [x] `v57000_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.9.0"` を期待していることを確認（更新対象）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `57.0.0` に更新
- [x] T2: `MILESTONE.md` に Language Power 2.0 宣言エントリを追加
  - [x] `## v57.0.0（2026-07-26）— Language Power 2.0` ヘッダー
  - [x] 宣言文（引用ブロック）を含める
  - [x] `**Language Power 2.0**` の宣言バージョン説明
  - [x] `**v56.1〜v56.9 達成内容:**` リスト（9件）
  - [x] `## v56.0.0` エントリの**前**（ファイル先頭寄り）に挿入
- [x] T3: `README.md` に Language Power 2.0 宣言の追記
  - [x] v56.0 宣言エントリの直後に挿入
  - [x] `**v57.0（2026-07-26）で、[Language Power 2.0](./MILESTONE.md) マイルストーンを宣言しました。**` 形式
  - [x] 主要機能の説明文を含める
- [x] T4: `CHANGELOG.md` に v57.0.0 エントリを追加
  - [x] `## [v57.0.0] — 2026-07-26 — Language Power 2.0 宣言` ヘッダー
  - [x] `[v57.0.0]` という文字列を含める（`changelog_has_v57_0_0` テスト対象）
- [x] T5: `fav/src/driver.rs` — `v57000_tests` モジュールを `v56900_tests` の直前に追加
  - [x] `cargo_toml_version_is_57_0_0`: `Cargo.toml` version が `"57.0.0"` である
  - [x] `changelog_has_v57_0_0`: `CHANGELOG.md` が `"[v57.0.0]"` を含む
  - [x] `milestone_has_language_power2`: `MILESTONE.md` が `"Language Power 2.0"` を含む
  - [x] `readme_mentions_language_power2`: `README.md` が `"Language Power 2.0"` を含む
- [x] T6: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.9.0"` → `"57.0.0"` に更新
  - [x] failure メッセージも `"should be 57.0.0"` に更新
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値も `"56.9.0"` → `"57.0.0"` に更新（rolling パターン）
  - [x] モジュール名 `v56300_tests` / 関数名は変更しない（慣例）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3252 tests passed, 0 failed**）
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` ok
  - [x] `v57000_tests::changelog_has_v57_0_0` ok
  - [x] `v57000_tests::milestone_has_language_power2` ok
  - [x] `v57000_tests::readme_mentions_language_power2` ok
  - [x] 既存 3248 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `versions/current.md` を v57.0.0 / 3252 tests に更新
- [x] T11: `versions/roadmap/roadmap-v56.1-v57.0.md` の v57.0.0 実績を COMPLETE に更新
  - [x] `3248 + 4 = 3252 tests passed, 0 failed（2026-07-26）` を追記
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.0.0 実績欄も COMPLETE に更新（テスト数推移テーブルの `~3250` → `3252` に精緻化）

---

## ★クリーンアップ

- [x] T13: `cargo clean` を実行（全ポスト処理完了・全テスト通過確認後）— 38.8 GiB 削除

---

## 完了確認

- [x] `cargo_toml_version_is_57_0_0` pass
- [x] `changelog_has_v57_0_0` pass
- [x] `milestone_has_language_power2` pass
- [x] `readme_mentions_language_power2` pass
- [x] **3252 tests passed, 0 failed**（かつ ≥ 3250）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `MILESTONE.md` に `"Language Power 2.0"` 宣言文エントリが追加されている
- [x] `README.md` に `"Language Power 2.0"` の言及が追加されている
- [x] `CHANGELOG.md` に `[v57.0.0]` エントリが追加されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.0.0"` になっている
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.0.0"` になっている（rolling 更新）
- [x] `versions/current.md` が v57.0.0 / 3252 tests を反映
- [x] T11 / T12 のロードマップ更新（実績 COMPLETE）が完了している
- [x] `cargo clean` 完了（★クリーンアップ）

---

## 実装メモ

- `MILESTONE.md` への挿入位置: `# Favnir Milestones` の直後、`## v56.0.0` の前
- `README.md` への挿入: v56.0 宣言行（`**v56.0（2026-07-24）で、` を検索）の直後
- `include_str!` パス（driver.rs から）:
  - `"../../CHANGELOG.md"` — `fav/src/` → `fav/` → プロジェクトルート
  - `"../../MILESTONE.md"` — 同上
  - `"../../README.md"` — 同上
- `v57000_tests` は `use super::*` 不要（`include_str!` のみ使用）
- T4（CHANGELOG 更新）は T5（driver.rs テスト追加）より先に行う
  （`changelog_has_v57_0_0` テストが先にビルドされると `[v57.0.0]` 不在でパニックする）
- **`v56900_tests::cargo_toml_version_is_56_9_0` も rolling バージョンチェックのため更新が必要**
  （spec T6 に記載なかったが実装時に判明 — 次スプリントでは v57000_tests も同様に更新する）
- `cargo clean` は T13 として最後に実行 — 次スプリントはクリーンな状態で開始
