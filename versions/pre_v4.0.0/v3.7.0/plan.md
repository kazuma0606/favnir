# Favnir v3.7.0 Implementation Plan

## Theme: `http` + `parquet` rune — REST / GraphQL / DWH 出力

---

## Phase 0: バージョン更新

- `Cargo.toml` version → `"3.7.0"`
- `Cargo.toml` に依存追加:
  - `ureq = "2"` — HTTP クライアント（同期・軽量・tokio 不要）
  - `tiny_http = "0.12"` — HTTP サーバー（ブロッキング）
  - `parquet = "52"` + `arrow = { version = "52", features = ["ipc"] }` — Parquet 読み書き
- `src/main.rs` ヘルプテキスト・バージョン表示更新

---

## Phase 1: 型登録 + namespace

**checker.rs**
- `HttpResponse` を `type_defs` に pre-register（`status: Int`, `body: String`, `content_type: String`）
- `HttpError` を `type_defs` に pre-register（`code: Int`, `message: String`, `status: Int`）
- `ParquetError` を `type_defs` に pre-register（`message: String`）
- `!Http` エフェクトを既存エフェクトリストに追加（`Effect::Http`）
- `Http` namespace をチェックリストに追加
- `Parquet` namespace をチェックリストに追加
- `Http.get_raw / post_raw / serve_raw` のシグネチャ登録
- `Parquet.write_raw / read_raw` のシグネチャ登録

**compiler.rs**
- `"Http"` を2箇所のグローバル登録ループに追加
- `"Parquet"` を2箇所のグローバル登録ループに追加
- `"HttpResponse"`, `"HttpError"`, `"ParquetError"` を type registration ループに追加

---

## Phase 2: VM プリミティブ

**vm.rs**

### HTTP クライアント（`ureq`）

```rust
"Http.get_raw" => {
    // ureq::get(url).call() → VMValue::Result(Ok(HttpResponse) | Err(HttpError))
}
"Http.post_raw" => {
    // ureq::post(url).set("Content-Type", ct).send_string(body)
}
```

エラーマッピング:
- 接続失敗 / DNS 解決失敗 → `code: 0`
- タイムアウト → `code: 1`
- HTTP エラーステータス（4xx/5xx）→ `code: 2, status: <code>`

### HTTP サーバー（`tiny_http`）

```rust
"Http.serve_raw" => {
    // tiny_http::Server::http("0.0.0.0:{port}")
    // ループして request を受け取り、handler を呼び出してレスポンス返す
    // handler = VMValue::Closure を呼び出す
}
```

### Parquet（`parquet` + `arrow`）

```rust
"Parquet.write_raw" => {
    // type_metas から型スキーマを取得
    // Arrow Schema 構築 → RecordBatch 作成 → Parquet ファイル書き込み
}
"Parquet.read_raw" => {
    // Parquet ファイル → RecordBatch → List<Map<String, String>> に変換
}
```

**vm_stdlib_tests.rs**: 8件の新テスト追加

| テスト名 | 内容 |
|---------|------|
| `http_get_raw_returns_ok_on_success` | ureq mock or httpbin.org GET |
| `http_get_raw_returns_err_on_bad_url` | 不正 URL → Err |
| `http_post_raw_sends_body` | POST ボディ確認 |
| `parquet_write_then_read_roundtrip` | 書いて読んでデータ一致 |
| `parquet_write_creates_file` | ファイルが存在する |
| `parquet_read_returns_err_on_missing` | 存在しないファイル → Err |
| `http_response_status_field` | resp.status が Int で取得できる |
| `parquet_row_count_matches` | 書き込み行数 == 読み込み行数 |

> **注意**: `http_get_raw` の外部ネットワーク依存テストは `#[ignore]` にして CI でスキップ可能にする。ローカルテストは `tiny_http` のエコーサーバーを別スレッドで立てる方式で対応。

---

## Phase 3: `runes/http/http.fav`

```
runes/
  http/
    http.fav   ← 6 関数
```

実装する関数:
1. `get(url)` → `Http.get_raw(url)`
2. `post(url, body)` → `Http.post_raw(url, body, "text/plain")`
3. `post_json(url, body)` → `Http.post_raw(url, body, "application/json")`
4. `get_body(url)` → get して `Result<String, HttpError>` を返す（resp.body）
5. `ok(status, body)` → `HttpResponse` レコードを作成する純粋関数
6. `error_response(status, message)` → エラー用 `HttpResponse` 作成

---

## Phase 4: `runes/http/http.test.fav`

テスト（10 件目標）:

| # | テスト名 | 内容 |
|---|---------|------|
| 1 | `http_get_returns_result` | 型が `Result<HttpResponse, HttpError>` |
| 2 | `http_get_bad_url_is_err` | 不正 URL → `Result.is_err` |
| 3 | `http_post_json_content_type` | Content-Type が json になる |
| 4 | `http_get_body_on_success` | get_body → Ok(String) |
| 5 | `http_get_body_on_bad_url` | get_body → Err |
| 6 | `http_ok_helper_status` | http.ok(200, "hi").status == 200 |
| 7 | `http_ok_helper_body` | http.ok(200, "hi").body == "hi" |
| 8 | `http_error_response_status` | http.error_response(404, "not found").status == 404 |
| 9 | `http_post_returns_result` | post → Result<HttpResponse, HttpError> |
| 10 | `http_get_httpbin_status_200` | httpbin.org GET → status == 200（`#[ignore]` 指定） |

---

## Phase 5: `runes/parquet/parquet.fav` + テスト

### `parquet.fav`（4 関数）

```
runes/
  parquet/
    parquet.fav
    parquet.test.fav
```

1. `write(path, type_name, rows)` → `Parquet.write_raw(path, type_name, rows)`
2. `read(path)` → `Parquet.read_raw(path)`
3. `append(path, type_name, rows)` → 既存読み込み + concat + 書き直し
4. `row_count(path)` → read して `List.length`（軽量版は Parquet メタデータから取得）

### `parquet.test.fav`（8 件）

| # | テスト名 | 内容 |
|---|---------|------|
| 1 | `parquet_write_returns_ok` | 正常書き込み |
| 2 | `parquet_read_after_write_count` | 書いた行数 == 読んだ行数 |
| 3 | `parquet_read_preserves_string_field` | String フィールドが保持される |
| 4 | `parquet_read_preserves_int_field` | Int フィールドが保持される |
| 5 | `parquet_read_missing_file_is_err` | 存在しないファイル → Err |
| 6 | `parquet_row_count_matches_write` | row_count == 書き込み行数 |
| 7 | `parquet_append_increases_count` | append 後に行数が増える |
| 8 | `parquet_write_empty_rows` | 0 件書き込みが Err にならない |

---

## Phase 6: driver 統合テスト + `fav build --graphql`

### driver.rs 統合テスト（6 件）

1. `http_rune_test_file_passes` → `run_fav_test_file_with_runes("runes/http/http.test.fav")`
2. `parquet_rune_test_file_passes` → `run_fav_test_file_with_runes("runes/parquet/parquet.test.fav")`
3. `http_get_body_in_favnir_source` — inline Favnir ソース
4. `parquet_write_read_roundtrip_in_favnir_source` — inline Favnir ソース
5. `http_ok_helper_in_favnir_source` — inline Favnir ソース
6. `fav_build_graphql_generates_type_block` — SDL 生成確認

### `fav build --graphql`

**driver.rs**
- `cmd_build_graphql(file: &str, out_path: &str)` — AST を解析して SDL を生成
  - `Item::TypeDef` → `type <Name> { ... }` ブロック
  - `Item::InterfaceDef` → `type Query { ... }` ブロック
  - Favnir 型 → GraphQL 型マッピング（`Int`→`Int!`, `String`→`String!`, `Option<T>`→`T`, `List<T>`→`[T!]!`, `Result<T,E>`→`T`）
  - ファイル書き出し or stdout

**main.rs**
- `fav build --graphql <file> [--out <path>]` パース
- ヘルプテキスト更新

---

## Phase 7: examples + docs

- `fav/examples/http_demo/main.fav` — GET + POST + エラーハンドリング例
- `fav/examples/parquet_demo/main.fav` — 書き込み + 読み込み + 行数確認例
- `fav/examples/graphql_schema_demo/main.fav` + `schema.graphql` — interface → SDL 例
- `versions/v3.7.0/langspec.md`
- `versions/v3.7.0/migration-guide.md`
- `versions/v3.7.0/progress.md` 全フェーズ完了に更新

---

## 依存関係

| クレート | バージョン | 用途 |
|---------|----------|------|
| `ureq` | `"2"` | HTTP クライアント（同期、軽量） |
| `tiny_http` | `"0.12"` | HTTP サーバー（ブロッキング） |
| `parquet` | `"52"` | Parquet ファイル読み書き |
| `arrow` | `"52"` | Arrow データ形式（Parquet の依存） |

> `ureq` は tokio 不要。VM がシングルスレッドで動作するため同期クライアントで十分。

---

## テスト目標

v3.6.0: ~790 tests → v3.7.0 目標: **~840 tests**

---

## 実装上の注意

### Http.serve_raw の設計

Favnir の VM は同期実行のため、`Http.serve_raw` は `tiny_http` のリクエストループを呼び出す。
Favnir クロージャを `handle_request` として受け取り、`VMValue::Closure` を VM コールする。
テストでは別スレッドでサーバーを立て、ureq でリクエストを送信して確認する。

### Parquet スキーマ解決

`Parquet.write_raw` は `type_metas` から型情報を取得する（`Gen.one_raw` と同じパターン）。
`type_metas` に登録されていない型名は `ParquetError` を返す。

### `fav build --graphql` の制約（v3.7.0 時点）

- サポートする interface は `method: ArgType -> Result<ReturnType, E>` 形式のみ
- Mutation は `interface` に `mut_` プレフィックスを付けると `type Mutation` に分類（将来拡張）
- 生成 SDL は Query のみ（Mutation/Subscription は将来バージョン）
