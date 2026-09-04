# Roadmap v94.1.0 〜 v95.0.0 — SAP Advanced 1.0

Date: 2026-08-25
Status: 完了（v95.0.0 宣言済み・2026-08-30）

マスターロードマップ: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)

---

## 前提

- 直前完了: v94.0.0「SAP Metadata Infer 1.0 宣言」（tests = 4,142）
- 本スプリントは SAP Advanced Era の第 5 スプリント（最終）
- 目標: v95.0.0「SAP Advanced 1.0 宣言」（tests = 4,164）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v94.0.0 になっていることを確認する
- `versions/v90-v95/v94.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `fav/src/sap_metadata.rs` が存在することを確認する（v93.1.0 完了済みの証拠）
- `runes/sap-odata/query_builder.fav` に `QueryBuilder` / `Page` が含まれることを確認する
- `fav/src/driver.rs` に `mod v94000_tests` が存在することを確認する（v94.0.0 完了済みの証拠）

### スプリントの性格

SAP Advanced Era の**集大成スプリント**。

OData `$batch` リクエストによるバルク操作、AWS Lambda SnapStart による コールドスタート最適化、
総合ベンチマーク、OSS コミュニティ向け整備を行い、**SAP Advanced 1.0** を宣言する。

A（基盤）30% + B（インフラ最適化）30% + C（OSS / ドキュメント）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v94.1.0 | `BatchRequest<T>` 型定義（OData $batch 基盤） | 4142 + 2 = 4144 | 未着手 |
| v94.2.0 | `ChangeSet` + `ctx.sap.batch()` 実装 | 4144 + 2 = 4146 | 未着手 |
| v94.3.0 | Lambda SnapStart 対応 Terraform（`infra/lambda/sap-sync/`） | 4146 + 2 = 4148 | 未着手 |
| v94.4.0 | コールドスタートベンチマーク（SnapStart あり vs なし） | 4148 + 2 = 4150 | 未着手 |
| v94.5.0 | `fav bench --sap`（SAP 総合ベンチマークレポート） | 4150 + 2 = 4152 | 未着手 |
| v94.6.0 | OSS 整備（CONTRIBUTING.md / ISSUES テンプレート / SAP 向け README） | 4152 + 2 = 4154 | 未着手 |
| v94.7.0 | E2E デモ更新（$batch + SnapStart を含む完全デモ） | 4154 + 2 = 4156 | 未着手 |
| v94.8.0 | サイトドキュメント完全化（SAP Advanced Era 総まとめ） | 4156 + 2 = 4158 | 未着手 |
| v94.9.0 | 安定化・コードフリーズ | 4158 + 2 = 4160 | 未着手 |
| v95.0.0 | SAP Advanced 1.0 宣言 ★クリーンアップ | 4160 + 4 = 4164 | 未着手 |

---

## v94.1.0 — `BatchRequest<T>` 型定義

OData `$batch` プロトコルに対応する `BatchRequest<T>` 型を定義する。
複数の CRUD 操作を 1 回の HTTP リクエストで送信できるようにする。

```favnir
-- バッチ操作の種類
type BatchOperation<T> =
    | BatchCreate(T)
    | BatchUpdate(String, T)    -- id, entity
    | BatchDelete(String)       -- id

-- バッチリクエスト型
type BatchRequest<T> = {
    entity_set:  String,
    operations:  List<BatchOperation<T>>
}

-- バッチレスポンス型
type BatchResponse<T> = {
    succeeded: List<T>,
    failed:    List<BatchError>
}

type BatchError = {
    index:   Int,
    message: String
}
```

**実装内容:**
- `runes/sap-odata/batch.fav`（新規作成）に `BatchRequest<T>` 等を定義
- `driver.rs` に `mod v94100_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4142 + 2 = 4144）
- `sap_batch_file_exists`: `runes/sap-odata/batch.fav` が存在する
- `batch_request_type_defined`: `batch.fav` に `BatchRequest` が含まれる

---

## v94.2.0 — `ChangeSet` + `ctx.sap.batch()` 実装

`ChangeSet`（トランザクション内操作グループ）と `ctx.sap.batch()` 呼び出しを実装する。

```favnir
-- ChangeSet: アトミックに実行する操作グループ
type ChangeSet<T> = {
    operations: List<BatchOperation<T>>
}

-- 使用例: 取引先を一括作成
fn bulk_create_business_partners(
    ctx: AppCtx,
    bps: List<BusinessPartner>
) -> Result<BatchResponse<BusinessPartner>, String> {
    bind ops <- Result.ok(List.map(bps, fn(bp) { BatchCreate(bp) }))
    bind req <- Result.ok(BatchRequest {
        entity_set: "A_BusinessPartner",
        operations: ops
    })
    ctx.sap.batch(req)
}
```

**実装内容:**
- `runes/sap-odata/batch.fav` に `ChangeSet` と `batch_request_builder` を追加
- `SapClient` interface に `batch` メソッドを追加
- `driver.rs` に `mod v94200_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4144 + 2 = 4146）
- `change_set_type_defined`: `batch.fav` に `ChangeSet` が含まれる
- `sap_client_has_batch_method`: `runes/sap-odata/types.fav` に `batch` が含まれる

---

## v94.3.0 — Lambda SnapStart 対応 Terraform

AWS Lambda SnapStart を有効化した SAP 同期 Lambda の Terraform を追加する。
コールドスタートを削減し、SAP からの大量データ処理を安定化させる。

```hcl
# infra/lambda/sap-sync/main.tf（抜粋）
resource "aws_lambda_function" "sap_sync" {
  function_name = "favnir-sap-sync"
  runtime       = "java21"
  snap_start {
    apply_on = "PublishedVersions"
  }
  environment {
    variables = {
      SAP_BASE_URL    = var.sap_base_url
      SAP_CLIENT_ID   = var.sap_client_id
    }
  }
}
```

**実装内容:**
- `infra/lambda/sap-sync/`（新規作成）に `main.tf` / `variables.tf` / `outputs.tf` を追加
- `driver.rs` に `mod v94300_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4146 + 2 = 4148）
- `lambda_sap_sync_infra_exists`: `infra/lambda/sap-sync/` ディレクトリが存在する
- `lambda_sap_sync_has_snap_start`: `infra/lambda/sap-sync/main.tf` に `snap_start` が含まれる

---

## v94.4.0 — コールドスタートベンチマーク

Lambda SnapStart あり / なし のコールドスタート時間を計測するベンチマークスクリプトを追加する。

```
$ ./scripts/bench_sap_coldstart.sh

SAP Sync Lambda Cold Start Benchmark
=====================================
Without SnapStart:
  P50: 3,421 ms
  P95: 4,892 ms
  P99: 6,204 ms

With SnapStart:
  P50:   248 ms  (-92.7%)
  P95:   312 ms  (-93.6%)
  P99:   387 ms  (-93.8%)

Recommendation: SnapStart reduces cold start by ~93%.
```

**実装内容:**
- `scripts/bench_sap_coldstart.sh` を新規作成
- ベンチマーク結果を `fav/tmp/sap_coldstart_bench.json` に記録するロジックを追加
- `driver.rs` に `mod v94400_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4148 + 2 = 4150）
- `bench_sap_coldstart_script_exists`: `scripts/bench_sap_coldstart.sh` が存在する
- `bench_sap_coldstart_output_path_defined`: スクリプトに `sap_coldstart_bench` が含まれる

---

## v94.5.0 — `fav bench --sap`（SAP 総合ベンチマーク）

SAP 関連の全ベンチマーク（クエリビルダー・バッチ・メタデータ解析）を一括実行するコマンドを追加する。

```
$ fav bench --sap

SAP Advanced Benchmark Suite
=============================
QueryBuilder:
  query() + 3 chains:              0.9 µs/op
  filter_to_odata_string (complex): 1.1 µs/op

BatchRequest:
  batch_request (100 ops):         12 µs/op
  change_set serialization:         8 µs/op

Metadata Infer:
  parse_edmx (A_BusinessPartner): 2.3 ms
  entity_type_to_favnir:          0.4 ms

Total: 4 benchmarks, all PASS
```

**実装内容:**
- `fav/src/bench.rs` に `bench_sap_all` 関数を追加
- `cli.fav` の `--sap` フラグで `bench_sap_all` を呼び出す
- `driver.rs` に `mod v94500_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4150 + 2 = 4152）
- `bench_sap_all_function_defined`: `bench.rs` に `bench_sap_all` が含まれる
- `cli_fav_has_bench_sap_flag`: `cli.fav` に `--sap` フラグ（bench コンテキスト）が含まれる

---

## v94.6.0 — OSS 整備

SAP Integration に関する OSS コミュニティ向けドキュメントを整備する。

**実装内容:**
- `CONTRIBUTING.md` に SAP テスト環境セットアップ手順を追加
- `runes/sap-odata/README.md` を新規作成（Rune の概要・使い方・設定方法）
- GitHub Issues テンプレート（`.github/ISSUE_TEMPLATE/sap-bug.md`）を追加
- `driver.rs` に `mod v94600_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4152 + 2 = 4154）
- `sap_odata_rune_readme_exists`: `runes/sap-odata/README.md` が存在する
- `sap_odata_rune_readme_has_setup`: `runes/sap-odata/README.md` に `setup` または `Setup` が含まれる

---

## v94.7.0 — E2E デモ更新

`infra/e2e-demo/sap-odata/` を SAP Advanced Era の全機能を含む完全デモに更新する。

```favnir
-- infra/e2e-demo/sap-odata/pipeline_advanced.fav
-- $batch + QueryBuilder<T> + fetch_all_pages を組み合わせた完全デモ

fn advanced_sap_pipeline(ctx: AppCtx) -> Result<String, String> {
    -- 1. QueryBuilder で全取引先を取得（自動ページング）
    bind q    <- query<BusinessPartner>()
    bind q    <- with_filter(q, Eq("Country", "JP"))
    bind bps  <- fetch_all_pages(ctx, q, 20, ctx.sap.business_partners_page)

    -- 2. S3 に保存
    bind _    <- ctx.s3.put_object("sap-sync", "bps_jp.json", List.length(bps))

    -- 3. バッチ更新（ステータスを "SYNCED" に）
    bind ops  <- List.map(bps, fn(bp) { BatchUpdate(bp.BusinessPartner, bp) })
    bind req  <- batch_request_builder("A_BusinessPartner", ops)
    bind resp <- ctx.sap.batch(req)

    Result.ok("synced " ++ Int.to_string(List.length(resp.succeeded)) ++ " business partners")
}
```

**実装内容:**
- `infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成
- `driver.rs` に `mod v94700_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4154 + 2 = 4156）
- `pipeline_advanced_fav_exists`: `infra/e2e-demo/sap-odata/pipeline_advanced.fav` が存在する
- `pipeline_advanced_uses_batch`: `pipeline_advanced.fav` に `ctx.sap.batch` が含まれる

---

## v94.8.0 — サイトドキュメント完全化

SAP Advanced Era（v90.1〜v94.7）の全機能をまとめた総合ドキュメントを作成する。

**追加・更新ドキュメント:**
- `site/content/docs/runes/sap-odata.mdx`: SAP Advanced Era 全機能（ctx.sap / QueryBuilder / $batch / Metadata Infer）
- `site/content/docs/cli/infer.mdx`: `--sap-metadata` / `--sap-metadata-file` の最終版
- `site/content/docs/guides/sap-integration.mdx`（新規）: SAP 統合の全体像ガイド

**完了条件**: Rust テスト 2 件（4156 + 2 = 4158）
- `docs_sap_integration_guide_exists`: `site/content/docs/guides/sap-integration.mdx` が存在する
- `docs_sap_integration_guide_mentions_batch`: `sap-integration.mdx` に `batch` または `BatchRequest` が含まれる

---

## v94.9.0 — 安定化・コードフリーズ

v94.1〜v94.8 の全機能を通しで確認する最終安定化スプリント。SAP Advanced Era 全体（v90.1〜v94.9）の総点検を実施する。

**実装内容:**
- `cargo test` 全 pass 確認（4,145 tests）
- SAP Advanced Era 全 4 スプリントの成果物（ファイル・テスト・ドキュメント）の存在確認
- `fav bench --sap` の全ベンチマーク pass 確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4158 + 2 = 4160）
- `sap_advanced_smoke_all_features`: 以下が全て存在することを確認
  - `runes/sap-odata/batch.fav`（$batch）
  - `runes/sap-odata/query_builder.fav`（QueryBuilder<T>）
  - `fav/src/sap_metadata.rs`（Metadata Infer）
  - `infra/lambda/sap-sync/main.tf`（SnapStart Lambda）
- `sap_advanced_era_doc_complete`: `site/content/docs/guides/sap-integration.mdx` が存在する

---

## v95.0.0 — SAP Advanced 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できる。
>  `QueryBuilder<T>` で型安全なクエリを組み立て、`fetch_all_pages` で全件自動取得できる。
>  `fav infer --sap-metadata` で SAP の型定義が自動生成される。
>  Lambda SnapStart でコールドスタートは 93% 削減される。
>  それが、Favnir SAP Advanced 1.0 である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `95.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v95.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テストを `95.0.0` に一括更新
- SAP Advanced Era 全 4 スプリントのロードマップを「完了」にマーク
  - `versions/roadmap/roadmap-v90.1-v91.0.md`
  - `versions/roadmap/roadmap-v91.1-v92.0.md`
  - `versions/roadmap/roadmap-v92.1-v93.0.md`
  - `versions/roadmap/roadmap-v93.1-v94.0.md`
  - `versions/roadmap/roadmap-v94.1-v95.0.md`

**完了条件**: `v95000_tests` 4 件（4160 + 4 = 4164）
- `cargo_toml_version_is_95_0_0`
- `changelog_has_v95_0_0`
- `milestone_has_sap_advanced`
- `readme_mentions_sap_advanced`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v94.0.0（ベース） | 4,142 | — |
| v94.1.0 | 4,144 | +2 |
| v94.2.0 | 4,146 | +2 |
| v94.3.0 | 4,148 | +2 |
| v94.4.0 | 4,150 | +2 |
| v94.5.0 | 4,152 | +2 |
| v94.6.0 | 4,154 | +2 |
| v94.7.0 | 4,156 | +2 |
| v94.8.0 | 4,158 | +2 |
| v94.9.0 | 4,160 | +2 |
| v95.0.0（宣言） | 4,164 | +4 |

**本スプリント合計**: +22 tests

---

## SAP Advanced Era 全体サマリー

| スプリント | バージョン範囲 | 宣言 | テスト増加 |
|---|---|---|---|
| Sprint 1 | v90.1.0〜v91.0.0 | SAP Ctx 統合 1.0 | +22 |
| Sprint 2 | v91.1.0〜v92.0.0 | SAP OData Query 1.0 | +22 |
| Sprint 3 | v92.1.0〜v93.0.0 | SAP QueryBuilder 1.0 | +22 |
| Sprint 4 | v93.1.0〜v94.0.0 | SAP Metadata Infer 1.0 | +22 |
| Sprint 5 | v94.1.0〜v95.0.0 | SAP Advanced 1.0 | +22 |
| **合計** | v90.1.0〜v95.0.0 | — | **+110** |

**テスト数推移**: 4,041（v90.0.0）→ 4,164（v95.0.0）
