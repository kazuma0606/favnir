# v15.5.0 Plan — `fav deploy`（AWS Lambda デプロイ CLI）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

### A-1: `fav/Cargo.toml` version 更新

```toml
version = "15.5.0"
```

---

## Phase B — `DeployConfig` 拡張（toml.rs）

### B-1: `DeployConfig` struct に `target` / `function_name` フィールド追加

```rust
pub struct DeployConfig {
    /// Deploy target platform (v15.5.0): "aws-lambda" | "azure-function"
    pub target: String,
    /// Lambda function name override (v15.5.0); defaults to fav.toml [project] name
    pub function_name: Option<String>,
    pub runtime: String,
    pub handler: String,
    pub memory: u32,
    pub timeout: u32,
    pub s3_bucket: Option<String>,
    pub role_arn: Option<String>,
    pub region: Option<String>,
}

impl Default for DeployConfig {
    fn default() -> Self {
        DeployConfig {
            target: "aws-lambda".to_string(),
            function_name: None,
            runtime: "provided.al2023".to_string(),  // al2 → al2023 に更新
            handler: "bootstrap".to_string(),
            memory: 256,
            timeout: 30,
            s3_bucket: None,
            role_arn: None,
            region: None,
        }
    }
}
```

### B-2: `parse_fav_toml` の "deploy" セクション解析に対応キー追加

```rust
"deploy" => {
    match key {
        "target"          => current.target        = val.to_string(),
        "function_name"   => current.function_name = Some(val.to_string()),
        "runtime"         => current.runtime       = val.to_string(),
        "handler"         => current.handler       = val.to_string(),
        "memory" | "memory_mb"     => current.memory  = val.parse().unwrap_or(256),
        "timeout" | "timeout_sec"  => current.timeout = val.parse().unwrap_or(30),
        "s3_bucket"       => current.s3_bucket     = Some(val.to_string()),
        "role_arn"        => current.role_arn       = Some(val.to_string()),
        "region"          => current.region         = Some(val.to_string()),
        _ => {}
    }
}
```

`memory_mb` / `timeout_sec` をロードマップ仕様のエイリアスとして追加。

---

## Phase C — `scripts/build-lambda-layer.sh` 作成

### C-1: `scripts/build-lambda-layer.sh` 新規作成

```bash
#!/usr/bin/env bash
# cross-compile fav → x86_64-unknown-linux-musl
# bootstrap スクリプト同梱
# function.zip として出力

cross build --release --target x86_64-unknown-linux-musl --bin fav
cp target/x86_64-unknown-linux-musl/release/fav /tmp/package/fav

# bootstrap: Lambda Runtime API ループ
cat > /tmp/package/bootstrap << 'EOF'
#!/bin/sh
FAV_FILE="${FAV_FILE:-/var/task/main.fav}"
while true; do
  REQUEST_ID=$(...)
  /var/task/fav run --legacy "$FAV_FILE"
  curl -X POST ".../response" ...
done
EOF
chmod +x /tmp/package/bootstrap

cd /tmp/package && zip function.zip fav bootstrap
```

---

## Phase D — `site/content/docs/deploy.mdx` 作成

### D-1: `site/content/docs/deploy.mdx` 新規作成

内容:
- `fav.toml [deploy]` 設定リファレンス表
- `fav deploy --dry-run` の実行例・出力例
- `scripts/build-lambda-layer.sh` の使い方
- 必要 IAM 権限 JSON
- Lambda での環境変数設定例

---

## Phase E — v155000_tests 追加（driver.rs）

### E-1: `v155000_tests` モジュール追加

```rust
// ── v155000_tests (v15.5.0) — fav deploy ─────────────────────────────────────
#[cfg(test)]
mod v155000_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn version_is_15_5_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"15.5.0\""), ...);
    }

    #[test]
    fn deploy_toml_schema_parses() {
        let toml_src = "[project]\nname = \"test\"\n...\n[deploy]\ntarget = \"aws-lambda\"\n\
                        function_name = \"my-fn\"\nruntime = \"provided.al2023\"\n\
                        memory_mb = 512\ntimeout_sec = 300\n...";
        let parsed = crate::toml::parse_fav_toml_pub(toml_src);
        let deploy = parsed.deploy.expect("deploy section should be parsed");
        assert_eq!(deploy.target, "aws-lambda");
        assert_eq!(deploy.function_name.as_deref(), Some("my-fn"));
        assert_eq!(deploy.runtime, "provided.al2023");
        assert_eq!(deploy.memory, 512);
        assert_eq!(deploy.timeout, 300);
    }

    #[test]
    fn deploy_cmd_exists() {
        let driver = fs::read_to_string("src/driver.rs").unwrap();
        assert!(driver.contains("fn cmd_deploy"), ...);
    }
}
```

---

## Phase F — テスト・コミット

### F-1: `cargo test v155000` → 3/3 パス最終確認

### F-2: `cargo test` → 全件パス（リグレッションなし）確認

### F-3: コミット

```
feat: v15.5.0 — fav deploy 完成（AWS Lambda デプロイ CLI）
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version 15.5.0 |
| `fav/src/toml.rs` | 更新 | DeployConfig に target / function_name 追加、memory_mb / timeout_sec エイリアス |
| `fav/src/driver.rs` | 更新 | v155000_tests 追加 |
| `scripts/build-lambda-layer.sh` | 新規 | Lambda 用クロスコンパイル + zip パッケージング |
| `site/content/docs/deploy.mdx` | 新規 | fav deploy ユーザーガイド |
| `versions/v15.5.0/spec.md` | 新規 | 仕様書 |
| `versions/v15.5.0/plan.md` | 新規 | 実装計画 |
| `versions/v15.5.0/tasks.md` | 新規 | タスクリスト |

---

## 実装上の注意点

1. **`cmd_deploy` は既実装**: v4.11.0 で実装済み。`package_project_zip` / `deploy_upload_to_s3` / `deploy_update_lambda` の 3 関数で構成。

2. **DeployConfig 既存フィールドの互換性**: `runtime` のデフォルトを `"provided.al2"` → `"provided.al2023"` に更新。既存 `fav.toml` で明示指定している場合は影響なし。

3. **`cross` crate について**: Windows 環境では `cargo build --target x86_64-unknown-linux-musl` は glibc の問題で動作しない場合がある。CI（Linux）または WSL2 での実行を推奨。`build-lambda-layer.sh` は `cross` を使用。

4. **`deploy_update_lambda` の初回デプロイ**: 関数が存在しない場合は `create-function` にフォールバックする実装が `driver.rs` に存在。`role_arn` が設定されていない場合は early exit。
