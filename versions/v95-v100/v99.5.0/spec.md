# Spec: v99.5.0 — GDPR データマスキング

## Background

v99.4.0 でマルチテナント対応（`TenantContext`）を追加した。
v99.5.0 では GDPR 準拠の個人情報（PII）保護を目的とし、`Masked<T>` 型ラッパーと
`UnmaskClient` ctx interface を追加する。

`Masked<T>` は純粋な型ラッパーであり、PII フィールドへの意図しないアクセスを
型レベルで防ぐ設計パターンを提供する。

> **Note**: ロードマップには `Effect::Unmask` を Rust `Effect` enum に追加し
> `checker.fav` の exhaustive match を更新するとあるが、`Effect` enum の変更は
> `effect_catalog.rs` / `checker.rs` / `compiler.rs` / `vm.rs` / `checker.fav` への
> 波及が伴うため、本バージョンのスコープ外とする。
> 本バージョンでは型定義（`Masked<T>` / `UnmaskClient`）とヘルパー関数
> （`mask` / `unmask_mock`）のみを提供し、実際の `!Unmask` effect 構文サポートは
> 将来バージョンで対応する。

## Goals

1. `runes/sap-odata/privacy.fav` — `Masked<T>` 型 + `UnmaskClient` interface + `mask` / `unmask_mock` 関数を新規作成
2. `runes/sap-odata/sap_odata.fav` — `use sap_odata.privacy` + 4 シンボル re-export 追加（`Masked<T>` / `UnmaskClient` / `mask` / `unmask_mock`）
3. `runes/ctx/ctx.fav` — `use sap_odata.privacy` + `AppCtx` に `unmask: UnmaskClient` フィールド追加
4. `fav/src/driver.rs` — `mod v99500_tests`（2 テスト）追加

## Syntax / API Examples

### privacy.fav

```favnir
-- runes/sap-odata/privacy.fav
-- GDPR データマスキング型定義（v99.5.0）

-- PII フィールドをラップする型（effect Unmask 宣言なし）
public type Masked<T> = { inner: T }

-- アンマスク操作を提供する interface
public interface UnmaskClient {
    fn unmask<T>(masked: Masked<T>) -> Result<T, String>
}

-- T を Masked<T> にラップする
public fn mask<T>(value: T) -> Masked<T> {
    Masked { inner: value }
}

-- テスト用モック: Masked<T> をアンマスクして T を返す
public fn unmask_mock<T>(masked: Masked<T>) -> Result<T, String> {
    Result.ok(masked.inner)
}
```

### ctx.fav への追加

```favnir
-- AppCtx フィールド追加（v99.5.0）
unmask: UnmaskClient,   -- GDPR アンマスク（v99.5.0 追加）
```

### 使用例（pipeline）

```favnir
import rune "sap-odata"

-- PII フィールドを含む型
type BusinessPartnerPii = {
    partner_id: String,
    email:      Masked<String>,
    phone:      Masked<String>
}

-- テスト用: マスクとアンマスクの確認
-- 本バージョンでは unmask_mock を使用（ctx.unmask.unmask() は UnmaskClient 実装後に使用可能）
pipeline mask_demo {
    stage Mask {
        bind masked_email <- Result.ok(mask("user@example.com"))
    }
    |> stage Unmask {
        bind raw_email <- unmask_mock(masked_email)
    }
}
```

## Success Criteria

- `runes/sap-odata/privacy.fav` が存在する
- `privacy.fav` に `Masked` が含まれる
- `privacy.fav` に `UnmaskClient` が含まれる
- `privacy.fav` に `mask` が含まれる
- `privacy.fav` に `unmask_mock` が含まれる
- `runes/sap-odata/sap_odata.fav` に `Masked` / `UnmaskClient` / `mask` / `unmask_mock` の re-export が含まれる（目視確認）
- `runes/ctx/ctx.fav` に `unmask: UnmaskClient` フィールドが含まれる（目視確認）
- `CHANGELOG.md` に `[v99.5.0]` エントリが含まれる
- `cargo test -- --test-threads=1` が 4,267 tests, 0 failures で通過する

> **Note**: `sap_odata.fav` の re-export 追加と `ctx.fav` の `unmask` フィールド追加は
> driver.rs テストではなく目視確認（Tasks T2 / T3）で検証する。
> テスト数は 4,267（+2）のまま維持する。

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/privacy.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.privacy` + 4 シンボル re-export 追加 |
| `runes/ctx/ctx.fav` | `use sap_odata.privacy` + `unmask: UnmaskClient` フィールド追加 |
| `fav/src/driver.rs` | 追記（`mod v99500_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.4.0 完了後の 4,265。v99.5.0 の目標は 4,265 + 2 = **4,267**。
