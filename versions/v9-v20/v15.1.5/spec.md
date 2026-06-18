# v15.1.5 Spec — CrossCloud 認証層 セキュア版（KMS 非対称署名）

Date: 2026-06-13
Branch: master（予定）

---

## テーマ

v15.1.0 の HMAC（対称暗号）を **AWS KMS の非対称署名（ECDSA P-256）** に置き換える。
加えて、v15.1.0 E2E デバッグで露見した実装上の問題点を修正・テストで固める。

---

## スコープ

### A: KMS 非対称署名（ロードマップ由来）

#### 背景

v15.1.0 の HMAC 方式は秘密鍵を両側（署名者 + 検証者）が保持する必要があり、
鍵漏洩リスクと鍵ローテーション時の同時更新コストという問題がある。

KMS 非対称署名では秘密鍵が KMS の外に出ず、署名者は KMS API 経由でのみ署名できる。
検証者は公開鍵のみ保持すればよい。

#### HMAC vs KMS 比較

| 項目 | v15.1.0 HMAC | v15.1.5 KMS (ECDSA P-256) |
|---|---|---|
| 秘密鍵の所在 | Secrets Manager（両側が知る） | KMS 内（外に出ない） |
| 検証側が必要なもの | 秘密鍵（共有） | 公開鍵のみ |
| 署名偽造のリスク | 秘密鍵漏洩で偽造可能 | KMS 外に秘密鍵が出ないため不可 |
| 鍵ローテーション | 両側同時更新が必要 | KMS 側だけ。公開鍵の再取得のみ |
| 実装複雑度 | 低 | 中（KMS API + ECDSA 検証） |

#### 署名フロー

```
[署名者: scripts/run_with_kms.sh]
  1. StringToSign を構築（v15.1.0 と同形式）
  2. aws kms sign \
       --key-id alias/crosscloud-signer \
       --signing-algorithm ECDSA_SHA_256 \
       --message-type RAW \
       --message file://<(echo -n "$STRING_TO_SIGN") \
     → DER エンコード署名バイト列 → base64
  3. X-Signature: <base64(DER署名)>
     X-KMS-Key-Id: alias/crosscloud-signer
     でリクエストを送信

[Lambda verifier_v2]
  4. X-KMS-Key-Id から kms:GetPublicKey → DER → PEM 変換
     （公開鍵はメモリキャッシュ、Lambda warm start に有効）
  5. StringToSign を再構築
  6. ECDSA 署名をローカル検証（Python cryptography ライブラリ）
  7. タイムスタンプ・nonce・Cognito JWT 検証は v15.1.0 と同じ
```

### B: v15.1.0 デバッグで判明した問題の修正・テスト化

v15.1.0 E2E で3つのバグが重なり、根本原因の特定に時間を要した（詳細: `debug-log.md`）。
再発防止のためテストを追加し、bootstrap のデバッグログを production 仕様に戻す。

| 問題 | v15.1.0 での対処 | v15.1.5 での追加対応 |
|---|---|---|
| `fav run --legacy` が `Result.err` で exit 0 | driver.rs 修正 | 専用リグレッションテスト追加 |
| `AWS_CONFIG` が `default()` で初期化 | vm.rs 修正 | — |
| `aws_post` がエラーボディを返さない | vm.rs 修正 | — |
| bootstrap にデバッグ用 base64 ログが残存 | — | production 仕様に戻す |

---

## 完了条件

1. `cargo test v15150` → 全テストパス
2. `cargo test` → リグレッションなし（1550+ パス）
3. `Cargo.toml version == "15.1.5"`
4. `scripts/run_with_kms.sh` が動作し valid request → 200
5. 改ざんリクエスト → 401（ECDSA 検証失敗）
6. `infra/e2e-demo/crosscloud/docs/auth-comparison.md` が存在する
7. bootstrap がデバッグログなしで動作する

---

## 既知の制約・スコープ外

- Azure Function が KMS `Sign` を呼ぶ統合は対象外（run_with_kms.sh で代替）
- v15.2.0 以降の GCP BigQuery / Kafka / fav deploy は本バージョンのスコープ外
- KMS E2E は費用節約のため terraform destroy を E2E 完了後に実施

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v15.1.5 セクション
- `versions/v15.1.0/debug-log.md` — デバッグ記録
- `versions/v15.1.0/architecture.md` — v15.1.0 アーキテクチャ
