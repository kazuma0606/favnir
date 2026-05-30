# Favnir v5.2.0 仕様書 — パッケージ仕様 + Registry 拡張

作成日: 2026-05-20

---

## 概要

v5.2.0 は `rune` パッケージマネージャの土台を固めるバージョン。
ローカル Rune に `rune.toml` を追加し、Registry をバージョン管理対応に拡張する。

---

## Phase A: `rune.toml` フォーマット定義

### A-1. Rune パッケージ用 `rune.toml`

各 Rune ディレクトリ（`runes/<name>/`）に配置。

```toml
[rune]
name        = "csv"
version     = "0.2.0"
description = "CSV parse/write with type-safe schema adaptation"
entry       = "csv.fav"
effects     = []

[dependencies]
# 他 Rune への依存（v5.6.0 で有効化）
```

| フィールド    | 型            | 必須 | 説明                                      |
|---------------|---------------|------|-------------------------------------------|
| `name`        | String        | Yes  | Rune 名（ディレクトリ名と一致）           |
| `version`     | String (semver) | Yes | セマンティックバージョン                  |
| `description` | String        | Yes  | 説明文（Registry に表示）                 |
| `entry`       | String        | Yes  | エントリポイント `.fav` ファイル名         |
| `effects`     | Array[String] | No   | 副作用宣言（例: `["!Io", "!AWS"]`）       |
| `[dependencies]` | Table      | No   | 依存 Rune（v5.6.0 まで無視）             |

### A-2. プロジェクト用 `rune.toml`

プロジェクトルートに配置（v5.3.0 の `rune install` が読む）。

```toml
[project]
name    = "my-pipeline"
version = "0.1.0"
favnir  = ">=5.2.0"

[runes]
csv  = "0.2.0"
http = "1.0.0"
auth = "0.3.0"
```

インストール先: `./rune_modules/<name>/`（v5.3.0 で実装、v5.2.0 では仕様定義のみ）

### A-3. 既存 15 Rune への `rune.toml` 追加

対象: `runes/auth`, `aws`, `csv`, `db`, `duckdb`, `env`, `gen`, `grpc`, `http`, `incremental`, `json`, `log`, `parquet`, `stat`, `validate`

各ディレクトリのエントリポイント（`<name>.fav` または `main.fav`）を確認し、適切な `rune.toml` を作成する。

---

## Phase B: 新規 VM Primitive

### B-1. `String.base64_decode`

```favnir
String.base64_decode(s: String) -> Result<List<Int>, String>
```

- Base64 文字列をバイト列（`List<Int>`）にデコード
- デコード失敗時は `Err(message)` を返す
- `String.base64_encode` の逆操作

### B-2. `AWS.s3_get_object_base64_raw`

```favnir
AWS.s3_get_object_base64_raw(bucket: String, key: String) -> Result<String, String>
```

- S3 オブジェクトをバイナリとして取得し、Base64 文字列で返す
- zip など非テキストデータの取得に使用
- Lambda API Gateway 経由でバイナリを返す際に必須

### B-3. `AWS.s3_put_bytes_raw`

```favnir
AWS.s3_put_bytes_raw(bucket: String, key: String, bytes: List<Int>) -> Result<Unit, String>
```

- `List<Int>` をバイト列として S3 に書き込む
- zip ファイル等のバイナリデータの保存に使用
- 既存の `AWS.s3_put_object_raw`（文字列専用）の補完

### B-4. `AWS.s3_list_objects_raw`

```favnir
AWS.s3_list_objects_raw(bucket: String, prefix: String) -> Result<List<String>, String>
```

- 指定プレフィックスに一致する S3 キーの一覧を返す
- `GET /runes/{name}/versions` の実装に使用
- 例: prefix=`"csv/"` → `["csv/0.1.0.zip", "csv/0.2.0.zip"]`

---

## Phase C: Registry S3 スキーマ変更

### C-1. S3 キー形式変更

| 変更前 | 変更後 |
|--------|--------|
| `{name}` | `{name}/{version}.zip` |

- バージョン別保存により複数バージョンが共存可能
- 旧データ（バージョンなしキー）は削除 OK
- 例: `csv/0.1.0.zip`, `csv/0.2.0.zip`

### C-2. DynamoDB スキーマ変更なし

- パーティションキー: `name`（変更なし）
- 属性: `version`（最新バージョン）, `description` — 変更なし
- バージョン一覧は S3 オブジェクト列挙で取得（DynamoDB への変更不要）

---

## Phase D: Registry API 拡張

### D-1. 新エンドポイント: `GET /runes/{name}/versions`

**レスポンス**:
```json
["0.1.0", "0.2.0", "0.3.0"]
```

**実装**:
1. `AWS.s3_list_objects_raw(pkg_bucket(), "{name}/")` でキー一覧取得
2. 各キー `{name}/{version}.zip` から `{version}` を抽出
3. バージョン一覧を JSON 配列で返す

### D-2. 新エンドポイント: `GET /runes/{name}/download`

**クエリパラメータ**: `?version=x.y.z`（省略時は DynamoDB の最新バージョンを使用）

**レスポンス**:
- `Content-Type: application/zip`
- Body: zip バイナリ（base64 エンコード済み）
- Lambda Response: `isBase64Encoded: true`

**実装**:
1. version 未指定時: DynamoDB から `version` 属性を取得
2. `AWS.s3_get_object_base64_raw(pkg_bucket(), "{name}/{version}.zip")`
3. レスポンスマップに `"is_base64": "true"` を含める

### D-3. `POST /runes/{name}` の変更

**変更前リクエストボディ**:
```json
{"version":"0.1.0","description":"CSV Rune"}
```

**変更後リクエストボディ**:
```json
{"version":"0.2.0","description":"CSV Rune","zip":"<base64エンコードされた zip データ>"}
```

**実装の変更**:
1. `zip` フィールドを JSON から取得（必須）
2. `String.base64_decode(zip_b64)` → `List<Int>`
3. `AWS.s3_put_bytes_raw(pkg_bucket(), "{name}/{version}.zip", bytes)` で保存
4. 旧 `AWS.s3_put_object_raw` の呼び出しを削除

---

## Phase E: Bootstrap 変更

`rune-registry/bootstrap` の Lambda レスポンス生成ロジックを更新。

**現状**:
```bash
# Favnir stdout: {"status":"201","body":"...","content_type":"text/plain"}
# → {"statusCode":201,"headers":{...},"body":"..."}
```

**v5.2.0 追加**:
```bash
# is_base64: "true" が含まれる場合
# → {"statusCode":200,"headers":{"Content-Type":"application/zip"},
#     "body":"<base64>","isBase64Encoded":true}
```

**クエリパラメータ対応**:
- `bootstrap` が `queryStringParameters.version` を `FAV_QUERY_VERSION` 環境変数にセット
- Favnir コード側で `Env.require_raw("FAV_QUERY_VERSION")` で読む

---

## Phase F: ルーター変更

現状のルーター: 2 セグメントパス（`/runes/{name}`）のみ対応。

v5.2.0 で 3 セグメントパス（`/runes/{name}/download`, `/runes/{name}/versions`）を追加。

**ルーティング拡張**:
```
GET /runes                         → handle_list()
GET /runes/{name}                  → handle_get(name)
GET /runes/{name}/versions         → handle_versions(name)
GET /runes/{name}/download         → handle_download(name, version)
POST /runes/{name}                 → handle_publish(name, body, auth)
```

**Favnir でのパス解析**:
- `String.starts_with(path, "/runes/")` → `rest = String.slice(path, 7, length(path))`
- `String.contains(rest, "/")` で 3 セグメント判定
- 3 セグメントの場合: スラッシュ位置で分割 → `name` + `sub` ("download" or "versions")

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `fav/src/backend/vm.rs` | `String.base64_decode`, `AWS.s3_get_object_base64_raw`, `AWS.s3_put_bytes_raw`, `AWS.s3_list_objects_raw` を追加 |
| `fav/src/middle/checker.rs` | 上記 4 関数の型シグネチャを追加 |
| `fav/src/backend/vm_stdlib_tests.rs` | `test_string_base64_decode` テストを追加 |
| `rune-registry/src/main.fav` | API 拡張（新エンドポイント + POST 変更）|
| `rune-registry/bootstrap` | `is_base64` 対応 + `FAV_QUERY_VERSION` 抽出 |
| `runes/auth/rune.toml` 〜 `runes/validate/rune.toml` | 15 ファイル新規作成 |
| `rune-registry/SPEC.md` | API 追記 |

---

## 完了条件

- [ ] `String.base64_decode` が正常にデコードし、不正入力で `Err` を返す
- [ ] `AWS.s3_get_object_base64_raw` / `AWS.s3_put_bytes_raw` / `AWS.s3_list_objects_raw` が型チェックを通る
- [ ] `cargo test` が全件 pass
- [ ] 全 15 Rune に `rune.toml` が存在する
- [ ] `POST /runes/{name}` が zip 受け付け + `{name}/{version}.zip` で保存
- [ ] `GET /runes/{name}/versions` が S3 オブジェクト一覧からバージョンリストを返す
- [ ] `GET /runes/{name}/download` が zip バイナリを返す
- [ ] Registry デプロイ後、curl でエンドツーエンドテストが通る
