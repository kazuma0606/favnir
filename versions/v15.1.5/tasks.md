# v15.1.5 Tasks — CrossCloud 認証層 セキュア版（KMS 非対称署名）

Date: 2026-06-13
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"15.1.5"` に変更

---

## Phase B — リグレッションテスト（v15.1.0 デバッグ教訓）

- [x] B-1: `fav/src/driver.rs` に `v15150_tests` モジュール追加（6テスト）

- [x] B-2: `driver.rs` に `is_result_err_value(v: &Value) -> bool` ヘルパー追加
  - `pub(crate) fn is_result_err_value(v: &Value) -> bool`
  - `cmd_run` は `if let Value::Variant(ref tag, ...)` の代わりにこれを使用

- [x] B-3: `cargo test v15150` → 6/6 パス確認

---

## Phase C — KMS Terraform リソース

- [x] C-1: `infra/e2e-demo/crosscloud/terraform/aws/auth.tf` に追記
  - `aws_kms_key` (crosscloud_signer, ECC_NIST_P256, SIGN_VERIFY)
  - `aws_kms_alias` (alias/crosscloud-signer)
  - Lambda IAM ポリシーに `kms:GetPublicKey` 権限追加

- [x] C-2: `infra/e2e-demo/crosscloud/terraform/aws/outputs.tf` に追記
  - `kms_key_arn` / `kms_key_alias` output

---

## Phase D — vm.rs / checker.rs 新 Primitive 追加

- [x] D-1: `fav/Cargo.toml` に `p256 = { version = "0.13", features = ["ecdsa", "pem"] }` 追加

- [x] D-2: `fav/src/backend/vm.rs` に `Crypto.ecdsa_verify_raw` primitive 追加
  - p256::ecdsa::VerifyingKey::from_public_key_pem + Signature::from_der で実装
  - 検証成功 → `ok(Unit)`, 失敗 → `err("ecdsa_verify_failed")`

- [x] D-3: `fav/src/backend/vm.rs` に `AWS.kms_get_public_key_raw` primitive 追加
  - KMS TrentService.GetPublicKey（SigV4）→ PublicKey フィールド抽出 → PEM 変換

- [x] D-4: `fav/src/middle/checker.rs` の `builtin_ret_ty` に追記完了

- [x] D-5: `cargo test` → 1556 + 705 pass / 0 fail / リグレッションなし

---

## Phase E — verifier_v2.fav + Lambda

- [x] E-1: `infra/e2e-demo/crosscloud/lambda/verifier_v2/verifier_v2.fav` 作成
  - get_kms_public_key / verify_ecdsa 実装
  - VERIFY_KMS_KEY_ID env var から KMS key ID を読み取り

- [x] E-2: `infra/e2e-demo/crosscloud/lambda/verifier_v2/Dockerfile` 作成

- [x] E-3: `infra/e2e-demo/crosscloud/lambda/verifier_v2/bootstrap` 作成
  - X-KMS-Key-Id → VERIFY_KMS_KEY_ID 追加
  - デバッグログなし（production 仕様）

- [x] E-4: v15.1.0 `lambda/verifier/bootstrap` のデバッグログ削除（前コミット済み）

---

## Phase F — スクリプト

- [x] F-1: `infra/e2e-demo/crosscloud/scripts/run_with_kms.sh` 作成
- [x] F-2: `infra/e2e-demo/crosscloud/scripts/reject_kms.sh` 作成（PASS=2 FAIL=0 期待）

---

## Phase G — ドキュメント

- [x] G-1: `infra/e2e-demo/crosscloud/docs/` ディレクトリ作成

- [x] G-2: `infra/e2e-demo/crosscloud/docs/auth-comparison.md` 作成
  - HMAC vs ECDSA/KMS 比較表・トレードオフ・本番ガイダンス記載

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

- [x] I-1: コミット完了（9c1872c）
  - `feat: v15.1.5 — CrossCloud KMS 非対称署名（ECDSA P-256）+ リグレッションテスト`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.1.5"` | [x] |
| `cargo test v15150` 全テストパス（6/6） | [x] |
| `cargo test` 全件パス（リグレッションなし、1556 pass）| [x] |
| `Crypto.ecdsa_verify_raw` primitive が vm.rs に存在する | [x] |
| `AWS.kms_get_public_key_raw` primitive が vm.rs に存在する | [x] |
| `auth.tf` に `ECC_NIST_P256` が含まれる | [x] |
| `lambda/verifier_v2/verifier_v2.fav` が存在する | [x] |
| `docs/auth-comparison.md` が存在する | [x] |
| `scripts/reject_kms.sh` が PASS=2 FAIL=0 を出力する（要 AWS 環境）| [ ] 未実施 |
| `run_with_kms.sh` が HTTP 200 を返す（要 AWS 環境）| [ ] 未実施 |
| S3 に auth-proof が保存される（要 AWS 環境）| [ ] 未実施 |
| `lambda/verifier/bootstrap` からデバッグログが削除されている | [x] |
| terraform destroy 完了 | [ ] Phase H（AWS 環境）実施後 |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.1.5/spec.md` | 仕様・スコープ |
| `versions/v15.1.5/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/v15.1.0/debug-log.md` | デバッグ記録（教訓） |
| `versions/v15.1.0/architecture.md` | v15.1.0 アーキテクチャ（ベース） |
| `versions/roadmap-v15.1-v16.0.md` | v15.1.5 セクション |
