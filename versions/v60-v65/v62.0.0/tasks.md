# v62.0.0 タスクリスト

Status: COMPLETE
Version: 62.0.0
Base tests: 3378
Target tests: 3382

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3378 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の現行バージョンが `61.0.0` であることを確認
- [x] `MILESTONE.md` に既存の "Language Polish" 言及（v32.0.0 エントリ）があることを確認
- [x] `README.md` に既存の "Language Polish" 言及があることを確認（`readme_mentions_language_polish` テスト事前通過の可能性確認）
- [x] `v61900_tests` が `driver.rs` に存在することを grep で確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version = "61.0.0"` を `version = "62.0.0"` に変更
- [x] `cargo build` でエラーなし

---

## T2: MILESTONE.md — Language Polish 宣言エントリ追加

- [x] 既存の最新エントリの直後に `## v62.0.0 — Language Polish（2026-08-01）` セクションを追加
- [x] 宣言文（spec.md から転記）を追加
- [x] v61.1〜v61.9 の機能一覧（OR パターン・as-pattern・個別ガード・record update・f-string 強化・型エラー差分表示・`_` 型プレースホルダー・`--strict` モード・安定化）を箇条書きで追加
- [x] テスト数 3382 を記載

---

## T3: README.md — v62.0 Language Polish 言及追加

- [x] バージョン履歴テーブルまたは概要セクションに v62.0 Language Polish を追記
  - 既存 "Language Polish"（v32.0）言及との重複は許容（世代が異なる）

---

## T4: CHANGELOG.md — v62.0.0 エントリ追加

- [x] `## [v62.0.0] — 2026-08-01 — Language Polish 宣言 ★クリーンアップ` エントリを先頭に追加
- [x] `### Added` 以下に v61.1〜v61.9 の主要機能を集約して記載

---

## T5: driver.rs — `v62000_tests` 追加

- [x] `v61900_tests` モジュールの**直後（ファイル末尾）**に `v62000_tests` モジュールを追加
- [x] `cargo_toml_version_is_62_0_0` テスト追加
  - `include_str!("../../Cargo.toml")` に `"version = \"62.0.0\""` が含まれることを `assert!`
- [x] `changelog_has_v62_0_0` テスト追加
  - `include_str!("../../../CHANGELOG.md")` に `"v62.0.0"` が含まれることを `assert!`
- [x] `milestone_has_language_polish` テスト追加
  - `include_str!("../../../MILESTONE.md")` に `"v62.0.0"` かつ `"Language Polish"` が含まれることを `assert!`
  - **注意**: v32.0.0 の既存記述があるため、単独 `"Language Polish"` チェックでは T2 追記前から通過してしまう。AND 条件必須。
- [x] `readme_mentions_language_polish` テスト追加
  - `include_str!("../../../README.md")` に `"v62.0"` かつ `"Language Polish"` が含まれることを `assert!`
  - **注意**: v32.0 の既存記述があるため、単独 `"Language Polish"` チェックでは T3 追記前から通過してしまう。AND 条件必須。
- [x] `cargo test v62000` で 4 件 PASS

---

## T6: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62000` で 4 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3382 tests passed, 0 failed を確認

---

## T7: ★クリーンアップ（cargo clean）

- [x] `cargo clean` を実行
- [x] `fav/tmp/hello.fav` の存在を確認（消えた場合は復元: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
- [x] `cargo build` でクリーン後ビルド成功確認
- [x] `cargo test -j 8 -- --test-threads=8` で 3382 tests passed, 0 failed を確認（クリーン後）

---

## T8: ドキュメント更新

- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` v62.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v62.0.0（3382 tests）に更新、「次」を次ロードマップに
- [x] `CHANGELOG.md` 確認（T4 で追加済みのため重複チェックのみ）
- [x] `site/` MDX — Language Polish v62.0 固有のサイトページは不要と判断（v61.1〜v61.9 の個別機能 MDX は各バージョンで対応済み）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

（実装後に記録）

---

## コードレビュー指摘対応

- **[spec][HIGH] `milestone_has_language_polish` / `readme_mentions_language_polish` 事前通過問題** — テスト条件を `"v62.0.0"` / `"v62.0"` との AND 条件に強化（spec/plan/tasks 更新済み）
- **[spec][MED] 挿入位置表記の修正** — 「直前」→「直後（ファイル末尾）」に修正
- **[impl] 旧バージョン `cargo_toml_version_is_X` 10 件 FAIL** — `"61.0.0"` → `"62.0.0"` に一括置換（全バージョン宣言時の標準手順）
- **[impl] `include_str!` パス誤り** — `../../Cargo.toml` → `../Cargo.toml`、`../../../CHANGELOG.md` → `../../CHANGELOG.md` 等に修正
- **[code-review][MED] エラーメッセージが旧バージョン `61.0.0` のまま** — `"Cargo.toml version should be 61.0.0"` 等 10 件を `62.0.0` に修正
- **[code-review][LOW] `v62000_tests` の `use super::*;` が不要** — 削除（`include_str!` 専用モジュール）
- **[code-review][LOW] `readme_mentions_language_polish` の `"v62.0"` が非対称** — `"v62.0.0"` に統一

## 完了サマリー

- Status: COMPLETE
- Tests: 3382 passed, 0 failed（ベース 3378 + 4）
- `★クリーンアップ`（cargo clean）: 完了（クリーン後 3382 PASS 確認）
- 完了日: 2026-08-01
