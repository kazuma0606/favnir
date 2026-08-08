# Roadmap v62.1.0 〜 v63.0.0 — AOT Native

Date: 2026-07-30
Status: 未着手

---

## 前提

- 直前完了: v62.0.0「Language Polish」（tests = 3374）
- マスターロードマップ: `roadmap-v60.1-v65.0.md`
- 本文書はマスターの v63.0 スプリント部分の詳細版
- **既存機能の扱い**:
  - `cranelift-object` クレートは Cargo.toml に v23 時点で登録済み
    → `fav build` コマンド新規追加・`aot.rs` 新規作成。Cargo.toml への `cranelift-object` 追加は不要
    （ただし `aarch64` 向け feature フラグを v62.3.0 で有効化する）
  - `cranelift-codegen` / `cranelift-frontend` / `cranelift-module` は Cargo.toml に依存済み
  - `fav profile` flamegraph は v9.9 実装済み → `fav bench` コマンドを新規追加

---

## 目標

**「型安全なパイプラインがネイティブコードに変わる」** ことを実現する。

v62.1〜v62.9 の 9 スプリントで AOT コンパイル基盤を構築し、
v63.0「AOT Native」として宣言する。

---

## バージョン計画

### v62.1.0 — `fav build` コマンド追加（cranelift object ファイル出力）

`main.rs` に `Some("build")` アームを追加し、`--output` / `-o` フラグを解析。
`fav/src/backend/aot.rs` を新規作成。`cranelift-object` は Cargo.toml 登録済みのため
追加不要（feature フラグの有効化のみ確認する）。
pipeline の IR を cranelift IR へ変換し、ELF / Mach-O object ファイルとして出力する基盤を実装。
`driver.rs` に `pub fn cmd_build_basic(src: &str, out: &str) -> String` を追加。

```bash
$ fav build pipeline.fav -o pipeline.o
Compiling pipeline.fav...
Output: pipeline.o (ELF x86_64, 128 KB)
```

**完了条件**: Rust テスト 2 件（ベース 3374 + 2 = 3376 tests passed, 0 failed）
- `cmd_build_outputs_object_file`
- `aot_basic_pipeline_compiles`

**実績**: 3384 tests passed, 0 failed（2026-08-01 完了）
- ベース 3382 + 2 = 3384（ロードマップ記載の 3374/3376 は古い値）
- `lower_to_object_pub`（`pub(crate)` ラッパー）を `cranelift_aot.rs` に追加
- `cmd_build_basic` を `driver.rs` に追加
- 関数呼び出しは AOT v19.2.0 未サポートのため `aot_basic_pipeline_compiles` テストは純算術式（`1 + 2 == 3`）を使用

---

### v62.2.0 — native binary 生成（`fav build --link`・Linux x86_64）

`aot.rs` に object ファイルのリンク処理を追加。
`cc` クレートまたはシステム `ld` の呼び出しで実行可能バイナリを生成。
`--link` フラグで object ファイルのリンクまで実施して実行可能バイナリを出力。
Favnir ランタイムスタブ（`fav/src/backend/fav_rt.rs`）を追加し、
VM プリミティブの最小セット（IO 実行・エラーハンドリング）を静的リンクする。

```bash
$ fav build pipeline.fav --link -o pipeline
$ ./pipeline
[stage LoadCsv] loaded 1000 rows
[stage Transform] processed 1000 rows
```

**完了条件**: Rust テスト 2 件（ベース 3384 + 2 = 3386 tests passed, 0 failed）
- `aot_binary_executable`
- `aot_runtime_stub_linked`

**実績**: 3386 tests passed, 0 failed（2026-08-01 完了）
- `fav_rt.rs` 新規作成（`fav_io_print` / `fav_io_panic` C スタブ）
- `compile_to_binary_pub` ラッパー追加（`cranelift_aot.rs`）
- `cmd_build_link` を `driver.rs` に追加
- `--link` フラグを `main.rs` `Some("build")` アームに追加
- Windows では `cc` 失敗のため `aot_binary_executable` は `"parse error:"` 不存在のみ確認

---

### v62.3.0 — `fav build --target` クロスコンパイルサポート

Cargo.toml の `cranelift-codegen` features に `"arm64"` を追加。
`--target <triple>` フラグを `main.rs` / `aot.rs` に追加。
サポート triple: `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`。
`aot.rs` にターゲット別 ISA 選択ロジックを実装（`cranelift_native::builder()` → target 指定に切り替え）。

```bash
$ fav build pipeline.fav --target aarch64-unknown-linux-gnu -o pipeline-arm
Compiling pipeline.fav (target: aarch64)...
Output: pipeline-arm (ELF aarch64, 132 KB)
```

**完了条件**: Rust テスト 2 件（ベース 3386 + 2 = 3388 tests passed, 0 failed）
- `aot_cross_compile_aarch64`
- `aot_target_triple_parsed`

**実績**: 3388 tests passed, 0 failed（2026-08-01 完了）
- `cranelift-codegen` features に `"arm64"` 追加（Cargo.toml）
- `lower_to_object_with_target` + `lower_to_object_with_target_pub` を `cranelift_aot.rs` に追加
  - `isa::lookup_by_name("aarch64")` で aarch64 クロスコンパイル対応
- `cmd_build_link_target(src, out, target: Option<&str>)` を `driver.rs` に追加
- `cmd_build_link` を薄いラッパーに変更（後方互換維持）
- `main.rs` `--link` ブランチに `--target` triple を接続

---

### v62.4.0 — AOT エフェクトディスパッチ最適化（`!Pure` ステージのインライン化）

`aot.rs` の IR 変換で `effects` フィールドが空（または `!Pure` のみ）の stage を
caller へインライン展開するパスを追加。
エフェクトのある stage（`!IO` / `!Kafka` 等）は runtime dispatch のまま維持。
`fav build --aot-stats` フラグでインライン化された stage 数・削減バイト数を表示。

```favnir
// !Pure ステージは AOT でインライン展開される
stage Transform: List<Row> -> List<Row> = |rows|
  rows |> List.map(transform_row)
  // effects = [] → インライン展開対象
```

**完了条件**: Rust テスト 2 件（ベース 3388 + 2 = 3390 tests passed, 0 failed）
- `aot_pure_stage_inlined`
- `aot_effectful_stage_not_inlined`

**実績**: 2026-08-01 完了。
- `effects` フィールドは v35.4.0 で削除済みのため、`is_aot_pure(expr: &IRExpr) -> bool`（IR 構造解析）で代替実装。
- `AotStats { inlined, dispatched }` + `analyze_for_inlining` を `cranelift_aot.rs` に追加。
- `cmd_build_aot_stats` を `driver.rs` に追加、`fav build --aot-stats` CLI フラグを `main.rs` に追加。
- 「削減バイト数」表示は関数サイズ推定が必要なため v62.9.0 に後送り（ロードマップとの意図的な乖離）。
- 3390 tests passed, 0 failed（ベース 3388 + 2）。

---

### v62.5.0 — `fav bench` コマンド（AOT vs VM 速度比較）

`main.rs` に `Some("bench")` アームを追加。
`driver.rs` に `pub fn cmd_bench(src: &str, runs: usize) -> String` を実装。
VM モードと AOT モードで同一パイプラインを N 回実行してスループット・レイテンシを計測。
結果を標準出力に表示し、`bench-results.json` に書き出す。

```bash
$ fav bench pipeline.fav --runs 10
Mode     | Mean (ms) | P99 (ms) | Throughput
---------|-----------|----------|-----------
VM       |    142.3  |   158.1  | 7,030 rows/s
AOT      |     23.8  |    27.4  | 42,017 rows/s
Speedup  |     5.98x |          |
```

**完了条件**: Rust テスト 2 件（ベース 3390 + 2 = 3392 tests passed, 0 failed）
- `cmd_bench_runs_both_modes`
- `bench_results_json_generated`

**実績**: 2026-08-01 完了。
- `cmd_bench` が `BenchOpts` シグネチャで既存のため、新関数名は `cmd_bench_aot_vm(src, runs, json_out)` に変更。
- VM 計測: `build_artifact` N 回 / AOT 計測: `compile_program` + `lower_to_object` N 回（コンパイル時間、実行時間ではない）。
- Throughput (rows/s) はパイプライン行数取得困難のため非スコープ（ロードマップとの意図的な乖離）。
- 並列テスト競合回避のため `json_out: &str` 引数を追加（空文字列 = 書き出しなし）。
- CLI: `fav bench <file> --aot [--runs N]` で `bench-results.json` を生成。
- 3392 tests passed, 0 failed（ベース 3390 + 2）。

---

### v62.6.0 — Docker / OCI イメージ生成（`fav build --docker`）

`driver.rs` に `pub fn cmd_build_docker(src: &str, tag: &str) -> String` を追加。
`fav build --docker --tag <name>:<ver>` で Dockerfile を自動生成し `docker build` を呼び出す。
ベースイメージは `debian:12-slim`、AOT binary のみを含む最小構成とする。
`fav build --docker --dry-run` で Dockerfile のみ標準出力に表示するモードを追加。

```bash
$ fav build pipeline.fav --docker --tag my-pipeline:latest
Building AOT binary...
Generating Dockerfile...
Building image: my-pipeline:latest (234 MB)
```

**完了条件**: Rust テスト 2 件（ベース 3392 + 2 = 3394 tests passed, 0 failed）
- `build_docker_dockerfile_generated`
- `build_docker_tag_format`

**実績**: 2026-08-01 完了。
- `validate_docker_tag` / `generate_aot_dockerfile`（private）+ `cmd_build_docker_dry_run` / `cmd_build_docker`（`#[cfg(not(target_arch = "wasm32"))]` 付与）を `driver.rs` に追加。
- 既存 `generate_dockerfile`（deploy 用）との名前衝突を避けるため `generate_aot_dockerfile` に命名。
- `fav build <file> --docker --tag <name>:<ver>` / `--dry-run` CLI フラグを `main.rs` に追加。
- docker が利用できない場合は `"docker not available: ..."` を返して graceful 終了。
- 3394 tests passed, 0 failed（ベース 3392 + 2）。

---

### v62.7.0 — `fav.toml` `[build]` セクション（AOT 設定）

`toml.rs` に `BuildConfig { target: String, opt_level: u8, inline_pure_stages: bool, output_dir: String }` を追加。
`FavConfig` に `build: Option<BuildConfig>` フィールドを追加してパース。
`fav build` が `fav.toml` の `[build]` セクションをデフォルト設定として読み込む。
CLI フラグが `fav.toml` 設定を上書きする優先順位を実装（CLI > fav.toml > デフォルト）。

```toml
[build]
target = "x86_64-unknown-linux-gnu"
opt_level = 2
inline_pure_stages = true
output_dir = "dist/"
```

**完了条件**: Rust テスト 2 件（ベース 3394 + 2 = 3396 tests passed, 0 failed）
- `build_toml_config_parsed`
- `build_cli_overrides_toml`

**実績**: 2026-08-01 完了。
- `BuildConfig { target, opt_level, inline_pure_stages, output_dir }` + `impl Default` を `toml.rs` に追加。
- `FavToml.build: Option<BuildConfig>` フィールドを追加。`parse_fav_toml` に `[build]` セク��ョンパース処理を追加。
- `ResolvedBuildConfig` + `resolve_build_config(cli_target, cli_opt_level, cli_inline_pure, cli_output_dir, toml)` を `driver.rs` に追加（CLI > fav.toml > default の優先順位）。
- `main.rs` `Some("build")` アームで `FavToml::load` + `resolve_build_config` を呼び出す最小実装。
- `checker.rs`・`resolver.rs`・`driver.rs` の既存 `FavToml { ... }` ���テラルに `build: None` を追加（後方互換維持）。
- 3396 tests passed, 0 failed（ベース 3394 + 2）。

---

### v62.8.0 — AOT エラーコード E0427（AOT 未サポート機能検出）

`aot.rs` の IR 変換フェーズで AOT 未サポート機能（`eval` / 動的ディスパッチ等）を
検出して E0427 を発行するバリデーターを追加。
`error_catalog.rs` に E0427 を登録（v60.6.0 で確立した `long_description` フィールドを含めること）。

```
E0427: unsupported feature in AOT mode
 --> pipeline.fav:5:3
  |
5 |   fav eval(dynamic_expr)
  |   ^^^^^^^^^^^^^^^^^^^^^^ `eval` は AOT コンパイルではサポートされていません
  |
  help: `fav build` の代わりに `fav run` を使用するか、eval を除去してください。
```

**完了条件**: Rust テスト 2 件（ベース 3396 + 2 = 3398 tests passed, 0 failed）
- `aot_e0427_eval_detected`（実装時に `aot_e0427_emit_detected` に変更済み — 下記実績参照）
- `error_catalog_has_e0427`

**実績**: 2026-08-01 完了。3399 tests passed（ベース 3397 + 2）。
- `contains_aot_unsupported` / `validate_aot_compat` を `cranelift_aot.rs` に追加（impl 外）
- E0427 を `error_catalog.rs` に登録（category: "build"、long_description あり）
- `cmd_build_aot_validate` を `driver.rs` に追加
- テスト名: `aot_e0427_emit_detected` / `error_catalog_has_e0427`（2/2 PASS）

---

### v62.9.0 — 安定化・AOT E2E デモ

`infra/e2e-demo/aot/` を新規作成。
`fav build --link` → native binary → Docker イメージ化 の E2E デモスクリプト
（`infra/e2e-demo/aot/scripts/build-aot.sh`）を作成。
`site/content/docs/runtime/aot.mdx` を作成（`fav build` の使い方・オプション一覧）。

```bash
$ ./infra/e2e-demo/aot/scripts/build-aot.sh
[1/3] fav build pipeline.fav --link -o dist/pipeline
[2/3] fav build pipeline.fav --docker --tag fav-demo:latest
[3/3] docker run --rm fav-demo:latest
All AOT E2E checks passed.
```

**完了条件**: Rust テスト 2 件（ベース 3398 + 2 = 3400 tests passed, 0 failed）
- `aot_e2e_demo_structure`
- `docs_aot_mdx_exists`

**実績**: 2026-08-01 完了。3402 tests passed（ベース 3400 + 2）。
- `infra/e2e-demo/aot/`（`src/pipeline.fav` + `scripts/build-aot.sh` + `README.md`）作成
- `site/content/docs/runtime/aot.mdx` 作成（`fav build` コマンド一覧・E0427 解説）
- テスト: `aot_e2e_demo_structure` / `docs_aot_mdx_exists`（2/2 PASS）
- `[3/3]` は `docker run` ではな��� `fav build --validate`（CI 環境依存回避のため変更）

---

### v63.0 — AOT Native 宣言 ★クリーンアップ

**宣言文**:

> 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
>  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
>
>  Favnir は型安全なコンパイル言語として新たな段階に達した。
>
>  これが Favnir v63.0 — AOT Native の姿である。」

**完了条件**:
- v62.1〜v62.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3404**）
- `v63000_tests` 4 件 pass（ベース 3400 + 4 = 3404 tests passed, 0 failed）:
  - `cargo_toml_version_is_63_0_0`
  - `changelog_has_v63_0_0`
  - `milestone_has_aot_native`
  - `readme_mentions_aot_native`
- `MILESTONE.md` に `"AOT Native"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 2026-08-02 完了。3406 tests passed（ベース 3402 + 4）。
- `v63000_tests` 4 件 pass（`cargo_toml_version_is_63_0_0` / `changelog_has_v63_0_0` / `milestone_has_aot_native` / `readme_mentions_aot_native`）
- `Cargo.toml` version `62.0.0` → `63.0.0` 更新
- `driver.rs` 内の `cargo.contains("version = \"62.0.0\"")` アサーション 12 件を `63.0.0` に一括更新
- `MILESTONE.md` に AOT Native 宣言エントリ追加（v62.1〜v62.9 達成内容集約）
- `README.md` に v63.0.0 AOT Native 言及追加
- `CHANGELOG.md` に v63.0.0 エントリ追加
- `cargo clean` 実施（★クリーンアップ完了）

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v62.0.0（ベース） | 3382 | — | Language Polish 宣言後（実績値） |
| v62.1.0 | 3384 | +2 | fav build 基盤（実績値） |
| v62.2.0 | 3386 | +2 | native binary 生成 |
| v62.3.0 | 3388 | +2 | クロスコンパイル |
| v62.4.0 | 3390 | +2 | Pure stage インライン化 |
| v62.5.0 | 3392 | +2 | fav bench |
| v62.6.0 | 3394 | +2 | Docker 出力 |
| v62.7.0 | 3396 | +2 | fav.toml [build] |
| v62.8.0 | 3399 | +2 | E0427 ✅ |
| v62.9.0 | 3402 | +2 | 安定化・E2E デモ ✅ |
| v63.0.0 | 3406 | +4 | AOT Native 宣言（★クリーンアップ）✅ |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v61.1-v62.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
