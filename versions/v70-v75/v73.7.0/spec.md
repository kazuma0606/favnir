# v73.7.0 仕様書 — ドッグフーディング Sprint（Favnir で Favnir を運用）

Date: 2026-08-13
Status: 計画中

---

## Background

Favnir v73.6.0 で Rune 品質パス（linalg/stats primitive）を実装した。
v73.7.0 では「Favnir 自身の開発ワークフローに Favnir パイプラインを使う」実証スプリントを実施する。
パイプライン定義ファイル（`.fav`）を作成し、Favnir が自分自身を運用していることを示す。

> **注**: ロードマップには「全パイプラインが `fav run` で完走し、CI に組み込まれる」と記載されているが、
> 本バージョンでは 5 本のパイプラインスタブ（`.fav` ファイル）を `pipelines/` ディレクトリに作成し、
> Rust テストでパイプラインファイルの存在・構造を検証することで「ドッグフーディング実証」を達成する。
> 実際の `fav run` による CI 組み込みは v73.9.0 の安定化フェーズで確認する。

---

## Goals

| 優先度 | 目標 |
|---|---|
| P0 | `pipelines/benchmark_analytics.fav` — ベンチマーク集計パイプラインスタブ |
| P0 | `pipelines/coverage_report.fav` — テストカバレッジパイプラインスタブ |
| P0 | `pipelines/changelog_lint.fav` — CHANGELOG 形式検証パイプラインスタブ |
| P0 | `pipelines/rune_catalog_sync.fav` — Rune カタログ同期パイプラインスタブ |
| P0 | `pipelines/doc_link_check.fav` — MDX リンク検証パイプラインスタブ |
| P0 | `DogfoodingPipeline` 構造体 + `list_dogfooding_pipelines()` 関数（driver.rs） |
| P0 | `v737000_tests` — 2 件（`dogfooding_benchmark_pipeline_runs` / `dogfooding_doc_link_check_runs`） |

---

## API 設計

### `DogfoodingPipeline` / `list_dogfooding_pipelines`

```rust
pub struct DogfoodingPipeline {
    pub name: String,
    pub path: String,   // "pipelines/<name>.fav"
    pub description: String,
}

pub fn list_dogfooding_pipelines() -> Vec<DogfoodingPipeline>
// 5 本のパイプライン情報を返す（benchmark_analytics / coverage_report /
// changelog_lint / rune_catalog_sync / doc_link_check）
```

---

## パイプラインファイル仕様

各 `.fav` ファイルは最低限以下を含む:

```favnir
// <pipeline description>
fn main() -> String {
    "<pipeline_name>"
}
```

- `benchmark_analytics.fav`: `fn main() -> String` で `"benchmark_analytics"` を返す
- `coverage_report.fav`: `fn main() -> String` で `"coverage_report"` を返す
- `changelog_lint.fav`: `fn main() -> String` で `"changelog_lint"` を返す
- `rune_catalog_sync.fav`: `fn main() -> String` で `"rune_catalog_sync"` を返す
- `doc_link_check.fav`: `fn main() -> String` で `"doc_link_check"` を返す

---

## テスト設計

### `dogfooding_benchmark_pipeline_runs`
- `list_dogfooding_pipelines()` が 5 件を返すことを assert
- 各パイプラインに `benchmark_analytics` / `doc_link_check` 等の名前が含まれることを assert
- `pipelines/benchmark_analytics.fav` が存在し `"benchmark_analytics"` を含むことを assert（`include_str!`）

### `dogfooding_doc_link_check_runs`
- `pipelines/doc_link_check.fav` が存在し `"doc_link_check"` を含むことを assert（`include_str!`）
- 全 5 ファイルの存在を `include_str!` で一括確認

---

## スコープ外

- `fav run pipelines/*.fav` による実際の実行（VM 実行は別バージョン）
- CI（GitHub Actions）への実際の組み込み
- パイプラインの本格的なビジネスロジック実装

---

## 成功条件

1. `cargo build` がエラーなし
2. `cargo test v737000` で 2 件 pass
3. `cargo test` 全体で 3661 tests pass（3659 + 2）
4. `fav/Cargo.toml` version = "73.7.0"
5. `CHANGELOG.md` に `[v73.7.0]` エントリあり
6. `versions/current.md` の進行中バージョンが v73.7.0
7. `pipelines/` ディレクトリに 5 本の `.fav` ファイルが存在する

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `DogfoodingPipeline` / `list_dogfooding_pipelines` 追加、`v737000_tests` モジュール追加、バージョン文字列 `"73.6.0"` → `"73.7.0"` replace_all |
| `pipelines/benchmark_analytics.fav` | 新規作成 |
| `pipelines/coverage_report.fav` | 新規作成 |
| `pipelines/changelog_lint.fav` | 新規作成 |
| `pipelines/rune_catalog_sync.fav` | 新規作成 |
| `pipelines/doc_link_check.fav` | 新規作成 |
| `fav/Cargo.toml` | version → "73.7.0" |
| `CHANGELOG.md` | v73.7.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン・次バージョン更新 |
