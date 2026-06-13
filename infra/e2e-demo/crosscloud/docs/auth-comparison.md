# CrossCloud 認証方式比較 — HMAC vs ECDSA/KMS

## 概要

CrossCloud 認証層の v15.1.0（HMAC-SHA256）と v15.1.5（ECDSA P-256 / AWS KMS）の比較。

---

## 比較表

| 項目 | v15.1.0: HMAC-SHA256 | v15.1.5: ECDSA P-256 / KMS |
|---|---|---|
| **秘密鍵の所在** | Secrets Manager（署名者・検証者が共有） | KMS 内（外部に出ない） |
| **検証側が必要なもの** | 秘密鍵（共有） | 公開鍵のみ |
| **署名偽造リスク** | 秘密鍵漏洩で偽造可能 | KMS 外に秘密鍵が出ないため不可 |
| **鍵ローテーション** | 署名者・検証者を同時に更新 | KMS 側のみ。公開鍵を再取得するだけ |
| **署名アルゴリズム** | HMAC-SHA256（対称） | ECDSA P-256（非対称） |
| **署名出力形式** | hex string | DER バイト列 → base64 |
| **AWS サービス依存** | Secrets Manager | KMS |
| **Lambda IAM 権限** | `secretsmanager:GetSecretValue` | `kms:GetPublicKey` |
| **署名者の権限** | Secrets Manager 読み取り | `kms:Sign` |
| **実装複雑度** | 低（openssl dgst 1コマンド） | 中（aws kms sign + DER/PEM 変換） |
| **コスト** | $0.40/10万 API calls（Secrets Manager） | $0.03/1万 API calls（KMS）+ $1/キー/月 |
| **Cold start 影響** | GetSecretValue 呼び出しあり | GetPublicKey 呼び出しあり（warm start でキャッシュ可） |

---

## 署名フロー比較

### v15.1.0: HMAC-SHA256

```
[署名者]
  1. Secrets Manager から HMAC シークレット取得
  2. StringToSign 構築（Method\nPath\nTimestamp\nNonce\nSHA256(Body)）
  3. HMAC-SHA256(secret, StringToSign) → hex → X-Signature ヘッダー

[Lambda verifier.fav]
  4. Secrets Manager から同じシークレット取得
  5. StringToSign を再構築
  6. HMAC 再計算 → ヘッダーの値と比較
```

### v15.1.5: ECDSA P-256 / KMS

```
[署名者]
  1. StringToSign 構築（同形式）
  2. aws kms sign --signing-algorithm ECDSA_SHA_256 --message-type RAW
     → DER エンコード署名バイト列 → base64 → X-Signature ヘッダー
  3. X-KMS-Key-Id ヘッダーにキー ID を付与

[Lambda verifier_v2.fav]
  4. kms:GetPublicKey → DER → PEM 変換（warm start でキャッシュ可）
  5. StringToSign を再構築
  6. ECDSA P-256 ローカル検証（p256 crate）
```

---

## Favnir primitive 対応

| 操作 | v15.1.0 | v15.1.5 |
|---|---|---|
| シークレット/鍵取得 | `AWS.secrets_get_raw(region, arn)` | `AWS.kms_get_public_key_raw(region, key_id)` |
| 署名検証 | `Crypto.hmac_sha256_raw(secret, data)` + 比較 | `Crypto.ecdsa_verify_raw(pem, message, sig_b64)` |

---

## トレードオフと選択ガイダンス

### HMAC-SHA256（v15.1.0）を選ぶケース

- **シンプルさ優先**: 実装・デバッグが容易
- **社内システム**: 署名者と検証者が同一組織・同一 AWS アカウント内
- **低トラフィック**: KMS 呼び出しコストを抑えたい
- **既存 HMAC インフラとの統合**: openssl / boto3 など広くサポートされている

### ECDSA P-256 / KMS（v15.1.5）を選ぶケース

- **マルチクラウド・ゼロトラスト**: 秘密鍵を検証者に渡したくない
- **クロス組織**: 署名者と検証者が異なる組織・異なるクラウド
- **コンプライアンス要件**: 秘密鍵の HSM 内保管が必要（PCI DSS / SOC2 等）
- **鍵ローテーションの簡素化**: 検証者側の更新なしに署名者だけローテーション可能

### 本番での推奨

- **同一 AWS アカウント内のサービス間**: HMAC でも十分だが、KMS のほうが監査ログ（CloudTrail）が充実
- **マルチクラウド / B2B 統合 / 高セキュリティ要件**: **ECDSA P-256 / KMS を推奨**

---

## 実装ファイル

| バージョン | Favnir ソース | bootstrap | Terraform |
|---|---|---|---|
| v15.1.0 (HMAC) | `lambda/verifier/verifier.fav` | `lambda/verifier/bootstrap` | `auth.tf`（Secrets Manager + Lambda） |
| v15.1.5 (ECDSA) | `lambda/verifier_v2/verifier_v2.fav` | `lambda/verifier_v2/bootstrap` | `auth.tf`（KMS + Lambda verifier_v2） |

---

## 参照

- `versions/v15.1.0/architecture.md` — v15.1.0 完成アーキテクチャ
- `versions/v15.1.5/spec.md` — v15.1.5 仕様
- `scripts/run_with_auth.sh` — HMAC 署名リクエスト送信スクリプト
- `scripts/run_with_kms.sh` — KMS ECDSA 署名リクエスト送信スクリプト
