# Plan: v99.2.0 — `!Audit` エフェクトマーカー + 監査ログ ctx interface

## 実装順序

### Step 1: audit.fav を新規作成

`runes/sap-odata/audit.fav` を作成。

```favnir
-- runes/sap-odata/audit.fav
-- SAP 監査ログ型定義（v99.2.0）

-- 監査イベント（SAP データアクセス 1 件につき 1 レコード）
public type AuditEvent = {
    actor:     String,
    action:    String,
    resource:  String,
    timestamp: String,
    result:    String
}

-- 監査証跡（複数 AuditEvent のコレクション）
public type AuditTrail = {
    events:    List<AuditEvent>,
    pipeline:  String,
    started_at: String
}

-- 監査ログクライアント interface（本番: CloudWatch / Splunk 等）
public interface AuditClient {
    fn log(event: AuditEvent) -> Result<Unit, String>
}

-- テスト用モック: 受け取った AuditEvent を受理して Result.ok を返す
public fn log_audit_event_mock(event: AuditEvent) -> Result<Unit, String> {
    Result.ok(Unit)
}
```

コメントは `--` スタイル。`//` 不使用。

---

### Step 2: sap_odata.fav に use と re-export を追加

`runes/sap-odata/sap_odata.fav` に以下を追加する:

1. `use` 宣言（他の `use` 行と同じ場所）:
   ```
   use sap_odata.audit
   ```

2. re-export ブロック（ファイル末尾、BTP 認証セクションの後）:
   ```favnir
   -- 監査ログ型 re-export（v99.2.0〜）
   public type AuditEvent  = audit.AuditEvent
   public type AuditTrail  = audit.AuditTrail
   public fn log_audit_event_mock(event: audit.AuditEvent) -> Result<Unit, String> {
       audit.log_audit_event_mock(event)
   }
   ```

---

### Step 3: ctx.fav に audit フィールドを追加

`runes/ctx/ctx.fav` に以下を追加する:

1. `use` 宣言ブロックに追加:
   ```
   use sap_odata.audit
   ```

2. `AppCtx` 型定義の `approval: ApprovalClient` フィールドの後に追記:
   ```
       audit: AuditClient,      -- 監査ログクライアント（runes/sap-odata/audit.fav: AuditClient）（v99.2.0 追加）
   ```

---

### Step 4: driver.rs に mod v99200_tests を追加

`mod v99100_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99200_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn audit_fav_exists() {
        std::fs::read_to_string(
            "../runes/sap-odata/audit.fav",
        )
        .expect("audit.fav should exist (v99.2.0)");
    }

    #[test]
    fn audit_fav_has_audit_event() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/audit.fav",
        )
        .expect("audit.fav should exist (v99.2.0)");
        assert!(
            content.contains("AuditEvent"),
            "audit.fav should define AuditEvent (v99.2.0)"
        );
        assert!(
            content.contains("AuditTrail"),
            "audit.fav should define AuditTrail (v99.2.0)"
        );
        assert!(
            content.contains("AuditClient"),
            "audit.fav should define AuditClient (v99.2.0)"
        );
        assert!(
            content.contains("log_audit_event_mock"),
            "audit.fav should define log_audit_event_mock (v99.2.0)"
        );
    }
}
```

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,261 tests, 0 failures

---

### Step 6: CHANGELOG.md に v99.2.0 エントリを追加

---

### Step 7: versions/current.md 更新

最新安定版を `v99.2.0` に更新（テスト数 4,261）。

---

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
