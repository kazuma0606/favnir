# Plan: v96.1.0 — `SapEnvironment` 型 + `ctx.sap_env()`

## 実装ステップ

### Step 1: `runes/sap-odata/types.fav` に `SapEnvironment` 型を追加

`SapClient` interface 定義の直後（ファイル末尾）に追加する。

```favnir
-- SAP 環境を型安全に表現する直和型（v96.1.0）
-- PRD: 本番環境 / QAS: 品質保証環境 / DEV: 開発・サンドボックス環境
-- Custom(String): 上記以外のカスタム環境（環境名を文字列で保持）
public type SapEnvironment =
    | Prd
    | Qas
    | Dev
    | Custom(String)

-- 環境名文字列から SapEnvironment を生成するユーティリティ（v96.1.0）
-- "PRD" → Prd, "QAS" → Qas, "DEV" → Dev, それ以外 → Custom(name)
public fn SapEnvironment.from_string(name: String) -> SapEnvironment {
    match name {
        "PRD" -> Prd
        "QAS" -> Qas
        "DEV" -> Dev
        _     -> Custom(name)
    }
}
```

### Step 2: `runes/ctx/ctx.fav` に `Ctx.sap_env()` 関数を追加

`Ctx.mock` 関数の直後（ファイル末尾）に追加する。

```favnir
-- 環境名から SapClient を取得する（v96.1.0 スタブ）
-- NOTE: v96.2.0 で fav.toml [sap.environments] と接続する。現バージョンはスタブ。
public fn Ctx.sap_env(name: String) -> Result<SapClient, String> {
    Result.err("sap_env not implemented: use Ctx.build() for now")
}
```

### Step 3: `fav/src/driver.rs` に `mod v96100_tests` を追加

`mod v96000_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v96100_tests {
    #[test]
    fn sap_environment_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("types.fav should exist");
        assert!(
            content.contains("SapEnvironment"),
            "types.fav should define SapEnvironment type"
        );
    }

    #[test]
    fn ctx_sap_env_fn_defined() {
        let content = std::fs::read_to_string("../runes/ctx/ctx.fav")
            .expect("ctx.fav should exist");
        assert!(
            content.contains("sap_env"),
            "ctx.fav should define Ctx.sap_env function"
        );
    }
}
```

## 依存関係

```
Step 1 (types.fav) → Step 2 (ctx.fav) → Step 3 (driver.rs テスト)
```

- Step 2 は Step 1 の型（`SapClient`）を参照するが、`SapClient` は既存のため Step 1 完了後でなくても追加可能
- Step 3 は Step 1・Step 2 の成果物に対してファイル内容をアサートするため、Step 1・2 完了後に実施する
