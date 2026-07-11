# v35.3.0 仕様書 — `fav ci init`

## Background

v35.1.0 / v35.2.0 では Lambda・Docker へのデプロイ自動化を実現した。
v35.3.0 は Deployment Story スプリントの第三版として、**GitHub Actions CI ワークフローの自動生成**を追加する。

既存の `fav new` テンプレートが `.github/workflows/ci.yml` を生成するが、それは単一ステップ（`fav check` のみ）の最小構成。
v35.3.0 の `fav ci init` は **check + lint + test の 3 ステップ**を持つ本格的な CI ワークフローを独立コマンドとして生成する。

### ロードマップの `fav/src/ci/github_actions.rs` について

ロードマップ（`roadmap-v35.1-v36.0.md` v35.3.0 セクション）は独立ファイル `fav/src/ci/github_actions.rs` を記載しているが、**v35.1.0 / v35.2.0 と同様に `driver.rs` への直接追加を採用する**。
理由: 独立モジュール化は v36.0 クリーンアップ時に一括実施する。

## Goals

1. `fav ci init` コマンドを追加する
2. `.github/workflows/ci.yml` を生成する（check + lint + test の 3 ステップ）
3. `--out-dir` でワークフローの出力先ディレクトリを変更できる
4. `--dry-run` で生成内容をプレビュー表示のみ（ファイル書き出しなし）

## Syntax / API

### コマンドライン

```bash
# カレントディレクトリに .github/workflows/ci.yml を生成
fav ci init

# --out-dir で出力先を変更
fav ci init --out-dir ./deploy-ci

# --dry-run でプレビュー表示のみ（ファイル書き出しなし）
fav ci init --dry-run
```

### 生成される `.github/workflows/ci.yml`

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install fav
        run: cargo install fav
      - name: Check
        run: fav check
      - name: Lint
        run: fav lint
      - name: Test
        run: fav test
```

### 内部動作フロー

```
1. fav.toml が存在すればプロジェクト名を取得（省略時 "fav-project"）
2. generate_ci_yaml(project_name) で YAML 文字列を生成
3. --dry-run の場合は標準出力に表示して終了
4. <out-dir>/.github/workflows/ci.yml に書き出す
   （親ディレクトリは自動作成）
5. 生成したパスを標準出力に表示
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 変更 | `generate_ci_yaml` 追加 + `cmd_ci_init` 追加 + v35300_ci_tests 追加 |
| `fav/src/main.rs` | 変更 | `Some("ci")` アーム追加 + ヘルプテキスト更新 |
| `fav/Cargo.toml` | 変更 | バージョンを `35.3.0` に更新 |
| `CHANGELOG.md` | 変更 | v35.3.0 エントリ追加 |

> サイトドキュメント（`site/content/docs/deploy/ci.mdx` など）は v35.8.0 で追加予定。本バージョンでは対象外。

## Success Criteria

1. `fav ci init` で `.github/workflows/ci.yml` が生成される
2. 生成された YAML に `check` / `lint` / `test` の 3 ステップが含まれる
3. `--dry-run` でファイルを書かずにプレビューが表示される
4. `cargo test` が 0 failures で通る（v35300_ci_tests の全テストが pass）
   ※ロードマップでは「Rust テスト 2 件」と記載しているが、`cargo_toml_version_is_35_3_0` を含め 6 件を実装する。
5. `CHANGELOG.md` に `## [35.3.0]` エントリが存在する

## 設計決定

### `cmd_ci_init` の出力先

- デフォルト: カレントディレクトリ（`"."` 相当）に `.github/workflows/ci.yml` を生成
- `--out-dir` 指定時: `<out-dir>/.github/workflows/ci.yml`
- 親ディレクトリ（`.github/workflows/`）は `write_text_file` 内の `create_dir_all` で自動作成される

### `generate_ci_yaml` の `project_name` 利用

v35.3.0 では YAML 内にプロジェクト名を埋め込まない（ジョブ名は `ci` 固定）。
将来的に workflow name や job name をカスタマイズしたい場合は引数を使う。

### `Some("ci")` アームのサブコマンド

```
fav ci init    → cmd_ci_init を呼ぶ
fav ci <other> → "error: unknown ci subcommand `<other>`" を stderr に出力して exit 1
fav ci (なし)  → usage を stderr に出力して exit 1
```

サブコマンドなしの場合は他のサブコマンド制御コマンド（`fav orchestrate`）と同様に exit 1 とする。

### `versions/current.md` 更新

マイナーバージョン実装では更新しない。

## Error Codes

新規エラーコードなし。ファイル書き出し失敗時は標準エラーにメッセージを出力して exit 1。
