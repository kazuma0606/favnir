# v74.3.0 タスクリスト — Documentation Site 2.0

Date: 2026-08-13
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.2.0` であることを確認
- [x] `cargo test` が 3673 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v742000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v743000_tests` が未存在であることを確認

---

## T1: MDX ファイル 3 件を作成

- [x] `site/content/docs/v2/` ディレクトリを作成した
- [x] `site/content/docs/v2/getting-started.mdx` を作成した
  - `"Getting Started"` タイトルを含む
  - `"Favnir"` の言及を含む
  - インストール手順・Hello World パイプライン例を含む
- [x] `site/content/docs/v2/migration-v35-v75.mdx` を作成した
  - `"Migration"` タイトルを含む
  - `"v35"` と `"v75"` の言及を含む
  - 主な変更点（`!Effect` 廃止・`ctx` 構文・`bind` 制限）を含む
- [x] `site/content/docs/v2/language-reference.mdx` を作成した
  - `"Language Reference"` タイトルを含む
  - `bind` / `stage` 等の構文一覧を含む

---

## T2: `v743000_tests` モジュールを `driver.rs` に追加

- [x] `// --- v74.3.0: Documentation Site 2.0 ---` セクションコメントを追加した
- [x] `v742000_tests` の直後に `v743000_tests` モジュールを追加した
- [x] `docs_site2_getting_started_exists` テストを実装した
  - `include_str!("../../site/content/docs/v2/getting-started.mdx")` を読み込む
  - `"Getting Started"` / `"Favnir"` を含むことを assert
- [x] `docs_site2_migration_guide_v35_to_v75` テストを実装した
  - `include_str!("../../site/content/docs/v2/migration-v35-v75.mdx")` を読み込む
  - `"Migration"` / `"v35"` / `"v75"` を含むことを assert
- [x] `cargo build` でエラーがないことを確認（`include_str!` はコンパイル時に解決される）

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.2.0"` → `version = "74.3.0"` に変更した
- [x] `driver.rs` 内の `version = "74.2.0"` 参照を `version = "74.3.0"` に replace_all した
- [x] `version should be 74.2.0` を `version should be 74.3.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.2.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.3.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v743000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3676 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.3.0]` エントリを先頭に追加した
  - Added: `site/content/docs/v2/` 以下 3 MDX ファイル（getting-started / migration-v35-v75 / language-reference）
  - Tests: 2 件、合計テスト数 3676（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v74.3.0)` に更新した
- [x] 「進行中バージョン」を `v74.3.0` に更新した
- [x] 「次に切る版」を `v74.4.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v743000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3676 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.3.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.3.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.3.0` であることを確認

---

## スコープ外（明示的除外）

- Cookbook / Rune Catalog / API Reference / Video Transcripts ページ（後続バージョンで対応）
- サイトのビルド・デプロイ（`scripts/deploy-site.sh` は別途実施）
- MILESTONE.md 更新（宣言バージョンではないため不要）
