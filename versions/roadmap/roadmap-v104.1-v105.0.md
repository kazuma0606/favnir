# Roadmap v104.1.0 〜 v105.0.0 — SAP Real-World Platform 1.0 宣言

Date: 2026-09-05
Status: 未着手

マスターロードマップ: [roadmap-v100.1-v105.0.md](roadmap-v100.1-v105.0.md)

---

## 前提

- 直前完了: v104.0.0「SAP Data Products 1.0 宣言」（tests = 4,367）
- 本スプリントは SAP Real-World Platform Era の第 5 スプリント（最終）
- 目標: v105.0.0「SAP Real-World Platform 1.0 宣言」（tests = 4,389）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v104.0.0 になっていることを確認する
- `fav/src/driver.rs` に `mod v104000_tests` が存在することを確認する
- `fav/Cargo.toml` の version が `104.0.0` であることを確認する
- `fav sap ping` コマンドが動作することを確認する（v101.5.0 完了済みの証拠）
- `fav catalog list` コマンドが動作することを確認する（v103.2.0 完了済みの証拠）
- `fav/tmp/hello.fav` が存在することを確認する

### スプリントの性格

SAP Real-World Platform Era の**統合・完成スプリント（最終章）**。

Sprint 1〜4 で「動く・本物のクライアント・外に出す・製品として管理する」を達成した。
Sprint 5 では全層を統合した E2E デモ・パフォーマンス計測・診断ツール・
OSS 公開準備を行い、「設計から実装、公開、運用まですべてが動く」プラットフォームとして宣言する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v104.1.0 | 総合 E2E シナリオ（SAP → pipeline → API 公開 → Data Product 登録） | 4367+2=4369 | 未着手 |
| v104.2.0 | パフォーマンス計測（BP 1万件取得 / ページネーション / Lambda コールドスタート） | 4369+2=4371 | 未着手 |
| v104.3.0 | `fav doctor --sap`（SAP 接続・設定・mock 動作を一括チェック） | 4371+2=4373 | 未着手 |
| v104.4.0 | `fav infer --from sap-mock`（mock JSON から型を自動生成） | 4373+2=4375 | 未着手 |
| v104.5.0 | エラーメッセージ改善（SAP OData エラーコード → 診断メッセージ強化） | 4375+2=4377 | 未着手 |
| v104.6.0 | `favnir4-showcase` E2E デモ更新（実際に動く全層縦断デモ） | 4377+2=4379 | 未着手 |
| v104.7.0 | 総合ドキュメント整備（SAP Real-World Platform 完全ガイド） | 4379+2=4381 | 未着手 |
| v104.8.0 | OSS 公開準備（CONTRIBUTING.md / examples/ / GitHub テンプレート整備） | 4381+2=4383 | 未着手 |
| v104.9.0 | コードフリーズ・最終確認 | 4383+2=4385 | 未着手 |
| v105.0.0 | SAP Real-World Platform 1.0 宣言 ★大クリーンアップ | 4385+4=4389 | 未着手 |

---

## v104.1.0 — 総合 E2E シナリオ

Sprint 1〜4 の全機能を組み合わせた縦断 E2E シナリオを実装・確認する。

**シナリオ: SAP BusinessPartner の全層縦断**

```
1. fav sap-mock --port 4004          # mock SAP 起動
2. fav run pipeline.fav              # SAP からデータ取得（Http.get_with_headers + Base64.encode）
3. fav serve api.fav --port 8080     # REST API として公開
4. curl http://localhost:8080/partners  # 外部から取得確認
5. fav catalog list ./products/      # Data Product として管理
6. fav mesh validate ./products/     # スキーマ契約チェック
7. fav mesh check-sla                # SLA 確認
8. fav catalog export --format openmetadata  # カタログ出力
```

**修正ファイル**: `infra/e2e-demo/sap-odata/scripts/run-full-e2e.sh`（新規）、`fav/src/driver.rs`

---

## v104.2.0 — パフォーマンス計測

本番規模のデータを想定したパフォーマンス計測を実施し、結果をドキュメントに記録する。

**計測項目**:

```bash
$ fav bench --sap --suite all
SAP Performance Benchmark

  BP 1,000件取得（1ページ）:        245ms  avg  (SLA: 500ms ✓)
  BP 10,000件取得（ページネーション）: 2.4s   avg  (SLA: 5s ✓)
  SalesOrder 5,000件取得:          1.8s   avg  (SLA: 5s ✓)
  POST BusinessPartner（CSRF付き）:  312ms  avg  (SLA: 1s ✓)

  fav serve GET /partners (100 並列): p50=45ms  p95=120ms  p99=280ms
  Lambda コールドスタート:             890ms  (SnapStart 適用後: 95ms)

Results saved: versions/v100-v105/v104.2.0/benchmark_results.md
```

**修正ファイル**: `fav/src/driver.rs`、`versions/v100-v105/v104.2.0/benchmark_results.md`（新規）

---

## v104.3.0 — `fav doctor --sap`

SAP 接続・設定・mock 動作・Rune ファイルの整合性を一括チェックする `fav doctor --sap` を追加する。
`fav doctor`（v54.5.0 で追加済み）のサブチェックとして実装する。

```bash
$ fav doctor --sap
SAP Doctor Check

  [✓] fav.toml [sap] セクションが存在する
  [✓] SAP_BASE_URL 環境変数が設定されている
  [✓] Http.get_with_headers が vm.rs に実装されている
  [✓] Base64.encode が vm.rs に実装されている
  [✓] runes/sap-odata/client.fav が存在する
  [✓] runes/sap-odata/csrf.fav が存在する
  [✓] infra/e2e-demo/sap-odata/mock/db.json が存在する
  [✓] docker-compose.yml が json-server を使用している
  [!] fav sap ping: SAP_BASE_URL に接続できません（mock が起動していない可能性）
      → fav sap-mock --port 4004 を実行してください

Result: 8/9 checks passed (1 warning)
```

**修正ファイル**: `fav/src/main.rs`（`doctor` サブコマンド `--sap` オプション追加）、`fav/src/driver.rs`

---

## v104.4.0 — `fav infer --from sap-mock`

`infra/e2e-demo/sap-odata/mock/db.json`（または `BusinessPartnerCollection.json` 等）の
JSON から Favnir 型定義を自動生成する `fav infer --from sap-mock` を追加する。
既存の `fav infer --from snowflake`（v10.8.0）/ `fav infer --from sap`（v90.x）と並ぶ
mock ファースト開発のための型推論コマンド。

```bash
$ fav infer --from sap-mock \
    --file infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json \
    --entity BusinessPartner \
    --out runes/sap-odata/generated/business_partner_inferred.fav

Inferring types from mock data...
  Source: BusinessPartnerCollection.json (3 records)

  Generated: runes/sap-odata/generated/business_partner_inferred.fav

type BusinessPartnerInferred = {
    BusinessPartner:         String,
    BusinessPartnerName:     String,
    BusinessPartnerCategory: String,
    Country:                 String,
    Region:                  String,
    CityName:                String
}
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`

---

## v104.5.0 — エラーメッセージ改善

SAP OData エラーコードを Favnir の診断メッセージとして分かりやすく表示する機能を追加する。
`SapApiError`（v100.8.0 で定義済み）を活用する。

**改善前**:
```
Error: Http.get_with_headers failed: 401 Unauthorized
```

**改善後**:
```
SAP OData Error [E0430]:
  Code:    /IWFND/MED/004
  Message: "Authentication failed"

  Hint: SAP_USER / SAP_PASS が正しく設定されているか確認してください。
        fav doctor --sap で接続設定を確認できます。
```

**修正ファイル**: `fav/src/frontend/parser.rs`（エラーコード定義）、`fav/src/driver.rs`

---

## v104.6.0 — `favnir4-showcase` E2E デモ更新

`infra/e2e-demo/favnir4-showcase/` を SAP Real-World Platform 全層縦断デモとして更新する。

**デモ構成**:
```
favnir4-showcase/
  pipeline.fav          -- SAP → 取得 → 変換
  api.fav               -- REST API 宣言
  products/
    business_partner_product.fav  -- Data Product 宣言
    sales_order_product.fav
  docker-compose.yml    -- mock SAP + favnir-runner
  scripts/
    run-showcase.sh     -- 全層縦断デモ実行スクリプト
  README.md             -- デモ手順
```

```bash
$ ./scripts/run-showcase.sh
[1/6] Starting SAP mock server...     ✓
[2/6] Fetching SAP data...            ✓ (245ms, 150 records)
[3/6] Starting REST API server...     ✓ http://localhost:8080
[4/6] Verifying API response...       ✓ 200 OK
[5/6] Checking Data Product SLA...    ✓ 99.95% availability
[6/6] Exporting catalog...            ✓ catalog.json

SAP Real-World Platform showcase: ALL PASSED
```

**修正ファイル**: `infra/e2e-demo/favnir4-showcase/`（全体更新）、`fav/src/driver.rs`

---

## v104.7.0 — 総合ドキュメント整備

**新規作成**:
- `site/content/docs/guides/sap-real-world-platform.mdx` — SAP Real-World Platform 完全ガイド（v101〜v104 全機能の統合ガイド）

**内容**:
- E2E セットアップ手順（Docker + fav run + fav serve）
- HTTP クライアント設定（CSRF / OAuth2 / ページネーション）
- REST API 公開手順（api キーワード / fav serve / Lambda）
- Data Product 管理手順（data_product / catalog / mesh validate）
- トラブルシューティング（fav doctor --sap / fav sap ping）

**修正ファイル**: 上記 1 ファイル（新規）、`fav/src/driver.rs`

---

## v104.8.0 — OSS 公開準備

GitHub Public 化・OSS コミュニティ向けの整備を行う。

**整備内容**:
- `CONTRIBUTING.md` — SAP 統合への貢献ガイド（mock 環境のセットアップ手順含む）
- `examples/sap-basic/` — 最小構成の SAP 連携 example（`main.fav` + `fav.toml`）
- `examples/sap-api/` — REST API 公開 example（`api.fav` + `fav serve` 手順）
- `.github/ISSUE_TEMPLATE/sap-bug-report.md` — SAP 関連バグレポートテンプレート
- `.github/ISSUE_TEMPLATE/feature-request.md` — 機能要望テンプレート

**修正ファイル**: 上記ファイル群（新規）、`fav/src/driver.rs`

---

## v104.9.0 — コードフリーズ・最終確認

- 全テスト通過確認（4,385 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- `infra/e2e-demo/favnir4-showcase/scripts/run-showcase.sh` が全 PASS になることを確認
- `fav doctor --sap` が 9/9 checks passed になることを確認（mock 起動時）
- 全 SAP ロードマップファイル（v100.1〜v105.0）の Status を「完了」に更新
- `versions/current.md` の次バージョン欄を v105.0.0 に更新

---

## v105.0.0 — SAP Real-World Platform 1.0 宣言 ★大クリーンアップ

**宣言文**:

> 「Favnir SAP Platform が、実際に動くプラットフォームになった。
>
>  `Http.get_with_headers` が SAP OData に接続し、
>  CSRF トークンが write を守り、
>  `@odata.nextLink` を追って全データを取り尽くし、
>  `fav serve` でデータを REST API として公開し、
>  `data_product` でチームが SLA とともに管理する。
>
>  `fav doctor --sap` が設定を診断し、
>  `run-showcase.sh` が全層の動作を証明する。
>
>  設計から実装、公開、運用まで——
>  これが、SAP Real-World Platform 1.0 である。」

**v105000_tests（4 テスト）**:
- `cargo_toml_version_is_105_0_0`
- `changelog_has_v105_0_0`
- `milestone_has_sap_real_world_platform`
- `sap_showcase_script_exists`

**大クリーンアップ**:
- `cargo clean` 実施（target/ ディレクトリ削除）
- `fav/tmp/hello.fav` 復元確認（cargo clean 後も存在することを確認）
- `cargo test` で 4,389 tests, 0 failures を再確認
- `cargo build` で `./target/debug/fav` を再生成
- 全 SAP ロードマップファイル（v100.1〜v105.0）の Status を「完了」に更新
- `roadmap-v100.1-v105.0.md` の Status を「完了」に更新

**修正ファイル**: `fav/Cargo.toml`（version → 105.0.0）、`MILESTONE.md`、`CHANGELOG.md`、`fav/src/driver.rs`、`versions/current.md`

---

## スプリント終了時の確認

- [ ] 4,389 tests, 0 failures
- [ ] `infra/e2e-demo/favnir4-showcase/scripts/run-showcase.sh` が ALL PASSED
- [ ] `fav doctor --sap` が全チェックをクリア（mock 起動時）
- [ ] `fav infer --from sap-mock` が動作する
- [ ] `examples/sap-basic/` と `examples/sap-api/` が存在する
- [ ] `site/content/docs/guides/sap-real-world-platform.mdx` が存在する
- [ ] `cargo clean` を実施する（★大クリーンアップ）
- [ ] `fav/tmp/hello.fav` が cargo clean 後も存在することを確認する
- [ ] `cargo test` で 4,389 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo build` で `./target/debug/fav` を再生成する
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v105.0.0 に更新
- [ ] `MILESTONE.md` に v105.0.0 エントリを追加（SAP Real-World Platform 1.0 宣言）
- [ ] 全ロードマップファイル（v100.1〜v105.0）の Status を「完了」に更新
- [ ] `roadmap-v100.1-v105.0.md` の Status を「完了」に更新
