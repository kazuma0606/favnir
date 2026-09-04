# Plan: v99.5.0 — GDPR データマスキング

## 実装順序

### Step 1: privacy.fav を新規作成

`runes/sap-odata/privacy.fav` を作成。

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

コメントは `--` スタイル。`//` 不使用。

---

### Step 2: sap_odata.fav に use と re-export を追加

`runes/sap-odata/sap_odata.fav` に以下を追加する:

1. `use` 宣言（既存 `use sap_odata.tenant` の直後・`use` ブロック末尾）:
   ```
   use sap_odata.privacy
   ```

2. re-export ブロック（ファイル末尾、マルチテナントセクションの後）:
   ```favnir
   -- GDPR データマスキング型 re-export（v99.5.0〜）
   public type Masked<T>    = privacy.Masked<T>
   public type UnmaskClient = privacy.UnmaskClient
   public fn mask<T>(value: T) -> Masked<T> {
       privacy.mask(value)
   }
   public fn unmask_mock<T>(masked: Masked<T>) -> Result<T, String> {
       privacy.unmask_mock(masked)
   }
   ```

3. 目視確認: `UnmaskClient` interface の `fn unmask<T>(masked: Masked<T>)` の引数型 `Masked<T>` が
   `sap_odata.fav` スコープで re-export 済みの `Masked<T>` エイリアスと一致することを確認する。
4. 目視確認: re-export 関数（`mask` / `unmask_mock`）の戻り型が `privacy.Masked<T>` ではなく
   `Masked<T>`（re-export 済みエイリアス）を使用していることを確認する。

---

### Step 3: ctx.fav に unmask フィールドを追加

`runes/ctx/ctx.fav` に以下を追加する:

1. `use` 宣言ブロックに追加（既存 `use sap_odata.tenant` の直後）:
   ```
   use sap_odata.privacy
   ```

2. `AppCtx` 型の `audit: AuditClient` 行の直後に追加:
   ```favnir
       unmask: UnmaskClient,   -- GDPR アンマスク（v99.5.0 追加）
   ```

---

### Step 4: driver.rs に mod v99500_tests を追加

`mod v99400_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99500_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn privacy_fav_exists() {
        std::fs::read_to_string(
            "../runes/sap-odata/privacy.fav",
        )
        .expect("privacy.fav should exist (v99.5.0)");
    }

    #[test]
    fn privacy_fav_has_masked() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/privacy.fav",
        )
        .expect("privacy.fav should exist (v99.5.0)");
        assert!(
            content.contains("Masked"),
            "privacy.fav should define Masked (v99.5.0)"
        );
        assert!(
            content.contains("UnmaskClient"),
            "privacy.fav should define UnmaskClient (v99.5.0)"
        );
        assert!(
            content.contains("mask"),
            "privacy.fav should define mask (v99.5.0)"
        );
        assert!(
            content.contains("unmask_mock"),
            "privacy.fav should define unmask_mock (v99.5.0)"
        );
    }
}
```

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,267 tests, 0 failures

---

### Step 6: CHANGELOG.md に v99.5.0 エントリを追加

---

### Step 7: versions/current.md 更新

最新安定版を `v99.5.0` に更新（テスト数 4,267）。

---

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
