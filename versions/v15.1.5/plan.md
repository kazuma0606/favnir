# v15.1.5 Plan — CrossCloud 認証層 セキュア版（KMS 非対称署名）

Date: 2026-06-13

---

## Phase A: Cargo バージョン更新

### A-1: `fav/Cargo.toml`
```toml
version = "15.1.5"
```

---

## Phase B: リグレッションテスト追加（v15.1.0 デバッグ教訓）

### B-1: `fav/src/driver.rs` — `v15150_tests` モジュール追加

```rust
#[cfg(test)]
mod v15150_tests {
    // B-1a: fav run --legacy が Result.err を返すと exit 1 になることを確認
    // ※ process::exit(1) はテストから直接確認できないため、
    //    exec_artifact_main_with_source の戻り値を確認 + cmd_run 側の exit 分岐を単体テスト
    fn legacy_run_result_err_triggers_exit_path()
    fn legacy_run_result_ok_does_not_trigger_exit_path()

    // B-1b: バージョン確認
    fn version_is_15_1_5()

    // B-1c: KMS インフラファイル確認
    fn crosscloud_kms_terraform_has_ecc_key()

    // B-1d: verifier_v2 の存在確認
    fn crosscloud_verifier_v2_exists()

    // B-1e: auth-comparison.md の存在確認
    fn crosscloud_auth_comparison_doc_exists()
}
```

### B-2: `fav run --legacy` の Result.err exit path 単体テスト設計

`process::exit(1)` はテスト内から呼べないため、以下のアプローチを取る:

```rust
// driver.rs に内部ヘルパーを追加（テスト可能な形で分離）
fn is_result_err_value(v: &Value) -> bool {
    matches!(v, Value::Variant(tag, _) if tag == "err")
}

// テスト:
fn legacy_run_result_err_triggers_exit_path() {
    let src = r#"public fn main(ctx: AppCtx) -> Result<Unit, String> { Result.err("test") }"#;
    let prog = Parser::parse_str(src, "t.fav").unwrap();
    let artifact = build_artifact_legacy(&prog);
    let result = exec_artifact_main_with_source(&artifact, None, None).unwrap();
    assert!(is_result_err_value(&result),
        "main が Result.err を返したとき exec_artifact_main_with_source は Variant(err) を返す");
}
```

---

## Phase C: KMS Terraform リソース追加

### C-1: `infra/e2e-demo/crosscloud/terraform/aws/auth.tf` 追記

```hcl
# KMS 非対称署名キー（ECDSA P-256）
resource "aws_kms_key" "crosscloud_signer" {
  description              = "CrossCloud request signing key (ECDSA P-256)"
  key_usage                = "SIGN_VERIFY"
  customer_master_key_spec = "ECC_NIST_P256"
  deletion_window_in_days  = 7

  tags = { Project = "favnir-crosscloud" }
}

resource "aws_kms_alias" "crosscloud_signer" {
  name          = "alias/crosscloud-signer"
  target_key_id = aws_kms_key.crosscloud_signer.key_id
}
```

### C-2: `aws_iam_role_policy.lambda_verifier_policy` に追記

```hcl
{
  Effect = "Allow"
  Action = ["kms:GetPublicKey"]
  Resource = [aws_kms_key.crosscloud_signer.arn]
}
```

### C-3: スクリプト実行ユーザー（ローカル実行用）の IAM 権限

```hcl
# outputs.tf に KMS key ARN 追加
output "kms_key_arn" {
  value = aws_kms_key.crosscloud_signer.arn
}
output "kms_key_alias" {
  value = aws_kms_alias.crosscloud_signer.name
}
```

---

## Phase D: Lambda verifier_v2

### D-1: `infra/e2e-demo/crosscloud/lambda/verifier_v2/` ディレクトリ作成

verifier.fav ベースで KMS 公開鍵検証に切り替えた Favnir 版 verifier。

ただし、**ECDSA P-256 の署名検証は vm.rs に新 primitive が必要**:
```rust
"Crypto.ecdsa_verify_raw" => {
    // (public_key_pem: String, message: String, signature_der_b64: String)
    //   -> Result<Unit, String>
}
```

#### D-1a: `vm.rs` — `Crypto.ecdsa_verify_raw` primitive 追加

依存: `p256` crate（純 Rust、追加依存最小）
```toml
# Cargo.toml
p256 = { version = "0.13", features = ["ecdsa", "pem"] }
```

実装:
```rust
"Crypto.ecdsa_verify_raw" => {
    // args: pub_key_pem, message_str, sig_der_b64
    // 1. base64 decode sig → DER bytes
    // 2. p256::ecdsa::VerifyingKey::from_public_key_pem(pub_key_pem)
    // 3. p256::ecdsa::Signature::from_der(&sig_bytes)
    // 4. verify(message_str.as_bytes(), &sig) → Result.ok(()) / Result.err("ecdsa_verify_failed")
}
```

#### D-1b: `checker.rs` — `Crypto.ecdsa_verify_raw` 追加

`builtin_ret_ty` の `Crypto.*` ブランチに追記。
エフェクト: `!Auth`（既存の `Crypto.*` と同じ）

#### D-1c: `checker.fav` — `crypto_fn` スキーム追加

#### D-1d: `lambda/verifier_v2/verifier_v2.fav` 実装

v15.1.0 の `verifier.fav` から変更点:
- `get_secret(region, secret_arn)` → `get_kms_public_key(region, key_id)` に置き換え
- `verify_hmac(...)` → `verify_ecdsa(pub_key_pem, sts, sig_b64)` に置き換え

```fav
fn get_kms_public_key(region: String, key_id: String) -> Result<String, String> !AWS {
  AWS.kms_get_public_key_raw(region, key_id)
}

fn verify_ecdsa(pub_key_pem: String, sts: String, sig_b64: String) -> Result<Unit, String> !Auth {
  Crypto.ecdsa_verify_raw(pub_key_pem, sts, sig_b64)
}
```

#### D-1e: `AWS.kms_get_public_key_raw` primitive 追加（vm.rs）

```rust
"AWS.kms_get_public_key_raw" => {
    // (region, key_id) -> Result<String, String>
    // KMS GetPublicKey API → DER → PEM 変換して返す
    // PEM 変換: p256::pkcs8::der::Document → base64 → PEM ヘッダー付与
}
```

### D-2: `lambda/verifier_v2/Dockerfile`

v15.1.0 の `lambda/verifier/Dockerfile` を元に `verifier_v2.fav` を参照するよう変更。
`fav` バイナリは同じもの（`Dockerfile.builder` でビルド済み）。

### D-3: `lambda/verifier_v2/bootstrap`

v15.1.0 の bootstrap から **デバッグログを削除**した production 版:

```sh
#!/bin/sh
set -euo pipefail
RUNTIME_API="http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime"

while true; do
  RESP_HEADERS=$(mktemp)
  EVENT=$(curl -sS -D "$RESP_HEADERS" "${RUNTIME_API}/invocation/next")
  REQUEST_ID=$(awk -v IGNORECASE=1 \
    '/lambda-runtime-aws-request-id/{gsub(/\r/,"",$2); print $2; exit}' \
    "$RESP_HEADERS")
  rm -f "$RESP_HEADERS"

  export VERIFY_REQUEST_ID="${REQUEST_ID}"
  # ... 環境変数マッピング（v15.1.0 と同じ） ...
  # X-KMS-Key-Id を追加で読む
  export VERIFY_KMS_KEY_ID=$(echo "$EVENT" | jq -r '.headers["x-kms-key-id"] // ""')

  EXIT_CODE=0
  OUTPUT=$(fav run --legacy /var/task/verifier_v2.fav 2>&1) || EXIT_CODE=$?

  if [ "$EXIT_CODE" -eq 0 ]; then
    PAYLOAD='{"statusCode":200,...}'
  else
    # エラー種別に応じて 401/409/500 を返す（v15.1.0 と同じロジック）
    ...
  fi

  curl -sS -X POST "${RUNTIME_API}/invocation/${REQUEST_ID}/response" \
    -H "Content-Type: application/json" -d "$PAYLOAD"
done
```

---

## Phase E: スクリプト

### E-1: `scripts/run_with_kms.sh`

```bash
#!/bin/bash
# run_with_kms.sh — KMS ECDSA 署名でリクエストを送信
# Usage: ./run_with_kms.sh <api_endpoint> <kms_key_id> <cognito_client_id> <username> <password>
set -euo pipefail

# 1. Cognito トークン取得（v15.1.0 の run_with_auth.sh と同じ）
# 2. StringToSign 構築（v15.1.0 と同形式）
# 3. KMS Sign API でバイナリ署名取得
SIG_B64=$(aws kms sign \
  --region "$REGION" \
  --key-id "$KMS_KEY_ID" \
  --signing-algorithm ECDSA_SHA_256 \
  --message-type RAW \
  --message fileb://<(printf '%s' "$STRING_TO_SIGN") \
  --query "Signature" --output text)

# 4. POST リクエスト送信
curl -sS -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIG_B64}" \
  -H "X-KMS-Key-Id: ${KMS_KEY_ID}" \
  ...
```

### E-2: `scripts/reject_kms.sh`

```bash
# ケース 1: 改ざんボディ（ECDSA 検証失敗） → 401
# ケース 2: 不正署名（ランダムバイト） → 401
# ケース 3: 異なる KMS キーで署名 → 401
```

---

## Phase F: ドキュメント

### F-1: `infra/e2e-demo/crosscloud/docs/auth-comparison.md`

```markdown
# CrossCloud 認証方式比較

## HMAC-SHA256（v15.1.0）vs ECDSA P-256 / KMS（v15.1.5）

...（比較表・トレードオフ・ユースケースガイダンス）
```

---

## Phase G: v15.1.0 bootstrap production 化

### G-1: `lambda/verifier/bootstrap` のデバッグログ削除

v15.1.0 では E2E デバッグのために base64 ログを仕込んでいたが、production では不要。

変更前:
```sh
echo "[DEBUG] EXIT_CODE=${EXIT_CODE}"
echo "[DEBUG] OUTPUT_B64=$(printf '%s' "$OUTPUT" | base64 | tr -d '\n')"
```

変更後: 削除（CloudWatch Logs への不要な出力を減らす）

---

## Phase H: ECR / Lambda デプロイ・E2E

### H-1: `scripts/build-and-push-verifier.sh` を verifier_v2 対応に更新

v15.1.0 と同じ Docker multi-stage ビルドフロー（`Dockerfile.builder` 流用）。
注意点: `--no-cache` を明示してキャッシュ誤使用を防ぐ。

### H-2: `terraform apply` → Cognito ユーザー作成 → E2E スクリプト実行

v15.1.0 の terraform に KMS キーリソースを追加した状態で apply。

### H-3: `terraform destroy`（E2E 完了後）

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version → 15.1.5 |
| `fav/Cargo.lock` | 自動更新 |
| `fav/src/backend/vm.rs` | `Crypto.ecdsa_verify_raw` + `AWS.kms_get_public_key_raw` primitive 追加 |
| `fav/src/driver.rs` | `v15150_tests` モジュール追加、`is_result_err_value` ヘルパー追加 |
| `fav/src/middle/checker.rs` | `ecdsa_verify_raw` / `kms_get_public_key_raw` を builtin_ret_ty に追加 |
| `runes/crosscloud/` (新規) | verifier_v2 用 helper rune（オプション） |
| `infra/.../terraform/aws/auth.tf` | `aws_kms_key` + `aws_kms_alias` 追加、Lambda IAM に `kms:GetPublicKey` 追加 |
| `infra/.../terraform/aws/outputs.tf` | KMS key ARN / alias output 追加 |
| `infra/.../lambda/verifier/bootstrap` | デバッグログ削除（production 化） |
| `infra/.../lambda/verifier_v2/` (新規) | verifier_v2.fav + Dockerfile + bootstrap |
| `infra/.../scripts/run_with_kms.sh` (新規) | KMS 署名スクリプト |
| `infra/.../scripts/reject_kms.sh` (新規) | KMS 検証ケーススクリプト |
| `infra/.../docs/auth-comparison.md` (新規) | HMAC vs KMS 比較ドキュメント |

---

## 新規 Cargo 依存

| Crate | 用途 | 追加理由 |
|---|---|---|
| `p256 = { version = "0.13", features = ["ecdsa", "pem"] }` | ECDSA P-256 署名検証 + PEM 変換 | 純 Rust、依存チェーン小 |

---

## 実装順序

```
A（バージョン）→ B（テスト）→ C（KMS Terraform）
→ D（vm.rs primitive + verifier_v2.fav）→ E（スクリプト）
→ F（ドキュメント）→ G（bootstrap 整理）→ H（E2E）
```

D の `vm.rs` 変更が最もリスクがある（新 primitive）ため、
先に `cargo test` で既存テストが全パスすることを確認してから E 以降に進む。
