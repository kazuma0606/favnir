# v35.1.0 実装計画 — `fav deploy --target lambda`

## 依存関係順序

```
toml.rs パース追加（DeployConfig）
    ↓
deploy/mod.rs + deploy/lambda.rs 新規作成
    ↓
main.rs — mod deploy; 追加 + Some("deploy") アーム追加
    ↓
examples/lambda-deploy/ 追加
    ↓
site/content/docs/deploy/lambda.mdx スタブ追加
    ↓
driver.rs — v35100_tests 追加 + 前バージョン固定テストスタブ化
    ↓
Cargo.toml バージョン → 35.1.0
    ↓
cargo test 全通過確認
    ↓
CHANGELOG 更新
```

---

## Step 1: `fav/src/toml.rs` — `[deploy]` セクションのパース追加

`FavConfig` 構造体（または同等の設定構造体）に `deploy: Option<DeployConfig>` フィールドを追加する。

```rust
#[derive(Debug, Default)]
pub struct DeployConfig {
    pub target: Option<String>,   // "lambda" / "docker" / "k8s"
    pub function: Option<String>, // Lambda 関数名
    pub region: Option<String>,   // AWS リージョン
    pub memory: Option<u32>,      // Lambda メモリ (MB)
    pub timeout: Option<u32>,     // Lambda タイムアウト (秒)
    pub output: Option<String>,   // --package-only 時の出力パス
}
```

`parse_fav_toml` 関数で `[deploy]` セクションを読み込み、`DeployConfig` にマッピングする。

---

## Step 2: `fav/src/deploy/mod.rs` 新規作成

```rust
pub mod lambda;
```

`fav/src/main.rs` の既存 `mod` 宣言群（41〜77 行付近）に `mod deploy;` を追加する。
`lib.rs` は本プロジェクトで mod 宣言に使用していないため、`main.rs` のみ更新する。

`#[cfg(not(target_arch = "wasm32"))]` ガードを付ける:

```rust
#[cfg(not(target_arch = "wasm32"))]
mod deploy;
```

---

## Step 3: `fav/src/deploy/lambda.rs` 新規作成

`zip` crate は Cargo.toml に既登録（v0.6、`deflate` feature）のため追加不要。

### `pub fn package_lambda(binary_path: &Path, output_zip: &Path) -> Result<(), String>`

1. `binary_path` のファイルを読み込む（`std::fs::read`）
2. `zip::ZipWriter` でアーカイブを作成
3. エントリ名 `"bootstrap"` で書き込む（Lambda provided.al2 規約）
4. `output_zip` に保存する

### `pub fn deploy_lambda(zip_path: &Path, function: &str, region: &str) -> Result<(), String>`

1. AWS CLI の存在確認: `Command::new("aws").arg("--version")` の exit code を確認
2. CLI が存在しない場合は警告を出して `Ok(())` を返す（フォールバック）
3. 存在する場合は以下を実行:
   ```
   aws lambda update-function-code
     --function-name <function>
     --zip-file fileb://<zip_path>
     --region <region>
   ```
4. exit code を確認し、0 以外はエラー返す

### `pub fn cmd_deploy(args: &[String], config: Option<&DeployConfig>) -> Result<(), String>`

CLI エントリポイント。引数パース（CLI 引数 > config > デフォルト）。

対応フラグ:
- `--target lambda`（必須または config から）
- `--function <name>`
- `--region <region>`（デフォルト: `ap-northeast-1`）
- `--package-only`（zip 生成のみ、deploy_lambda をスキップ）
- `--output <path>`（--package-only 時の zip 出力先、デフォルト: `bootstrap.zip`）

フロー: `package_lambda` → （`--package-only` でなければ）`deploy_lambda`

全体を `#[cfg(not(target_arch = "wasm32"))]` で囲む。

---

## Step 4: `fav/src/main.rs` — `Some("deploy")` アーム追加

既存の `match subcommand` パターンに追加:

```rust
#[cfg(not(target_arch = "wasm32"))]
Some("deploy") => {
    let args: Vec<String> = std::env::args().skip(2).collect();
    let config = load_fav_toml().ok().and_then(|c| c.deploy);
    if let Err(e) = deploy::lambda::cmd_deploy(&args, config.as_ref()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
```

ヘルプテキストにも `deploy` を追記する。

---

## Step 5: `examples/lambda-deploy/` 追加

```
examples/lambda-deploy/
├── fav.toml          [deploy] セクションを含む
├── src/
│   └── main.fav      サンプルパイプライン（IO.println を使った hello world 程度）
└── README.md         30 分でデプロイできる手順書
```

`fav.toml` の最小構成:
```toml
[package]
name = "lambda-demo"
version = "0.1.0"

[deploy]
target = "lambda"
function = "lambda-demo"
region = "ap-northeast-1"
memory = 256
timeout = 30
```

---

## Step 6: `site/content/docs/deploy/lambda.mdx` スタブ追加

```mdx
---
title: Lambda Deploy
---

# Deploying to AWS Lambda

> This page is a stub. Full documentation will be added in v35.8.

Use `fav deploy --target lambda` to deploy your pipeline to AWS Lambda.
```

v35.8 で充実化する予定のため、スタブとして存在するだけでよい。

---

## Step 7: `fav/src/driver.rs` — v35100_tests モジュール追加

前バージョン固定テスト（`cargo_toml_version_is_35_X_X` など）をスタブ化する。

v35100_tests に追加するテスト（7 件）:
- `cargo_toml_version_is_35_1_0` — Cargo.toml に `35.1.0` が含まれる
- `deploy_command_exists_in_main` — `main.rs` に `Some("deploy")` が含まれる
- `lambda_package_creates_zip` — `package_lambda` が bootstrap.zip を生成する
- `lambda_zip_contains_bootstrap_entry` — zip の中に `"bootstrap"` エントリがある
- `deploy_config_parse_from_toml` — `[deploy]` セクションが DeployConfig にパースされる
- `examples_lambda_deploy_exists` — `examples/lambda-deploy/fav.toml` が存在する
- `changelog_has_v35_1_0` — `CHANGELOG.md` に `[35.1.0]` が含まれる

---

## Step 8: `fav/Cargo.toml` バージョン更新

```toml
version = "35.1.0"
```

---

## Step 9: `cargo test` 全通過確認

```
cargo test 2>&1 | tail -5
# expected: test result: ok. XXXX passed; 0 failed
```

v35100_tests の 7 テストが pass することを確認する。

---

## Step 10: `CHANGELOG.md` 更新

テスト全通過確認後に追加する（テスト未通過のまま changelog を更新しない）。

先頭に以下を追加:

```markdown
## [35.1.0] — 2026-07-06

### Added
- `fav deploy --target lambda` — Lambda への自動デプロイコマンド
- `fav deploy --package-only` — bootstrap.zip 生成のみ（アップロードなし）
- `fav.toml` の `[deploy]` セクションで関数名・リージョン・メモリ・タイムアウトを設定可能
- `examples/lambda-deploy/` — Lambda デプロイデモプロジェクト
- `site/content/docs/deploy/lambda.mdx` — Lambda デプロイガイドスタブ
```
