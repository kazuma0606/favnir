# v15.1.0 実装デバッグログ — 詰まった箇所の記録

Date: 2026-06-13

## 概要

v15.1.0 の Lambda E2E（reject_cases.sh PASS=5 FAIL=0）達成まで、
3つの独立したバグが重なり合い、原因特定に時間を要した。

---

## Bug 1: Lambda スレッドパニック（EAGAIN）

### 症状
```
thread 'main' panicked at src/main.rs:241:10:
failed to spawn main thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
```

### 原因
`main()` が `SELF_HOST_STACK_SIZE = 256MB` のスタックでスレッドをスポーンしようとした。
Lambda の仮想スタック制限（128MB メモリ設定）では EAGAIN になる。

### 修正
```rust
// fav/src/main.rs
let stack_size = if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
    8 * 1024 * 1024  // 8MB for Lambda
} else {
    SELF_HOST_STACK_SIZE
};
```

### 教訓
Lambda コンテナでは仮想スタックサイズに制限がある。`AWS_LAMBDA_FUNCTION_NAME` で検出できる。

---

## Bug 2: `!Auth` エフェクト未宣言

### 症状
```
E0311: function uses !Auth effect but declaration does not declare it
```

### 原因
`verifier.fav` の `build_sts` / `verify_hmac` が `Crypto.*` を呼ぶが、
関数シグネチャに `!Auth` を宣言していなかった。

```fav
// Before (error)
fn build_sts(...) -> String {

// After (fixed)
fn build_sts(...) -> String !Auth {
```

### 教訓
`!Auth` は `Crypto.*` 系のすべての関数シグネチャに必要。
呼び出し元チェーンをさかのぼって全関数に追加が必要。

---

## Bug 3: `fav run --legacy` が `Result.err` で exit 0 を返す（最も時間がかかった）

### 症状
- Lambda が常に HTTP 200 を返す（invalid_signature でも）
- CloudWatch ログ: `EXIT_CODE=0`、OUTPUT は `[1/5] HMAC シークレット取得中...` のみ

### なぜ発見が遅れたか
Bug 4（後述）が同時に発生していたため、
「get_secret が失敗 → chain escape → Result.err → exit 1」のはずが
「Result.err → exit 0（バグ）」になっており、bootstrap は常に成功扱いしていた。

exit code 0 なのに stdout が途中で止まっている、という矛盾から
「exit code のバグ」に気づくまでに base64 デバッグログを仕込む工程が必要だった。

### 原因
```rust
// driver.rs の cmd_run（--legacy パス）
exec_artifact_main_with_source(&artifact, db_url, Some(&source_path2))
    .unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
// ← unwrap_or_else は Rust レベルの Err のみを処理
// Favnir の Result.err は Ok(Value::Variant("err", ...)) として返るため
// process::exit(1) が呼ばれない
```

### 修正
```rust
// cmd_run の --legacy パスに追加
let result = exec_artifact_main_with_source(&artifact, db_url, Some(&source_path2))
    .unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(1);
    });
// Favnir の Result.err も exit 1 として扱う
if let Value::Variant(ref tag, ref payload) = result {
    if tag == "err" {
        let msg = payload.as_deref().map(|v| v.repr()).unwrap_or_default();
        eprintln!("error: {msg}");
        process::exit(1);
    }
}
```

### 教訓
`exec_artifact_main_with_source` は Rust レベルの実行エラーのみを `Err` として返す。
Favnir プログラムが `Result.err` を返した場合は `Ok(Value::Variant("err", ...))` になる。
`fav run` コマンドとして呼ぶ場合は、この Variant も exit 1 として扱う必要がある。
**テストが必要**: `pub fn main(...) -> Result<Unit, String> { Result.err("x") }` で exit 1 を確認。

---

## Bug 4: `AWS_CONFIG` thread-local が `from_env()` ではなく `default()` で初期化

### 症状
```
HTTP 400: {"__type":"InvalidSignatureException",
           "message":"Credential should be scoped to a valid region. "}
```

### 原因
```rust
// vm.rs
thread_local! {
    static AWS_CONFIG: std::cell::RefCell<AwsConfig> =
        // ← default() はハードコード値（region="us-east-1", key="test"）
        std::cell::RefCell::new(AwsConfig::default());
}

impl Default for AwsConfig {
    fn default() -> Self {
        AwsConfig {
            region: "us-east-1".to_string(),  // ← Lambda では ap-northeast-1 が必要
            access_key: "test".to_string(),    // ← Lambda では IAM 認証情報が必要
            secret_key: "test".to_string(),
            session_token: None,
        }
    }
}
```

LocalStack テストでは `set_aws_config()` で上書きするため顕在化しなかった。
`fav run --legacy` では `set_aws_config()` が呼ばれないため、default 値のまま使われた。

### 修正
```rust
thread_local! {
    static AWS_CONFIG: std::cell::RefCell<AwsConfig> =
        std::cell::RefCell::new(AwsConfig::from_env());  // ← 環境変数から読む
}
```

`from_env()` は環境変数が未設定の場合は同じ default 値を返すため、
LocalStack テストへの影響なし。

### 教訓
- thread-local の初期化は **起動時に一度だけ**評価される。テスト環境では `set_aws_config()` が
  上書きするので顕在化しないバグが、Lambda 実環境で初めて現れるパターン。
- デバッグは `aws_post` のエラーボディを取得できるかどうかで大きく変わる。
  当初 `map_err(|e| e.to_string())` ではステータスコードしか見えなかった。

---

## デバッグ手法として有効だったもの

### Bootstrap に base64 デバッグログを仕込む
```sh
EXIT_CODE=0
OUTPUT=$(fav run --legacy /var/task/verifier.fav 2>&1) || EXIT_CODE=$?
echo "[DEBUG] EXIT_CODE=${EXIT_CODE}"
echo "[DEBUG] OUTPUT_B64=$(printf '%s' "$OUTPUT" | base64 | tr -d '\n')"
```
→ CloudWatch Logs で `aws logs filter-log-events` して base64 デコードすると
  fav の完全な stdout/stderr が確認できる。

### aws_post のエラーボディ取得
```rust
// Before（ステータスコードのみ）
req.send_string(body).map_err(|e| e.to_string())

// After（レスポンスボディ含む）
match req.send_string(body) {
    Ok(r) => r.into_string().map_err(|e| e.to_string()),
    Err(ureq::Error::Status(code, resp)) => {
        let body = resp.into_string().unwrap_or_default();
        Err(format!("HTTP {code}: {body}"))
    }
    Err(e) => Err(e.to_string()),
}
```

---

## Docker ビルドに関する注意点

| 問題 | 対処 |
|---|---|
| `target/` が 179GB でコンテキスト転送に時間がかかる | `.dockerignore` に `target/` を追加 |
| `CXXFLAGS="/EHsc /utf-8"` (Windows MSVC) が Linux ビルドに混入 | `ENV CXXFLAGS=""` を Dockerfile に追加 |
| ECR push で OCI manifest エラー（Lambda非対応） | `docker buildx build --provenance=false --push` を使う |
| ソース変更後にキャッシュが使われ再ビルドされない | `docker build --no-cache` を使う |
| Lambda が旧イメージを使い続ける | `aws lambda update-function-code` + `wait function-updated` |

---

## Git Bash 環境での注意点

| 問題 | 対処 |
|---|---|
| `uuidgen` が存在しない | `/tmp/uuidgen` ラッパー: `powershell.exe -Command "[guid]::NewGuid().ToString()"` |
| `aws logs` で `/aws/lambda/...` パスが Windows パスに変換される | `MSYS_NO_PATHCONV=1` を付ける、または ARN 形式で指定 |
| `date` コマンドが見つからない（PATH 問題） | `PATH="/usr/bin:$PATH"` で明示的に追加 |
