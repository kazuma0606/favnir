# Plan: v99.1.0 — OAuth2 PKCE / SAP BTP Trust Configuration

## 実装順序

### Step 1: btp_auth.fav を新規作成

`runes/sap-odata/btp_auth.fav` を作成。

```favnir
-- runes/sap-odata/btp_auth.fav
-- SAP BTP OAuth2 PKCE 認証型定義（v99.1.0）

-- BTP サービスキー認証情報
public type BtpCredential = {
    client_id:     String,
    client_secret: String,
    token_url:     String,
    scope:         List<String>
}

-- BTP アクセストークン
public type BtpToken = {
    access_token: String,
    expires_in:   Int,
    token_type:   String
}

-- テスト用モック: BtpCredential からダミー BtpToken を返す
public fn acquire_token_mock(cred: BtpCredential) -> BtpToken {
    BtpToken {
        access_token: String.concat(["mock_token_for_", cred.client_id]),
        expires_in:   3600,
        token_type:   "Bearer"
    }
}
```

コメントは `--` スタイル。`//` 不使用。

---

### Step 1.5: sap_odata.fav に use と re-export を追加

`runes/sap-odata/sap_odata.fav` に以下を追加する:

1. `use` 宣言（他の `use` 行と同じ場所）:
   ```
   use sap_odata.btp_auth
   ```

2. re-export ブロック（ファイル末尾、BTP 認証 セクションとして）:
   ```favnir
   -- BTP 認証型 re-export（v99.1.0〜）
   public type BtpCredential = btp_auth.BtpCredential
   public type BtpToken      = btp_auth.BtpToken
   public fn acquire_token_mock(cred: btp_auth.BtpCredential) -> btp_auth.BtpToken {
       btp_auth.acquire_token_mock(cred)
   }
   ```

---

### Step 2: driver.rs に mod v99100_tests を追加

`mod v99000_tests` の直後に追加：

```rust
#[cfg(test)]
mod v99100_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn btp_auth_fav_exists() {
        std::fs::read_to_string(
            "../runes/sap-odata/btp_auth.fav",
        )
        .expect("btp_auth.fav should exist (v99.1.0)");
    }

    #[test]
    fn btp_auth_has_btp_credential() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/btp_auth.fav",
        )
        .expect("btp_auth.fav should exist");
        assert!(
            content.contains("BtpCredential"),
            "btp_auth.fav should define BtpCredential (v99.1.0)"
        );
    }
}
```

---

### Step 3: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,259 tests, 0 failures

---

### Step 4: CHANGELOG.md に v99.1.0 エントリを追加

---

### Step 5: versions/current.md 更新

最新安定版を `v99.1.0` に更新（テスト数 4,259）。

---

### Step 6: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
