# v35.7.0 実装計画 — docs_server.rs !Effect 完全除去

## 前提確認

- `docs_server.rs` の IO 関数修正は sprint（v35.0B）で完了済み
- `CHANGELOG.md` に `[v35.7.0]` エントリは既存
- `v35700_tests` モジュール（5 件）は driver.rs に pre-existing
- `v35600_tests::cargo_toml_version_is_35_6_0` は現在ライブアサーション → バンプ前にスタブ化が必須

## 実装ステップ

### Step 1: v35600_tests::cargo_toml_version_is_35_6_0 をスタブ化

**ファイル**: `fav/src/driver.rs`

```rust
// before:
fn cargo_toml_version_is_35_6_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.6.0"), "Cargo.toml must contain version 35.6.0");
}

// after:
fn cargo_toml_version_is_35_6_0() {
    // stubbed: version bumped to 35.7.0
}
```

### Step 2: v35700_tests::cargo_toml_version_is_35_7_0 を生きたアサーションに修正

**ファイル**: `fav/src/driver.rs`

```rust
// before (半スタブ — driver.rs line 42236 の実際のコード):
fn cargo_toml_version_is_35_7_0() {
    // Stubbed: version bumped to 35.8.0 in v35.0C
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35."), "Cargo.toml must contain a 35.x version");
}

// after (生きたアサーション):
fn cargo_toml_version_is_35_7_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.7.0"), "Cargo.toml must contain version 35.7.0");
}
```

コメント行（"Stubbed: version bumped to 35.8.0 in v35.0C"）も削除する。

### Step 3: Cargo.toml バージョンを 35.7.0 に更新

**ファイル**: `fav/Cargo.toml`

```toml
# before:
version = "35.6.0"

# after:
version = "35.7.0"
```

Step 1 完了後（v35600 スタブ化後）に実施すること。

### Step 4: cargo test 全通過を確認

```
cargo test --manifest-path fav/Cargo.toml 2>&1 | tail -5
```

期待: `test result: ok. N passed; 0 failed` （N ≥ 2646）

### Step 5: v35700_tests 5 件が pass することを確認

```
cargo test v35700 --manifest-path fav/Cargo.toml 2>&1
```

### Step 6: ドキュメント更新

- `versions/v30-v35/v35.7.0/tasks.md` を COMPLETE ステータスに更新
- `versions/current.md` を v35.7.0（最新安定版）・v35.8.0（次バージョン）に更新

差分例:
```
# 最新安定版
- before: **v35.6.0** — ctx 構文統一 + Production Ready 宣言
+ after:  **v35.7.0** — docs_server.rs !Effect 完全除去（v35.0B）

# 次に切る版
- before: **v35.7.0** — docs_server.rs !Effect 完全除去（v35.0B）
+ after:  **v35.8.0** — LSP / エラーカタログ / MCP / help !Effect 廃止完結（v35.0C）
```

## 実装不要な作業（sprint 完了済み）

- `docs_server.rs` IO_FUNCTIONS の signature / effects 修正（完了済み）
- `CHANGELOG.md` への `[v35.7.0]` 追記（完了済み）
- `v35700_tests` モジュール追加（完了済み、driver.rs に pre-existing）
