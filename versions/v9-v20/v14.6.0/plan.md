# v14.6.0 Plan — ドキュメント整備（README + CHANGELOG）

Date: 2026-06-12

---

## Phase A — `CHANGELOG.md` 追記

### A-1: v14.5.0〜v14.1.0 のエントリを追加

追加場所: `## [v14.0.0]` ブロックの **直前**（最新が上になる順序を維持）

追加順序: v14.5.0 → v14.4.0 → v14.3.0 → v14.2.0 → v14.1.0（上から新しい順）

#### v14.5.0 エントリ（コピペ用）

```markdown
## [v14.5.0] — 2026-06-12 — Azure Blob Storage Rune

### New Features
- `azure_blob_sign` ヘルパー関数（`vm.rs`）: HMAC-SHA256 + base64 による Azure Shared Key 署名
  - 既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` + `chrono` を使用（新規 crate なし）
- `AzureBlob.put_raw(account, key, container, blob_name, body)` VM primitive
- `AzureBlob.get_raw(account, key, container, blob_name)` VM primitive
- `AzureBlob.list_raw(account, key, container, prefix)` VM primitive（XML → JSON 配列）
- `AzureBlob.delete_raw(account, key, container, blob_name)` VM primitive
- `checker.rs`: `require_azure_storage_effect` (E0317) — `!AzureStorage` 未宣言エラー
- `checker.rs`: `("AzureBlob", "put_raw/get_raw/list_raw/delete_raw")` in `builtin_ret_ty`
- `checker.rs`: `"AzureBlob"` を `BUILTIN_EFFECTS` に追加
- `runes/azure-blob/azure_blob.fav`: `put/get/list/delete` ctx-aware ラッパー（`ctx: String`）
- `runes/azure-blob/rune.toml`: rune メタデータ

### Notes
- テスト: v145000_tests 4 件（version_is_14_5_0 / azure_blob_put_raw_registered / azure_storage_effect_required / azure_blob_rune_file_present）
- `let` 構文は rune ファイル内でパースエラーになるため、引数はインライン化
- `import rune "ctx"` は使用不可（runes/ctx/ctx.fav 未存在）→ `ctx: String` で代替
```

#### v14.4.0 エントリ（コピペ用）

```markdown
## [v14.4.0] — 2026-06-12 — AWS Rune 正式パッケージング

### New Features
- `AWS.secrets_get_raw(region, secret_name)` VM primitive（SigV4 + ureq で Secrets Manager API 呼び出し）
- `checker.rs`: `("AWS", "secrets_get_raw")` を `builtin_ret_ty` に追加
- `checker.rs`: `("Ctx", "aws_get_field_raw")` を `builtin_ret_ty` に追加
- `runes/aws/secrets.fav`: `secrets_get(ctx: String, secret_name: String)` ラッパー
- `runes/aws/s3.fav`: `s3_put/s3_get/s3_delete/s3_list` ctx-aware ラッパー追加
- `runes/aws/rune.toml`: version `14.4.0`、description 更新

### Notes
- テスト: v144000_tests 4 件（version_is_14_4_0 / secrets_get_raw_registered / aws_ctx_field_raw_registered / aws_rune_s3_ctx_functions_present）
- LocalStack エンドポイント対応（`config.endpoint_url` がある場合は `/` で置換）
```

#### v14.3.0 エントリ（コピペ用）

```markdown
## [v14.3.0] — 2026-06-12 — Azure lineage + !AzureStorage エフェクト

### New Features
- `ast::Effect::AzureStorage` 追加（parser / lineage / checker で認識）
- `lineage.rs`: `EffectKind::AzureDbRead` / `AzureDbWrite` / `AzureBlobRead` / `AzureBlobWrite` 追加
- `lineage.rs`: `collect_azure_blob_call_kinds` / `collect_azure_db_call_kinds` 追加
- `checker.rs`: `BUILTIN_EFFECTS` に `"AzureStorage"` 追加
- `fav explain --lineage` 出力に Azure エフェクトが表示されるよう更新

### Notes
- テスト: v143000_tests
```

#### v14.2.0 エントリ（コピペ用）

```markdown
## [v14.2.0] — 2026-06-12 — AzureCtx / AwsCtx + fav.toml [azure]

### New Features
- `Ctx.build_aws_raw(region, s3_bucket, db_url)` VM primitive
- `Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)` VM primitive
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx JSON からフィールドを取得
- `Ctx.azure_get_field_raw(ctx, field)` VM primitive — AzureCtx JSON からフィールドを取得
- `fav.toml` に `[azure]` セクション追加（`postgres_url` / `storage_account` / `storage_key` / `container`）
- `inject_azure_config` — fav.toml の [azure] セクションを env var 展開して ctx に注入

### Notes
- テスト: v142000_tests
```

#### v14.1.0 エントリ（コピペ用）

```markdown
## [v14.1.0] — 2026-06-12 — Azure PostgreSQL Rune

### New Features
- `AzurePostgres.execute_raw(conn_str, sql, params)` VM primitive（tokio-postgres + tokio-runtime-1）
- `AzurePostgres.query_raw(conn_str, sql, params)` VM primitive（JSON 配列文字列として返す）
- `checker.rs`: `AzurePostgres` namespace を `builtin_ret_ty` / `BUILTIN_EFFECTS` に追加
- `checker.rs`: `require_azure_db_effect` (E0316) — `!AzureDb` 未宣言エラー
- `ast::Effect::AzureDb` 追加
- `lineage.rs`: `!AzureDb(read/write)` 区別追加
- `runes/azure-postgres/azure_postgres.fav`: `execute/query_rows` ctx-aware ラッパー
- `runes/azure-postgres/rune.toml`: rune メタデータ

### Notes
- テスト: v141000_tests
- SSL: `sslmode=require` を接続文字列に付加して Azure DB for PostgreSQL の SSL 必須要件に対応
```

---

## Phase B — `README.md` 更新

### B-1: 「現在の状態」見出しを更新

**変更箇所（line ~80）:**

```markdown
# Before
**v10.0.0（2026-06-03）— OSS 公開準備完了**
テスト: **1260 件すべて通過**

# After
**v14.6.0（2026-06-12）— ドキュメント整備完了**
テスト: **1530+ 件すべて通過**
```

本文の「v14.0.0 で…」の説明文に以下を追記（v14.0.0 の段落の後）:

```markdown
v14.1.0〜v14.5.0（2026-06-12）で、クロスクラウド基盤を整備しました。
Azure DB for PostgreSQL・Azure Blob Storage のネイティブ対応、AWS Secrets Manager 統合、
および CrossCloud E2E デモ（v15.0.0）に向けた Rune エコシステムを拡充しました。
v14.6.0（2026-06-12）で、ドキュメント整備を完了しました。
```

### B-2: 機能一覧表に Azure 行を追加

**変更箇所（Rune エコシステム行）:**

```markdown
# Before
| **Rune エコシステム** | AWS / DuckDB / SQL / DB / fs / Parquet | ✓ |
| | http / grpc / graphql（`!Http` エフェクト） | ✓ |
| | llm（`!Llm` エフェクト、Claude / OpenAI） | ✓ |
| | snowflake（`!Snowflake` エフェクト） | ✓ |

# After
| **Rune エコシステム** | AWS（S3 / SQS / DynamoDB / Secrets Manager） | ✓ |
| | Azure Blob Storage（`AzureBlob.*`、Shared Key 認証） | ✓ |
| | Azure PostgreSQL（`AzurePostgres.*`、SSL 対応） | ✓ |
| | http / grpc / graphql | ✓ |
| | llm（Claude / OpenAI） | ✓ |
| | snowflake | ✓ |
| | DuckDB / SQL / DB / fs / Parquet / json / csv / gen 等 | ✓ |
```

### B-3: コード例の注記追加

**変更箇所（「基本パイプライン」コードブロック周辺）:**

旧 `!Effect` スタイルのコード例（`stage ParseCsv: String -> List<Row> !Io` 等）に以下の注記を追加:

```markdown
> **注記**: 以下のコード例は `--legacy` モード（旧 `!Effect` スタイル）で動作します。
> v14.0.0 以降の標準スタイルは `fn load(ctx: LoadCtx) -> ...` の Capability Context 形式です。
> 新規コードは「Capability Context（v14.0.0〜）」セクションのスタイルを推奨します。
```

### B-4: ロードマップ表に v14.1.0〜v14.6.0 を追記

**変更箇所（「ロードマップ」表、`v14.0.0` 行の直後）:**

```markdown
| v14.1.0〜v14.5.0 | Azure PostgreSQL / AzureCtx / Azure Blob Storage Rune / AWS Secrets Manager | 完了 |
| **v14.6.0** | **ドキュメント整備**（README / CHANGELOG） | **完了** |
```

---

## Phase C — `fav/src/driver.rs`: v146000_tests + バージョンバンプ

### C-1: `v146000_tests` モジュールを追加

追加場所: `v145000_tests` ブロックの直前

```rust
// ── v146000_tests (v14.6.0) — ドキュメント整備 ────────────────────────────────
#[cfg(test)]
mod v146000_tests {
    #[test]
    fn version_is_14_6_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.6.0");
    }

    #[test]
    fn changelog_has_v14_5_0_entry() {
        let changelog = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("CHANGELOG.md")
        ).expect("CHANGELOG.md should exist");
        assert!(changelog.contains("[v14.5.0]"),
            "CHANGELOG.md should contain [v14.5.0] entry");
        assert!(changelog.contains("[v14.1.0]"),
            "CHANGELOG.md should contain [v14.1.0] entry");
    }

    #[test]
    fn readme_mentions_azure_blob() {
        let readme = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("README.md")
        ).expect("README.md should exist");
        assert!(readme.contains("AzureBlob") || readme.contains("Azure Blob"),
            "README.md should mention Azure Blob Storage");
        assert!(readme.contains("v14.5.0") || readme.contains("v14.6.0"),
            "README.md should mention v14.5.0 or v14.6.0 in roadmap");
    }
}
```

### C-2: `v145000_tests` の `version_is_14_5_0` を `>=` 比較に修正

```rust
// Before
assert_eq!(env!("CARGO_PKG_VERSION"), "14.5.0");

// After
assert!(env!("CARGO_PKG_VERSION") >= "14.5.0",
    "expected >= 14.5.0, got {}", env!("CARGO_PKG_VERSION"));
```

### C-3: `fav/Cargo.toml` バージョンを `14.6.0` にバンプ

```toml
version = "14.6.0"
```

---

## Phase D — 確認 + コミット

### D-1: `cargo test v146000` で 3 件全パス

```
test driver::v146000_tests::version_is_14_6_0 ... ok
test driver::v146000_tests::changelog_has_v14_5_0_entry ... ok
test driver::v146000_tests::readme_mentions_azure_blob ... ok
```

### D-2: `cargo test` 全件パス

### D-3: git commit

```bash
git commit -m "feat: v14.6.0 — ドキュメント整備（README + CHANGELOG v14.1.0〜v14.5.0）"
```

---

## 実装上の注意

- CHANGELOG エントリは `## [v14.5.0]` → `## [v14.4.0]` → `## [v14.3.0]` → `## [v14.2.0]` → `## [v14.1.0]` の順（新しい順）で、既存の `## [v14.0.0]` の直前に挿入する
- README のコード例を書き直す必要はない。注記（blockquote）を追加するだけでよい
- テストは内容ではなく「存在確認」レベルで十分（CHANGELOG に `[v14.5.0]` 文字列が含まれるか等）
