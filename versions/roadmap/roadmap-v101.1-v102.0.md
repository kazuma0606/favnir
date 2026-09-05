# Roadmap v101.1.0 〜 v102.0.0 — SAP HTTP Layer 1.0

Date: 2026-09-05
Status: 未着手

マスターロードマップ: [roadmap-v100.1-v105.0.md](roadmap-v100.1-v105.0.md)

---

## 前提

- 直前完了: v101.0.0「SAP E2E Foundation 1.0 宣言」（tests = 4,301）
- 本スプリントは SAP Real-World Platform Era の第 2 スプリント
- 目標: v102.0.0「SAP HTTP Layer 1.0 宣言」（tests = 4,323）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v101.0.0 になっていることを確認する
- `fav/src/driver.rs` に `mod v101000_tests` が存在することを確認する
- `fav/Cargo.toml` の version が `101.0.0` であることを確認する
- `Http.get_with_headers` が vm.rs に実装されていることを確認する（v100.1.0 完了済みの証拠）
- `docker compose up` で mock サーバーが起動することを確認する

### スプリントの性格

SAP Real-World Platform Era の**HTTP クライアント完成スプリント**。

Sprint 1 で「動く」状態を達成した。Sprint 2 では本番 SAP への接続に必要な
CSRF トークン・OAuth2 Bearer トークン・OData ページネーションを実装し、
「本物の SAP HTTP クライアント」として完成させる。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v101.1.0 | CSRF トークン取得・付与（SAP write 操作の必須要件） | 4301+2=4303 | 未着手 |
| v101.2.0 | OAuth2 Bearer トークン認証（Basic → Bearer 切り替え） | 4303+2=4305 | 未着手 |
| v101.3.0 | OData ページネーション自動追跡（`@odata.nextLink` ループ） | 4305+2=4307 | 未着手 |
| v101.4.0 | タイムアウト / リトライ設定（`SapClientConfig` 拡張） | 4307+2=4309 | 未着手 |
| v101.5.0 | `fav sap ping` コマンド（SAP 接続テスト・認証確認） | 4309+2=4311 | 未着手 |
| v101.6.0 | `fav sap-mock` コマンド（ローカル mock サーバー起動） | 4311+2=4313 | 未着手 |
| v101.7.0 | `Http.post_with_headers` / `Http.patch_with_headers` / `Http.delete_with_headers` 追加 | 4313+2=4315 | 未着手 |
| v101.8.0 | サイトドキュメント（SAP HTTP クライアント完全ガイド） | 4315+2=4317 | 未着手 |
| v101.9.0 | 安定化・コードフリーズ | 4317+2=4319 | 未着手 |
| v102.0.0 | SAP HTTP Layer 1.0 宣言 ★クリーンアップ | 4319+4=4323 | 未着手 |

---

## v101.1.0 — CSRF トークン

SAP OData では POST / PUT / PATCH / DELETE の前に CSRF トークンを取得する必要がある。
`X-CSRF-Token: Fetch` ヘッダーで GET リクエストを送り、レスポンスヘッダーの
`X-CSRF-Token` 値を後続の write リクエストに付与する。

```favnir
-- CSRF トークン取得（runes/sap-odata/csrf.fav に追加）
fn fetch_csrf_token(cfg: SapConfig) -> Result<String, String> {
    bind resp <- Http.get_with_headers_raw(
        String.concat([cfg.base_url, "/"]),
        [
            ("Authorization", basic_auth_header(cfg.username, cfg.password)),
            ("X-CSRF-Token", "Fetch"),
            ("sap-client",   cfg.client)
        ]
    )
    -- レスポンスヘッダーから X-CSRF-Token を抽出
    Http.get_response_header(resp, "X-CSRF-Token")
}

-- CSRF トークンを付与した POST
fn odata_post_with_csrf(cfg: SapConfig, entity_set: String, body: String) -> Result<String, String> {
    bind token <- fetch_csrf_token(cfg)
    Http.post_with_headers(
        String.concat([cfg.base_url, "/", entity_set]),
        body,
        [
            ("Authorization", basic_auth_header(cfg.username, cfg.password)),
            ("X-CSRF-Token",  token),
            ("Content-Type",  "application/json"),
            ("Accept",        "application/json")
        ]
    )
}
```

**修正ファイル**: `runes/sap-odata/csrf.fav`（新規）、`fav/src/backend/vm.rs`（`Http.get_with_headers_raw` 追加）、`fav/src/driver.rs`

---

## v101.2.0 — OAuth2 Bearer トークン認証

Basic 認証（ユーザー名/パスワード）に加え、SAP BTP の OAuth2 Bearer トークン認証に対応する。
`BtpCredential` 型（v99.1.0 で定義済み）を使ってトークンを取得し、
以後の SAP API 呼び出しに Bearer ヘッダーとして付与する。

```favnir
-- Bearer トークンで認証ヘッダーを生成
fn bearer_auth_header(token: String) -> String {
    String.concat(["Bearer ", token])
}

-- OAuth2 トークン取得（client_credentials フロー）
fn acquire_oauth2_token(cred: BtpCredential) -> Result<BtpToken, String> {
    bind form_body <- Result.ok(String.concat([
        "grant_type=client_credentials&client_id=", cred.client_id,
        "&client_secret=", cred.client_secret
    ]))
    bind resp <- Http.post_with_headers(cred.token_url, form_body, [
        ("Content-Type", "application/x-www-form-urlencoded")
    ])
    bind parsed <- Json.parse(resp)
    bind token  <- Json.get_string(parsed, "access_token")
    bind exp    <- Json.get_int(parsed, "expires_in")
    Result.ok(BtpToken { access_token: token, expires_in: exp, token_type: "Bearer" })
}
```

**修正ファイル**: `runes/sap-odata/btp_auth.fav`（新規）、`fav/src/driver.rs`

---

## v101.3.0 — OData ページネーション自動追跡

SAP OData は大量データ取得時に `@odata.nextLink` でページネーションを行う。
現状は 1 ページ目のみ取得される。全ページを自動的に追跡するヘルパーを追加する。

```favnir
-- 全ページを自動取得（runes/sap-odata/pagination.fav に追加）
fn odata_fetch_all<T>(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    parse_fn:   fn(String) -> Result<List<T>, String>
) -> Result<List<T>, String> {
    bind first_resp  <- odata_list(cfg, entity_set, params)
    bind first_items <- parse_fn(first_resp)
    bind next_link   <- Result.ok(odata_next_link(first_resp))
    -- 再帰的に次ページを取得（nextLink が None になるまで）
    odata_collect_pages(cfg, first_items, next_link, parse_fn)
}

fn odata_next_link(json: String) -> Option<String> {
    -- "@odata.nextLink" フィールドの有無を確認
    Json.get_string_opt(Json.parse_unsafe(json), "@odata.nextLink")
}
```

**修正ファイル**: `runes/sap-odata/pagination.fav`（新規）、`fav/src/driver.rs`

---

## v101.4.0 — タイムアウト / リトライ設定

`SapConfig` にタイムアウトとリトライ設定を追加し、本番環境での安定性を高める。

```favnir
-- SapConfig を拡張（runes/sap-odata/types.fav）
type SapConfig = {
    base_url:   String,
    client:     String,
    username:   String,
    password:   String,
    timeout_ms: Int,        -- リクエストタイムアウト（ミリ秒）。デフォルト 30000
    max_retry:  Int         -- リトライ回数。デフォルト 3
}

-- デフォルト設定で SapConfig を生成するヘルパー
fn sap_config_default(base_url: String, client: String, user: String, pass: String) -> SapConfig {
    SapConfig {
        base_url:   base_url,
        client:     client,
        username:   user,
        password:   pass,
        timeout_ms: 30000,
        max_retry:  3
    }
}
```

**修正ファイル**: `runes/sap-odata/types.fav`（`SapConfig` 拡張）、`runes/sap-odata/client.fav`（タイムアウト付き HTTP 呼び出しに更新）、`fav/src/driver.rs`

---

## v101.5.0 — `fav sap ping` コマンド

SAP への接続・認証確認を一発で行う `fav sap ping` サブコマンドを追加する。
`fav.toml` の `[sap]` セクションを読み込み、実際に HTTP GET を送って疎通確認する。

```bash
$ fav sap ping
Checking SAP connection...
  URL:    https://my.sap.example.com/sap/opu/odata/sap
  Client: 100
  Auth:   Basic

  GET /A_BusinessPartner?$top=1 ... 200 OK (245ms)

SAP connection: OK
```

**修正ファイル**: `fav/src/main.rs`（`sap` サブコマンド追加）、`fav/src/driver.rs`

---

## v101.6.0 — `fav sap-mock` コマンド

ローカル開発用に mock SAP サーバーをワンコマンドで起動する `fav sap-mock` を追加する。
`json-server` を使い、`infra/e2e-demo/sap-odata/mock/db.json` を自動的にサーブする。

```bash
$ fav sap-mock --port 4004
Starting SAP mock server...
  Data: infra/e2e-demo/sap-odata/mock/db.json
  Port: 4004

  Endpoints:
    GET  http://localhost:4004/BusinessPartnerCollection
    GET  http://localhost:4004/SalesOrderCollection
    POST http://localhost:4004/BusinessPartnerCollection

SAP mock server running. Press Ctrl+C to stop.
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v101.7.0 — `Http.post_with_headers` / `Http.patch_with_headers` / `Http.delete_with_headers`

v100.1.0 で `Http.get_with_headers` を追加したが、write 系（POST / PATCH / DELETE）の
`with_headers` バリアントが未実装。SAP の書き込み操作に必要なため追加する。

```rust
// vm.rs に追加するディスパッチ（イメージ）
"Http.post_with_headers" => {
    // args[0]: url, args[1]: body, args[2]: List<(String, String)>
}
"Http.patch_with_headers" => {
    // args[0]: url, args[1]: body, args[2]: List<(String, String)>
}
"Http.delete_with_headers" => {
    // args[0]: url, args[1]: List<(String, String)>
}
```

**修正ファイル**: `fav/src/backend/vm.rs`、`runes/sap-odata/client.fav`（write 系関数を更新）、`fav/src/driver.rs`

---

## v101.8.0 — サイトドキュメント

SAP HTTP クライアントの完全ガイドを作成する。

**新規作成**:
- `site/content/docs/guides/sap-http-client.mdx` — 認証・CSRF・ページネーション・リトライのガイド

**修正ファイル**: 上記 1 ファイル（新規）、`fav/src/driver.rs`

---

## v101.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,319 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- `versions/current.md` の次バージョン欄を v102.0.0 に更新

---

## v102.0.0 — SAP HTTP Layer 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「Favnir の SAP クライアントが、本物になった。
>
>  CSRF トークンが write を守り、
>  OAuth2 Bearer が認証し、
>  `@odata.nextLink` を追って全データを取り尽くす。
>
>  これが、SAP HTTP Layer 1.0 である。」

**v102000_tests（4 テスト）**:
- `cargo_toml_version_is_102_0_0`
- `changelog_has_v102_0_0`
- `sap_http_client_guide_exists`
- `sap_pagination_fav_exists`

**クリーンアップ**:
- `cargo clean` 実施
- `cargo test` で 4,323 tests, 0 failures を再確認
- `cargo build` で `./target/debug/fav` を再生成

---

## スプリント終了時の確認

- [ ] 4,323 tests, 0 failures
- [ ] `fav sap ping` コマンドが動作する
- [ ] `fav sap-mock` コマンドが動作する
- [ ] CSRF トークン取得・付与が実装されている（`runes/sap-odata/csrf.fav`）
- [ ] OAuth2 Bearer 認証が実装されている（`runes/sap-odata/btp_auth.fav`）
- [ ] OData ページネーション追跡が実装されている（`runes/sap-odata/pagination.fav`）
- [ ] `cargo clean` を実施する
- [ ] `cargo test` で 4,323 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v102.0.0 に更新
- [ ] `MILESTONE.md` に v102.0.0 エントリを追加
- [ ] `roadmap-v100.1-v105.0.md` の Sprint 2 状態を「完了」に更新
