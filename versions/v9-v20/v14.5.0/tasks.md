# v14.5.0 Tasks — Azure Blob Storage Rune (runes/azure-blob/)

Date: 2026-06-12
Branch: master

---

## Phase A — `fav/src/backend/vm.rs`: AzureBlob VM プリミティブ追加

- [x] A-1: `azure_blob_sign` ヘルパー関数を追加
  - 追加場所: `url_encode` の直後（`// ── Env config` の前）
  - 既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` を使用（新規 crate 不要）
  - 引数: `account, key_b64, method, content_type, content_length, x_ms_blob_type, canonical_resource`
  - 返り値: `Result<(String, String), String>`（x-ms-date, Authorization ヘッダー値）
  - chrono で RFC 1123 日付生成、HMAC-SHA256 + base64 で Shared Key 署名

- [x] A-2: `AzureBlob.put_raw` ハンドラ追加
  - 引数 5 個: `account, key, container, blob_name, body`
  - ureq PUT + Shared Key 署名、BlockBlob タイプ
  - 返り値: `Result<Unit, String>`

- [x] A-3: `AzureBlob.get_raw` ハンドラ追加
  - 引数 4 個: `account, key, container, blob_name`
  - ureq GET + Shared Key 署名
  - 返り値: `Result<String, String>`

- [x] A-4: `AzureBlob.list_raw` ハンドラ追加
  - 引数 4 個: `account, key, container, prefix`
  - ureq GET + Shared Key 署名、`extract_xml_tags` で `<Name>` 抽出
  - canonical_resource: `"/{account}/{container}\ncomp:list\nprefix:{prefix}\nrestype:container"`
  - 返り値: `Result<String, String>`（`["blob1", "blob2", ...]`）

- [x] A-5: `AzureBlob.delete_raw` ハンドラ追加
  - 引数 4 個: `account, key, container, blob_name`
  - `ureq::request("DELETE", &url)` + Shared Key 署名
  - 返り値: `Result<Unit, String>`

- [x] A-6: `cargo build` でコンパイルエラーなし確認

---

## Phase B — `fav/src/middle/checker.rs`: AzureBlob namespace 登録

- [x] B-1: `require_azure_storage_effect` 追加（`require_azure_db_effect` の直後）
  ```rust
  fn require_azure_storage_effect(&mut self, span: &Span) {
      if !self.has_effect(|e| matches!(e, Effect::AzureStorage)) {
          self.type_error(
              "E0317",
              "AzureBlob.* call requires `!AzureStorage` effect on enclosing fn/stage",
              span,
          );
      }
  }
  ```

- [x] B-2: `builtin_ret_ty` に `AzureBlob.*` 追加（`("AzurePostgres", _)` ブロックの直後）
  - `("AzureBlob", "put_raw")` → `require_azure_storage_effect` + `Result<Unit, String>`
  - `("AzureBlob", "get_raw")` → `require_azure_storage_effect` + `Result<String, String>`
  - `("AzureBlob", "list_raw")` → `require_azure_storage_effect` + `Result<String, String>`
  - `("AzureBlob", "delete_raw")` → `require_azure_storage_effect` + `Result<Unit, String>`
  - `("AzureBlob", _)` → `require_azure_storage_effect` + `Type::Unknown`（フォールバック）

- [x] B-3: `BUILTIN_EFFECTS`（~line 1422）に `"AzureBlob"` 追加
  （`"AzurePostgres"` の隣）

- [x] B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C — `runes/azure-blob/` 新規作成

- [x] C-1: `C:\Users\yoshi\favnir\runes\azure-blob\azure_blob.fav` を新規作成
  - `put(ctx: String, blob_name: String, body: String) -> Result<Unit, String> !AzureStorage`
  - `get(ctx: String, blob_name: String) -> Result<String, String> !AzureStorage`
  - `list(ctx: String, prefix: String) -> Result<String, String> !AzureStorage`
  - `delete(ctx: String, blob_name: String) -> Result<Unit, String> !AzureStorage`
  - 各関数: `Ctx.azure_get_field_raw(ctx, "storage_account/storage_key/container")` でフィールド取得
  - **注記**: `import rune "ctx"` は省略（`runes/ctx/ctx.fav` 未存在）。`ctx: String` で代替。
  - 関数本体は引数を直接インライン化（`let` 構文は rune ファイルでパースエラーになる）

- [x] C-2: `C:\Users\yoshi\favnir\runes\azure-blob\rune.toml` を新規作成
  ```toml
  [rune]
  name        = "azure-blob"
  version     = "14.5.0"
  description = "Azure Blob Storage: put/get/list/delete with Shared Key authentication"
  entry       = "azure_blob.fav"
  effects     = ["!AzureStorage"]

  [dependencies]
  ```

- [x] C-3: `cargo test` でリグレッションなし確認

---

## Phase D — `fav/src/driver.rs`: v145000_tests + バージョンバンプ

- [x] D-1: `v145000_tests` モジュールを追加（`v144000_tests` の直前）
  - [x] `version_is_14_5_0` — `CARGO_PKG_VERSION == "14.5.0"` 確認
  - [x] `azure_blob_put_raw_registered` — `AzureBlob.put_raw` で E0007 が出ない確認
  - [x] `azure_storage_effect_required` — `!AzureStorage` なしで E0317 が出る確認
  - [x] `azure_blob_rune_file_present` — `runes/azure-blob/azure_blob.fav` に `fn put`/`fn get` が存在

- [x] D-2: `v144000_tests` の `version_is_14_4_0` を `>=` 比較に修正
  ```rust
  assert!(env!("CARGO_PKG_VERSION") >= "14.4.0", ...);
  ```

- [x] D-3: `fav/Cargo.toml` バージョンを `"14.5.0"` にバンプ

- [x] D-4: `cargo test v145000` で 4 件全パス確認

---

## Phase E — 全テスト + コミット

- [x] E-1: `cargo test v145000` 全 4 件パス
- [x] E-2: `cargo test` 全件パス（リグレッションなし）
- [x] E-3: `git commit -m "feat: v14.5.0 — Azure Blob Storage Rune (AzureBlob.put_raw/get_raw/list_raw/delete_raw)"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `AzureBlob.put_raw` が E0007 を出さない | [x] |
| `AzureBlob.put_raw` を `!AzureStorage` なしで呼ぶと E0317 | [x] |
| `runes/azure-blob/azure_blob.fav` が存在し `fn put`/`fn get` を含む | [x] |
| `cargo test v145000` 全 4 件パス | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `CARGO_PKG_VERSION == "14.5.0"` | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.5.0/spec.md` | 仕様・ユーザー体験 |
| `versions/v14.5.0/plan.md` | 実装詳細・コードスニペット |
| `versions/v14.4.0/tasks.md` | 先行バージョンのパターン参照（実装メモ含む） |
| `versions/roadmap-v14.1-v15.0.md` | v14.5.0 の位置づけ・依存関係 |

## 実装メモ

- `azure_blob_sign` の StringToSign: 13 フィールド（method + 11 空行 + canonical_headers + canonical_resource）
- `x-ms-blob-type` は PUT 時のみ設定（GET/DELETE/LIST では省略）
- `ms_headers` は sort してから canonical_headers に結合（`x-ms-date`, `x-ms-version`, `x-ms-blob-type` の順）
- LIST の canonical_resource は query params をアルファベット順にソート: `comp:list\nprefix:{prefix}\nrestype:container`
- `extract_xml_tags` ヘルパー（既存）で XML `<Name>` タグを抽出し JSON 配列文字列を生成
- `rune.toml` の `effects = ["!AzureStorage"]` はメタデータのみ（VM ではチェックしない）
