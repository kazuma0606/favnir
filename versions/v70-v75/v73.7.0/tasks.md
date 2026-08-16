# v73.7.0 タスクリスト — ドッグフーディング Sprint

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.6.0` であることを確認
- [x] `cargo test` が 3659 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v736000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v737000_tests` が未存在であることを確認
- [x] `pipelines/` ディレクトリが未存在であることを確認

---

## T1: `pipelines/` ディレクトリに 5 本の `.fav` ファイルを作成

- [x] `fav/pipelines/benchmark_analytics.fav` を作成した（`"benchmark_analytics"` を含む）
- [x] `fav/pipelines/coverage_report.fav` を作成した（`"coverage_report"` を含む）
- [x] `fav/pipelines/changelog_lint.fav` を作成した（`"changelog_lint"` を含む）
- [x] `fav/pipelines/rune_catalog_sync.fav` を作成した（`"rune_catalog_sync"` を含む）
- [x] `fav/pipelines/doc_link_check.fav` を作成した（`"doc_link_check"` を含む）
- [x] 各ファイルがコメント行 + `fn main() -> String` 形式であることを確認

---

## T2: `DogfoodingPipeline` 構造体 + `list_dogfooding_pipelines` 追加

- [x] `DogfoodingPipeline { name: String, path: String, description: String }` を `driver.rs` に追加した
- [x] `pub struct` であることを確認
- [x] `pub fn list_dogfooding_pipelines() -> Vec<DogfoodingPipeline>` を実装した（5 件を返す）
  - `benchmark_analytics` / `coverage_report` / `changelog_lint` / `rune_catalog_sync` / `doc_link_check`
  - 各エントリの `path = "pipelines/<name>.fav"` であることを確認
  - 各エントリの `description` が空でないことを確認
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v737000_tests` モジュール追加

- [x] `v736000_tests` の直後に `v737000_tests` モジュールを追加した
- [x] `use super::list_dogfooding_pipelines` を追加した
- [x] `dogfooding_benchmark_pipeline_runs` テストを実装した
  - `list_dogfooding_pipelines()` が 5 件を返すことを assert
  - `"benchmark_analytics"` / `"doc_link_check"` が names に含まれることを assert
  - `include_str!("../pipelines/benchmark_analytics.fav")` が `"benchmark_analytics"` を含むことを assert
  - `bench.path == "pipelines/benchmark_analytics.fav"` を assert
  - `bench.description` が空でないことを assert
- [x] `dogfooding_doc_link_check_runs` テストを実装した
  - `include_str!("../pipelines/doc_link_check.fav")` が `"doc_link_check"` を含むことを assert
  - 全 5 ファイルを `include_str!` で読み込み存在確認
  - 全 `path` が `"pipelines/"` で始まり `".fav"` で終わることを assert
- [x] `cargo test v737000` で 2 件 pass することを確認

---

## T4: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.6.0"` → `version = "73.7.0"` に変更した
- [x] `driver.rs` 内の `"73.6.0"` を `"73.7.0"` に replace_all した（バージョン検証テスト文字列を含む）
- [x] 残存 `73.6.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "73.7.0"` を含むことを確認

---

## T5: 部分テスト確認

- [x] T4 のバージョン更新後も `cargo test v737000` で引き続き 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3661 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v73.7.0]` エントリを先頭に追加した
  - Added: 5 本のドッグフーディングパイプライン / `DogfoodingPipeline` / `list_dogfooding_pipelines`
  - Tests: 2 件、合計テスト数 3661（+2）

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.7.0)` に更新した
- [x] 「進行中バージョン」を `v73.7.0` に更新した
- [x] 「次に切る版」を `v73.8.0` に更新した

---

## T9: 最終確認（T7・T8 完了後）

- [x] `cargo test v737000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3661 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.7.0` であることを確認
- [x] `pipelines/` に 5 本の `.fav` ファイルが存在することを確認
- [x] `DogfoodingPipeline` / `list_dogfooding_pipelines` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.7.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.7.0` であることを確認

---

## スコープ外（明示的除外）

- `fav run pipelines/*.fav` による実際の VM 実行
- GitHub Actions への CI 組み込み
- パイプラインの本格的なビジネスロジック実装
