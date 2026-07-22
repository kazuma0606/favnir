# Plan — v54.9.0 — v55.0 前調整・安定化

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `54.9.0` に更新。

```toml
[package]
version = "54.9.0"
```

### Step 2: production3-overview.mdx 補完

`site/content/docs/production3-overview.mdx` の `## v54` セクションに
v54.6〜v54.9 最終整備内容を追記する。

追記内容:
```markdown
**v54.6〜v54.9 最終整備:**
- README / CONTRIBUTING 最終更新（Production 3.0 言及・`fav doctor` / `fav bench` 手順追記）（v54.6）
- 本ドキュメント（`production3-overview.mdx`）新規作成（v54.7）
- `MILESTONE.md` に `v55.0.0（予定）— Production 3.0` エントリ追加（v54.8）
- v55.0 前調整・安定化・コードフリーズ（v54.9）
```

### Step 3: driver.rs に v54900_tests 追加

`fav/src/driver.rs` の `v54800_tests` モジュールの直前に `v54900_tests` モジュールを挿入。

```rust
// -- v54900_tests (v54.9.0) -- v55.0 前調整・安定化 --
#[cfg(test)]
mod v54900_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_54_9_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"54.9.0\""),
            "Cargo.toml version should be 54.9.0"
        );
    }

    #[test]
    fn production3_overview_doc_complete() {
        let doc = include_str!("../../site/content/docs/production3-overview.mdx");
        assert!(doc.contains("## v51"), "production3-overview.mdx should have v51 section");
        assert!(doc.contains("## v52"), "production3-overview.mdx should have v52 section");
        assert!(doc.contains("## v53"), "production3-overview.mdx should have v53 section");
        assert!(doc.contains("## v54"), "production3-overview.mdx should have v54 section");
        assert!(doc.contains("## v55"), "production3-overview.mdx should have v55 section");
        assert!(
            doc.contains("v54.6"),
            "production3-overview.mdx should mention v54.6 final polish"
        );
    }
}
```

### Step 4: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3203 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン（warnings/errors なし）

### Step 5: ポスト処理

- `CHANGELOG.md` に v54.9.0 エントリ追加
- `versions/current.md` を v54.9.0 / 3203 tests に更新
- `versions/roadmap/roadmap-v54.1-v55.0.md` の v54.9.0 実績を COMPLETE に更新

---

## 注意事項

- `cargo_toml_version_is_X` テストは Cargo.toml バージョン更新と同時に古いバージョンを期待するテストが失敗する。
  v54.8.0 の `cargo_toml_version_is_54_8_0` は `v54900_tests` 追加後は空になる（Cargo.toml が 54.9.0 になるため）。
  これは設計上の意図的な動作。
- `production3_overview_doc_complete` の `include_str!("../../site/content/docs/production3-overview.mdx")`
  は `fav/src/driver.rs` からの相対パスで `favnir/site/content/docs/production3-overview.mdx` を指す。
