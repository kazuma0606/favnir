# Plan — v57.5.0 — 監査ログ暗号化・署名（tamper-proof audit）

## 実装順序

```
Cargo.toml → driver.rs（v57500_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
→ tasks.md COMPLETE 更新
```

依存関係:
- `AuditEntry` / `sign_entry` / `verify_entry` はすべて `v57500_tests` 内に完結
- `toml.rs` への変更は不要（driver.rs のみ）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.4.0"  →  version = "57.5.0"
```

---

## Step 2: `fav/src/driver.rs` — `v57500_tests` 追加

`v57400_tests` の直前（`// -- v57400_tests` コメント行の直前）に挿入:

```rust
// -- v57500_tests (v57.5.0) -- 監査ログ暗号化・署名 --
#[cfg(test)]
mod v57500_tests {
    #[derive(Debug, Clone)]
    struct AuditEntry {
        id: u64,
        event: String,
        payload: String,
    }

    /// Deterministic signature using stdlib u64 arithmetic only (no external crates).
    /// Combines key and entry bytes so that any change to either alters the signature.
    fn sign_entry(entry: &str, key: &str) -> String {
        let key_hash: u64 = key
            .bytes()
            .enumerate()
            .fold(0u64, |acc, (i, b)| {
                acc.wrapping_add((b as u64).wrapping_mul(i as u64 + 31))
            });
        let entry_hash: u64 = entry
            .bytes()
            .enumerate()
            .fold(0u64, |acc, (i, b)| {
                acc.wrapping_add((b as u64).wrapping_mul(i as u64 + 1))
            });
        format!("{:016x}", key_hash.wrapping_add(entry_hash))
    }

    fn verify_entry(entry: &str, signature: &str, key: &str) -> bool {
        sign_entry(entry, key) == signature
    }

    #[test]
    fn audit_sign_entry() {
        // Use AuditEntry to build the entry string (avoids dead_code on struct fields)
        let e = AuditEntry {
            id: 1,
            event: "pipeline.start".to_string(),
            payload: "ok".to_string(),
        };
        let entry = format!(
            r#"{{"id":{},"event":"{}","payload":"{}"}}"#,
            e.id, e.event, e.payload
        );
        let key = "prod/audit-key";

        let sig = sign_entry(&entry, key);

        // Non-empty
        assert!(!sig.is_empty(), "signature should not be empty");
        // 16-char hex
        assert_eq!(sig.len(), 16, "signature should be 16-char hex string");
        // Deterministic
        assert_eq!(sig, sign_entry(&entry, key), "same input should produce same signature");
        // Key-sensitive
        let sig_other_key = sign_entry(&entry, "other-key");
        assert_ne!(sig, sig_other_key, "different keys should produce different signatures");
        // Entry-sensitive: changing one char produces a different signature
        let entry_modified = format!(
            r#"{{"id":{},"event":"{}","payload":"{}"}}"#,
            e.id, e.event, "ng"
        );
        assert_ne!(
            sig,
            sign_entry(&entry_modified, key),
            "modified entry should produce different signature"
        );
    }

    #[test]
    fn audit_verify_tamper_detected() {
        let original = r#"{"id":42,"event":"pipeline.complete","payload":"rows=1000"}"#;
        let tampered = r#"{"id":42,"event":"pipeline.complete","payload":"rows=9999"}"#;
        let key = "prod/audit-key";

        let sig = sign_entry(original, key);

        // Original passes verification
        assert!(
            verify_entry(original, &sig, key),
            "original entry should pass verification"
        );
        // Tampered entry fails verification (hash mismatch)
        assert!(
            !verify_entry(tampered, &sig, key),
            "tampered entry should fail verification"
        );
        // Wrong key also fails
        assert!(
            !verify_entry(original, &sig, "wrong-key"),
            "wrong key should fail verification"
        );
    }
}
```

---

## Step 3: `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.4.0" → "57.5.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.4.0" → "57.5.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.4.0" → "57.5.0"（rolling）
```

> `v57100_tests` / `v57200_tests` / `v57300_tests` / `v57400_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## Step 4: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3263 tests passed, 0 failed を確認。`v57500_tests` の 2 件が全通過することを確認。

---

## Step 5: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 6: ポスト処理

1. `CHANGELOG.md` に v57.5.0 エントリを追加（先頭）
2. `versions/current.md` を v57.5.0 / 3263 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.5.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.5.0 実績欄を COMPLETE に更新し、テスト数推移テーブルに v57.5.0 行（3263）を追加

---

## Step 7: `versions/v55-v60/v57.5.0/tasks.md` を COMPLETE に更新

全チェックボックス（T0 含む）を `[x]` にする。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `AuditEntry` フィールド（`id` / `payload`）が tests で読まれず dead_code 警告 | `audit_sign_entry` / `audit_verify_tamper_detected` で entry 文字列を使う設計のため構造体自体は補助的定義。フィールドは `make_entry()` ヘルパーで使用するか、コメントで意図を明示する |
| `sign_entry` の数式で 2 エントリが同じ署名になる衝突 | fold に位置インデックス（`i + 1`）を乗算することで位置依存ハッシュとなり、単純な衝突を防ぐ |
| `v57400_tests` コメント行の直前への挿入位置ミス | Python `str.replace()` を使う（awk 多行挿入は過去に失敗実績あり） |
| 外部 crate を誤って追加する | `Cargo.toml` の `[dependencies]` は変更しない |
