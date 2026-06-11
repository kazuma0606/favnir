# v14.2.0 Spec — AzureCtx / AwsCtx + fav.toml [azure]

Date: 2026-06-12

---

## 目的

CrossCloud パイプライン（AWS → Azure）を記述するための **コンテキスト型** と
**設定ファイル統合** を実装する。

v14.1.0 では `AzurePostgres.execute_raw` / `AzurePostgres.query_raw` の VM プリミティブを追加した。
v14.2.0 では「接続情報をどこからどう渡すか」を解決する。

---

## ユーザー体験（Before / After）

### Before（v14.1.0 まで）

```fav
// 接続文字列をハードコードするしかない
bind result <- AzurePostgres.execute_raw(
    "postgresql://user:pass@host/db?sslmode=require",
    "INSERT INTO ...",
    "[]"
)
```

### After（v14.2.0）

```toml
# fav.toml
[azure]
postgres_url    = "${AZURE_POSTGRES_URL}"
storage_account = "${AZURE_STORAGE_ACCOUNT}"
storage_key     = "${AZURE_STORAGE_KEY}"
container       = "favnir-proof"
```

```fav
import rune "ctx"

// AwsCtx / AzureCtx を型として扱える
public fn migrate(aws_ctx: AwsCtx, azure_ctx: AzureCtx) -> Result<Unit, String> !AzureDb {
    bind result <- AzurePostgres.execute_raw(
        Ctx.azure_postgres_url(azure_ctx),
        "INSERT INTO ...",
        "[]"
    )
    Result.ok(())
}

public fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind aws_ctx   <- Ctx.build_aws("ap-northeast-1", "my-bucket", "postgresql://...")
    bind azure_ctx <- Ctx.build_azure(
        "${AZURE_POSTGRES_URL}", "${AZURE_STORAGE_ACCOUNT}",
        "${AZURE_STORAGE_KEY}", "favnir-proof"
    )
    migrate(aws_ctx, azure_ctx)
}
```

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `fav.toml [azure]` セクション | postgres_url / storage_account / storage_key / container のパース + 環境変数展開 |
| `Ctx.build_aws_raw` VM プリミティブ | AwsCtx レコードを構築して文字列シリアライズして返す |
| `Ctx.build_azure_raw` VM プリミティブ | AzureCtx レコードを構築して文字列シリアライズして返す |
| `runes/ctx/crosscloud.fav` | `AwsCtx` / `AzureCtx` / `CrossCloudCtx` 型定義と builder 関数 |
| `inject_azure_config` | fav.toml の [azure] を env var として VM に注入 |
| `v142000_tests` | 4 件のテスト |

### Out of Scope

- Azure Blob Storage 操作（v14.5.0）
- CrossCloud E2E デモ（v15.0.0）
- `fav explain` の Azure リネージ改善（v14.3.0）

---

## 型設計

```fav
// runes/ctx/crosscloud.fav

// 名目型ラッパー（Nominal wrapper）
type AwsCtx(Record)
    // フィールド: region, s3_bucket, db_url

type AzureCtx(Record)
    // フィールド: postgres_url, storage_account, storage_key, container

// CrossCloudCtx はオプショナル（コメントで代替案として提示）
// type CrossCloudCtx(Record)

public fn build_aws(
    region: String,
    s3_bucket: String,
    db_url: String
) -> Result<AwsCtx, String> {
    Ctx.build_aws_raw(region, s3_bucket, db_url)
}

public fn build_azure(
    postgres_url: String,
    storage_account: String,
    storage_key: String,
    container: String
) -> Result<AzureCtx, String> {
    Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)
}

public fn azure_postgres_url(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "postgres_url")
}

public fn azure_storage_account(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "storage_account")
}

public fn azure_storage_key(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "storage_key")
}

public fn azure_container(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "container")
}
```

---

## fav.toml [azure] スキーマ

```toml
[azure]
postgres_url    = "${AZURE_POSTGRES_URL}"          # 必須（CrossCloud デモ用）
storage_account = "${AZURE_STORAGE_ACCOUNT}"       # オプション（Blob 保存用）
storage_key     = "${AZURE_STORAGE_KEY}"           # オプション
container       = "favnir-proof"                   # オプション、デフォルト "favnir-proof"
```

環境変数展開: `${VAR_NAME}` 形式。既存の `expand_env_vars` 関数を流用。

---

## 実装ノート

- `Ctx.build_aws_raw` / `Ctx.build_azure_raw` は JSON 文字列を返す（既存の `Ctx.build_raw` と同じパターン）
- `AwsCtx` / `AzureCtx` は名目型ラッパー。checker.fav の `infer_hm` では `Unknown` として扱う（builtin_ret_ty に追加）
- `inject_azure_config` は `AZURE_POSTGRES_URL` / `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_KEY` / `AZURE_CONTAINER` を環境変数として設定
- `fav new --template crosscloud` は v15.0.0 に先送り

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `fav.toml` に `[azure]` を書いてエラーなし | ✅ |
| `Ctx.build_aws_raw` が E0007 を出さない | ✅ |
| `Ctx.build_azure_raw` が E0007 を出さない | ✅ |
| `runes/ctx/crosscloud.fav` が `fav check` をパス | ✅ |
| `cargo test v142000` 全 4 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.2.0"` | ✅ |
