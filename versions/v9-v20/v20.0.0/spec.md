# v20.0.0 — Production Performance マイルストーン宣言 仕様書

## 概要

v19.x シリーズ（v19.1.0〜v19.8.0）の集大成として、Favnir が「本番で速い言語」であることを正式宣言する。
新機能の実装はなく、ドキュメント整備・ベンチマーク追加・バージョン宣言が主な作業。

---

## 実装済み機能（v19.x で完了）

| バージョン | 機能 |
|---|---|
| v19.1.0 | `#[streaming]` 遅延評価パイプライン（定常メモリ・大規模 CSV 対応）|
| v19.2.0 | AOT コンパイル（Cranelift バックエンド、`fav build --target native`）|
| v19.3.0 | インクリメンタルコンパイル（SHA-256 フィンガープリント + `.fav_cache/`）|
| v19.4.0 | 並列コンパイル（Rayon + petgraph トポロジカルソート）|
| v19.5.0 | Apache Arrow 統合（`VMValue::ArrowBatch`、`write_parquet`/`read_parquet`）|
| v19.6.0 | WASM 最適化（デッドコード除去、サイズ削減）|
| v19.7.0 | 事前コンパイル（`fav compile` / `fav run --precompiled`、Lambda 対応）|
| v19.8.0 | プロファイリング強化（inferno フレームグラフ SVG、`--format=flamegraph/text/json`）|

---

## v20.0.0 実装内容

### 1. バージョン更新
- `fav/Cargo.toml`: `version = "19.8.0"` → `"20.0.0"`

### 2. CHANGELOG.md 更新
v19.1.0〜v19.8.0 の全エントリを追加（形式: `## [X.Y.Z] - YYYY-MM-DD`）

### 3. README.md 更新
- 「現在のバージョン」を v20.0.0 に更新
- Production Performance マイルストーン達成を記載
  - streaming / AOT / Arrow / precompiled の実績
  - ベンチマーク結果（10GB CSV 処理 / Lambda コールドスタート）
- バージョン履歴表に v19.1.0〜v20.0.0 エントリ追加

### 4. site/content/docs/performance/ ドキュメント（6 ファイル）
- `streaming.mdx`: `#[streaming]` アノテーション使い方ガイド
- `native-build.mdx`: `fav build --target native` ガイド
- `incremental.mdx`: インクリメンタルコンパイルガイド
- `arrow.mdx`: Apache Arrow 統合ガイド
- `precompiled.mdx`: `.favc` 事前コンパイルガイド（Lambda 向け）
- `profiling.mdx`: フレームグラフ＋テキスト/JSON レポートガイド

> Note: `tools/precompiled.mdx` と `tools/profiling.mdx` は v19.7/v19.8 で作成済み。
> `performance/` サブディレクトリに performance-focused な詳細版を新規作成する。

### 5. benchmarks/ ディレクトリ（新規）
- `10gb_csv.fav`: 10GB CSV 処理ベンチマーク Favnir スクリプト
- `lambda_coldstart.sh`: Lambda コールドスタート計測シェルスクリプト
- `results.md`: ベンチマーク結果の記録

### 6. テスト（v200000_tests、5件）
- `version_is_20_0_0`: Cargo.toml に "20.0.0" が含まれる
- `changelog_has_v19_entries`: CHANGELOG.md に "v19." エントリが含まれる
- `readme_mentions_streaming`: README.md に "streaming" が含まれる
- `readme_mentions_native_build`: README.md に "native" が含まれる
- `benchmarks_dir_exists`: `benchmarks/` ディレクトリが存在する

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| Cargo.toml が `20.0.0` になっている | [ ] |
| CHANGELOG.md に v19.1.0〜v19.8.0 エントリがある | [ ] |
| README.md に Production Performance マイルストーン記述がある | [ ] |
| `site/content/docs/performance/` に 6 ファイルが存在する | [ ] |
| `benchmarks/` ディレクトリに 3 ファイルが存在する | [ ] |
| `cargo test v200000` — 5/5 PASS | [ ] |
| `cargo test` — リグレッションなし | [ ] |
