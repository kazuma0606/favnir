# v73.8.0 仕様書 — GitHub Actions 公式 Action

Date: 2026-08-13

---

## Background

Favnir を CI/CD パイプラインに組み込む際、利用者は毎回バイナリのダウンロードスクリプトを自前で書く必要がある。
GitHub Actions の `uses: favnir/setup-fav@v1` という公式 Action を提供することで、1 行でインストールが完了し、`fav check` / `fav test` / `fav quality` / `fav audit` がそのまま CI で実行できるようになる。

---

## Goals

1. `.github/actions/setup-fav/action.yml` を作成する（composite action 形式）
2. OS 別（linux / macos / windows）バイナリ URL 形式を定義する
3. GitHub Releases からバイナリをダウンロードしてパスに追加する手順を記述する
4. 使用例・バッジ・マトリックスビルドサンプルを `README.md` に追記する
5. Rust テスト 2 件を `driver.rs` に追加する

---

## API / YAML 例

```yaml
# .github/workflows/favnir-ci.yml
steps:
  - uses: favnir/setup-fav@v1
    with:
      version: "75.0.0"

  - name: Type Check
    run: fav check pipeline.fav

  - name: Test
    run: fav test pipeline.fav

  - name: Quality Gate
    run: fav quality report pipeline.fav --min-score 80 --fail-below

  - name: Audit
    run: fav audit --deny-high
```

### action.yml（composite action）

```yaml
name: Setup Favnir
description: Install the fav compiler from GitHub Releases
inputs:
  version:
    description: Favnir version to install
    required: true
    default: latest
runs:
  using: composite
  steps:
    - name: Download fav binary
      shell: bash
      run: |
        OS=$(uname -s | tr '[:upper:]' '[:lower:]')
        ARCH=$(uname -m)
        URL="https://github.com/favnir/favnir/releases/download/v${{ inputs.version }}/fav-${OS}-${ARCH}"
        curl -sL "$URL" -o fav && chmod +x fav
        echo "$(pwd)" >> $GITHUB_PATH
```

### バイナリ URL フォーマット

```
https://github.com/favnir/favnir/releases/download/v{version}/fav-{os}-{arch}
```

- `os`: `linux` / `darwin` / `windows`
- `arch`: `x86_64` / `aarch64`

---

## Success Criteria

1. `.github/actions/setup-fav/action.yml` が存在し、`name`・`inputs.version`・`runs.using: composite` を含む
2. `.github/actions/setup-fav/README.md` が存在し、使用例・バッジ・マトリックスサンプルを含む
3. `driver.rs` に `GithubActionConfig` 構造体 + `format_github_action_url` 関数を追加（URL は `https://github.com/favnir/favnir/releases/download/v{version}/fav-{os}-{arch}` 形式であること）
4. `v738000_tests` モジュールに 2 件のテストが存在し pass する
   - `github_action_setup_fav_action_yml_valid`
   - `github_action_fav_binary_url_format`
5. `cargo test` で 3663 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし（ファイル生成・テストのみ）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `GithubActionConfig` 構造体 + `format_github_action_url` 関数 + `v738000_tests` モジュール追加（`include_str!("../../.github/actions/setup-fav/action.yml")` で参照、`../../` = `favnir/` を指す） |
| `fav/Cargo.toml` | `version = "73.8.0"` に更新 |
| `CHANGELOG.md` | v73.8.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |

## Files to Create

| ファイル | 内容 |
|---|---|
| `.github/actions/setup-fav/action.yml` | composite action 定義 |
| `.github/actions/setup-fav/README.md` | 使用例・バッジ・マトリックスサンプル |
