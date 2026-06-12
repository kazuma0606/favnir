# v14.5.0 Plan — 技術実装設計

Date: 2026-06-12

---

## 実装順序（Phase A → E）

```
A: fav/src/backend/vm.rs — AzureBlob.* 4 primitives + azure_blob_sign ヘルパー
    ↓
B: fav/src/middle/checker.rs — AzureBlob namespace + require_azure_storage_effect
    ↓
C: runes/azure-blob/ — azure_blob.fav + rune.toml 新規作成
    ↓
D: fav/src/driver.rs — v145000_tests (4件) + Cargo.toml バンプ
    ↓
E: 全テスト + コミット
```

---

## Phase A: `fav/src/backend/vm.rs`

### A-1: `azure_blob_sign` ヘルパー関数（Shared Key 署名）

`aws_post` / `aws_sigv4_headers` のパターンを参照し、Azure Blob 用の Shared Key 署名を実装する。
**新規 crate は追加しない**（既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` を使用）。

追加場所: `AzurePostgres.execute_raw` の前（`// ── AzurePostgres` コメントの直前、vm.rs ~9818）。

```rust
/// Azure Blob Storage — Shared Key 署名ヘッダーを生成する
/// Returns: (x-ms-date, Authorization) ヘッダー値
fn azure_blob_sign(
    account: &str,
    key_b64: &str,
    method: &str,           // "PUT" / "GET" / "DELETE"
    content_type: &str,     // "application/octet-stream" or ""
    content_length: usize,  // body の byte 長
    x_ms_blob_type: &str,   // "BlockBlob" (PUT 時のみ) or ""
    canonical_resource: &str, // "/{account}/{container}/{blob}" または クエリ付き
) -> Result<(String, String), String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};

    // x-ms-date (RFC 1123)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let date = fmt_rfc1123(now); // 既存の fmt_rfc1123 または同等実装

    // CanonicalizedHeaders (x-ms-* を辞書順で)
    let mut ms_headers: Vec<(String, String)> = vec![
        ("x-ms-date".to_string(), date.clone()),
        ("x-ms-version".to_string(), "2020-10-02".to_string()),
    ];
    if !x_ms_blob_type.is_empty() {
        ms_headers.push(("x-ms-blob-type".to_string(), x_ms_blob_type.to_string()));
    }
    ms_headers.sort_by(|a, b| a.0.cmp(&b.0));
    let canonical_headers: String = ms_headers
        .iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect();

    // StringToSign
    let content_len_str = if content_length > 0 {
        content_length.to_string()
    } else {
        "".to_string()
    };
    let string_to_sign = format!(
        "{}\n\n{}\n{}\n{}{}",
        method,
        content_type,
        content_len_str,
        canonical_headers,
        canonical_resource
    );

    // HMAC-SHA256
    let key_bytes = B64.decode(key_b64)
        .map_err(|e| format!("azure_blob_sign: invalid storage key: {}", e))?;
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&key_bytes)
        .map_err(|e| format!("azure_blob_sign: hmac error: {}", e))?;
    mac.update(string_to_sign.as_bytes());
    let sig = B64.encode(mac.finalize().into_bytes());

    let auth = format!("SharedKey {}:{}", account, sig);
    Ok((date, auth))
}
```

**注意**: StringToSign の厳密フォーマットは Azure ドキュメントの "Shared Key Lite" 形式を採用。
Content-MD5 は空で渡す（小規模ブロブ向け）。

### A-2: `AzureBlob.put_raw` ハンドラ

`"AWS.secrets_get_raw"` の直後（Email.send_raw の前）の ureq POST パターンを参考に追加。

```rust
// ── Azure Blob Storage primitives (v14.5.0) ───────────────────────────────
"AzureBlob.put_raw" => {
    // AzureBlob.put_raw(account, key, container, blob_name, body) -> Result<Unit, String>
    let mut it = args.into_iter();
    let account    = vm_string(it.next().ok_or("put_raw: missing account")?,    "AzureBlob.put_raw")?;
    let key        = vm_string(it.next().ok_or("put_raw: missing key")?,        "AzureBlob.put_raw")?;
    let container  = vm_string(it.next().ok_or("put_raw: missing container")?,  "AzureBlob.put_raw")?;
    let blob_name  = vm_string(it.next().ok_or("put_raw: missing blob_name")?,  "AzureBlob.put_raw")?;
    let body       = vm_string(it.next().ok_or("put_raw: missing body")?,       "AzureBlob.put_raw")?;

    let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
    let (date, auth) = match azure_blob_sign(
        &account, &key, "PUT",
        "application/octet-stream", body.len(),
        "BlockBlob", &canonical_resource,
    ) {
        Ok(h) => h,
        Err(e) => return Ok(err_vm(VMValue::Str(e))),
    };

    let url = format!(
        "https://{}.blob.core.windows.net/{}/{}",
        account, container, blob_name
    );
    let result = ureq::put(&url)
        .header("x-ms-date", &date)
        .header("x-ms-version", "2020-10-02")
        .header("x-ms-blob-type", "BlockBlob")
        .header("Content-Type", "application/octet-stream")
        .header("Authorization", &auth)
        .send_bytes(body.as_bytes());

    Ok(match result {
        Ok(_)  => ok_vm(VMValue::Unit),
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    })
}
```

### A-3: `AzureBlob.get_raw` ハンドラ

```rust
"AzureBlob.get_raw" => {
    let mut it = args.into_iter();
    let account   = vm_string(it.next().ok_or("get_raw: missing account")?,   "AzureBlob.get_raw")?;
    let key       = vm_string(it.next().ok_or("get_raw: missing key")?,       "AzureBlob.get_raw")?;
    let container = vm_string(it.next().ok_or("get_raw: missing container")?, "AzureBlob.get_raw")?;
    let blob_name = vm_string(it.next().ok_or("get_raw: missing blob_name")?, "AzureBlob.get_raw")?;

    let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
    let (date, auth) = match azure_blob_sign(
        &account, &key, "GET", "", 0, "", &canonical_resource,
    ) {
        Ok(h) => h,
        Err(e) => return Ok(err_vm(VMValue::Str(e))),
    };

    let url = format!(
        "https://{}.blob.core.windows.net/{}/{}",
        account, container, blob_name
    );
    let result = ureq::get(&url)
        .header("x-ms-date", &date)
        .header("x-ms-version", "2020-10-02")
        .header("Authorization", &auth)
        .call();

    Ok(match result {
        Ok(resp) => match resp.into_body().read_to_string() {
            Ok(body) => ok_vm(VMValue::Str(body)),
            Err(e)   => err_vm(VMValue::Str(e.to_string())),
        },
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    })
}
```

### A-4: `AzureBlob.list_raw` ハンドラ

list_raw は XML レスポンスを JSON 配列（blob name の String 配列）に変換して返す。

```rust
"AzureBlob.list_raw" => {
    let mut it = args.into_iter();
    let account   = vm_string(it.next().ok_or("list_raw: missing account")?,   "AzureBlob.list_raw")?;
    let key       = vm_string(it.next().ok_or("list_raw: missing key")?,       "AzureBlob.list_raw")?;
    let container = vm_string(it.next().ok_or("list_raw: missing container")?, "AzureBlob.list_raw")?;
    let prefix    = vm_string(it.next().ok_or("list_raw: missing prefix")?,    "AzureBlob.list_raw")?;

    let query = format!("restype=container&comp=list&prefix={}", url_encode(&prefix));
    let canonical_resource = format!(
        "/{}/{}?comp=list&prefix={}",
        account, container, url_encode(&prefix)
    );
    let (date, auth) = match azure_blob_sign(
        &account, &key, "GET", "", 0, "", &canonical_resource,
    ) {
        Ok(h) => h,
        Err(e) => return Ok(err_vm(VMValue::Str(e))),
    };

    let url = format!(
        "https://{}.blob.core.windows.net/{}?{}",
        account, container, query
    );
    let result = ureq::get(&url)
        .header("x-ms-date", &date)
        .header("x-ms-version", "2020-10-02")
        .header("Authorization", &auth)
        .call();

    Ok(match result {
        Ok(resp) => match resp.into_body().read_to_string() {
            Ok(xml) => {
                // <Name>...</Name> を抽出して JSON 配列に変換
                let names = extract_xml_tags(&xml, "Name"); // 既存ヘルパー流用
                let json = serde_json::to_string(&names).unwrap_or("[]".to_string());
                ok_vm(VMValue::Str(json))
            }
            Err(e) => err_vm(VMValue::Str(e.to_string())),
        },
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    })
}
```

### A-5: `AzureBlob.delete_raw` ハンドラ

```rust
"AzureBlob.delete_raw" => {
    let mut it = args.into_iter();
    let account   = vm_string(it.next().ok_or("delete_raw: missing account")?,   "AzureBlob.delete_raw")?;
    let key       = vm_string(it.next().ok_or("delete_raw: missing key")?,       "AzureBlob.delete_raw")?;
    let container = vm_string(it.next().ok_or("delete_raw: missing container")?, "AzureBlob.delete_raw")?;
    let blob_name = vm_string(it.next().ok_or("delete_raw: missing blob_name")?, "AzureBlob.delete_raw")?;

    let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
    let (date, auth) = match azure_blob_sign(
        &account, &key, "DELETE", "", 0, "", &canonical_resource,
    ) {
        Ok(h) => h,
        Err(e) => return Ok(err_vm(VMValue::Str(e))),
    };

    let url = format!(
        "https://{}.blob.core.windows.net/{}/{}",
        account, container, blob_name
    );
    let result = ureq::delete(&url)
        .header("x-ms-date", &date)
        .header("x-ms-version", "2020-10-02")
        .header("Authorization", &auth)
        .call();

    Ok(match result {
        Ok(_)  => ok_vm(VMValue::Unit),
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    })
}
```

### A-6: `cargo build` でコンパイルエラーなし確認

---

## Phase B: `fav/src/middle/checker.rs`

### B-1: `require_azure_storage_effect` 追加

`require_azure_db_effect`（~line 5028）の直後に追加:

```rust
fn require_azure_storage_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::AzureStorage)) {
        self.type_error(
            "E0317",
            "AzureBlob.* call requires `!AzureStorage` effect on enclosing fn/stage",
            span,
        );
    }
}
```

### B-2: `builtin_ret_ty` に `AzureBlob.*` 追加

`("AzurePostgres", _)` ブロックの直後に追加:

```rust
// AzureBlob (v14.5.0) — require !AzureStorage effect
("AzureBlob", "put_raw") => {
    self.require_azure_storage_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
("AzureBlob", "get_raw") => {
    self.require_azure_storage_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("AzureBlob", "list_raw") => {
    self.require_azure_storage_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("AzureBlob", "delete_raw") => {
    self.require_azure_storage_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
("AzureBlob", _) => {
    self.require_azure_storage_effect(span);
    Some(Type::Unknown)
}
```

### B-3: `BUILTIN_EFFECTS` に `"AzureBlob"` 追加

既存の `"AzurePostgres"` の隣（~line 1422）に追加:

```rust
"AzureBlob",
```

### B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C: `runes/azure-blob/` 新規作成

### C-1: `runes/azure-blob/azure_blob.fav`

spec.md の関数設計を参照。4 関数（put / get / list / delete）。

**注記**: `import rune "ctx"` は `runes/ctx/ctx.fav` 未存在のため省略。
`ctx: String` で代替（`Ctx.azure_get_field_raw` は String 引数を受け付ける）。

### C-2: `runes/azure-blob/rune.toml`

```toml
[rune]
name        = "azure-blob"
version     = "14.5.0"
description = "Azure Blob Storage: put/get/list/delete with Shared Key authentication"
entry       = "azure_blob.fav"
effects     = ["!AzureStorage"]

[dependencies]
```

### C-3: `cargo test` でリグレッションなし確認

---

## Phase D: `fav/src/driver.rs` + `Cargo.toml`

### D-1: `v145000_tests` モジュール追加（`v144000_tests` の直後）

```rust
#[cfg(test)]
mod v145000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn version_is_14_5_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.5.0");
    }

    #[test]
    fn azure_blob_put_raw_registered() {
        // AzureBlob.put_raw が E0007 を出さないことを確認
        let src = r#"
public fn save(account: String, key: String, container: String) -> Result<Unit, String> !AzureStorage {
    AzureBlob.put_raw(account, key, container, "proof.json", "{}")
}
"#;
        let prog = Parser::parse_str(src, "blob_test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        let e0007: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0007" && e.message.contains("put_raw"))
            .collect();
        assert!(e0007.is_empty(),
            "AzureBlob.put_raw should not produce E0007, got: {:?}", e0007);
    }

    #[test]
    fn azure_storage_effect_required() {
        // !AzureStorage なしで AzureBlob.put_raw を呼ぶと E0317 が出ることを確認
        let src = r#"
public fn save(account: String, key: String, container: String) -> Result<Unit, String> {
    AzureBlob.put_raw(account, key, container, "proof.json", "{}")
}
"#;
        let prog = Parser::parse_str(src, "blob_effect_test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        let e0317: Vec<_> = errors.iter()
            .filter(|e| e.code == "E0317")
            .collect();
        assert!(!e0317.is_empty(),
            "AzureBlob.put_raw without !AzureStorage should produce E0317");
    }

    #[test]
    fn azure_blob_rune_file_present() {
        // runes/azure-blob/azure_blob.fav が存在し put/get 関数を含む確認
        let rune_fav = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("runes/azure-blob/azure_blob.fav")
        ).expect("azure_blob.fav should exist");
        assert!(rune_fav.contains("fn put"),
            "azure_blob.fav should contain fn put");
        assert!(rune_fav.contains("fn get"),
            "azure_blob.fav should contain fn get");
    }
}
```

### D-2: `v144000_tests` の `version_is_14_4_0` を `>=` 比較に修正

```rust
assert!(env!("CARGO_PKG_VERSION") >= "14.4.0", ...);
```

### D-3: `fav/Cargo.toml` バージョンを `"14.5.0"` にバンプ

### D-4: `cargo test v145000` で 4 件全パス確認

---

## 参照先ファイル（実装時に確認すること）

| ファイル | 参照目的 |
|---|---|
| `fav/src/backend/vm.rs:9818` | `// ── AzurePostgres` セクション — 追加場所の前 |
| `fav/src/backend/vm.rs:12605` | `sqs_send_message_raw` — ureq POST パターン |
| `fav/src/backend/vm.rs:12944` | `AWS.secrets_get_raw` — ureq POST + JSON パターン |
| `fav/src/backend/vm.rs:~12603` | `url_encode` / `extract_xml_tags` ヘルパー — 流用 |
| `fav/src/middle/checker.rs:5028` | `require_azure_db_effect` — E0316 パターン |
| `fav/src/middle/checker.rs:5845` | `("AzurePostgres", ...)` ブロック — 追加場所の前 |
| `runes/aws/secrets.fav` | rune 関数スタイル参照 |
| `versions/v14.4.0/tasks.md` 実装メモ | `let` 構文制限 / `import rune "ctx"` 制限 |

---

## 実装上の注意点

1. **Azure Shared Key 署名のフォーマット**:
   StringToSign の改行区切りは厳密に仕様通り。Content-MD5 は空文字列（改行は含む）。
   `x-ms-date` を Date ヘッダーの代わりに使う場合、StringToSign の Date 行は空にする（改行のみ）。
   実際のフォーマット:
   ```
   PUT\n\napplication/octet-stream\n\nx-ms-blob-type:BlockBlob\nx-ms-date:{date}\nx-ms-version:2020-10-02\n/{account}/{container}/{blob}
   ```

2. **ureq の HTTP メソッド**:
   `ureq::delete(&url)` が使えない場合は `ureq::request("DELETE", &url)` で代替。

3. **list_raw の XML 解析**:
   `extract_xml_tags(&xml, "Name")` は既存ヘルパー（SQS の MessageId 抽出と同じ関数）で流用可能。

4. **`fmt_rfc1123` の確認**:
   vm.rs に既存の RFC 1123 フォーマット関数があるか確認。なければ `chrono` crate（既存依存）または手動実装。

5. **Windows 開発環境**:
   ureq の HTTPS は Windows でも動作する（rustls ベース）。Azure Blob への HTTPS 接続で追加設定不要。
