# v15.1.5 デバッグ記録

Date: 2026-06-14

---

## 詰まった点 1: PEM base64 の行折り返し

### 症状

`reject_kms.sh` ケース 1（改ざんボディ）は 401 になるのに、ケース 2（ランダム署名）が 500 を返す。

### 原因

`AWS.kms_get_public_key_raw` の初期実装で、KMS `GetPublicKey` レスポンスの `PublicKey` フィールド（base64 DER）をそのまま PEM ヘッダーで囲んでいた。

```rust
// NG: base64 を改行なしで埋め込む
let pem = format!("-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n", der_b64.trim());
```

p256 クレートの `VerifyingKey::from_public_key_pem` は RFC 7468 に準拠しており、base64 は **64 文字ごとに改行**が必要。改行なしの場合 `PEM Base64 error: invalid Base64 encoding` が発生する。

改ざんボディのケース（ケース 1）では正当な署名 DER が送られてくるため `Signature::from_der` まで到達して検証に失敗し `err("ecdsa_verify_failed")` → 401 を返せていた。ランダム署名（ケース 2）では KMS 公開鍵の取得自体が PEM エラーで失敗するため `?` 演算子でエラーが伝播し、そのエラー文字列が `"invalid_signature|ecdsa_verify_failed"` にマッチせず 500 になっていた。

### 修正

```rust
let b64_clean: String = der_b64.trim().chars().filter(|c| !c.is_whitespace()).collect();
let b64_wrapped: String = b64_clean.as_bytes().chunks(64)
    .map(|c| std::str::from_utf8(c).unwrap_or(""))
    .collect::<Vec<_>>()
    .join("\n");
let pem = format!("-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n", b64_wrapped);
```

### 診断手順

bootstrap に一時デバッグ echo を追加して CloudWatch Logs で確認した。

```sh
# 追加（status=500 の else ブランチ）
echo "[DEBUG OUTPUT] $(printf '%s' "$OUTPUT" | tr -d '\033\r' | tr '\n' '|')"
```

ログに `PEM Base64 error: invalid Base64 encoding` が出ており、PEM 構築の問題と特定できた。確認後に削除。

---

## 詰まった点 2: DER parse 失敗が 500 になる

### 症状

PEM 修正後も `reject_kms.sh` ケース 2（ランダム 64 バイト base64 署名）が 500 を返す。

### 原因

`Signature::from_der` がランダムバイト列を DER 署名として解析しようとして失敗する際、`?` 演算子でエラーが伝播する。エラー文字列は `"Crypto.ecdsa_verify_raw: parse DER sig: ..."` という形式になるため、bootstrap の `grep -q "invalid_signature\|ecdsa_verify_failed"` にマッチせず、else ブランチ（STATUS=500）に落ちていた。

```rust
// NG: ? でエラーを伝播すると bootstrap が 401 に変換できない
let sig = Signature::from_der(&sig_bytes)
    .map_err(|e| format!("Crypto.ecdsa_verify_raw: parse DER sig: {e}"))?;
```

### 修正

PEM parse・DER parse の両方で `match` を使い、失敗時は明示的に `err("ecdsa_verify_failed")` を返す。

```rust
let verifying_key = match VerifyingKey::from_public_key_pem(pem.trim()) {
    Ok(k) => k,
    Err(_) => return Ok(VMValue::Variant(
        "err".into(),
        Some(Box::new(VMValue::Str("ecdsa_verify_failed".into()))),
    )),
};

let sig = match Signature::from_der(&sig_bytes) {
    Ok(s) => s,
    Err(_) => return Ok(VMValue::Variant(
        "err".into(),
        Some(Box::new(VMValue::Str("ecdsa_verify_failed".into()))),
    )),
};
```

### 教訓

Lambda 側の Favnir コードが `err("...")` を返せば bootstrap がエラー種別を判断できる。しかし VM Primitive が `?` でエラーを伝播すると bootstrap には素の Rust エラー文字列が届き、grep でのハンドリングが壊れる。**署名・認証系 primitive は全て `match` で明示的に `err("固定キー")` を返すべき。**

---

## 詰まった点 3: Docker build --no-cache の必要性

### 症状

`vm.rs` を修正したにもかかわらず、Lambda の動作が変わらない。

### 原因

`docker build` はレイヤーキャッシュを利用するため、`COPY` や `RUN cargo build` のレイヤーがキャッシュされていると Rust ソースが変わっても再コンパイルされない（"CACHED" と表示されビルドが約 40 秒で完了するが、実際には古いバイナリが使われる）。

### 修正

Rust ソースを変更した後の `Dockerfile.builder` ビルドは必ず `--no-cache` を付ける。

```bash
docker build --no-cache -f fav/Dockerfile.builder --tag fav-builder:latest fav/
```

---

## 詰まった点 4: docker cp のパスが違う

### 症状

`docker cp fav-builder-tmp:/usr/local/bin/fav` が失敗する。

### 原因

`Dockerfile.builder` の `WORKDIR` は `/build` であり、`cargo build --release` の成果物は `/build/target/release/fav` に置かれる。`/usr/local/bin/fav` は存在しない。

### 修正

```bash
docker cp fav-builder-tmp:/build/target/release/fav /tmp/fav_linux
```

---

## 詰まった点 5: terraform apply を 2 回実行する必要がある

### 症状

`terraform apply` で Lambda 関数の作成に失敗する（`image_uri` が参照する ECR リポジトリのイメージが存在しない）。

### 原因

Lambda の `image_uri` は ECR リポジトリのイメージを参照するが、ECR リポジトリ作成直後はイメージが存在しない。Terraform は `aws_ecr_repository` と `aws_lambda_function` を同一 apply で作ろうとするが、Lambda はイメージ存在を検証するため失敗する。

### 正しい手順

1. `terraform apply`（ECR リポジトリのみ作成、Lambda は失敗してもよい）
2. `docker buildx build --push` で ECR にイメージを push
3. `terraform apply` 再実行（Lambda 作成が成功する）

---

## 詰まった点 6: terraform destroy に -var が必要

### 症状

`terraform destroy -auto-approve` が `No value for required variable` で失敗する。

### 原因

`variables.tf` に `hmac_secret`・`azure_storage_key`・`azure_storage_account`・`azure_conn_str`・`azure_container`・`rds_password` の 6 変数が必須（デフォルトなし）として定義されている。destroy でも変数の評価が必要。

### 修正

```bash
terraform destroy -auto-approve \
  -var="azure_storage_key=dummy" \
  -var="hmac_secret=dummy" \
  -var="azure_storage_account=dummy" \
  -var="azure_conn_str=dummy" \
  -var="azure_container=dummy" \
  -var="rds_password=dummy"
```

実際のリソース削除には変数の値は使われないため dummy で問題ない。

---

## 詰まった点 7: clippy -D warnings（doc comment / unused import）

### 症状

コミット直後に pre-commit フックが clippy エラーを検出。

1. `vm.rs` — `AWS.kms_get_public_key_raw` 内の `use base64::Engine;` が未使用（同スコープ外で既に `use` 済み）
2. `driver.rs` — `is_result_err_value` と `cmd_run` の doc comment が混在し、リスト項目のインデントが不正（`///   - item` 形式は clippy `doc_list_item_without_indentation` を起こす）

### 修正

- `use base64::Engine;` を削除
- doc comment を関数ごとに分離し、リスト形式を `/// \n/// - item` の標準形式に統一
