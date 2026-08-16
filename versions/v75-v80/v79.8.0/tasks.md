# v79.8.0 タスクリスト — ドキュメント完全化（v3 リファレンス）

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.7.0` であることを確認
- [x] `cargo test` が全 pass（3801 tests = v79.7.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `site/content/docs/v3/temporal.mdx` がまだ存在しないことを確認（重複追加防止）
- [x] `site/content/docs/v3/migration-v2-v3.mdx` がまだ存在しないことを確認（重複追加防止）

---

## T1: `site/content/docs/v3/temporal.mdx` 作成

- [x] `site/content/docs/v3/` ディレクトリを作成する
- [x] `site/content/docs/v3/temporal.mdx` を新規作成する
  - `FreshnessPolicy` という文字列を含む
  - `AsOfQuery` という文字列を含む
  - `SCD` という文字列を含む（SCD2 の説明）
  - Favnir コードサンプルは `bind` 構文を使用（`let` 不使用）

---

## T2: `site/content/docs/v3/migration-v2-v3.mdx` 作成

- [x] `site/content/docs/v3/migration-v2-v3.mdx` を新規作成する
  - `v2` という文字列を含む
  - `v3` という文字列を含む
  - `Temporal` という文字列を含む
- (スコープ外) `provenance.mdx` / `verifiable.mdx` / `execution-effects.mdx` は後続コミットで追加

---

## T3: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.8.0 エントリを追加する（形式: `## [v79.8.0] — 2026-08-16 — ドキュメント完全化（v3 リファレンス）`）
- [x] Added セクション（temporal.mdx / migration-v2-v3.mdx 追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T4: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.8.0: ドキュメント完全化（v3 リファレンス）---` コメントを追加する
- [x] `v798000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const TEMPORAL` / `const MIGRATION` を配置する（`include_str!` パス: `../../site/content/docs/v3/temporal.mdx` / `../../site/content/docs/v3/migration-v2-v3.mdx`）
- [x] `docs_v3_temporal_exists` テストを実装する
  - `FreshnessPolicy` / `AsOfQuery` / `SCD` を assert
- [x] `docs_v3_migration_guide_exists` テストを実装する
  - `v2` / `v3` / `Temporal` を assert
- [x] `cargo test v798000` で 2 件が pass することを確認する

---

## T5: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.7.0"` → `"79.8.0"` に変更する
- [x] driver.rs 内の escaped `\"79.7.0\"` を `\"79.8.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.7.0` を `79.8.0` に更新する
- [x] **更新後に** `grep -c "79\.7\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.7.0: OSS 公開強化・コミュニティ整備 ---` コメント行の 1 件のみ

---

## T6: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.8.0**（ドキュメント完全化 v3 リファレンス）` に更新する
- [x] `## 次に切る版` 欄を `**v79.9.0**（安定化・コードフリーズ）` に更新する

---

## T7: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3803 tests）
- [x] `cargo test v798000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.8.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.8.0]` であることを確認する
- [x] `site/content/docs/v3/temporal.mdx` に `FreshnessPolicy` / `AsOfQuery` / `SCD` が含まれることを確認する
- [x] `site/content/docs/v3/migration-v2-v3.mdx` に `v2` / `v3` / `Temporal` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `docs_v3_temporal_exists` が pass
- [x] `docs_v3_migration_guide_exists` が pass
- [x] テスト総数: 3803（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.8.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.8.0" に更新済み
- [x] `versions/current.md` が v79.8.0 に更新済み
- [x] `changelog_has_v79_8_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
- [x] `provenance.mdx` / `verifiable.mdx` / `execution-effects.mdx` 追加: 対象外（後続コミットで追加）
