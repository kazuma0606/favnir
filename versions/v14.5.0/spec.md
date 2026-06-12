# v14.5.0 Spec — Azure Blob Storage Rune (runes/azure-blob/)

Date: 2026-06-12

---

## 目的

CrossCloud E2E デモ（v15.0.0）で「証跡 JSON を Azure Blob に保存」するための
`runes/azure-blob/` Rune を正式実装する。

`AzurePostgres` 相当の VM プリミティブを `AzureBlob` として追加し、
`Ctx.azure_get_field_raw` で AzureCtx からアカウント・キー・コンテナを取り出して使う。

---

## 現状（v14.4.0 時点）

| リソース | 状態 |
|---|---|
| `ast::Effect::AzureStorage` | ✅ v14.3.0 で追加済み |
| `BUILTIN_EFFECTS` に `"AzureStorage"` | ✅ v14.3.0 で追加済み |
| Parser が `!AzureStorage` を認識 | ✅ v14.3.0 で追加済み |
| `lineage.rs` の `collect_azure_blob_call_kinds` | ✅ v14.3.0 で追加済み |
| `Ctx.build_azure_raw` VM primitive | ✅ v14.2.0 で追加済み |
| `Ctx.azure_get_field_raw` VM primitive | ✅ v14.2.0 で追加済み |
| `AzureBlob.*` VM primitives | **未実装** |
| `checker.rs` の `AzureBlob` namespace | **未実装** |
| `runes/azure-blob/` | **未実装** |

---

## ユーザー体験（Before / After）

### Before（v14.4.0 まで）

```fav
// Azure Blob への保存は AzureBlob VM primitive 未実装のため不可
```

### After（v14.5.0）

```fav
import rune "ctx"
import rune "azure-blob"

public fn save_proof(azure_ctx: AzureCtx, proof_json: String) -> Result<Unit, String> !AzureStorage {
    // AzureCtx から storage_account / storage_key / container を自動取得
    azure_blob.put(azure_ctx, "proof/migrate.json", proof_json)
}
```

または VM primitive 直呼び:

```fav
bind _ <- AzureBlob.put_raw(account, key, container, "proof/migrate.json", proof_json)
```

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `AzureBlob.put_raw` VM primitive | PUT ブロブ (Shared Key 認証) |
| `AzureBlob.get_raw` VM primitive | GET ブロブ |
| `AzureBlob.list_raw` VM primitive | コンテナ一覧 (XML → JSON 変換) |
| `AzureBlob.delete_raw` VM primitive | DELETE ブロブ |
| `require_azure_storage_effect` + E0317 | `!AzureStorage` 未宣言エラー |
| `checker.rs` の `AzureBlob` namespace 登録 | builtin_ret_ty + BUILTIN_EFFECTS 更新 |
| `runes/azure-blob/azure_blob.fav` | ctx-aware ラッパー 4 関数 |
| `runes/azure-blob/rune.toml` | rune メタデータ |
| `v145000_tests` (4 件) | version / E0007 / E0317 / ファイル存在確認 |
| Cargo.toml バージョン `14.5.0` | |

### Out of Scope

- SAS トークン認証（v15.x 以降）
- Azure Data Lake Storage Gen2（別 Rune）
- `azure_storage` / `azure_storage_blobs` crate の導入（ureq + Shared Key で実装）
- CrossCloud E2E デモ（v15.0.0）

---

## 関数設計

### VM Primitives

```
AzureBlob.put_raw(account: String, key: String, container: String, blob_name: String, body: String)
    -> Result<Unit, String>   !AzureStorage 必須

AzureBlob.get_raw(account: String, key: String, container: String, blob_name: String)
    -> Result<String, String>  !AzureStorage 必須

AzureBlob.list_raw(account: String, key: String, container: String, prefix: String)
    -> Result<String, String>  !AzureStorage 必須  ※ JSON 配列文字列を返す

AzureBlob.delete_raw(account: String, key: String, container: String, blob_name: String)
    -> Result<Unit, String>   !AzureStorage 必須
```

### `runes/azure-blob/azure_blob.fav`（新規）

```fav
// runes/azure-blob/azure_blob.fav — Azure Blob Storage wrapper (v14.5.0)
// ctx: AzureCtx (String JSON from Ctx.build_azure_raw)
// フィールド: storage_account, storage_key, container

/// Upload a blob to Azure Blob Storage using AzureCtx.
public fn put(ctx: String, blob_name: String, body: String) -> Result<Unit, String> !AzureStorage {
    AzureBlob.put_raw(
        Ctx.azure_get_field_raw(ctx, "storage_account"),
        Ctx.azure_get_field_raw(ctx, "storage_key"),
        Ctx.azure_get_field_raw(ctx, "container"),
        blob_name,
        body
    )
}

/// Download a blob from Azure Blob Storage using AzureCtx.
public fn get(ctx: String, blob_name: String) -> Result<String, String> !AzureStorage {
    AzureBlob.get_raw(
        Ctx.azure_get_field_raw(ctx, "storage_account"),
        Ctx.azure_get_field_raw(ctx, "storage_key"),
        Ctx.azure_get_field_raw(ctx, "container"),
        blob_name
    )
}

/// List blobs in the container (returns JSON array of names) using AzureCtx.
public fn list(ctx: String, prefix: String) -> Result<String, String> !AzureStorage {
    AzureBlob.list_raw(
        Ctx.azure_get_field_raw(ctx, "storage_account"),
        Ctx.azure_get_field_raw(ctx, "storage_key"),
        Ctx.azure_get_field_raw(ctx, "container"),
        prefix
    )
}

/// Delete a blob from Azure Blob Storage using AzureCtx.
public fn delete(ctx: String, blob_name: String) -> Result<Unit, String> !AzureStorage {
    AzureBlob.delete_raw(
        Ctx.azure_get_field_raw(ctx, "storage_account"),
        Ctx.azure_get_field_raw(ctx, "storage_key"),
        Ctx.azure_get_field_raw(ctx, "container"),
        blob_name
    )
}
```

---

## Azure Blob Storage REST API 仕様

| 操作 | Method | URL パターン |
|---|---|---|
| PUT blob | PUT | `https://{account}.blob.core.windows.net/{container}/{blob_name}` |
| GET blob | GET | `https://{account}.blob.core.windows.net/{container}/{blob_name}` |
| LIST blobs | GET | `https://{account}.blob.core.windows.net/{container}?restype=container&comp=list&prefix={prefix}` |
| DELETE blob | DELETE | `https://{account}.blob.core.windows.net/{container}/{blob_name}` |

認証: **Azure Shared Key** — `Authorization: SharedKey {account}:{HMAC-SHA256}` ヘッダー

crate 依存: 既存の `hmac = "0.12"` + `sha2 = "0.10"` + `base64 = "0.22"` で実装可能（新規 crate 不要）。

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `AzureBlob.put_raw` が E0007 を出さない | ✅ |
| `AzureBlob.put_raw` を `!AzureStorage` なしで呼ぶと E0317 | ✅ |
| `runes/azure-blob/azure_blob.fav` が存在する | ✅ |
| `cargo test v145000` 全 4 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.5.0"` | ✅ |
