# Favnir v5.2.0 実装計画

作成日: 2026-05-20

---

## Phase A: `rune.toml` フォーマット定義

### 変更ファイル: `runes/*/rune.toml`（15 ファイル新規作成）

各 Rune のエントリポイントと副作用を確認してから作成する。既存 `runes/<name>/` ディレクトリ内の `.fav` ファイルを読んでエントリポイント名を特定すること。

**テンプレート**:
```toml
[rune]
name        = "<name>"
version     = "0.1.0"
description = "<description>"
entry       = "<name>.fav"
effects     = []

[dependencies]
```

**各 Rune の初期バージョンは `0.1.0`**（未 publish のため）。エフェクトは後で正確化（v5.5.0 publish 時）。

---

## Phase B: 新規 VM Primitive

### 変更ファイル: `fav/src/backend/vm.rs`

#### B-1. `String.base64_decode`

既存の `String.base64_encode` の近くに追加。

```rust
"String.base64_decode" => {
    let s = get_string!(args, 0, "String.base64_decode");
    match BASE64_STANDARD.decode(s.as_bytes()) {
        Ok(bytes) => {
            let list = bytes.into_iter()
                .map(|b| VMValue::Int(b as i64))
                .collect::<Vec<_>>();
            VMValue::List(Rc::new(RefCell::new(list)))
        }
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    }
}
```

注意: `base64` crate は既に `Cargo.toml` に存在する（`String.base64_encode` が使用中）。`BASE64_STANDARD` の use 文も確認すること。

#### B-2. `AWS.s3_get_object_base64_raw`

既存の `AWS.s3_*` ブロックの近くに追加。

```rust
"AWS.s3_get_object_base64_raw" => {
    let bucket = get_string!(args, 0, "AWS.s3_get_object_base64_raw");
    let key    = get_string!(args, 1, "AWS.s3_get_object_base64_raw");
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_s3::Client::new(&config);
        match client.get_object().bucket(&bucket).key(&key).send().await {
            Err(e) => err_vm(VMValue::Str(e.to_string())),
            Ok(resp) => {
                match resp.body.collect().await {
                    Err(e) => err_vm(VMValue::Str(e.to_string())),
                    Ok(agg) => {
                        let encoded = BASE64_STANDARD.encode(agg.into_bytes());
                        ok_vm(VMValue::Str(encoded))
                    }
                }
            }
        }
    })
}
```

#### B-3. `AWS.s3_put_bytes_raw`

```rust
"AWS.s3_put_bytes_raw" => {
    let bucket = get_string!(args, 0, "AWS.s3_put_bytes_raw");
    let key    = get_string!(args, 1, "AWS.s3_put_bytes_raw");
    let bytes_val = args.get(2).cloned().unwrap_or(VMValue::List(Rc::new(RefCell::new(vec![]))));
    let bytes: Vec<u8> = match &bytes_val {
        VMValue::List(lst) => lst.borrow().iter().map(|v| match v {
            VMValue::Int(n) => (n & 0xFF) as u8,
            _ => 0u8,
        }).collect(),
        _ => vec![],
    };
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_s3::Client::new(&config);
        let body = aws_sdk_s3::primitives::ByteStream::from(bytes);
        match client.put_object().bucket(&bucket).key(&key).body(body).send().await {
            Err(e) => err_vm(VMValue::Str(e.to_string())),
            Ok(_)  => ok_vm(VMValue::Unit),
        }
    })
}
```

#### B-4. `AWS.s3_list_objects_raw`

```rust
"AWS.s3_list_objects_raw" => {
    let bucket = get_string!(args, 0, "AWS.s3_list_objects_raw");
    let prefix = get_string!(args, 1, "AWS.s3_list_objects_raw");
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_s3::Client::new(&config);
        match client.list_objects_v2().bucket(&bucket).prefix(&prefix).send().await {
            Err(e) => err_vm(VMValue::Str(e.to_string())),
            Ok(resp) => {
                let keys: Vec<VMValue> = resp.contents()
                    .iter()
                    .filter_map(|obj| obj.key().map(|k| VMValue::Str(k.to_string())))
                    .collect();
                ok_vm(VMValue::List(Rc::new(RefCell::new(keys))))
            }
        }
    })
}
```

### 変更ファイル: `fav/src/middle/checker.rs`

既存の `("AWS", ...)` ブロックの近くに追加:

```rust
("String", "base64_decode") => Some(Type::Result(
    Box::new(Type::List(Box::new(Type::Int))),
    Box::new(Type::String),
)),
("AWS", "s3_get_object_base64_raw") => Some(Type::Result(
    Box::new(Type::String),
    Box::new(Type::String),
)),
("AWS", "s3_put_bytes_raw") => Some(Type::Result(
    Box::new(Type::Unit),
    Box::new(Type::String),
)),
("AWS", "s3_list_objects_raw") => Some(Type::Result(
    Box::new(Type::List(Box::new(Type::String))),
    Box::new(Type::String),
)),
```

### 変更ファイル: `fav/src/backend/vm_stdlib_tests.rs`

```rust
#[test]
fn test_string_base64_decode() {
    // "hello" → [104, 101, 108, 108, 111]
    let src = r#"
        bind res <- String.base64_decode("aGVsbG8=");
        match res {
          Err(_) => -1
          Ok(bytes) => List.length(bytes)
        }
    "#;
    assert_eq!(run_expr(src), VMValue::Int(5));
}

#[test]
fn test_string_base64_decode_invalid() {
    let src = r#"
        bind res <- String.base64_decode("not!!valid%%base64");
        match res {
          Err(_) => 0
          Ok(_)  => 1
        }
    "#;
    assert_eq!(run_expr(src), VMValue::Int(0));
}

#[test]
fn test_string_base64_roundtrip() {
    let src = r#"
        let encoded = String.base64_encode("favnir");
        bind decoded_bytes <- String.base64_decode(encoded);
        match decoded_bytes {
          Err(_) => -1
          Ok(bs) => List.length(bs)
        }
    "#;
    assert_eq!(run_expr(src), VMValue::Int(6));
}
```

注意: `let` は Favnir キーワードではないため、`test_string_base64_roundtrip` の `let` 行は別の書き方に変更すること（インライン式またはネストで対応）。

---

## Phase C: Registry S3 キー変更

### 変更ファイル: `rune-registry/src/main.fav`

#### `save_rune` 関数の変更

```favnir
fn save_rune(name: String, version: String, description: String, zip_b64: String) -> Result<Unit, String> !AWS {
  bind decoded <- String.base64_decode(zip_b64);
  match decoded {
    Err(e) => Result.err(e)
    Ok(bytes) => {
      bind put_db <- AWS.dynamo_put_item_raw(
        db_table(),
        Map.set(Map.set(Map.set((), "name", name), "version", version), "description", description)
      );
      match put_db {
        Err(e) => Result.err(e)
        Ok(_)  => {
          bind s3_key <- String.concat(String.concat(name, "/"), String.concat(version, ".zip"));
          AWS.s3_put_bytes_raw(pkg_bucket(), s3_key, bytes)
        }
      }
    }
  }
}
```

#### `handle_publish` 関数の変更

`zip` フィールドを必須として取得:

```favnir
fn handle_publish(name: String, body: String, auth: String) -> Map<String, String> !AWS {
  if Http.check_basic_auth(auth, "admin", "adminuser") {
    bind fields_res <- Json.parse_raw(body);
    match fields_res {
      Err(_)     => resp_text(400, "invalid JSON body")
      Ok(fields) => {
        bind version_opt     <- Map.get(fields, "version");
        bind description_opt <- Map.get(fields, "description");
        bind zip_opt         <- Map.get(fields, "zip");
        bind version     <- match version_opt     { None => "0.1.0"  Some(v) => v };
        bind description <- match description_opt { None => ""       Some(d) => d };
        match zip_opt {
          None       => resp_text(400, "missing zip field")
          Some(zip_b64) => {
            bind saved <- save_rune(name, version, description, zip_b64);
            match saved {
              Err(e) => resp_text(500, e)
              Ok(_)  => resp_text(201, "published")
            }
          }
        }
      }
    }
  } else {
    resp_text(401, "Unauthorized")
  }
}
```

---

## Phase D: 新エンドポイント実装

### 変更ファイル: `rune-registry/src/main.fav`

#### `handle_versions` 関数

```favnir
fn handle_versions(name: String) -> Map<String, String> !AWS {
  bind prefix <- String.concat(name, "/");
  bind list_res <- AWS.s3_list_objects_raw(pkg_bucket(), prefix);
  match list_res {
    Err(e)   => resp_text(500, e)
    Ok(keys) => {
      bind prefix_len <- String.length(prefix);
      bind suffix_len <- String.length(".zip");
      bind versions <- List.map(keys, |k| {
        bind total <- String.length(k);
        String.slice(k, prefix_len, total - suffix_len)
      });
      resp_json(Json.write_array_raw(List.map(versions, |v| Map.set((), "v", v))))
    }
  }
}
```

注意: `Json.write_array_raw` が受け取るのは `List<Map<String,String>>` 形式。バージョン文字列の JSON 配列を直接作れない場合は、文字列ジョインで組み立てる代替案を検討する。

#### `handle_download` 関数

```favnir
fn handle_download(name: String, version: String) -> Map<String, String> !AWS {
  bind s3_key <- String.concat(String.concat(name, "/"), String.concat(version, ".zip"));
  bind get_res <- AWS.s3_get_object_base64_raw(pkg_bucket(), s3_key);
  match get_res {
    Err(_) => resp_text(404, "version not found")
    Ok(b64) =>
      Map.set(Map.set(Map.set(Map.set((), "status", 200), "body", b64), "content_type", "application/zip"), "is_base64", "true")
  }
}
```

#### ルーター変更

3 セグメントパスの判定を追加:

```favnir
fn route(req: Map<String, String>) -> Map<String, String> !AWS {
  // ... 既存の bind ...
  if method == "GET" {
    if path == "/runes" {
      handle_list()
    } else {
      if String.starts_with(path, "/runes/") {
        bind rest <- String.slice(path, 7, String.length(path));
        if String.contains(rest, "/") {
          // 3 セグメント: /runes/{name}/{sub}
          bind slash_pos <- String.index_of(rest, "/");
          bind rune_name <- String.slice(rest, 0, slash_pos);
          bind sub       <- String.slice(rest, slash_pos + 1, String.length(rest));
          if sub == "versions" {
            handle_versions(rune_name)
          } else {
            if sub == "download" {
              bind ver_opt <- Map.get(req, "query_version");
              bind ver <- match ver_opt { None => "" Some(v) => v };
              if ver == "" {
                // バージョン未指定: DynamoDB から最新取得
                bind db_res <- get_rune_db(rune_name);
                match db_res {
                  Err(e)  => resp_text(500, e)
                  Ok(opt) => match opt {
                    None       => resp_text(404, "rune not found")
                    Some(item) => {
                      bind ver_from_db_opt <- Map.get(item, "version");
                      bind ver_from_db <- match ver_from_db_opt { None => "0.1.0" Some(v) => v };
                      handle_download(rune_name, ver_from_db)
                    }
                  }
                }
              } else {
                handle_download(rune_name, ver)
              }
            } else {
              resp_text(404, "not found")
            }
          }
        } else {
          handle_get(rest)
        }
      } else {
        resp_text(404, "not found")
      }
    }
  } else {
    // ... POST など既存のまま ...
  }
}
```

注意: `String.contains`, `String.index_of` の存在を checker.rs で確認すること。なければ代替実装を検討（`String.chars` + `List.find_index`）。

#### `main` 関数への `FAV_QUERY_VERSION` 追加

```favnir
public fn main() -> Unit !Io !Env !AWS {
  // ... 既存の bind ...
  bind query_ver_r <- Env.require_raw("FAV_QUERY_VERSION");
  bind query_ver   <- match query_ver_r { Err(_) => "" Ok(v) => v };
  bind res <- route(
    Map.set(
      Map.set(Map.set(Map.set(Map.set((), "method", method), "path", path), "body", body),
        "authorization", auth),
      "query_version", query_ver
    )
  );
  IO.println(Json.write_raw(res))
}
```

---

## Phase E: Bootstrap 変更

### 変更ファイル: `rune-registry/bootstrap`

**クエリパラメータ抽出（追加）**:
```bash
FAV_QUERY_VERSION=$(echo "$EVENT" | jq -r '.queryStringParameters.version // ""')
export FAV_QUERY_VERSION
```

**`is_base64` 対応（追加）**:
```bash
IS_B64=$(echo "$RESPONSE" | jq -r '.is_base64 // "false"')
if [ "$IS_B64" = "true" ]; then
  LAMBDA_RESPONSE=$(echo "$RESPONSE" | jq '{
    statusCode: (.status | tonumber),
    headers: {"Content-Type": .content_type},
    body: .body,
    isBase64Encoded: true
  }')
else
  LAMBDA_RESPONSE=$(echo "$RESPONSE" | jq '{
    statusCode: (.status | tonumber),
    headers: {"Content-Type": .content_type},
    body: .body
  }')
fi
```

---

## Phase F: デプロイ + テスト

### デプロイ手順

```bash
# master push で GitHub Actions が自動デプロイ
git add rune-registry/ runes/*/rune.toml fav/src/backend/vm.rs fav/src/middle/checker.rs
git commit -m "feat: v5.2.0 rune.toml + registry versioned packages"
git push
```

### エンドツーエンドテスト

```bash
BASE="https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com"

# 1. publish (zip は base64 エンコードしたダミー zip)
ZIP_B64=$(echo "PK dummy" | base64)
curl -X POST "$BASE/runes/csv" \
  -H "Authorization: Basic YWRtaW46YWRtaW51c2Vy" \
  -H "Content-Type: application/json" \
  -d "{\"version\":\"0.2.0\",\"description\":\"CSV Rune\",\"zip\":\"$ZIP_B64\"}"
# → 201 published

# 2. versions
curl "$BASE/runes/csv/versions"
# → ["0.2.0"]

# 3. download
curl "$BASE/runes/csv/download?version=0.2.0" -o csv-0.2.0.zip
# → zip ファイルが保存される

# 4. 既存エンドポイントの確認
curl "$BASE/runes"
curl "$BASE/runes/csv"
```

---

## 実装上の注意事項

### `String.contains` / `String.index_of` の存在確認

vm.rs の AWS ブロック実装前に、`String.contains` と `String.index_of` が既存 VM に存在するか grep で確認する。存在しない場合:

- `String.contains(s, sub)` の代替: `String.split(s, sub)` の結果リストが 2 件以上あれば true
- `String.index_of(s, sub)` の代替: `String.chars` + `List.find_index` で自前実装、またはこのバージョンで VM primitive として追加

### `Json.write_array_raw` の制約

`Json.write_array_raw` は `List<Map<String,String>>` を受け取る。バージョン文字列の配列を直接返すには文字列ジョインで JSON 配列を組み立てる代替案:

```favnir
bind json_arr <- String.concat("[", String.concat(
  List.join(List.map(versions, |v| String.concat("\"", String.concat(v, "\""))), ","),
  "]"
));
resp_json(json_arr)
```

`String.concat` の多用は煩雑なため、`String.join` (または `List.join`) の存在を確認すること。

### `base64` crate の import

vm.rs の既存 `String.base64_encode` 実装で使っている crate/関数名（`BASE64_STANDARD` or `STANDARD` など）を読んで合わせる。
