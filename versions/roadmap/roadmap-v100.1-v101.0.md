# Roadmap v100.1.0 〜 v101.0.0 — SAP E2E Foundation 1.0

Date: 2026-09-05
Status: 未着手

マスターロードマップ: [roadmap-v100.1-v105.0.md](roadmap-v100.1-v105.0.md)

---

## 前提

- 直前完了: v100.0.0「Favnir SAP Platform 1.0 宣言」（tests = 4,279）
- 本スプリントは SAP Real-World Platform Era の第 1 スプリント
- 目標: v101.0.0「SAP E2E Foundation 1.0 宣言」（tests = 4,301）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v100.0.0 になっていることを確認する
- `fav/src/driver.rs` に `mod v100000_tests` が存在することを確認する（v100.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `100.0.0` であることを確認する
- `infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json` が存在することを確認する

### スプリントの性格

SAP Real-World Platform Era の**修正・実証スプリント**。

新機能追加より「既存コードが実際に動く」状態の達成を最優先とする。
`Http.get_with_headers` と `Base64.encode` は `runes/sap-odata/client.fav` が呼び出すが
VM に未実装のため、現状では `fav run pipeline.fav` がランタイムエラーになる。
本スプリントでこれを修正し、Docker Compose で mock SAP に実際に HTTP 接続する
E2E テストを通すことをゴールとする。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v100.1.0 | `Http.get_with_headers(url, List<(String,String)>)` を vm.rs に実装 | 4279+2=4281 | 未着手 |
| v100.2.0 | `Base64.encode` を vm.rs に追加（`String.base64_encode` のエイリアス） | 4281+2=4283 | 未着手 |
| v100.3.0 | docker-compose.yml の mock サーバーを `json-server` に差し替え | 4283+2=4285 | 未着手 |
| v100.4.0 | `fav run` が SAP client Rune をランタイムエラーなく実行できることを確認するテスト | 4285+2=4287 | 未着手 |
| v100.5.0 | Docker Compose 起動 → pipeline.fav が mock に HTTP 接続して結果を返す E2E 確認 | 4287+2=4289 | 未着手 |
| v100.6.0 | OData v4 JSON レスポンスパース（`value` 配列の抽出・型変換ヘルパー） | 4289+2=4291 | 未着手 |
| v100.7.0 | SAP OData write 系（POST / PATCH / DELETE）の mock 通信確認 | 4291+2=4293 | 未着手 |
| v100.8.0 | SAP OData エラーレスポンス（`error.code` / `error.message`）の型パース | 4293+2=4295 | 未着手 |
| v100.9.0 | 安定化・コードフリーズ | 4295+2=4297 | 未着手 |
| v101.0.0 | SAP E2E Foundation 1.0 宣言 ★クリーンアップ | 4297+4=4301 | 未着手 |

---

## v100.1.0 — `Http.get_with_headers` VM 実装

`runes/sap-odata/client.fav` が呼び出す `Http.get_with_headers(url, List<(String,String)>)` が
`fav/src/backend/vm.rs` に存在しない。`fav run` 実行時にランタイムエラーになる。

既存の `Http.get_raw_headers`（`Map<String,String>` 形式）とは引数型が異なるため、
`List<(String,String)>` タプルリスト形式の新関数として追加する。

```rust
// vm.rs に追加するディスパッチ（イメージ）
"Http.get_with_headers" => {
    // args[0]: url (String)
    // args[1]: List<(String, String)> — ヘッダータプルのリスト
    let url = vm_string(args[0], "Http.get_with_headers")?;
    let mut req = ureq::get(&url);
    for tuple in header_list {
        req = req.set(&key, &val);
    }
    // ... レスポンスを Result<String, String> で返す
}
```

**修正ファイル**: `fav/src/backend/vm.rs`、`fav/src/driver.rs`（テスト 2 件追加）

---

## v100.2.0 — `Base64.encode` VM 実装

`runes/sap-odata/client.fav` の `basic_auth_header` 関数が `Base64.encode` を呼び出すが未実装。
`String.base64_encode` として実装済みの処理を `Base64.encode` エイリアスとして追加する。

```rust
// vm.rs に追加するディスパッチ（イメージ）
"Base64.encode" => {
    // String.base64_encode と同実装
    use base64::Engine;
    let s = vm_string(args[0], "Base64.encode")?;
    Ok(VMValue::Str(
        base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
    ))
}
"Base64.decode" => { /* String.base64_decode と同実装 */ }
```

**修正ファイル**: `fav/src/backend/vm.rs`、`fav/src/driver.rs`（テスト 2 件追加）

---

## v100.3.0 — docker-compose.yml mock サーバー修正

現在の `docker-compose.yml` は `@sap-ux/mockserver-main` を使用しているが、
このパッケージは実在しない（または動作しない）。
`json-server`（広く使われる npm パッケージ）に差し替えて動作する mock サーバーを構築する。

mock データ（`BusinessPartnerCollection.json` / `SalesOrderCollection.json`）は既存のものを流用する。

```yaml
# docker-compose.yml 修正後イメージ
services:
  sap-mock:
    image: node:20-alpine
    working_dir: /app
    command: >
      sh -c "npm install -g json-server &&
             json-server --watch /app/db.json --port 4004 --host 0.0.0.0"
    ports:
      - "4004:4004"
    volumes:
      - ./mock:/app
```

**修正ファイル**: `infra/e2e-demo/sap-odata/docker-compose.yml`、`infra/e2e-demo/sap-odata/mock/db.json`（新規、json-server 用統合 JSON）、`fav/src/driver.rs`

---

## v100.4.0 — SAP client Rune 実行テスト

v100.1〜v100.2 の修正により `Http.get_with_headers` / `Base64.encode` が使えるようになった。
`fav run` で `runes/sap-odata/client.fav` がランタイムエラーなく実行できることを
driver.rs のテストで確認する。

**テスト内容**:
- `sap_http_primitives_are_registered`: vm.rs に `Http.get_with_headers` / `Base64.encode` が
  ディスパッチ対象として登録されていることを文字列検索で確認
- `sap_client_fav_compiles_without_error`: `runes/sap-odata/client.fav` が
  `fav check` を通ることを確認

**修正ファイル**: `fav/src/driver.rs`

---

## v100.5.0 — Docker E2E 実証

Docker Compose を起動し、`pipeline.fav` が mock SAP サーバーに実際に HTTP 接続して
JSON レスポンスを受け取ることを確認する。

**手順**:
1. `docker compose -f infra/e2e-demo/sap-odata/docker-compose.yml up -d`
2. mock が起動するまで待機（`http://localhost:4004/BusinessPartnerCollection` が返ること）
3. `fav run infra/e2e-demo/sap-odata/pipeline.fav` を実行
4. エラーなく終了することを確認

**テスト内容** (driver.rs):
- `sap_e2e_mock_data_files_exist`: mock JSON ファイルが存在することを確認
- `sap_e2e_docker_compose_has_json_server`: docker-compose.yml に `json-server` が含まれることを確認

**修正ファイル**: `fav/src/driver.rs`、`infra/e2e-demo/sap-odata/scripts/run-e2e.sh`（新規）

---

## v100.6.0 — OData v4 JSON レスポンスパース

SAP OData v4 のレスポンスは以下の形式で返る。現状は生 JSON 文字列のまま扱っている。
`value` 配列の抽出と基本的な型変換ヘルパーを追加する。

```favnir
-- OData v4 レスポンス形式
-- { "@odata.context": "...", "value": [...] }

-- ヘルパー関数（runes/sap-odata/parser.fav に追加）
fn odata_extract_value(json: String) -> Result<String, String> {
    -- JSON から "value" 配列を抽出して返す
    Json.get_array(Json.parse(json), "value")
}

fn odata_parse_count(json: String) -> Result<Int, String> {
    -- "@odata.count" フィールドを取得
    Json.get_int(Json.parse(json), "@odata.count")
}
```

**修正ファイル**: `runes/sap-odata/parser.fav`（新規）、`fav/src/driver.rs`

---

## v100.7.0 — SAP OData write 系 mock 通信確認

POST（作成）/ PATCH（更新）/ DELETE（削除）の HTTP 通信を mock で確認する。
`Http.post_with_headers` / `Http.patch_with_headers` / `Http.delete_with_headers` が
未実装の場合は v100.7.0 で追加する。

```favnir
-- POST: BusinessPartner 作成
fn create_business_partner(cfg: SapConfig, bp: NewBusinessPartner) -> Result<String, String> {
    bind url  <- Result.ok(String.concat([cfg.base_url, "/A_BusinessPartner"]))
    bind body <- Json.encode(bp)
    Http.post_with_headers(url, body, [
        ("Authorization", basic_auth_header(cfg.username, cfg.password)),
        ("Content-Type",  "application/json"),
        ("Accept",        "application/json")
    ])
}
```

**修正ファイル**: `fav/src/backend/vm.rs`（`Http.post_with_headers` 等の追加）、`runes/sap-odata/client.fav`、`fav/src/driver.rs`

---

## v100.8.0 — SAP OData エラーレスポンスパース

SAP OData がエラーを返す場合、以下の形式でレスポンスが返る。
これを `SapApiError` 型にパースするヘルパーを追加する。

```favnir
-- SAP OData エラーレスポンス形式
-- { "error": { "code": "...","message": { "lang": "en", "value": "..." } } }

type SapApiError = {
    code:    String,
    message: String
}

fn parse_sap_error(json: String) -> Result<SapApiError, String> {
    bind parsed  <- Json.parse(json)
    bind error   <- Json.get_object(parsed, "error")
    bind code    <- Json.get_string(error, "code")
    bind msg_obj <- Json.get_object(error, "message")
    bind message <- Json.get_string(msg_obj, "value")
    Result.ok(SapApiError { code: code, message: message })
}
```

**修正ファイル**: `runes/sap-odata/parser.fav`（追記）、`runes/sap-odata/types.fav`（`SapApiError` 追加）、`fav/src/driver.rs`

---

## v100.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,297 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- `versions/current.md` の次バージョン欄を v101.0.0 に更新

---

## v101.0.0 — SAP E2E Foundation 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「Favnir の SAP 統合が、初めて実際に動いた。
>
>  `Http.get_with_headers` が SAP OData に接続し、
>  `Base64.encode` が認証ヘッダーを生成し、
>  Docker の mock サーバーが JSON を返し、
>  `fav run pipeline.fav` がエラーなく終わる。
>
>  設計から動作へ——これが、SAP E2E Foundation 1.0 である。」

**v101000_tests（4 テスト）**:
- `cargo_toml_version_is_101_0_0`
- `changelog_has_v101_0_0`
- `milestone_has_sap_e2e_foundation`
- `sap_mock_docker_compose_uses_json_server`

**クリーンアップ**:
- `cargo clean` 実施
- `fav/tmp/hello.fav` 復元確認
- `cargo test` で 4,301 tests, 0 failures を再確認
- `cargo build` で `./target/debug/fav` を再生成

**修正ファイル**: `fav/Cargo.toml`（version → 101.0.0）、`MILESTONE.md`、`CHANGELOG.md`、`fav/src/driver.rs`、`versions/current.md`

---

## スプリント終了時の確認

- [ ] 4,301 tests, 0 failures
- [ ] `Http.get_with_headers` が vm.rs に実装されている
- [ ] `Base64.encode` が vm.rs に実装されている
- [ ] docker-compose.yml が `json-server` を使用している
- [ ] `fav run infra/e2e-demo/sap-odata/pipeline.fav` がエラーなく終了する（Docker 起動時）
- [ ] `cargo clean` を実施する
- [ ] `cargo test` で 4,301 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v101.0.0 に更新
- [ ] `MILESTONE.md` に v101.0.0 エントリを追加
- [ ] `roadmap-v100.1-v105.0.md` の Sprint 1 状態を「完了」に更新
