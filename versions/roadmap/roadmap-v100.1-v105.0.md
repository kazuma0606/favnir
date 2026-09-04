# Favnir ロードマップ v100.1〜v105.0 — SAP Real-World Platform Era

Date: 2026-09-05
Status: 計画中（v100.0.0 完了時点）

---

## 背景と方針

v100.0.0「Favnir SAP Platform 1.0」をもって、SAP 統合の**設計フェーズ**が完成した。
型定義・Rune シグネチャ・pipeline ファイル・ドキュメントが揃い、
「型で語れる SAP 統合」として完成した。

しかし v100.0.0 には**構造的な未解決課題**が残っている。

**課題 1 — VM プリミティブの欠落（最優先）**
`runes/sap-odata/client.fav` が呼び出す `Http.get_with_headers` と `Base64.encode` が
VM（`vm.rs`）に実装されていない。
現時点で `fav run pipeline.fav` を実行しても**ランタイムエラーになる**。

**課題 2 — E2E テストが未実施**
`infra/e2e-demo/sap-odata/docker-compose.yml` の mock サーバーに使用している
`@sap-ux/mockserver-main` は実在しない npm パッケージ。
Docker Compose を起動して SAP mock と実際に HTTP 通信した実績がない。

**課題 3 — SAP HTTP クライアントの本番機能不足**
CSRF トークン（SAP write 操作の必須要件）/ OAuth2 Bearer トークン /
OData ページネーション（`$skiptoken` / `nextLink`）が未実装。
本番 SAP への投入には HTTP クライアント層の完成が必要。

**課題 4 — SAP データの「外に出す」手段がない**
設計した SAP パイプラインは「SAP から取得して内部処理」どまり。
取得したデータを REST API として公開する手段（`fav serve` / Lambda ハンドラ生成）がない。

**課題 5 — データ製品（Data Product）概念の欠如**
SAP データを「誰が所有し、どの SLA で提供し、どのスキーマで公開するか」を
コードで宣言する仕組みがない。

v100.1〜v105.0 では、これら 5 課題を段階的に解決し、
Favnir を「SAP データが実際に動き、外に出て、管理される、本物のプラットフォーム」として宣言する。

```
v101.0 — SAP E2E Foundation 1.0  : 「Http.get_with_headers が動き、Docker E2E が通る」
v102.0 — SAP HTTP Layer 1.0      : 「CSRF / OAuth2 / ページネーション — 本物の SAP クライアント」
v103.0 — SAP API Exposure 1.0   : 「SAP データを REST API として型安全に外に出す」
v104.0 — SAP Data Products 1.0  : 「SLA・オーナー・スキーマをコードで宣言し管理する」
v105.0 — SAP Real-World Platform 1.0 宣言 : 「設計から運用まで — 本物の SAP プラットフォーム」
```

### 設計方針

- **動くことが最優先**: 型設計の拡充より「既存コードが実際に動く」状態を先に達成する。
  Sprint 1（v100.1〜v101.0）は新機能追加よりバグ修正・E2E 検証を優先する。
- **mock ファースト**: 本番 SAP 環境は不要。`json-server` ベースの Docker mock で
  全 E2E が通ることを CI の合格基準とする。
- **ctx パターン維持**: 新機能はすべて `AppCtx` の interface フィールドとして追加する。
  `ctx.api.*` / `ctx.catalog.*` 等の追加も同方針に従う。
- **既存コードの修正**: `runes/sap-odata/client.fav` の `Base64.encode` は
  `String.base64_encode` に統一するか、VM に `Base64.encode` エイリアスを追加する。

### 現状評価（v100.0.0 時点）

| カテゴリ | 状態 | 評価 |
|---|---|---|
| SAP 型定義（BP / SO / Material / PO / JE 等） | 完成（設計レベル） | ★★★★★ |
| Rune シグネチャ / pipeline ファイル | 完成（設計レベル） | ★★★★★ |
| `Http.get_with_headers` VM 実装 | **未実装（ランタイムエラー）** | ★☆☆☆☆ |
| `Base64.encode` VM 実装 | **未実装（ランタイムエラー）** | ★☆☆☆☆ |
| Docker E2E テスト（mock サーバー起動 + HTTP 通信） | **未実施** | ★☆☆☆☆ |
| CSRF トークン / OAuth2 Bearer | **未実装** | ★☆☆☆☆ |
| OData ページネーション（nextLink 自動追跡） | **未実装** | ★★☆☆☆ |
| REST API 公開（`fav serve` / Lambda ハンドラ生成） | **未実装** | ★☆☆☆☆ |
| Data Product 宣言 / カタログ管理 | **未実装** | ★☆☆☆☆ |
| OpenAPI スキーマ自動出力 | **未実装** | ★☆☆☆☆ |

---

## テスト数推移（本スプリント全体）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v100.0.0（ベース） | 4,279 | — |
| v100.1.0〜v100.9.0 | +2 × 9 = +18 | 4,297 |
| v101.0.0（宣言） | +4 | 4,301 |
| v101.1.0〜v101.9.0 | +2 × 9 = +18 | 4,319 |
| v102.0.0（宣言） | +4 | 4,323 |
| v102.1.0〜v102.9.0 | +2 × 9 = +18 | 4,341 |
| v103.0.0（宣言） | +4 | 4,345 |
| v103.1.0〜v103.9.0 | +2 × 9 = +18 | 4,363 |
| v104.0.0（宣言） | +4 | 4,367 |
| v104.1.0〜v104.9.0 | +2 × 9 = +18 | 4,385 |
| v105.0.0（宣言） | +4 | 4,389 |

**本スプリント合計**: +110 tests（4,279 → 4,389）

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 1: SAP E2E Foundation（v100.1〜v101.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 設計から動作へ — VM プリミティブ修正と Docker E2E 実証。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v100.1.0 | `Http.get_with_headers(url, List<(String,String)>)` を vm.rs に実装 | 4279+2=4281 | 未着手 |
| v100.2.0 | `Base64.encode` を vm.rs に追加（`String.base64_encode` のエイリアス） | 4281+2=4283 | 未着手 |
| v100.3.0 | `docker-compose.yml` mock サーバーを `json-server` に差し替え（実在パッケージ） | 4283+2=4285 | 未着手 |
| v100.4.0 | `fav run runes/sap-odata/client.fav` がランタイムエラーなく通ることを確認するテスト追加 | 4285+2=4287 | 未着手 |
| v100.5.0 | Docker Compose 起動 → `pipeline.fav` が mock に HTTP 接続して結果を返す E2E 確認 | 4287+2=4289 | 未着手 |
| v100.6.0 | OData v4 JSON レスポンスパース（`value` 配列の抽出・型変換） | 4289+2=4291 | 未着手 |
| v100.7.0 | SAP OData write 系（POST / PATCH / DELETE）の mock 通信確認 | 4291+2=4293 | 未着手 |
| v100.8.0 | SAP OData エラーレスポンス（`error.code` / `error.message`）の型パース | 4293+2=4295 | 未着手 |
| v100.9.0 | 安定化・コードフリーズ | 4295+2=4297 | 未着手 |
| v101.0.0 | SAP E2E Foundation 1.0 宣言 ★クリーンアップ | 4297+4=4301 | 未着手 |

詳細: [roadmap-v100.1-v101.0.md](roadmap-v100.1-v101.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 2: SAP HTTP Layer（v101.1〜v102.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: CSRF トークン / OAuth2 / ページネーション — 本物の SAP HTTP クライアント完成。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v101.1.0 | CSRF トークン取得・付与（`X-CSRF-Token: Fetch` → write 操作ヘッダー添付） | 4301+2=4303 | 未着手 |
| v101.2.0 | OAuth2 Bearer トークン認証（Basic → Bearer 切り替え、`BtpCredential` 対応） | 4303+2=4305 | 未着手 |
| v101.3.0 | OData ページネーション自動追跡（`@odata.nextLink` / `$skiptoken` ループ） | 4305+2=4307 | 未着手 |
| v101.4.0 | タイムアウト / リトライ設定（`SapClientConfig` に `timeout_ms` / `retry` フィールド追加） | 4307+2=4309 | 未着手 |
| v101.5.0 | `fav sap ping` コマンド（SAP 接続テスト・認証確認） | 4309+2=4311 | 未着手 |
| v101.6.0 | `fav sap-mock` コマンド（ローカル mock サーバー起動 / `json-server` ラッパー） | 4311+2=4313 | 未着手 |
| v101.7.0 | `Http.post_with_headers` / `Http.patch_with_headers` / `Http.delete_with_headers` 追加 | 4313+2=4315 | 未着手 |
| v101.8.0 | サイトドキュメント（SAP HTTP クライアント完全ガイド） | 4315+2=4317 | 未着手 |
| v101.9.0 | 安定化・コードフリーズ | 4317+2=4319 | 未着手 |
| v102.0.0 | SAP HTTP Layer 1.0 宣言 ★クリーンアップ | 4319+4=4323 | 未着手 |

詳細: [roadmap-v101.1-v102.0.md](roadmap-v101.1-v102.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 3: SAP API Exposure（v102.1〜v103.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SAP データを REST API として型安全に外に出す。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v102.1.0 | `api` キーワード定義（REST エンドポイント宣言構文） | 4323+2=4325 | 未着手 |
| v102.2.0 | `fav serve` — ローカル REST サーバー起動（`api` 宣言から自動生成） | 4325+2=4327 | 未着手 |
| v102.3.0 | Lambda ハンドラ自動生成（`fav build --target lambda-api`） | 4327+2=4329 | 未着手 |
| v102.4.0 | OpenAPI スキーマ自動出力（`fav api-spec --format openapi`） | 4329+2=4331 | 未着手 |
| v102.5.0 | Bearer トークン認証ミドルウェア（API ゲートウェイ連携） | 4331+2=4333 | 未着手 |
| v102.6.0 | レスポンスのページネーション（`Page<T>` → `Link: <next>` ヘッダー） | 4333+2=4335 | 未着手 |
| v102.7.0 | E2E デモ（SAP BP → `fav serve` → curl で取得 → Lambda デプロイ） | 4335+2=4337 | 未着手 |
| v102.8.0 | サイトドキュメント（SAP API Exposure ガイド） | 4337+2=4339 | 未着手 |
| v102.9.0 | 安定化・コードフリーズ | 4339+2=4341 | 未着手 |
| v103.0.0 | SAP API Exposure 1.0 宣言 ★クリーンアップ | 4341+4=4345 | 未着手 |

詳細: [roadmap-v102.1-v103.0.md](roadmap-v102.1-v103.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 4: SAP Data Products（v103.1〜v104.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SLA・オーナー・スキーマをコードで宣言し、SAP データを製品として管理する。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v103.1.0 | `data_product` キーワード（オーナー・SLA・スキーマの宣言構文） | 4345+2=4347 | 未着手 |
| v103.2.0 | `fav catalog list` — データ製品一覧表示 | 4347+2=4349 | 未着手 |
| v103.3.0 | データ製品間スキーマ契約チェック（`fav mesh validate`） | 4349+2=4351 | 未着手 |
| v103.4.0 | SLA 違反検出（`SlaViolation` 型 + `fav mesh check-sla`） | 4351+2=4353 | 未着手 |
| v103.5.0 | データ製品バージョン管理（`version` フィールド / 後方互換性チェック） | 4353+2=4355 | 未着手 |
| v103.6.0 | カタログ JSON 出力（OpenMetadata / DataHub 互換フォーマット） | 4355+2=4357 | 未着手 |
| v103.7.0 | E2E デモ（SAP BP / SalesOrder を data_product 化 → catalog 登録 → SLA チェック） | 4357+2=4359 | 未着手 |
| v103.8.0 | サイトドキュメント（Data Products ガイド） | 4359+2=4361 | 未着手 |
| v103.9.0 | 安定化・コードフリーズ | 4361+2=4363 | 未着手 |
| v104.0.0 | SAP Data Products 1.0 宣言 ★クリーンアップ | 4363+4=4367 | 未着手 |

詳細: [roadmap-v103.1-v104.0.md](roadmap-v103.1-v104.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 5: SAP Real-World Platform 1.0 宣言（v104.1〜v105.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 設計・実装・公開・運用 — 全層が実際に動く SAP プラットフォームの完成宣言。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v104.1.0 | 総合 E2E シナリオ（SAP → pipeline → API 公開 → Data Product 登録） | 4367+2=4369 | 未着手 |
| v104.2.0 | パフォーマンス計測（BP 1万件取得 / ページネーション時間 / Lambda コールドスタート） | 4369+2=4371 | 未着手 |
| v104.3.0 | `fav doctor --sap`（SAP 接続・設定・mock 動作を一括チェック） | 4371+2=4373 | 未着手 |
| v104.4.0 | `fav infer --from sap-mock`（mock JSON から型を自動生成） | 4373+2=4375 | 未着手 |
| v104.5.0 | エラーメッセージ改善（SAP OData エラーコード → Favnir E0xxx 対応表） | 4375+2=4377 | 未着手 |
| v104.6.0 | `favnir4-showcase` E2E デモ更新（実際に動く SAP + API + Data Product） | 4377+2=4379 | 未着手 |
| v104.7.0 | 総合ドキュメント整備（SAP Real-World Platform 完全ガイド） | 4379+2=4381 | 未着手 |
| v104.8.0 | OSS 公開準備（CONTRIBUTING.md / examples/ 整備） | 4381+2=4383 | 未着手 |
| v104.9.0 | コードフリーズ・最終確認 | 4383+2=4385 | 未着手 |
| v105.0.0 | SAP Real-World Platform 1.0 宣言 ★大クリーンアップ | 4385+4=4389 | 未着手 |

詳細: [roadmap-v104.1-v105.0.md](roadmap-v104.1-v105.0.md)

---

## スプリント総括

| スプリント | バージョン範囲 | テーマ | 宣言バージョン | テスト累計 |
|---|---|---|---|---|
| Sprint 1 | v100.1〜v101.0 | SAP E2E Foundation（VM 修正・mock・E2E 実証） | v101.0.0 | 4,301 |
| Sprint 2 | v101.1〜v102.0 | SAP HTTP Layer（CSRF / OAuth2 / ページネーション） | v102.0.0 | 4,323 |
| Sprint 3 | v102.1〜v103.0 | SAP API Exposure（fav serve / Lambda / OpenAPI） | v103.0.0 | 4,345 |
| Sprint 4 | v103.1〜v104.0 | SAP Data Products（data_product / catalog / SLA） | v104.0.0 | 4,367 |
| Sprint 5 | v104.1〜v105.0 | SAP Real-World Platform 1.0 宣言（統合・OSS 準備） | v105.0.0 | 4,389 |

**合計**: +110 tests（4,279 → 4,389）

### 参考リンク

- 前フェーズ（完了）: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)
- Sprint 1 詳細: [roadmap-v100.1-v101.0.md](roadmap-v100.1-v101.0.md)
- Sprint 2 詳細: [roadmap-v101.1-v102.0.md](roadmap-v101.1-v102.0.md)
- Sprint 3 詳細: [roadmap-v102.1-v103.0.md](roadmap-v102.1-v103.0.md)
- Sprint 4 詳細: [roadmap-v103.1-v104.0.md](roadmap-v103.1-v104.0.md)
- Sprint 5 詳細: [roadmap-v104.1-v105.0.md](roadmap-v104.1-v105.0.md)
- 進行状況: [../current.md](../current.md)
- マイルストーン: [../../MILESTONE.md](../../MILESTONE.md)

---

## 宣言文（予定）

> 「Favnir SAP Platform が、実際に動くプラットフォームになった。
>
>  `Http.get_with_headers` が SAP OData に接続し、
>  CSRF トークンで書き込み、OAuth2 で認証し、
>  `fav serve` でデータを REST API として公開し、
>  `data_product` でチームが管理する。
>
>  設計から実装、公開、運用まで——それが、SAP Real-World Platform 1.0 である。」
