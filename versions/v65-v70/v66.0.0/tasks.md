# v66.0.0 タスクリスト

Status: COMPLETE
Version: 66.0.0
Base tests: 3471
Target tests: 3475
Actual tests: 3475

---

## T0: 事前確認

- [x] `cargo test --bin fav` でベース 3471 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（本バージョンで更新する）
- [x] `driver.rs` に `v65900_tests` が存在することを確認（`v66000_tests` の挿入位置）
- [x] `driver.rs` に `v66000_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65900_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.9.0` であることを確認

---

## T1: `driver.rs` — `v66000_tests` 追加

- [x] `// -- v65900_tests (v65.9.0)` コメントの直前に `v66000_tests` を挿入（4 テスト関数）
  - [x] `cargo_toml_version_is_66_0_0` — `../Cargo.toml` に `version = "66.0.0"` を含む
  - [x] `changelog_has_v66_0_0` — `../../CHANGELOG.md` に `v66.0.0` を含む
  - [x] `milestone_has_math_science` — `../../MILESTONE.md` に `Math & Science` を含む
  - [x] `readme_mentions_math_science` — `../../README.md` に `Math & Science` または `v66.0` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）

---

## T2: `fav/Cargo.toml` — version 更新

- [x] `version = "65.0.0"` → `version = "66.0.0"` に変更

---

## T3: `MILESTONE.md` — v66.0.0 エントリ追加

- [x] 既存 `## v65.0.0` エントリの直前に v66.0.0 エントリを挿入（先頭エントリ）
- [x] 宣言文（「行列の次元は...」）を含む
- [x] `"Math & Science"` 文字列を含む（`milestone_has_math_science` テスト要件）
- [x] v65.1〜v65.9 の達成内容リストを含む
- [x] `**テスト数**: 3475` を含む

---

## T4: `README.md` — v66.0.0 宣言追加

- [x] 既存バージョン履歴の先頭または適切な箇所に v66.0.0 の言及を追加
- [x] `"Math & Science"` または `"v66.0"` を含む（`readme_mentions_math_science` テスト要件）

---

## T5: `CHANGELOG.md` — v66.0.0 エントリ追加

- [x] 既存 `## [v65.0.0]` エントリの直前に v66.0.0 エントリを挿入（先頭エントリ）
- [x] `"v66.0.0"` を含む（`changelog_has_v66_0_0` テスト要件）
- [x] v65.1〜v65.9 の全変更を一括追記（CHANGELOG 方針に従い）
- [x] `Changed` セクションに `Cargo.toml version "65.0.0" → "66.0.0"` を記載

---

## T6: テスト確認（cargo clean 前）

- [x] `cargo build` でエラーなし
- [x] `cargo test --bin fav v66000_tests` で 4 件 PASS
  - [x] `cargo_toml_version_is_66_0_0` PASS
  - [x] `changelog_has_v66_0_0` PASS
  - [x] `milestone_has_math_science` PASS
  - [x] `readme_mentions_math_science` PASS

---

## T7: `cargo clean` ★クリーンアップ

- [x] `cargo clean` を実行（8.9GB 削除）
- [x] `fav/tmp/hello.fav` が存在することを確認（削除なし）

---

## T8: フルテスト確認

- [x] `cargo test -j 8 -- --test-threads=8` で 3475 tests passed, 0 failed を確認

---

## T9: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v66.0.0 行を「完了」に更新
- [x] `versions/current.md` を更新（最新安定版 → v66.0.0、次バージョン → v66.1.0）
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

---

## コードレビュー指摘と対応

- v66.0.0 コードレビュー: 旧 `cargo_toml_version_is_XX` テスト 14 件がスタブのまま古い version 文字列（65.0.0）を参照していた → 全件 `66.0.0` に置換（driver.rs sed replace_all）
