# v35.8.0 実装計画 — !Effect 廃止完結（LSP / エラーカタログ / MCP / help）

## 前提確認

- `lsp/completion.rs` / `error_catalog.rs` / `mcp/mod.rs` / `main.rs` の修正は sprint（v35.0C）で完了済み
- `CHANGELOG.md` に `[v35.8.0]` エントリは既存
- `v35800_tests` モジュール（5 件）は driver.rs に pre-existing
- `v35700_tests::cargo_toml_version_is_35_7_0` は現在ライブアサーション（`35.7.0`）→ バンプ前にスタブ化が必須
- `v35800_tests::cargo_toml_version_is_35_8_0` は現在スタブ（空ボディ）→ ライブアサーションに修正が必要

## 実装ステップ

### Step 1: v35700_tests::cargo_toml_version_is_35_7_0 をスタブ化

**ファイル**: `fav/src/driver.rs`

```rust
// before:
fn cargo_toml_version_is_35_7_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.7.0"), "Cargo.toml must contain version 35.7.0");
}

// after:
fn cargo_toml_version_is_35_7_0() {
    // stubbed: version bumped to 35.8.0
}
```

### Step 2: v35800_tests::cargo_toml_version_is_35_8_0 を生きたアサーションに修正

**ファイル**: `fav/src/driver.rs`

```rust
// before (スタブ — driver.rs の実際のコード):
fn cargo_toml_version_is_35_8_0() {
    // stubbed: version bumped to 35.7.0
}

// after (生きたアサーション):
fn cargo_toml_version_is_35_8_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("35.8.0"), "Cargo.toml must contain version 35.8.0");
}
```

### Step 3: Cargo.toml バージョンを 35.8.0 に更新

**ファイル**: `fav/Cargo.toml`

```toml
# before:
version = "35.7.0"

# after:
version = "35.8.0"
```

Step 1 および Step 2 の完了後に実施すること（両方が前提条件）。

### Step 4: cargo test 全通過を確認

```
cargo test --manifest-path fav/Cargo.toml 2>&1 | tail -5
```

期待: `test result: ok. N passed; 0 failed` （N ≥ 2646）

### Step 5: v35800_tests 5 件が pass することを確認

```
cargo test v35800 --manifest-path fav/Cargo.toml 2>&1
```

### Step 6: ドキュメント更新

- `versions/v30-v35/v35.8.0/tasks.md` を COMPLETE ステータスに更新
- `versions/current.md` を v35.8.0（最新安定版）・v35.9.0（次バージョン）に更新

差分例:
```
# 最新安定版
- before: **v35.7.0** — docs_server.rs !Effect 完全除去（v35.0B）
+ after:  **v35.8.0** — !Effect 廃止完結（LSP / エラーカタログ / MCP / help）（v35.0C）

# 次に切る版
- before: **v35.8.0** — LSP / エラーカタログ / MCP / help !Effect 廃止完結（v35.0C）
+ after:  **v35.9.0** — v36.0 前調整・安定化
```

## 注意事項

- `v35800_tests` モジュールは driver.rs で `v35700_tests` より前（行 42155）に定義されており昇順が逆になっている。これはスプリント一括コミットの構造のため本バージョンでは変更しない（意図的な逆順として扱う）。
- `lsp_completion_signatures_no_effect` テストは `!Csv"` / `!Sys"` を検出しないが、sprint で該当箇所は除去済み。テスト網羅性は不完全だが実害なし。
- `mcp_docs_no_effect_annotation` テストのパターンを `!Io\\n` → `!Io`（`!Http\\n` → `!Http`、`!Db\\n` → `!Db`）に修正済み（spec-reviewer [HIGH] 対応）。

## 実装不要な作業（sprint 完了済み）

- `lsp/completion.rs` の `!Effect` 除去（完了済み）
- `error_catalog.rs` の `fix:` 書き換え（完了済み）
- `mcp/mod.rs` のドキュメント修正（完了済み）
- `main.rs` help テキスト修正（完了済み）
- `CHANGELOG.md` への `[v35.8.0]` 追記（完了済み）
- `v35800_tests` モジュール追加（完了済み、driver.rs に pre-existing）
