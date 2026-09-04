---
name: SAP OData Bug Report
about: SAP OData Rune に関するバグ報告（接続エラー・型不一致・$batch 失敗など）
labels: bug, sap-odata
assignees: ''
---

## 環境

- **Favnir バージョン**: （例: v94.5.0 — `./target/debug/fav --version` で確認）
- **SAP S/4HANA バージョン**: （例: S/4HANA 2023 FPS01）
- **SAP Gateway バージョン**: （例: 7.50）
- **OS**: （例: macOS 14.5 / Ubuntu 22.04 / Windows 11）
- **Rust バージョン**: （`rustc --version` で確認）

## 再現手順

1. `fav.toml` の `[sap]` 設定:
   ```toml
   [sap]
   base_url  = "..."
   client_id = "..."
   ```
2. 実行したコマンド:
   ```bash
   ./target/debug/fav ...
   ```
3. 使用した Favnir コード（最小再現例）:
   ```favnir
   ...
   ```

## 期待する動作

<!-- どのような結果を期待していたか記述してください -->

## 実際の動作

<!-- 実際に起きたこと（エラーメッセージ・スタックトレース等）を記述してください -->

## エラーログ

```
（エラー出力をここに貼り付けてください）
```

## 追加情報

- `fav doctor` の出力（接続エラーの場合のみ）:
  ```
  （出力をここに貼り付けてください — 接続設定がない場合は省略可）
  ```
- 関連する SAP OData エンドポイント（例: `A_BusinessPartner`）:
- 試したワークアラウンドがあれば記述してください:
