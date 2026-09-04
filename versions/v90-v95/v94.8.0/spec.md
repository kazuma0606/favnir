# Spec: v94.8.0 — サイトドキュメント完全化（SAP Advanced Era 総まとめ）

## Background

v90.1〜v94.7 で実装した SAP Advanced Era の全機能（`ctx.sap.batch` / QueryBuilder<T> / Metadata Infer / Lambda SnapStart / E2E デモ）がコードとして完成しているが、サイトドキュメントに散在している。本バージョンでは：

- `site/content/docs/runes/sap-odata.mdx` に `$batch` セクションを追加し、業務シナリオ表にシナリオ 5 を追記する
- `site/content/docs/cli/infer.mdx` に `--sap-metadata` / `--sap-metadata-file` フラグの最終版説明を追記する
- `site/content/docs/guides/sap-integration.mdx`（新規）を作成し、SAP 統合の全体像ガイドとして公開する

## Goals

1. `site/content/docs/guides/sap-integration.mdx` を新規作成する
   - SAP Advanced Era の全体像（ctx.sap / QueryBuilder / $batch / Metadata Infer / SnapStart）を 1 ページにまとめる
   - `batch` / `BatchRequest<T>` / `BatchOperation<T>` の使用例を含める（テスト要件）
2. `site/content/docs/runes/sap-odata.mdx` に `$batch` セクションを追加する
   - `ctx.sap.batch(req)` の使用例・型定義表・`BatchOperation<T>` ADT の説明
   - `SapClient` メソッド表に `ctx.sap.batch(req)` を追記
   - 業務シナリオ表にシナリオ 5（`advanced_sap_pipeline`）を追記
3. `site/content/docs/cli/infer.mdx` に `--sap-metadata` / `--sap-metadata-file` フラグを追記する
4. `driver.rs` に `mod v94800_tests`（テスト 2 件）を追加する

## Syntax / API Examples

### `$batch` 使用例（sap-integration.mdx / sap-odata.mdx に掲載）

```favnir
import rune "sap-odata"

fn cleanup_partners(ctx: AppCtx) -> Result<String, String> {
    bind bps  <- ctx.sap.business_partners(BusinessPartnerFilter {
        country:       Option.some("JP"),
        category:      Option.none(),
        changed_after: Option.none(),
        top:           Option.some(200)
    })
    bind ops  <- List.map(bps, fn(bp) { BatchDelete(bp.partner_id) })
    bind req  <- batch_request_builder("A_BusinessPartner", ops)
    bind resp <- ctx.sap.batch(req)
    Result.ok(String.concat("deleted ", Int.to_string(List.length(resp.succeeded))))
}
```

### `BatchOperation<T>` ADT

| バリアント | シグネチャ | 説明 |
|---|---|---|
| `BatchCreate(T)` | `BatchCreate(body: T)` | エンティティ新規作成 |
| `BatchUpdate(String, T)` | `BatchUpdate(key, body: T)` | 既存エンティティ更新 |
| `BatchDelete(String)` | `BatchDelete(key)` | エンティティ削除 |

### `batch_request_builder<T>` シグネチャ

```favnir
fn batch_request_builder<T>(entity_set: String, ops: List<BatchOperation<T>>) -> BatchRequest<T>
```

### `SapClient.batch` シグネチャ

```favnir
fn batch(ctx: SapClient, req: BatchRequest<String>) -> Result<BatchResponse<String>, String>
```

### `fav infer --sap-metadata` 使用例（infer.mdx に追記）

```bash
fav infer --from sap --sap-metadata-file ./metadata.xml --output src/sap_types.fav
fav infer --from sap --sap-metadata https://my.sap.example.com/odata/v4/$metadata
```

## Success Criteria

1. `site/content/docs/guides/sap-integration.mdx` が存在する
2. `sap-integration.mdx` に `batch` または `BatchRequest` が含まれる
3. `site/content/docs/runes/sap-odata.mdx` に `$batch` セクション（`ctx.sap.batch`）が追加されている
4. `site/content/docs/cli/infer.mdx` に `--sap-metadata` が記載されている
5. `cargo test` で 4,158 tests（+2）、0 failures
6. `cargo clippy --locked -- -D warnings` pass
7. `fav fmt --check` pass（compiler.fav / checker.fav）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `site/content/docs/guides/sap-integration.mdx` | **新規作成** | SAP 統合ガイド全体像（batch / QueryBuilder / SnapStart / Metadata Infer） |
| `site/content/docs/runes/sap-odata.mdx` | **更新** | `$batch` セクション追加・SapClient 表・業務シナリオ表更新 |
| `site/content/docs/cli/infer.mdx` | **更新** | `--sap-metadata` / `--sap-metadata-file` フラグ追記 |
| `fav/src/driver.rs` | **更新** | `mod v94800_tests`（テスト 2 件）追加 |
| `CHANGELOG.md` | **更新** | v94.8.0 エントリ追加 |
