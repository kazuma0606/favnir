# Roadmap v102.1.0 〜 v103.0.0 — SAP API Exposure 1.0

Date: 2026-09-05
Status: 未着手

マスターロードマップ: [roadmap-v100.1-v105.0.md](roadmap-v100.1-v105.0.md)

---

## 前提

- 直前完了: v102.0.0「SAP HTTP Layer 1.0 宣言」（tests = 4,323）
- 本スプリントは SAP Real-World Platform Era の第 3 スプリント
- 目標: v103.0.0「SAP API Exposure 1.0 宣言」（tests = 4,345）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v102.0.0 になっていることを確認する
- `fav/src/driver.rs` に `mod v102000_tests` が存在することを確認する
- `fav/Cargo.toml` の version が `102.0.0` であることを確認する
- `runes/sap-odata/csrf.fav` が存在することを確認する（v101.1.0 完了済みの証拠）
- `runes/sap-odata/pagination.fav` が存在することを確認する（v101.3.0 完了済みの証拠）

### スプリントの性格

SAP Real-World Platform Era の**外部公開スプリント**。

Sprint 1・2 で「SAP からデータを取得できる」状態になった。
Sprint 3 では取得したデータを REST API として外部に公開する仕組みを追加する。
`fav serve` でローカル REST サーバーを起動し、Lambda ハンドラとして自動生成し、
OpenAPI スキーマを出力する。「SAP → Favnir → 外部アプリ」の縦断を実現する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v102.1.0 | `api` キーワード定義（REST エンドポイント宣言構文） | 4323+2=4325 | 未着手 |
| v102.2.0 | `fav serve` — ローカル REST サーバー起動 | 4325+2=4327 | 未着手 |
| v102.3.0 | Lambda ハンドラ自動生成（`fav build --target lambda-api`） | 4327+2=4329 | 未着手 |
| v102.4.0 | OpenAPI スキーマ自動出力（`fav api-spec --format openapi`） | 4329+2=4331 | 未着手 |
| v102.5.0 | Bearer トークン認証ミドルウェア（API ゲートウェイ連携） | 4331+2=4333 | 未着手 |
| v102.6.0 | レスポンスページネーション（`Page<T>` → `Link: <next>` ヘッダー） | 4333+2=4335 | 未着手 |
| v102.7.0 | E2E デモ（SAP BP → `fav serve` → curl 取得 → Lambda デプロイ） | 4335+2=4337 | 未着手 |
| v102.8.0 | サイトドキュメント（SAP API Exposure ガイド） | 4337+2=4339 | 未着手 |
| v102.9.0 | 安定化・コードフリーズ | 4339+2=4341 | 未着手 |
| v103.0.0 | SAP API Exposure 1.0 宣言 ★クリーンアップ | 4341+4=4345 | 未着手 |

---

## v102.1.0 — `api` キーワード定義

Favnir の型から REST エンドポイントを宣言する `api` キーワードを追加する。
パーサー・AST・コンパイラへの追加を行う。

```favnir
-- REST エンドポイント宣言（構文定義）
api BusinessPartnerApi {
    GET  /partners                  -> Page<BusinessPartner>
    GET  /partners/{id: String}     -> BusinessPartner
    POST /partners                  -> BusinessPartner  with NewBusinessPartner
}

-- api 宣言は pipeline と接続される
api BusinessPartnerApi {
    GET /partners -> Page<BusinessPartner>
        = fn(ctx: AppCtx, page: Int, top: Int) -> Result<Page<BusinessPartner>, String> {
            bind partners <- ctx.sap.business_partners(BusinessPartnerFilter {
                country: Option.none(), category: Option.none(),
                changed_after: Option.none(), top: Option.some(top)
            })
            Result.ok(Page { value: partners, total: List.length(partners) })
        }
}
```

**修正ファイル**: `fav/src/frontend/lexer.rs`（`api` トークン追加）、`fav/src/frontend/parser.rs`（api 宣言パース）、`fav/src/ast.rs`（`ApiDecl` ノード追加）、`fav/src/driver.rs`

---

## v102.2.0 — `fav serve`

`api` 宣言から自動的にローカル REST サーバーを起動する `fav serve` コマンドを追加する。
ureq ではなく `tiny_http` クレートを使い、リクエストをディスパッチする。

```bash
$ fav serve api.fav --port 8080
Starting Favnir API server...
  File: api.fav
  Port: 8080

  Routes:
    GET  http://localhost:8080/partners
    GET  http://localhost:8080/partners/{id}
    POST http://localhost:8080/partners

Server running. Press Ctrl+C to stop.
```

```bash
$ curl http://localhost:8080/partners?top=5
{
  "value": [...],
  "total": 5
}
```

**修正ファイル**: `fav/src/main.rs`（`serve` サブコマンド追加）、`fav/Cargo.toml`（`tiny_http` 依存追加）、`fav/src/driver.rs`

---

## v102.3.0 — Lambda ハンドラ自動生成

`api` 宣言から AWS Lambda ハンドラ（`handler.rs`）を自動生成する
`fav build --target lambda-api` オプションを追加する。

```bash
$ fav build --target lambda-api api.fav --out ./lambda-handler/
Generating Lambda handler...
  File:   api.fav
  Output: ./lambda-handler/

  Generated:
    ./lambda-handler/src/main.rs    -- Lambda エントリポイント
    ./lambda-handler/Cargo.toml     -- 依存関係
    ./lambda-handler/Makefile       -- デプロイ用

$ cd lambda-handler && make deploy
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v102.4.0 — OpenAPI スキーマ自動出力

`api` 宣言から OpenAPI 3.0 スキーマを自動生成する `fav api-spec` コマンドを追加する。
Favnir の型定義をそのまま JSON Schema に変換する。

```bash
$ fav api-spec api.fav --format openapi --out openapi.json
Generated: openapi.json

$ cat openapi.json
{
  "openapi": "3.0.0",
  "info": { "title": "BusinessPartnerApi", "version": "1.0.0" },
  "paths": {
    "/partners": {
      "get": {
        "summary": "List BusinessPartner",
        "parameters": [...],
        "responses": { "200": { "schema": { ... } } }
      }
    }
  }
}
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v102.5.0 — Bearer トークン認証ミドルウェア

`fav serve` で起動したサーバーに Bearer トークン認証を追加する。
`Authorization: Bearer <token>` ヘッダーを検証し、不正なリクエストを 401 で返す。

```favnir
-- api 宣言に認証設定を追加
api BusinessPartnerApi {
    auth: Bearer   -- Authorization: Bearer <token> を必須化
    GET /partners -> Page<BusinessPartner> = ...
}
```

```bash
$ curl -H "Authorization: Bearer mytoken" http://localhost:8080/partners
# 200 OK

$ curl http://localhost:8080/partners
# 401 Unauthorized
```

**修正ファイル**: `fav/src/frontend/parser.rs`（`auth` フィールドパース）、`fav/src/ast.rs`、`fav/src/driver.rs`

---

## v102.6.0 — レスポンスページネーション

`Page<T>` を返す API エンドポイントが OData スタイルのページネーションヘッダーを
自動的に付与するようにする。`Link: <next>` ヘッダーと `X-Total-Count` ヘッダーを追加する。

```favnir
type Page<T> = {
    value:     List<T>,
    total:     Int,
    next_link: Option<String>
}
```

```bash
$ curl -v http://localhost:8080/partners?top=10&skip=0
< Link: <http://localhost:8080/partners?top=10&skip=10>; rel="next"
< X-Total-Count: 150
```

**修正ファイル**: `fav/src/driver.rs`、`runes/sap-odata/types.fav`（`Page<T>` 型定義の整合確認）

---

## v102.7.0 — E2E デモ

SAP mock → `fav serve` → curl → Lambda デプロイの縦断 E2E デモを作成する。

**デモ手順**:
1. `fav sap-mock --port 4004` で mock SAP 起動
2. `fav serve infra/e2e-demo/sap-odata/api.fav --port 8080` で API サーバー起動
3. `curl http://localhost:8080/partners` で取得確認
4. `fav build --target lambda-api infra/e2e-demo/sap-odata/api.fav` で Lambda ハンドラ生成
5. `fav api-spec infra/e2e-demo/sap-odata/api.fav --format openapi` でスキーマ確認

**新規作成**: `infra/e2e-demo/sap-odata/api.fav`（API 宣言ファイル）

**修正ファイル**: `fav/src/driver.rs`

---

## v102.8.0 — サイトドキュメント

**新規作成**:
- `site/content/docs/guides/sap-api-exposure.mdx` — SAP データを REST API として公開するガイド

**修正ファイル**: `fav/src/driver.rs`

---

## v102.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,341 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- `versions/current.md` の次バージョン欄を v103.0.0 に更新

---

## v103.0.0 — SAP API Exposure 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「SAP のデータが、REST API になった。
>
>  `api` 宣言が型からエンドポイントを生み、
>  `fav serve` がローカルで動き、
>  Lambda ハンドラが自動生成され、
>  OpenAPI スキーマが外部ツールとつながる。
>
>  SAP → Favnir → 外の世界——これが、SAP API Exposure 1.0 である。」

**v103000_tests（4 テスト）**:
- `cargo_toml_version_is_103_0_0`
- `changelog_has_v103_0_0`
- `sap_api_exposure_guide_exists`
- `sap_e2e_api_fav_exists`

**クリーンアップ**:
- `cargo clean` 実施
- `cargo test` で 4,345 tests, 0 failures を再確認
- `cargo build` で `./target/debug/fav` を再生成

---

## スプリント終了時の確認

- [ ] 4,345 tests, 0 failures
- [ ] `api` キーワードがパーサーで解析できる
- [ ] `fav serve api.fav` がローカル REST サーバーを起動する
- [ ] `fav build --target lambda-api` が Lambda ハンドラを生成する
- [ ] `fav api-spec --format openapi` が OpenAPI JSON を出力する
- [ ] `infra/e2e-demo/sap-odata/api.fav` が存在する
- [ ] `cargo clean` を実施する
- [ ] `cargo test` で 4,345 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v103.0.0 に更新
- [ ] `MILESTONE.md` に v103.0.0 エントリを追加
- [ ] `roadmap-v100.1-v105.0.md` の Sprint 3 状態を「完了」に更新
