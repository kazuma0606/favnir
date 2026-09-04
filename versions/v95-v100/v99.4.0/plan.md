# Plan: v99.4.0 — マルチテナント対応

## 実装順序

### Step 1: tenant.fav を新規作成

`runes/sap-odata/tenant.fav` を作成。

```favnir
-- runes/sap-odata/tenant.fav
-- SAP マルチテナント型定義（v99.4.0）

use sap_odata.types

-- テナント識別子
public type TenantId = String

-- テナントコンテキスト（テナントごとの SAP 接続設定）
-- SapEnvironment は v96.1.0 で定義済み（Prd / Qas / Dev / Custom(String)）
public type TenantContext = {
    tenant_id: TenantId,
    sap_env:   types.SapEnvironment,
    schema:    String
}

-- テスト用モック: TenantId から TenantContext を生成する
public fn tenant_context_mock(tenant_id: TenantId) -> TenantContext {
    TenantContext {
        tenant_id: tenant_id,
        sap_env:   types.SapEnvironment.Dev,
        schema:    String.concat(["schema_", tenant_id])
    }
}
```

コメントは `--` スタイル。`//` 不使用。

---

### Step 2: sap_odata.fav に use と re-export を追加

`runes/sap-odata/sap_odata.fav` に以下を追加する:

1. `use` 宣言（既存 `use sap_odata.resilience` の直後・`use` ブロック末尾）:
   ```
   use sap_odata.tenant
   ```

2. re-export ブロック（ファイル末尾、Circuit Breaker セクションの後）:
   ```favnir
   -- マルチテナント型 re-export（v99.4.0〜）
   public type TenantId      = tenant.TenantId
   public type TenantContext = tenant.TenantContext
   public fn tenant_context_mock(tenant_id: tenant.TenantId) -> TenantContext {
       tenant.tenant_context_mock(tenant_id)
   }
   ```

---

### Step 3: ctx.fav に Ctx.for_tenant_mock を追加

`runes/ctx/ctx.fav` に以下を追加する:

1. `use` 宣言ブロックに追加:
   ```
   use sap_odata.tenant
   ```

2. `Ctx.sap_env` 関数の後に追加:
   ```favnir
   -- テスト用 TenantContext を生成する（v99.4.0）
   public fn Ctx.for_tenant_mock(tenant_id: String) -> tenant.TenantContext {
       tenant.tenant_context_mock(tenant_id)
   }
   ```

---

### Step 4: driver.rs に mod v99400_tests を追加

`mod v99300_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99400_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn tenant_fav_exists() {
        std::fs::read_to_string(
            "../runes/sap-odata/tenant.fav",
        )
        .expect("tenant.fav should exist (v99.4.0)");
    }

    #[test]
    fn tenant_fav_has_tenant_context() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/tenant.fav",
        )
        .expect("tenant.fav should exist (v99.4.0)");
        assert!(
            content.contains("TenantId"),
            "tenant.fav should define TenantId (v99.4.0)"
        );
        assert!(
            content.contains("TenantContext"),
            "tenant.fav should define TenantContext (v99.4.0)"
        );
        assert!(
            content.contains("tenant_context_mock"),
            "tenant.fav should define tenant_context_mock (v99.4.0)"
        );
    }
}
```

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,265 tests, 0 failures

---

### Step 6: CHANGELOG.md に v99.4.0 エントリを追加

---

### Step 7: versions/current.md 更新

最新安定版を `v99.4.0` に更新（テスト数 4,265）。

---

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
