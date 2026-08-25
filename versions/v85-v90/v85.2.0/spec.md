# Spec: v85.2.0 — `SapConfig` Favnir 型 + `sap_config_from_env()`

## Background

v85.1.0 で Rust 側の `fav.toml [sap]` 解析・env 注入基盤が完成した。
本バージョンでは SAP 接続設定を **Favnir の型** として表現し、
`runes/sap-odata/types.fav` に `SapConfig` 型と `sap_config_from_env()` 関数を定義する。

これにより Favnir コードから `bind cfg <- sap_odata.sap_config_from_env()` で設定を取得し、
後続バージョンで実装する SAP Rune 関数に渡せるようになる。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.2.0 セクション）

## Goals

- `runes/sap-odata/` ディレクトリを作成する（骨格 — v85.4.0 で完全化）
- `runes/sap-odata/types.fav` に `SapConfig` 型と `sap_config_from_env()` を定義する
- Rust テスト 2 件を追加して **3,935 tests** を達成する

## API / Type Definitions

```favnir
-- runes/sap-odata/types.fav

type SapConfig = {
    base_url: String,
    client:   String,
    username: String,
    password: String,
    auth:     String
}

-- Env.require は Result<String, String> を返す → bind を使う
-- Env.get_or はデフォルト値付きで String を直接返す → bind 不要
public fn sap_config_from_env() -> Result<SapConfig, String> {
    bind base_url <- Env.require("SAP_BASE_URL")
    bind username <- Env.require("SAP_USER")
    bind password <- Env.require("SAP_PASS")
    Result.ok(SapConfig {
        base_url,
        username,
        password,
        client: Env.get_or("SAP_CLIENT", "100"),
        auth:   Env.get_or("SAP_AUTH", "basic")
    })
}
```

### 使用例

```favnir
fn connect(ctx: AppCtx) -> Result<Unit, String> {
    bind cfg <- sap_odata.sap_config_from_env()
    -- cfg.base_url / cfg.username / cfg.password 等を利用して API 呼び出し
    Result.ok(Unit)
}
```

## Success Criteria

- `cargo test` が **3,935 tests**, 0 failures
- `sap_config_from_env_returns_ok_when_vars_set`:
  - `SAP_BASE_URL` / `SAP_USER` / `SAP_PASS` が設定された状態で、
    `types.fav` に `sap_config_from_env` 関数が定義されていることを確認する（ファイル存在 + 文字列チェック）
- `sap_config_from_env_returns_err_when_base_url_missing`:
  - `types.fav` に `Env.require("SAP_BASE_URL")` が含まれることを確認する（エラーパス検証）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/types.fav` | 新規作成 | `SapConfig` 型 + `sap_config_from_env()` 関数 |
| `fav/src/driver.rs` | 追記 | `mod v85200_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `runes/sap-odata/` ディレクトリは本バージョンで作成する（`rune.toml` / `sap_odata.fav` は v85.4.0 で追加）
- Rust テストは Favnir ファイルの存在・内容をチェックする形式（`include_str!` 相当のファイル読み取り）
- `sap_config_from_env` テストは env var を実際に注入・実行するのではなく、ファイル内容の文字列確認で行う
- `Env.require` は `Result<String, String>` を返すため `bind` が必須
- `Env.get_or` はデフォルト値付きで `String` を直接返すため `bind` は使わない
- テストのファイルパス: `../runes/sap-odata/types.fav`（`cargo test` は `fav/` をカレントディレクトリとして実行するため `..` 1 段で `runes/` に到達する）
