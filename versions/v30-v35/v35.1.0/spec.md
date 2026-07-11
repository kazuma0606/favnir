# v35.1.0 仕様書 — `fav deploy --target lambda`

## Background

v35.0「Production Ready」では `fav build --target native`（v33.1 実装済み）で
ネイティブバイナリを生成し、**手動で** Lambda にデプロイする形での達成を宣言した。

v35.1.0 は「手動デプロイ」を **`fav deploy` CLI による自動化** に引き上げる。
Deployment Story スプリント（v35.1〜v36.0）の第一版。

## Goals

1. `fav deploy --target lambda` コマンドを追加する
2. 生成済みネイティブバイナリを Lambda 用の `bootstrap.zip` にパッケージングする
3. AWS CLI 経由で `aws lambda update-function-code` を実行する
4. 基本設定を `fav.toml` の `[deploy]` セクションから読み込む
5. `examples/lambda-deploy/` にデモプロジェクトを追加する

## Syntax / API

### コマンドライン

```bash
# fav.toml の [deploy] セクションから設定を読み込んで実行
fav deploy --target lambda

# コマンドラインで直接指定
fav deploy --target lambda --function my-pipeline --region ap-northeast-1

# zip のみ生成（アップロードしない）
fav deploy --target lambda --package-only --output bootstrap.zip
```

### fav.toml の `[deploy]` セクション

```toml
[deploy]
target = "lambda"
function = "my-pipeline"
region = "ap-northeast-1"
memory = 512
timeout = 60
```

### 内部動作フロー

```
1. fav.toml [deploy] または CLI 引数から設定を読み込む
2. fav build --target native が生成した native バイナリを探す
   （デフォルト: target/native/pipeline または fav.toml の [build.output]）
3. bootstrap.zip を作成
   - バイナリを "bootstrap" という名前で zip に含める（Lambda provided.al2 規約）
4. AWS CLI で更新（--package-only 指定時はスキップ）
   aws lambda update-function-code \
     --function-name <function> \
     --zip-file fileb://bootstrap.zip \
     --region <region>
5. 成功/失敗を標準出力に報告
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/main.rs` | 変更 | `mod deploy;` 宣言追加 + `Some("deploy")` アーム追加 |
| `fav/src/deploy/lambda.rs` | 新規 | Lambda パッケージング + デプロイロジック |
| `fav/src/deploy/mod.rs` | 新規 | deploy モジュール宣言（`pub mod lambda;`） |
| `fav/src/toml.rs` | 変更 | `[deploy]` セクションのパース追加（`DeployConfig` 構造体） |
| `fav/Cargo.toml` | 確認のみ | `zip` crate は v0.6 で既存登録済み、追加不要 |
| `fav/src/driver.rs` | 変更 | v35100_tests モジュール追加 |
| `examples/lambda-deploy/` | 新規 | デモプロジェクト（fav.toml + main.fav + README） |
| `site/content/docs/deploy/lambda.mdx` | 新規（スタブ） | Lambda デプロイガイドのスタブ（v35.8 で充実化） |
| `CHANGELOG.md` | 変更 | v35.1.0 エントリ追加 |

## Success Criteria

1. `fav deploy --target lambda --function test-fn --package-only` が `bootstrap.zip` を生成する
2. `bootstrap.zip` に `"bootstrap"` というエントリが含まれている
3. `fav.toml` の `[deploy]` セクションが正しくパースされる
4. `fav deploy` コマンドが `Some("deploy")` アームで処理される
5. `examples/lambda-deploy/fav.toml` が存在する
6. `site/content/docs/deploy/lambda.mdx` のスタブが存在する
7. `cargo test` が 0 failures で通る（v35100_tests 7 件 pass）
8. `CHANGELOG.md` に `## [35.1.0]` エントリが存在する

## 設計決定

### AWS CLI 経由（AWS SDK 非使用）

ロードマップ記述の「AWS SDK 経由」を **AWS CLI 経由（`std::process::Command`）に変更** した。

理由:
- AWS SDK（`aws-sdk-lambda`）は多数の依存を追加しバイナリサイズが増大する
- AWS CLI は Lambda デプロイ環境に通常インストールされており、既存の認証設定を流用できる
- `--package-only` フラグで AWS CLI 非依存の動作も可能

AWS CLI がインストールされていない場合は警告を出し、`--package-only` 相当の動作にフォールバックする。
ロードマップ（`roadmap-v35.1-v36.0.md`）の v35.1 セクションには「AWS CLI 経由」と同期済み。

### WASM ビルド除外

`deploy` モジュール全体を `#[cfg(not(target_arch = "wasm32"))]` で除外する。
`main.rs` の `Some("deploy")` アームも同様に除外する。

### `mod deploy;` の置き場所

`fav/src/main.rs` に他の `mod` 宣言と並べて追加する（`lib.rs` は使用しないプロジェクト構成）。

### `versions/current.md` 更新

マイナーバージョン実装では更新しない。`current.md` の「進行中バージョン」欄の更新は
マイルストーン版（x.0.0）のみで実施する。

## Error Codes

新規エラーコードなし。ビルド済みバイナリが見つからない場合は標準エラーにメッセージを出力して exit 1。
