# Plan — v55.9.0 — 安定化・コードフリーズ（Streaming Native 2.0 前調整）

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.9.0` に更新。

```toml
[package]
version = "55.9.0"
```

---

### Step 2: `streaming-native2-overview.mdx` 作成

`site/content/docs/streaming-native2-overview.mdx` を新規作成する。

#### 必須セクション

1. **frontmatter** — `title` / `description`
2. **宣言文**（引用ブロック）— ロードマップ記載の宣言文を引用
3. **機能一覧テーブル** — v55.1〜v55.8 の全機能を表形式で列挙
4. **クイックスタート**:
   - fav.toml `[stream]` 設定例（buffer_size / delivery / checkpoint_interval_sec / checkpoint_store / session_gap_sec / watermark_max_late_sec）
   - Stateful stage コード例
   - CEP stage コード例（`fn is_start` / `fn is_end` を事前定義してから使用）
   - チェックポイント CLI 操作例（`fav checkpoint list` / `--resume-from`）
5. **詳細ドキュメントリンク** — streaming-v2 / stateful-pipeline / cep-patterns
6. **次のステップ: v56.0** — replay API・状態復元の予告

#### キーワード必須チェック

| キーワード | 含む箇所 |
|---|---|
| `"Streaming Native 2.0"` | タイトル・本文 |
| `"Exactly-once"` | 機能一覧テーブル・設定例 |
| `"CEP"` | 機能一覧テーブル・コード例 |
| `"Stateful"` | 機能一覧テーブル・コード例 |

---

### Step 3: `driver.rs` — `v55900_tests` モジュール追加

`v55800_tests` の直前（`// -- v55800_tests` コメント行の前）に挿入する。

```rust
// -- v55900_tests (v55.9.0) -- 安定化・コードフリーズ（Streaming Native 2.0 前調整）--
#[cfg(test)]
mod v55900_tests {
    #[test]
    fn cargo_toml_version_is_55_9_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"55.9.0\""),
            "Cargo.toml version should be 55.9.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn streaming_native2_overview_exists() {
        let src = include_str!("../../site/content/docs/streaming-native2-overview.mdx");
        assert!(src.contains("Streaming Native 2.0"), "streaming-native2-overview.mdx must mention Streaming Native 2.0");
        assert!(src.contains("Exactly-once"), "streaming-native2-overview.mdx must mention Exactly-once");
        assert!(src.contains("CEP"), "streaming-native2-overview.mdx must mention CEP");
        assert!(src.contains("Stateful"), "streaming-native2-overview.mdx must mention Stateful");
    }
}
```

---

### Step 4: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep -E "^test result|v55900|FAILED"
```

期待結果: `3224 tests passed, 0 failed`、`cargo_toml_version_is_55_9_0 ... ok`、`streaming_native2_overview_exists ... ok`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

期待結果: クリーン

---

### Step 5: ポスト処理

- `CHANGELOG.md` に v55.9.0 エントリ追加
- `versions/current.md` を v55.9.0 / 3224 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.9.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.9.0 実績欄も COMPLETE に更新

---

## 注意事項

- **コードフリーズ**: 新機能追加は行わない。既存コードの変更も最小限（Cargo.toml バージョンと driver.rs テスト追加のみ）。
- CEP コード例で `yield is_start;` / `yield is_end;` を使う場合は、
  直前に `fn is_start(e: Event) -> Bool { e.type == "start" }` を定義すること（コードレビュー対応）。
- `streaming-native2-overview.mdx` は `site/content/docs/` 直下に配置する（`runtime/` サブディレクトリではない）。
- `cargo_toml_version_is_55_9_0` アサートメッセージの `find(|l| l.contains("version"))` は
  Cargo.toml 先頭付近の `version = "55.9.0"` 行（line 3）を優先的に取得するため問題なし。
