# v79.6.0 タスクリスト — ドッグフーディング強化

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.5.0` であることを確認
- [x] `cargo test` が全 pass（3797 tests = v79.5.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `fav/pipelines/` ディレクトリに `release.fav` / `health-check.fav` がまだ存在しないことを確認（重複追加防止）

---

## T1: `fav/pipelines/` ファイル作成

- [x] `fav/pipelines/release.fav` を新規作成する
  - `bump_version` 関数を含む（`String.replace(source, old_ver, new_ver)` 3引数）
  - `prepend_changelog` 関数を含む（`String.concat` 2引数）
  - `release_pipeline` 関数を含む（明示パラメータで ctx.current_version 等を使わない）
  - すべて `bind` 構文を使用（`let` 不使用）
- [x] `fav/pipelines/health-check.fav` を新規作成する
  - `run_tests` 関数を含む（`ctx.io.println` 使用）
  - `run_verify` 関数を含む（`"fav verify"` 文字列をログ出力に含める）
  - `health_check_pipeline` 関数を含む

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.6.0 エントリを追加する（形式: `## [v79.6.0] — 2026-08-16 — ドッグフーディング強化`）
- [x] Added セクション（release.fav / health-check.fav 追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.6.0: ドッグフーディング強化 ---` コメントを追加する
- [x] `v796000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const RELEASE` / `const HEALTH` を配置する（パス: `../pipelines/release.fav` / `../pipelines/health-check.fav`）
- [x] `dogfood_release_pipeline_exists` テストを実装する
  - `release_pipeline` / `bump_version` / `prepend_changelog` を assert
- [x] `dogfood_health_check_pipeline_exists` テストを実装する
  - `health_check_pipeline` / `fav verify` を assert
- [x] `cargo test v796000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.5.0"` → `"79.6.0"` に変更する
- [x] driver.rs 内の escaped `\"79.5.0\"` を `\"79.6.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.5.0` を `79.6.0` に更新する
- [x] **更新後に** `grep -c "79\.5\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.5.0: Execution Effects showcase パイプライン ---` コメント行の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.6.0**（ドッグフーディング強化）` に更新する
- [x] `## 次に切る版` 欄を `**v79.7.0**（OSS 公開強化・コミュニティ整備）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3799 tests）
- [x] `cargo test v796000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.6.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.6.0]` であることを確認する
- [x] `fav/pipelines/release.fav` に `release_pipeline` / `bump_version` / `prepend_changelog` が含まれることを確認する
- [x] `fav/pipelines/health-check.fav` に `health_check_pipeline` / `fav verify` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `dogfood_release_pipeline_exists` が pass
- [x] `dogfood_health_check_pipeline_exists` が pass
- [x] テスト総数: 3799（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.6.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.6.0" に更新済み
- [x] `versions/current.md` が v79.6.0 に更新済み
- [x] `changelog_has_v79_6_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
