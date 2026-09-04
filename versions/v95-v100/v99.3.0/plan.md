# Plan: v99.3.0 — Rate Limiting / Circuit Breaker

## 実装順序

### Step 1: resilience.fav を新規作成

`runes/sap-odata/resilience.fav` を作成。

```favnir
-- runes/sap-odata/resilience.fav
-- SAP API Rate Limiting / Circuit Breaker 型定義（v99.3.0）

-- Circuit Breaker の状態
public type CircuitState =
    | Closed     -- 通常動作
    | Open       -- トリップ中（リクエストを遮断）
    | HalfOpen   -- 復旧試行中

-- Circuit Breaker 設定
public type CircuitBreaker<T> = {
    state:            CircuitState,
    failure_count:    Int,
    threshold:        Int,
    reset_timeout_ms: Int,
    tag:              String
}

-- デフォルト設定の CircuitBreaker を生成する
public fn circuit_breaker_default<T>(tag: String) -> CircuitBreaker<T> {
    CircuitBreaker {
        state:            Closed,
        failure_count:    0,
        threshold:        5,
        reset_timeout_ms: 30000,
        tag:              tag
    }
}

-- テスト用モック: Closed 状態なら value を返す、Open なら Result.err を返す
public fn circuit_breaker_call_mock<T>(cb: CircuitBreaker<T>, value: T) -> Result<T, String> {
    match cb.state {
        Closed   -> Result.ok(value)
        HalfOpen -> Result.ok(value)
        Open     -> Result.err(String.concat(["circuit open: ", cb.tag]))
    }
}
```

コメントは `--` スタイル。`//` 不使用。

---

### Step 2: sap_odata.fav に use と re-export を追加

`runes/sap-odata/sap_odata.fav` に以下を追加する:

1. `use` 宣言（他の `use` 行と同じ場所）:
   ```
   use sap_odata.resilience
   ```

2. re-export ブロック（ファイル末尾、監査ログセクションの後）:
   ```favnir
   -- Circuit Breaker 型 re-export（v99.3.0〜）
   public type CircuitState        = resilience.CircuitState
   public type CircuitBreaker<T>   = resilience.CircuitBreaker<T>
   public fn circuit_breaker_default<T>(tag: String) -> resilience.CircuitBreaker<T> {
       resilience.circuit_breaker_default(tag)
   }
   public fn circuit_breaker_call_mock<T>(cb: resilience.CircuitBreaker<T>, value: T) -> Result<T, String> {
       resilience.circuit_breaker_call_mock(cb, value)
   }
   ```

---

### Step 3: driver.rs に mod v99300_tests を追加

`mod v99200_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99300_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn resilience_fav_exists() {
        std::fs::read_to_string(
            "../runes/sap-odata/resilience.fav",
        )
        .expect("resilience.fav should exist (v99.3.0)");
    }

    #[test]
    fn resilience_fav_has_circuit_breaker() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/resilience.fav",
        )
        .expect("resilience.fav should exist (v99.3.0)");
        assert!(
            content.contains("CircuitState"),
            "resilience.fav should define CircuitState (v99.3.0)"
        );
        assert!(
            content.contains("CircuitBreaker"),
            "resilience.fav should define CircuitBreaker (v99.3.0)"
        );
        assert!(
            content.contains("circuit_breaker_call_mock"),
            "resilience.fav should define circuit_breaker_call_mock (v99.3.0)"
        );
    }
}
```

---

### Step 4: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,263 tests, 0 failures

---

### Step 5: CHANGELOG.md に v99.3.0 エントリを追加

---

### Step 6: versions/current.md 更新

最新安定版を `v99.3.0` に更新（テスト数 4,263）。

---

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
