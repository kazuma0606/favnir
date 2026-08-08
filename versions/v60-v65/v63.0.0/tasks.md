# v63.0.0 タスクリスト

Status: COMPLETE
Version: 63.0.0
Base tests: 3402
Target tests: 3406

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3402 tests passed, 0 failed を確認
  （ロードマップ記載 3400 より +2 — v62.8.0 code-reviewer 対応で `aot_no_emit_passes` 追加のため）
- [x] `fav/Cargo.toml` の現行バージョンが `62.0.0` であることを確認
- [x] `MILESTONE.md` に `"v63.0.0"` かつ `"AOT Native"` の組み合わせが **存在しない** ことを確認
- [x] `MILESTONE.md` に `"v63.0.0"` が **存在しない** ことを確認
- [x] `README.md` に `"v63.0.0"` が **存在しない** ことを確認
- [x] `driver.rs` に `v62000_tests` が存在することを確認（挿入位置確認）

---

## T1: `driver.rs` — `v63000_tests` 追加

- [x] `v62000_tests` の閉じ括弧（`}`）の**直後**（`v62900_tests` の直前）に以下を挿入:
  （注意: `include_str!` のみ使用のため `use super::*;` は不要）
  ```rust
  // -- v63000_tests (v63.0.0) -- AOT Native 宣言 --
  #[cfg(test)]
  mod v63000_tests {
      #[test]
      fn cargo_toml_version_is_63_0_0() {
          let cargo = include_str!("../Cargo.toml");
          assert!(
              cargo.contains("version = \"63.0.0\""),
              "Cargo.toml should contain version = \"63.0.0\"; got: {:?}",
              &cargo[..200.min(cargo.len())]
          );
      }

      #[test]
      fn changelog_has_v63_0_0() {
          let cl = include_str!("../../CHANGELOG.md");
          assert!(
              cl.contains("v63.0.0"),
              "CHANGELOG.md should contain v63.0.0 entry"
          );
      }

      #[test]
      fn milestone_has_aot_native() {
          let ms = include_str!("../../MILESTONE.md");
          assert!(
              ms.contains("v63.0.0") && ms.contains("AOT Native"),
              "MILESTONE.md should contain both v63.0.0 and AOT Native"
          );
      }

      #[test]
      fn readme_mentions_aot_native() {
          let readme = include_str!("../../README.md");
          assert!(
              readme.contains("v63.0.0") && readme.contains("AOT Native"),
              "README.md should contain both v63.0.0 and AOT Native"
          );
      }
  }
  ```
- [x] `cargo build` でエラーなし（この時点でテストは FAIL — 想定内）

---

## T2: `fav/Cargo.toml` — バージョン更新

- [x] `version = "62.0.0"` を `version = "63.0.0"` に変更
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — 旧バージョンアサーション一括置換

- [x] `driver.rs` 内の `cargo.contains("version = \"62.0.0\"")` を
  `cargo.contains("version = \"63.0.0\"")` に **一括置換**（12 件実施）
  （テスト関数名 `fn cargo_toml_version_is_62_0_0()` は変更しない）
- [x] `cargo build` でエラーなし

---

## T4: `CHANGELOG.md` — v63.0.0 エントリ追加

- [x] `CHANGELOG.md` 先頭に以下を追加:
  ```markdown
  ## [v63.0.0] — 2026-08-02 — AOT Native 宣言 ★クリーンアップ

  ### Added
  - `MILESTONE.md` に AOT Native 宣言エントリを追加（v62.1〜v62.9 の全 AOT 機能集約）
  - Rust テスト 4 件追加（`v63000_tests`）:
    `cargo_toml_version_is_63_0_0` / `changelog_has_v63_0_0` / `milestone_has_aot_native` / `readme_mentions_aot_native`

  ### Changed
  - `fav/Cargo.toml` バージョンを `62.0.0` → `63.0.0` に更新
  - `driver.rs` 内の `cargo.contains("version = \"62.0.0\"")` アサーション 12 件を `63.0.0` に一括更新

  ### Notes
  - ★クリーンアップ（`cargo clean`）実施済み

  ---
  ```
- [x] `cargo test v63000` → `changelog_has_v63_0_0` PASS

---

## T5: `MILESTONE.md` — AOT Native 宣言エントリ追加

- [x] 既存の最新エントリ（v62.0.0 Language Polish）の直後に以下を追加:
  ```markdown
  ## v63.0.0（2026-08-02）— AOT Native

  > 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
  >  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
  >
  >  Favnir は型安全なコンパイル言語として新たな段階に達した。
  >
  >  これが Favnir v63.0 — AOT Native の姿である。」

  **AOT Native** の宣言バージョン。...

  **テスト数**: 3406
  ```
- [x] `cargo test v63000` → `milestone_has_aot_native` PASS

---

## T6: `README.md` — v63.0.0 AOT Native 言及追加

- [x] v63.0.0 AOT Native 段落を README.md に追加
- [x] `cargo test v63000` → `readme_mentions_aot_native` PASS

---

## T7: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v63000` で 4 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3406 tests passed, 0 failed を確認（実測値）

---

## T8: ★クリーンアップ（cargo clean）

- [x] `cargo clean` を実行（17434 files, 13.0 GiB 削除）
- [x] `fav/tmp/hello.fav` の存在を確認（消えておらず、内容正常）
- [x] `cargo build` でクリーン後ビルド成功確認
- [x] `cargo test -j 8 -- --test-threads=8` で 3406 tests passed, 0 failed を確認（クリーン後）

---

## T9: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v63.0 セクションに実績を追記
- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` テスト数推移表（v63.0.0 行）を `3404` → `3406` に更新
- [x] `versions/roadmap/roadmap-v60.1-v65.0.md` テスト数推移表（v63.0.0 行）を `3396` → `3406` に更新
- [x] `versions/current.md` の「進行中」を v63.0.0（3406 tests）に更新、「次」を次ロードマップに
- [x] `CHANGELOG.md` 確認（T4 で追加済み）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- [MED] アサート失敗時のエラーメッセージ文字列に `"62.0.0"` が 10 箇所残存 → 全件 `"63.0.0"` に修正済み
  （対象: `v61000_tests`〜`v56300_tests` の `cargo_toml_version_is_*` 系テスト）
- [LOW] `v62000_tests::cargo_toml_version_is_62_0_0` の assert 条件が `63.0.0` を検証 → 設計上の意図（関数名は変更しない仕様）のため対応不要

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3406 passed, 0 failed（クリーン後も確認済み）
- `★クリーンアップ`（cargo clean）: 完了（17434 files, 13.0 GiB 削除）
- 主要実装: Cargo.toml バージョン更新 / MILESTONE.md AOT Native 宣言 / README.md 言及追加 / CHANGELOG.md / `v63000_tests` 4 件
- 完了日: 2026-08-02
