# Spec: v85.6.0 — `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク）

## Background

v85.5.0 で `odata_get` / `odata_list` の HTTP クライアント基盤を実装した。
本バージョンでは SAP 固有のエラー応答を型で表現し、わかりやすいエラーメッセージを返す基盤を追加する。
`runes/sap-odata/types.fav` に `SapErrorCode` 列挙型と `SapError` レコード型を追加する。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.6.0 セクション）

## Goals

- `runes/sap-odata/types.fav` に `SapErrorCode` 列挙型を追加する
- `runes/sap-odata/types.fav` に `SapError` レコード型を追加する
- Rust テスト 2 件を追加して **3,943 tests** を達成する

## Files to Create / Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/types.fav` | 追記 | `SapErrorCode` 列挙型 + `SapError` レコード型 |
| `fav/src/driver.rs` | 追記 | `mod v85600_tests`（テスト 2 件） |

## 型定義（`types.fav` に追記）

```favnir
-- SAP OData エラーコード列挙型
-- HTTP ステータスコードとのマッピング:
--   400 → BadRequest / 401 → Unauthorized / 403 → Forbidden
--   404 → NotFound / 5xx → ServerError / 接続失敗 → NetworkError
type SapErrorCode = NotFound | Unauthorized | Forbidden | BadRequest | ServerError | NetworkError

-- SAP OData エラー型
-- OData v4 エラーレスポンス（{"error": {"code": "...", "message": "..."}}）に対応
type SapError = {
    code:    SapErrorCode,
    message: String,
    detail:  Option<String>
}
```

### HTTP ステータスコードとのマッピング

| HTTP ステータス | `SapErrorCode` |
|---|---|
| 400 Bad Request | `BadRequest` |
| 401 Unauthorized | `Unauthorized` |
| 403 Forbidden | `Forbidden` |
| 404 Not Found | `NotFound` |
| 5xx Server Error | `ServerError` |
| 接続失敗（タイムアウト等） | `NetworkError` |

### OData v4 エラーレスポンス形式

```json
{
  "error": {
    "code": "404",
    "message": "Entity not found",
    "innererror": {
      "errordetails": [...]
    }
  }
}
```

`SapError.detail` には `innererror` の文字列表現を格納する（後続バージョンで実装）。

## Success Criteria

- `cargo test` が **3,943 tests**, 0 failures
- `sap_error_type_exists`:
  - `runes/sap-odata/types.fav` に `SapError` が含まれる（ファイル内容チェック）
- `sap_error_code_variants_exist`:
  - `runes/sap-odata/types.fav` に `SapErrorCode` が含まれる（ファイル内容チェック）

## Error Codes

新規 Favnir エラーコードなし（`SapErrorCode` は Favnir の型定義であり、コンパイラエラーコードではない）。

## 注記

- `SapError` を使った実際のエラーハンドリング（`odata_get` / `odata_list` の戻り型変更）は v85.9.0 安定化バージョンで実施
- テストはファイル内容の文字列チェックのみ
- テストのファイルパス: `../runes/sap-odata/types.fav`（`cargo test` は `fav/` をカレントとして実行）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
