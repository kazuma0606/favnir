# Spec: v96.1.0 — `SapEnvironment` 型 + `ctx.sap_env()`

## Background

SAP 本番環境では PRD（本番）/ QAS（品質保証）/ DEV（開発）の3環境を常に意識する必要がある。
v96.0.0 までの実装では `SapConfig` の `base_url` を直接書き換える運用しかなく、
パイプライン内で安全に環境を切り替える型安全な手段が存在しなかった。

v96.1.0 では `SapEnvironment` 直和型を導入し、`ctx.sap_env("PRD")` で
名前から環境別 `SapClient` を取得できるようにする。

これは SAP Multi-system 1.0（v97.0.0）スプリントの起点であり、
後続の `fav.toml [sap.environments]` マルチ環境設定（v96.2.0）の基盤となる。

## Goals

1. `SapEnvironment` 直和型を `runes/sap-odata/types.fav` に追加する
2. `Ctx.sap_env(name: String) -> Result<SapClient, String>` を `runes/ctx/ctx.fav` に追加する
3. `driver.rs` に `mod v96100_tests`（2 テスト）を追加する

## 型定義

```favnir
-- SAP 環境を型安全に表現する直和型（v96.1.0）
-- PRD: 本番環境 / QAS: 品質保証環境 / DEV: 開発・サンドボックス環境
-- Custom(String): 上記以外のカスタム環境（環境名を文字列で保持）
public type SapEnvironment =
    | Prd
    | Qas
    | Dev
    | Custom(String)
```

## API 仕様

```favnir
-- 環境名文字列から SapEnvironment を生成するユーティリティ関数
-- "PRD" → Prd, "QAS" → Qas, "DEV" → Dev, それ以外 → Custom(name)
public fn SapEnvironment.from_string(name: String) -> SapEnvironment {
    match name {
        "PRD" -> Prd
        "QAS" -> Qas
        "DEV" -> Dev
        _     -> Custom(name)
    }
}

-- 環境名から SapClient を取得する（v96.1.0 スタブ）
-- NOTE: v96.2.0 で fav.toml [sap.environments] のマルチ環境設定と接続する。
-- 現バージョンはスタブとして Result.err を返す。
public fn Ctx.sap_env(name: String) -> Result<SapClient, String> {
    Result.err("sap_env not implemented: use Ctx.build() for now")
}
```

## 利用例

```favnir
-- pipeline 内で環境を切り替える使用例（v96.2.0 以降で実動する）
pipeline fetch_prd_data !SapOData {
    stage Fetch {
        bind sap_prd <- ctx.sap_env("PRD")
        bind bps     <- sap_prd.business_partners(BusinessPartnerFilter {
            country: Option.some("JP"), category: Option.none(),
            changed_after: Option.none(), top: Option.some(100)
        })
    }
}
```

## Success Criteria

- `runes/sap-odata/types.fav` に `SapEnvironment` 型（`Prd / Qas / Dev / Custom(String)`）と `SapEnvironment.from_string` 関数が追加される
- `runes/ctx/ctx.fav` に `Ctx.sap_env(name: String) -> Result<SapClient, String>` 関数が追加される
- `cargo test` で 4,190 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/types.fav` | `SapEnvironment` 直和型 + `SapEnvironment.from_string` 関数を追加 |
| `runes/ctx/ctx.fav` | `Ctx.sap_env(name: String) -> Result<SapClient, String>` スタブ関数を追加 |
| `fav/src/driver.rs` | `mod v96100_tests`（2 テスト）を追加 |
