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

- [x] H-1: `Dockerfile.builder` でLinux バイナリをビルド（`--no-cache` 付き）
  - PEM base64 行折り返し修正後に再ビルド（`--no-cache`）実施
  - DER parse 失敗時も `err("ecdsa_verify_failed")` を返すよう修正後に再ビルド

- [x] H-2: ECR ログイン → crosscloud-verifier-v2 リポジトリ作成（terraform apply 済み）
  - `847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/crosscloud-verifier-v2`

- [x] H-3: verifier_v2 イメージをビルド・ECR push
  - `docker buildx build --platform linux/amd64 --provenance=false --push`
  - マニフェスト: `sha256:095eab676122d7e225c822d24db82ccf439eda971cfef141ee80417be4b4220e`

- [x] H-4: `terraform apply` (KMS リソース + Lambda verifier_v2 追加分)
  - KMS key ARN: `arn:aws:kms:ap-northeast-1:847333136058:key/5548aced-0bc4-4863-8615-be58bd6d3bb6`
  - API endpoint: `https://3e5ithnoj7.execute-api.ap-northeast-1.amazonaws.com`
  - Cognito: user pool `ap-northeast-1_aTkU4j9ez`, client `37d18bg84udnel648nph430915`

- [x] H-5: `reject_kms.sh` 実行 → **PASS=2 FAIL=0**
  - ケース1: 改ざんボディ → 401 [PASS]
  - ケース2: ランダム署名（不正 DER）→ 401 [PASS]

- [x] H-6: `run_with_kms.sh` 実行 → **HTTP 200**
  - KMS 署名検証通過 / ECS タスク起動確認
  - ECS task ARN: `arn:aws:ecs:ap-northeast-1:847333136058:task/favnir-crosscloud/b942d15df51e41b5a5cb6d7d255a2b93`

- [x] H-7: S3 証跡確認
  - `s3://favnir-crosscloud-proof-dev/auth-proof/327c9085-cd05-481f-b6d5-83c88f6bf9dd.json`
  - `{"status":"ok","request_id":"327c9085-cd05-481f-b6d5-83c88f6bf9dd","task_arn":"arn:aws:ecs:ap-northeast-1:847333136058:task/favnir-crosscloud/b942d15df51e41b5a5cb6d7d255a2b93"}`

- [x] H-8: `terraform destroy` 完了

---

## Phase I — コミット

- [x] I-1: Phase A-G コミット完了（9c1872c）
  - `feat: v15.1.5 — CrossCloud KMS 非対称署名（ECDSA P-256）+ リグレッションテスト`

- [x] I-2: Phase H（E2E）完了コミット

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
| `scripts/reject_kms.sh` が PASS=2 FAIL=0 を出力する（要 AWS 環境）| [x] PASS=2 FAIL=0 |
| `run_with_kms.sh` が HTTP 200 を返す（要 AWS 環境）| [x] HTTP 200 |
| S3 に auth-proof が保存される（要 AWS 環境）| [x] 327c9085-cd05-481f-b6d5-83c88f6bf9dd.json |
| `lambda/verifier/bootstrap` からデバッグログが削除されている | [x] |
| terraform destroy 完了 | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.1.5/spec.md` | 仕様・スコープ |
| `versions/v15.1.5/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/v15.1.0/debug-log.md` | デバッグ記録（教訓） |
| `versions/v15.1.0/architecture.md` | v15.1.0 アーキテクチャ（ベース） |
| `versions/roadmap-v15.1-v16.0.md` | v15.1.5 セクション |
