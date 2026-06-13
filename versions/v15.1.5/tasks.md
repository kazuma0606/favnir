# v15.1.5 Tasks — CrossCloud 認証層 セキュア版（KMS 非対称署名）

Date: 2026-06-13
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"15.1.5"` に変更

---

## Phase B — リグレッションテスト（v15.1.0 デバッグ教訓）

- [ ] B-1: `fav/src/driver.rs` に `v15150_tests` モジュール追加

  ```rust
  #[cfg(test)]
  mod v15150_tests {
      fn version_is_15_1_5()
      fn legacy_run_result_err_triggers_exit_path()
      fn legacy_run_result_ok_does_not_trigger_exit_path()
      fn crosscloud_kms_terraform_has_ecc_key()
      fn crosscloud_verifier_v2_exists()
      fn crosscloud_auth_comparison_doc_exists()
  }
  ```

- [ ] B-2: `driver.rs` に `is_result_err_value(v: &Value) -> bool` ヘルパー追加
  - テスト可能な形で Result.err 判定を分離
  - `cmd_run` の `if let Value::Variant(...)` を本ヘルパーに委譲

- [ ] B-3: `cargo test v15150` → 全テストパス確認

---

## Phase C — KMS Terraform リソース

- [ ] C-1: `infra/e2e-demo/crosscloud/terraform/aws/auth.tf` に追記
  - `aws_kms_key` (crosscloud_signer, ECC_NIST_P256, SIGN_VERIFY)
  - `aws_kms_alias` (alias/crosscloud-signer)
  - Lambda IAM ポリシーに `kms:GetPublicKey` 権限追加

- [ ] C-2: `infra/e2e-demo/crosscloud/terraform/aws/outputs.tf` に追記
  - `kms_key_arn` / `kms_key_alias` output

---

## Phase D — vm.rs / checker.rs 新 Primitive 追加

- [ ] D-1: `fav/Cargo.toml` に `p256` crate 追加
  ```toml
  p256 = { version = "0.13", features = ["ecdsa", "pem"] }
  ```

- [ ] D-2: `fav/src/backend/vm.rs` に `Crypto.ecdsa_verify_raw` primitive 追加
  - 引数: `(pub_key_pem: String, message: String, sig_der_b64: String) -> Result<Unit, String>`
  - `p256::ecdsa::VerifyingKey::from_public_key_pem` + `p256::ecdsa::Signature::from_der`
  - 検証成功 → `ok(())`, 失敗 → `err("ecdsa_verify_failed")`

- [ ] D-3: `fav/src/backend/vm.rs` に `AWS.kms_get_public_key_raw` primitive 追加
  - 引数: `(region: String, key_id: String) -> Result<String, String>`
  - KMS GetPublicKey API（SigV4）→ DER → PEM 変換して返す

- [ ] D-4: `fav/src/middle/checker.rs` の `builtin_ret_ty` に追記
  - `"Crypto.ecdsa_verify_raw"` → `Result<Unit, String>`（!Auth エフェクト）
  - `"AWS.kms_get_public_key_raw"` → `Result<String, String>`（!AWS エフェクト）

- [ ] D-5: `cargo test` → 全テストパス確認（リグレッションなし）

---

## Phase E — verifier_v2.fav + Lambda

- [ ] E-1: `infra/e2e-demo/crosscloud/lambda/verifier_v2/verifier_v2.fav` 作成
  - v15.1.0 `verifier.fav` の `get_secret` / `verify_hmac` を KMS 版に置き換え
  - `get_kms_public_key(region, key_id) -> Result<String, String> !AWS`
  - `verify_ecdsa(pub_key_pem, sts, sig_b64) -> Result<Unit, String> !Auth`
  - env var 追加: `VERIFY_KMS_KEY_ID`, `KMS_KEY_ARN`

- [ ] E-2: `infra/e2e-demo/crosscloud/lambda/verifier_v2/Dockerfile` 作成
  - ベース: `public.ecr.aws/lambda/provided:al2023`
  - v15.1.0 `Dockerfile` を元に `verifier_v2.fav` を参照

- [ ] E-3: `infra/e2e-demo/crosscloud/lambda/verifier_v2/bootstrap` 作成
  - v15.1.0 `bootstrap` をベースに:
    1. `X-KMS-Key-Id` ヘッダーを `VERIFY_KMS_KEY_ID` env var に追加
    2. デバッグ base64 ログを**削除**（production 仕様）

- [ ] E-4: v15.1.0 `lambda/verifier/bootstrap` のデバッグログ削除
  - `echo "[DEBUG] EXIT_CODE=..."` の行を削除
  - `echo "[DEBUG] OUTPUT_B64=..."` の行を削除

---

## Phase F — スクリプト

- [ ] F-1: `infra/e2e-demo/crosscloud/scripts/run_with_kms.sh` 作成
  - Cognito 認証 → StringToSign 構築 → `aws kms sign` → API POST
  - `--query "Signature" --output text` で base64 署名を取得

- [ ] F-2: `infra/e2e-demo/crosscloud/scripts/reject_kms.sh` 作成
  - ケース 1: 改ざんボディ（ECDSA 検証失敗）→ 401 期待
  - ケース 2: ランダム署名（不正 DER バイト列）→ 401 期待
  - PASS=2 FAIL=0 を確認

---

## Phase G — ドキュメント

- [ ] G-1: `infra/e2e-demo/crosscloud/docs/` ディレクトリ作成

- [ ] G-2: `infra/e2e-demo/crosscloud/docs/auth-comparison.md` 作成
  - HMAC vs ECDSA/KMS の比較表（ロードマップ記載の内容）
  - それぞれのユースケース・トレードオフ
  - 「どちらを本番で使うべきか」のガイダンス

---

## Phase H — ECR / Lambda デプロイ・E2E

- [ ] H-1: `Dockerfile.builder` でLinux バイナリをビルド（`--no-cache` 付き）
  ```bash
  docker build --no-cache -f fav/Dockerfile.builder --tag fav-builder:latest fav/
  ```
  ※ `--no-cache` を付けないと変更が反映されないことがある

- [ ] H-2: ECR ログイン → crosscloud-verifier-v2 リポジトリ作成（terraform apply 後）

- [ ] H-3: verifier_v2 イメージをビルド・ECR push
  ```bash
  docker buildx build --platform linux/amd64 --provenance=false \
    -t <ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/crosscloud-verifier-v2:latest \
    --push lambda/verifier_v2/
  ```

- [ ] H-4: `terraform apply` (KMS リソース + Lambda verifier_v2 追加分)
  - Cognito ユーザー再作成（terraform destroy 済みのため）
  - `aws cognito-idp admin-set-user-password` で permanent パスワード設定

- [ ] H-5: `reject_kms.sh` 実行 → PASS=2 FAIL=0 確認

- [ ] H-6: `run_with_kms.sh` 実行 → HTTP 200 + ECS タスク起動確認

- [ ] H-7: S3 証跡確認
  ```bash
  aws s3 ls s3://<proof-bucket>/auth-proof/
  ```

- [ ] H-8: `terraform destroy`（E2E 完了後）

---

## Phase I — コミット

- [ ] I-1: `git commit -m "feat: v15.1.5 — CrossCloud KMS 非対称署名（ECDSA P-256）+ リグレッションテスト"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.1.5"` | [ ] |
| `cargo test v15150` 全テストパス | [ ] |
| `cargo test` 全件パス（リグレッションなし、1550+ パス）| [ ] |
| `Crypto.ecdsa_verify_raw` primitive が vm.rs に存在する | [ ] |
| `AWS.kms_get_public_key_raw` primitive が vm.rs に存在する | [ ] |
| `auth.tf` に `ECC_NIST_P256` が含まれる | [ ] |
| `lambda/verifier_v2/verifier_v2.fav` が存在する | [ ] |
| `docs/auth-comparison.md` が存在する | [ ] |
| `scripts/reject_kms.sh` が PASS=2 FAIL=0 を出力する（要 AWS 環境）| [ ] |
| `run_with_kms.sh` が HTTP 200 を返す（要 AWS 環境）| [ ] |
| S3 に auth-proof が保存される（要 AWS 環境）| [ ] |
| `lambda/verifier/bootstrap` からデバッグログが削除されている | [ ] |
| terraform destroy 完了 | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.1.5/spec.md` | 仕様・スコープ |
| `versions/v15.1.5/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/v15.1.0/debug-log.md` | デバッグ記録（教訓） |
| `versions/v15.1.0/architecture.md` | v15.1.0 アーキテクチャ（ベース） |
| `versions/roadmap-v15.1-v16.0.md` | v15.1.5 セクション |
