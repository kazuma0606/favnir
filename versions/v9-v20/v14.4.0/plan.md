# v14.4.0 Plan — 技術実装設計

Date: 2026-06-12

---

## 実装順序（Phase A → F）

```
A: fav/src/backend/vm.rs — Ctx.aws_get_field_raw + AWS.secrets_get_raw 追加
    ↓
B: fav/src/middle/checker.rs — builtin_ret_ty / ns_env_def に登録
    ↓
C: runes/aws/secrets.fav — Secrets Manager ラッパー（新規）
    ↓
D: runes/aws/s3.fav — ctx-aware ラッパー追加
    ↓
E: runes/aws/aws.fav + rune.toml 更新
    ↓
F: fav/src/driver.rs — v144000_tests + Cargo.toml バンプ
```

---

## Phase A: `fav/src/backend/vm.rs`

### A-1: `Ctx.aws_get_field_raw` 追加

`Ctx.azure_get_field_raw`（vm.rs ~13230）の直後に追加。
パターンは完全に同じ — JSON パースしてフィールドを返す。

```rust
"Ctx.aws_get_field_raw" => {
    // Ctx.aws_get_field_raw(ctx: AwsCtx, field: String) -> String
    if args.len() != 2 {
        return Err("Ctx.aws_get_field_raw requires 2 arguments".to_string());
    }
    let ctx_str = as_string(&args[0], "ctx")?;
    let field   = as_string(&args[1], "field")?;
    // ctx_str は "ok({...})" 形式または生 JSON
    let json_str = ctx_str.trim_start_matches("ok(").trim_end_matches(')');
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let val = parsed.get(&field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Value::String(val))
}
```

### A-2: `AWS.secrets_get_raw` 追加

既存の S3 primitives ブロック（vm.rs ~12417）の末尾（`aws_dynamo_*` の後）に追加。
**実装方針: 既存 ureq + SigV4 パターンを踏襲**（`aws-sdk-secretsmanager` crate は追加しない）。

Secrets Manager API のエンドポイント:
```
POST https://secretsmanager.{region}.amazonaws.com/
Content-Type: application/x-amz-json-1.1
X-Amz-Target: secretsmanager.GetSecretValue

Body: {"SecretId": "<secret_name>"}
Response: {"SecretString": "<value>", ...}
```

SigV4 署名は既存の `sign_request` / `aws_sigv4_headers` ヘルパーを使う
（`s3_get_object_raw` や `sqs_send_message_raw` と同じ関数を呼ぶ）。

```rust
"AWS.secrets_get_raw" => {
    // AWS.secrets_get_raw(region: String, secret_name: String) -> Result<String, String>
    if args.len() != 2 {
        return Err("AWS.secrets_get_raw requires 2 arguments".to_string());
    }
    let region      = as_string(&args[0], "region")?;
    let secret_name = as_string(&args[1], "secret_name")?;

    let config = get_aws_config();
    let endpoint = if let Some(ref ep) = config.endpoint_url {
        format!("{}/", ep)
    } else {
        format!("https://secretsmanager.{}.amazonaws.com/", region)
    };

    let body = format!(r#"{{"SecretId":"{}"}}"#, secret_name.replace('"', "\\\""));

    // SigV4 ヘッダー生成（既存ヘルパー流用）
    // service = "secretsmanager", target = "secretsmanager.GetSecretValue"
    // ureq POST → parse SecretString from JSON response
    // ok(secret_string) or err(error_message)
    // ...（実装時に既存 sqs_send_message_raw のパターンを参照）
}
```

### A-3: `cargo build` でコンパイルエラーなし確認

---

## Phase B: `fav/src/middle/checker.rs`

### B-1: `builtin_ret_ty` に追加

`("Ctx", "azure_get_field_raw")` の直後に:

```rust
("Ctx", "aws_get_field_raw") => Some(Type::String),
```

`("AWS", "dynamo_scan_raw")` の後（既存の `("AWS", ...)` ブロック末尾付近）に:

```rust
("AWS", "secrets_get_raw") => Some(Type::Result(
    Box::new(Type::String),
    Box::new(Type::String),
)),
```

### B-2: `ns_env_def` の `"Ctx"` ブロックに `"aws_get_field_raw"` 追加

既存の `"azure_get_field_raw"` が登録されている箇所の直後に追加。

### B-3: `require_aws_effect` 追加

`secrets_get_raw` は `!AWS` エフェクトを要求する。
既存 `("AWS", "s3_get_object_raw")` の match 節と同じパターン:

```rust
("AWS", "secrets_get_raw") => {
    self.require_aws_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
```

### B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C: `runes/aws/secrets.fav`（新規作成）

```fav
// runes/aws/secrets.fav — AWS Secrets Manager wrapper (v14.4.0)

import rune "ctx"

/// Retrieve a secret value from AWS Secrets Manager.
/// `ctx` provides the AWS region.
/// `secret_name` is the secret's name or ARN.
public fn secrets_get(ctx: AwsCtx, secret_name: String) -> Result<String, String> !AWS {
    let region = Ctx.aws_get_field_raw(ctx, "region")
    AWS.secrets_get_raw(region, secret_name)
}
```

**注意**: `import rune "ctx"` が必要（`AwsCtx` 型と `Ctx.aws_get_field_raw` を参照するため）。
`runes/ctx/crosscloud.fav` が `AwsCtx` を定義しているので、ctx rune を import することで使える。

---

## Phase D: `runes/aws/s3.fav` 拡張

**既存関数はそのまま保持**し、ctx-aware ラッパーを末尾に追加。

```fav
// ── ctx-aware ラッパー（v14.4.0） ─────────────────────────────────────────────
// AwsCtx から s3_bucket を自動取得するショートカット関数。
// 既存の get_object(bucket, key) はそのまま残す（後方互換）。

import rune "ctx"

/// Put an object using the bucket from AwsCtx.
public fn s3_put(ctx: AwsCtx, key: String, body: String) -> Result<Unit, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_put_object_raw(bucket, key, body)
}

/// Get an object using the bucket from AwsCtx.
public fn s3_get(ctx: AwsCtx, key: String) -> Result<String, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_get_object_raw(bucket, key)
}

/// Delete an object using the bucket from AwsCtx.
public fn s3_delete(ctx: AwsCtx, key: String) -> Result<Unit, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_delete_object_raw(bucket, key)
}

/// List objects using the bucket from AwsCtx.
public fn s3_list(ctx: AwsCtx, prefix: String) -> Result<List<String>, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_list_objects_raw(bucket, prefix)
}
```

---

## Phase E: `runes/aws/aws.fav` + `rune.toml` 更新

### E-1: `aws.fav` に `use secrets.*` 追加

```fav
// AWS Rune — barrel module (v4.11.0, updated v14.4.0)

use s3.*
use sqs.*
use dynamodb.*
use secrets.*
```

### E-2: `rune.toml` の description 更新

```toml
[rune]
name        = "aws"
version     = "14.4.0"
description = "AWS SDK: S3, SQS, DynamoDB, Secrets Manager operations with SigV4 signing"
entry       = "aws.fav"
effects     = ["!AWS"]
```

---

## Phase F: `fav/src/driver.rs` — v144000_tests + Cargo.toml バンプ

### F-1: `v144000_tests` モジュール追加（`v143000_tests` の直後推奨）

```rust
#[cfg(test)]
mod v144000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn version_is_14_4_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.4.0");
    }

    #[test]
    fn secrets_get_raw_registered() {
        // AWS.secrets_get_raw が E0007 を出さないことを確認
        let src = r#"
public fn main(ctx: AppCtx) -> Unit !AWS {
    bind secret <- AWS.secrets_get_raw("ap-northeast-1", "prod/my-secret")
    ctx.io.println(secret)
}
"#;
        let prog = Parser::parse_str(src, "secrets_test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        let e0007: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0007" && e.message.contains("secrets_get_raw"))
            .collect();
        assert!(e0007.is_empty(),
            "AWS.secrets_get_raw should not produce E0007, got: {:?}", e0007);
    }

    #[test]
    fn aws_ctx_field_raw_registered() {
        // Ctx.aws_get_field_raw が E0007 を出さないことを確認
        let src = r#"
public fn main(ctx: AppCtx) -> Unit {
    bind aws_ctx <- Ctx.build_aws_raw("ap-northeast-1", "my-bucket", "postgresql://localhost/db")
    let region = Ctx.aws_get_field_raw(aws_ctx, "region")
    ctx.io.println(region)
}
"#;
        let prog = Parser::parse_str(src, "aws_field_test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        let e0007: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0007" && e.message.contains("aws_get_field_raw"))
            .collect();
        assert!(e0007.is_empty(),
            "Ctx.aws_get_field_raw should not produce E0007, got: {:?}", e0007);
    }

    #[test]
    fn aws_rune_s3_ctx_functions_present() {
        // runes/aws/s3.fav に s3_put / s3_get が存在することを parse で確認
        let s3_fav = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("runes/aws/s3.fav")
        ).expect("s3.fav should exist");
        assert!(s3_fav.contains("fn s3_put"),
            "s3.fav should contain fn s3_put");
        assert!(s3_fav.contains("fn s3_get"),
            "s3.fav should contain fn s3_get");
    }
}
```

### F-2: `version_is_14_3_0` を `>=` 比較に修正

`v143000_tests` の `assert_eq!(env!("CARGO_PKG_VERSION"), "14.3.0")` を:
```rust
assert!(env!("CARGO_PKG_VERSION") >= "14.3.0", ...);
```
に変更。

### F-3: `fav/Cargo.toml` バージョンバンプ

```toml
version = "14.4.0"
```

---

## 実装上の注意点

1. **`AWS.secrets_get_raw` の SigV4 署名**:
   既存の `sqs_send_message_raw` が POST + JSON body で SigV4 を使っている。
   Secrets Manager も同じく POST + JSON。`service = "secretsmanager"` に変えるだけで
   同じヘルパーが流用できるはず。
   実装時は `sqs_send_message_raw`（vm.rs ~12605）のパターンを参照。

2. **`import rune "ctx"` の扱い**:
   `secrets.fav` と `s3.fav`（の ctx-aware 部分）は `import rune "ctx"` が必要。
   rune ファイル内での rune import は Favnir の rune ローダーが解決する。
   `runes/ctx/crosscloud.fav` が `AwsCtx` を定義しているので問題ない。
   ただし、型チェック時に `AwsCtx` が `Unknown` になる可能性があるため、
   `Ctx.aws_get_field_raw` の引数は実質 String として扱われる（既存動作）。

3. **後方互換性**:
   `s3.fav` の既存関数（`get_object`, `put_object` など）はそのまま残す。
   ctx-aware ラッパーは `s3_put`, `s3_get` など **別名** で追加する。

4. **`aws_rune_test_file_passes`（既存テスト）**:
   このテストは `runes/aws/aws.test.fav` を実行する。
   今回の変更で `aws.fav` に `use secrets.*` を追加するが、`secrets.fav` が
   正しく作成されていれば既存テストに影響しない。

5. **`s3.fav` への `import rune "ctx"` 追加**:
   既存の `s3.fav` 先頭に `import rune "ctx"` を追加する。
   既存関数（`get_object` など）は `ctx` を使わないので影響なし。

---

## 参照先ファイル（実装時に確認すること）

| ファイル | 参照目的 |
|---|---|
| `fav/src/backend/vm.rs:13220-13245` | `Ctx.azure_get_field_raw` — コピー元パターン |
| `fav/src/backend/vm.rs:12605-12640` | `AWS.sqs_send_message_raw` — SigV4 POST パターン |
| `fav/src/middle/checker.rs:6290-6300` | `("Ctx", "azure_get_field_raw")` — builtin_ret_ty パターン |
| `fav/src/middle/checker.rs:6180-6270` | `("AWS", ...)` ブロック — require_aws_effect パターン |
| `runes/aws/s3.fav` | 既存関数（後方互換のため保持） |
| `runes/aws/aws.fav` | バレルモジュール（use secrets.* 追加） |
| `runes/ctx/crosscloud.fav` | `AwsCtx` 型定義 |
