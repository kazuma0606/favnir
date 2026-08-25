# Spec: v85.4.0 — `runes/sap-odata/` 骨格 + `rune.toml`

## Background

v85.2.0 で `runes/sap-odata/types.fav` を作成し、v85.3.0 で Docker Compose モック環境を整備した。
本バージョンでは `rune.toml` と `sap_odata.fav`（エントリポイント）を追加し、
`sap-odata` Rune を Rune Registry に登録できる完全な骨格構造を完成させる。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.4.0 セクション）

## Goals

- `runes/sap-odata/rune.toml` を作成する（Rune メタデータ）
- `runes/sap-odata/sap_odata.fav` を作成する（エントリポイント — `types.fav` を use）
- `runes/sap-odata/sap_odata.test.fav` を作成する（テストファイル骨格）
- Rust テスト 2 件を追加して **3,939 tests** を達成する

## Files to Create / Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/rune.toml` | 新規作成 | Rune メタデータ（name / version / entry / description） |
| `runes/sap-odata/sap_odata.fav` | 新規作成 | エントリポイント（`types.fav` を use） |
| `runes/sap-odata/sap_odata.test.fav` | 新規作成 | テストファイル骨格 |
| `fav/src/driver.rs` | 追記 | `mod v85400_tests`（テスト 2 件） |

### 既存ファイル（変更なし）

- `runes/sap-odata/types.fav` — v85.2.0 で作成済み（`SapConfig` 型 + `sap_config_from_env()`）

## `rune.toml` 設計

```toml
[rune]
name        = "sap-odata"
version     = "85.4.0"
entry       = "sap_odata.fav"
description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"
```

`effects` フィールドは省略（ctx パターンのため `!Sap` エフェクト不要）。

## `sap_odata.fav` 設計

エントリポイントとして `types.fav` を use し、後続バージョンで追加される関数群（`client.fav` 等）への参照を準備する。

```favnir
-- sap-odata Rune エントリポイント（v85.4.0）
-- 後続バージョンで client.fav / error.fav 等を use 追加する。

use sap_odata.types

-- re-export: 利用者が sap_odata.SapConfig / sap_odata.sap_config_from_env を使えるようにする
public type SapConfig   = types.SapConfig
public fn   sap_config_from_env() -> Result<SapConfig, String> {
    types.sap_config_from_env()
}
```

## `sap_odata.test.fav` 設計

```favnir
-- sap-odata Rune テスト骨格（v85.4.0）
-- 後続バージョンで E2E テストを追加する。

fn test_sap_config_fields_exist() -> Bool {
    -- SapConfig のフィールドが定義されていることを確認する（骨格テスト）
    True
}
```

## Success Criteria

- `cargo test` が **3,939 tests**, 0 failures
- `sap_odata_rune_toml_exists`:
  - `runes/sap-odata/rune.toml` が存在する（ファイル存在チェック）
- `sap_odata_rune_entry_exists`:
  - `runes/sap-odata/sap_odata.fav` が存在する（ファイル存在チェック）

## Error Codes

新規エラーコードなし。

## 注記

- `runes/sap-odata/types.fav` は v85.2.0 で作成済みのため本バージョンで変更しない
- `sap_odata.fav` は後続バージョン（v85.5.0〜）で `client.fav` を use 追加する拡張点
- テストのファイルパス: `../runes/sap-odata/rune.toml`（`cargo test` は `fav/` をカレントとして実行するため `../` 1 段で `favnir/` に到達）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
