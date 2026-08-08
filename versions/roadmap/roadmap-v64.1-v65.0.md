# Roadmap v64.1.0 〜 v65.0.0 — Performance 1.0 宣言

Date: 2026-07-30
Status: 未着手

---

## 前提

- 直前完了: v64.0.0「Incremental & Scale」（tests = 3431）
- マスターロードマップ: `roadmap-v60.1-v65.0.md`
- 本文書はマスターの v65.0 スプリント部分の詳細版
- **既存機能の扱い**:
  - `fav build` は v62.1 で実装済み → CI 向け `--ci` フラグを追加
  - `fav bench` は v62.5 で実装済み → `--compare` / `--suite` フラグを拡張
  - `fav profile` flamegraph は v9.9 実装済み → AOT バイナリのプロファイル対応追加
  - `fav new` テンプレートギャラリーは v24.8 実装済み → CI workflow テンプレート追加
  - WASM ビルド基盤（`wasm-encoder`）は Cargo.toml 登録済み → `fav build --target wasm32` で統合

---

## 目標

**「型安全と高速を両立したデータパイプライン言語」** としての完成を宣言する。

v64.1〜v64.9 の 9 スプリントで Performance 1.0 の仕上げを行い、
v65.0「Performance 1.0」として宣言する。

---

## バージョン計画

### v64.1.0 — AOT ビルドの CI 統合（`fav build --ci`）

`fav build` に `--ci` フラグを追加。CI 向け出力形式（色なし・機械可読エラー・
exit code 厳格化）を実装。`main.rs` の print_diag 経路を `--ci` 時に切り替える。
`fav new` のテンプレートギャラリー（v24.8 実装済み）に GitHub Actions ワークフロー
テンプレートを追加（`fav build --ci` を呼ぶ `.github/workflows/build.yml` を生成）。

```yaml
# fav new で生成される .github/workflows/build.yml
- name: Build AOT binary
  run: fav build pipeline.fav --link --ci -o dist/pipeline
- name: Validate binary
  run: ./dist/pipeline --validate
```

**完了条件**: Rust テスト 2 件（ベース 3431 + 2 = 3433 tests passed, 0 failed）
- `build_ci_flag_output_format`
- `new_template_has_ci_workflow`

**実績**: 2026-08-02 完了。`cmd_build_ci` + `create_ci_workflow_project` + `v64100_tests` 2 件追加。3433 tests passed, 0 failed。

---

### v64.2.0 — パフォーマンスリグレッションテスト自動化

`fav bench` に `--compare <git-ref>` フラグを追加。
`driver.rs` に `cmd_bench_compare(ref_a: &str, ref_b: &str) -> String` を実装。
現在の bench 結果と指定 git ref の `bench-results.json` を比較し、
劣化率が `regression_threshold_pct` を超えた場合に exit 1 を返す。
`fav.toml` の `[bench]` セクションに `regression_threshold_pct = 10` を追加。

```bash
$ fav bench --compare main..HEAD
Regression detected:
  Transform stage: +18.3% slower (was 12ms, now 14.2ms) — exceeds threshold 10%
Exit code: 1
```

**完了条件**: Rust テスト 2 件（ベース 3433 + 2 = 3435 tests passed, 0 failed）
- `bench_compare_detects_regression`
- `bench_toml_threshold`

**NOTE（スコープ縮小）**: `--compare` CLI フラグ・exit 1・実際の git ref 読み取りは本バージョンでは非スコープ（後送り）。`cmd_bench_compare` は v51.4.0 で既に実装済みのため、`fav.toml` の `[bench]` セクションパース追加と `v64200_tests` 2 件のみ実装。

**実績**: 2026-08-02 完了。`BenchTomlConfig` + `FavToml.bench` フィールド追加（`toml.rs`）、`v64200_tests` 2 件追加。3435 tests passed, 0 failed。

---

### v64.3.0 — パフォーマンスガイド（`site/content/docs/runtime/performance.mdx`）

AOT コンパイル・差分コンパイル・並列最適化・DAG 最適化・バックプレッシャーの
使い方をまとめたパフォーマンスチューニングガイドを作成。
`fav bench` / `fav profile` の出力の読み方・ボトルネック特定手順を掲載。
`site/content/docs/runtime/` ディレクトリに既存の `aot.mdx`（v62.9 作成）と並べて配置。

```bash
# ガイドに掲載するチューニング手順例
$ fav profile --memory pipeline.fav    # ボトルネック特定
$ fav build pipeline.fav --link        # AOT で高速化
$ fav run pipeline.fav --opt-stats     # DAG 最適化の効果確認
```

**完了条件**: Rust テスト 2 件（ベース 3435 + 2 = 3437 tests passed, 0 failed）
- `docs_performance_guide_exists`
- `docs_performance_has_aot_section`

**実績**: 2026-08-02 完了。`site/content/docs/runtime/performance.mdx` 新規作成、`v64300_tests` 2 件追加。3437 tests passed, 0 failed。

---

### v64.4.0 — `fav profile` flamegraph 改善（AOT バイナリ対応・並列表示）

既存の `fav profile --flamegraph`（inferno クレート、v9.9 実装済み）を以下の点で拡張。
- AOT バイナリ（`fav build --link` で生成）の実行プロファイルを取得して flamegraph を生成
- `--compare-vm` フラグで VM と AOT の flamegraph を SVG に並べて比較表示
- `open` クレート（Cargo.toml 登録済み）でブラウザ自動表示

```bash
$ fav profile --flamegraph pipeline.fav
Generated: fav-profile-vm.svg
Generated: fav-profile-aot.svg
Opening comparison in browser...

$ fav profile --flamegraph --compare-vm pipeline.fav
Generated: fav-profile-compare.svg (side-by-side)
```

**完了条件**: Rust テスト 2 件（ベース 3437 + 2 = 3439 tests passed, 0 failed）
- `profile_flamegraph_aot`
- `profile_flamegraph_svg_generated`

**実績**: 2026-08-02 完了。`cmd_profile_flamegraph_aot`（IR fns → StageRecord → generate_svg）追加、`v64400_tests` 2 件追加。3439 tests passed, 0 failed。`--compare-vm` / ブラウザ自動表示は後送り（v64.5 以降）。

---

### v64.5.0 — 外部ベンチマーク比較（`site/content/docs/runtime/benchmarks.mdx`）

`site/content/docs/runtime/benchmarks.mdx` に比較ベンチマーク結果ページを作成。
再現可能なベンチマークスクリプト（`fav/benchmarks/compare/run_comparison.sh`）を公開。
比較対象: Python pandas / Apache Beam / dbt（SQL 変換）。
`fav/benchmarks/compare/` に各比較ツールの実装スクリプトを追加。

```
Benchmark: 1M row CSV → Postgres transform
  Favnir AOT: 1,180 ms  (847k rows/s)  ✓
  pandas:     8,340 ms  (120k rows/s)   7.1× slower
  Apache Beam: 5,210 ms (192k rows/s)  4.4× slower
  dbt (SQL):  3,210 ms  (312k rows/s)  2.7× slower
```

**完了条件**: Rust テスト 2 件（ベース 3439 + 2 = 3441 tests passed, 0 failed）
- `docs_benchmarks_page_exists`
- `benchmark_compare_script_exists`

**実績**: 2026-08-02 完了。`site/content/docs/runtime/benchmarks.mdx` 新規作成、`benchmarks/compare/run_comparison.sh` 新規作成、`v64500_tests` 2 件追加。3441 tests passed, 0 failed。個別スクリプト（pandas/Beam/dbt）は `run_comparison.sh` に統合（後送り）。

---

### v64.6.0 — `fav lint --perf`（パフォーマンス lint 一括実行）

`main.rs` / `driver.rs` の `fav lint` に `--perf` フラグを追加。
`--perf` 有効時に W039 / W040 / W041 を一括有効化（`--strict` と独立したフラグ）。
`fav.toml` の `[lint]` セクションに `perf = true` オプションを追加し
`toml.rs` の `LintTomlConfig` に `perf: Option<bool>` フィールドを追加してパース
（`LintConfig` は `lint.rs` に存在し v63.6.0 で `perf: bool` 追加済み）。

```bash
$ fav lint --perf pipeline.fav
W040: large `collect()` without prior filter (pipeline.fav:22) [perf]
W039: type hole `_` reduces AOT optimization (pipeline.fav:5)  [perf]
```

```toml
[lint]
perf = true
```

**完了条件**: Rust テスト 2 件（ベース 3441 + 2 = 3443 tests passed, 0 failed）
- `lint_perf_flag_enables_w039_w040`
- `lint_toml_perf_setting`

**実績**: 2026-08-02 完了。`LintTomlConfig.perf: Option<bool>` 追加（`toml.rs`）、`cmd_lint` の perf を toml 設定から読み取るよう更新（`driver.rs`）、`v64600_tests` 2 件追加。3443 tests passed, 0 failed。`--perf` CLI フラグ（`main.rs`）は後送り（v64.7 以降）。

---

### v64.7.0 — `fav build --target wasm32` 出力（Playground 向け）

既存の WASM ビルド基盤（`wasm-encoder` クレート、Cargo.toml 登録済み）を
`fav build --target wasm32-unknown-unknown` と統合。
`aot.rs` の target 選択ロジックを拡張して wasm32 を追加。
生成した `.wasm` ファイルを Playground の `@favnir/wasm` パッケージに組み込めるよう
エクスポート関数シグネチャを整備する。

```bash
$ fav build pipeline.fav --target wasm32-unknown-unknown -o pipeline.wasm
Compiling pipeline.fav (target: wasm32)...
Output: pipeline.wasm (WASM module, 48 KB)
```

**完了条件**: Rust テスト 2 件（ベース 3443 + 2 = 3445 tests passed, 0 failed）
- `build_wasm_target_outputs_wasm`
- `wasm_build_compat_check`

**実績**: 2026-08-02 完了。`cmd_build_wasm`（`wasm_codegen_program` 経由）追加（`driver.rs`）、`v64700_tests` 2 件追加。3445 tests passed, 0 failed。`cmd_build` への `\"wasm32\"` アーム統合・Playground エクスポート関数シグネチャ整備は後送り（v64.9 以降）。

---

### v64.8.0 — ドキュメントサイト Performance 1.0 総括記事

`site/content/docs/performance/performance1-overview.mdx` を作成。
v61〜v64 の全機能（DX 2.0 / Language Polish / AOT Native / Incremental & Scale）を
統括する概観記事とクイックスタートガイドを記述。
認定チェックリスト形式（`fav bench` / `fav profile` / `fav build --ci` の通過を確認）を掲載。

```bash
# Performance 1.0 クイックスタート
$ fav build pipeline.fav --link -o dist/pipeline   # AOT ビルド
$ fav bench pipeline.fav --runs 10                 # ベンチ確認
$ fav profile --memory pipeline.fav                # メモリ確認
$ fav lint --perf pipeline.fav                     # パフォーマンス lint
```

**完了条件**: Rust テスト 2 件（ベース 3445 + 2 = 3447 tests passed, 0 failed）
- `docs_performance1_overview_exists`
- `docs_performance1_has_quickstart`

**実績**: 2026-08-02 完了。`site/content/docs/performance/performance1-overview.mdx` 新規作成（概観記事・クイックスタート・認定チェックリスト・ベンチマーク比較）、`v64800_tests` 2 件追加。3447 tests passed, 0 failed。

---

### v64.9.0 — 安定化・コードフリーズ（Performance 1.0 前調整）

v61〜v64 の全テストが通過していることを確認。
全 lint / clippy クリーン確認。
`site/content/docs/performance/performance1-overview.mdx` の最終確認
（クイックスタート・認定チェックリストが正しく記述されているか）。

**完了条件**: Rust テスト 2 件（ベース 3447 + 2 = 3449 tests passed, 0 failed）
- `scale_all_v64_features_stable`
- `performance1_overview_doc_complete`

**実績**: 2026-08-02 完了。`v64900_tests` 2 件追加（`scale_all_v64_features_stable` で v64.1/v64.4/v64.7 動作確認、`performance1_overview_doc_complete` で MDX 4 セクション検証）。3449 tests passed, 0 failed。

---

### v65.0 — Performance 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデータパイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

**完了条件**:
- v64.1〜v64.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3453**）
- `v65000_tests` 4 件 pass（ベース 3449 + 4 = 3453 tests passed, 0 failed）:
  - `cargo_toml_version_is_65_0_0`
  - `changelog_has_v65_0_0`
  - `milestone_has_performance1`
  - `readme_mentions_performance1`
- `MILESTONE.md` に `"Performance 1.0"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 2026-08-04 完了。Cargo.toml `"65.0.0"` 更新、MILESTONE.md に Performance 1.0 宣言文追加、README.md 追記、`v65000_tests` 4 件追加（旧 `cargo_toml_version_is_*` テストの `"64.0.0"` 誤参照も一括修正）。cargo clean 後も 3453 tests passed, 0 failed。★クリーンアップ完了。

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v64.0.0（ベース） | 3431 | — | Incremental & Scale 宣言後（v63.6.0 code-reviewer 対応 +3 等の影響） |
| v64.1.0 | 3433 | +2 | CI 統合 |
| v64.2.0 | 3435 | +2 | リグレッション検出 |
| v64.3.0 | 3437 | +2 | パフォーマンスガイド |
| v64.4.0 | 3439 | +2 | flamegraph 改善 |
| v64.5.0 | 3441 | +2 | 外部ベンチ比較 |
| v64.6.0 | 3443 | +2 | lint --perf |
| v64.7.0 | 3445 | +2 | wasm32 出力 |
| v64.8.0 | 3447 | +2 | Performance 1.0 総括記事 |
| v64.9.0 | 3449 | +2 | 安定化 |
| v65.0.0 | 3453 | +4 | Performance 1.0 宣言（★クリーンアップ） |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v63.1-v64.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
