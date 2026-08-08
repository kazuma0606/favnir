# Plan — v56.9.0 — 安定化・コードフリーズ（Language Power 2.0 前調整）

## 実装順序

```
Cargo.toml → language-power2-overview.mdx（新規）→ driver.rs（テスト追加 + バージョンチェック更新）
```

依存関係:
- `language-power2-overview.mdx` は `driver.rs` の `include_str!` 参照より先に作成が必要
- `Cargo.toml` 更新は最初に行う

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "56.9.0"
```

---

## Step 2: `site/content/docs/language-power2-overview.mdx` — 新規作成

Language Power 2.0 全機能の俯瞰ページ。`docs/` 直下（`language/` サブディレクトリではない）に作成する。

**構成**:
1. ページ概要 — Language Power 2.0 の位置づけ
2. 機能一覧テーブル — バージョン / 機能 / 主な追加内容
3. 各機能のサマリーセクション（v56.1〜v56.8）
4. 関連ドキュメントリンク
5. 次のステップ（v57.0 宣言への案内）

**アサーション対象となるキーワード**:
- `"Language Power 2.0"` — ページタイトル / 概要
- `"bounded-generics"` — 境界付きジェネリクスへのリンク

---

## Step 3: `fav/src/driver.rs` — `v56900_tests` 追加

`v56800_tests` モジュールの直前に挿入する。

```rust
// -- v56900_tests (v56.9.0) -- 安定化・Language Power 2.0 コードフリーズ --
#[cfg(test)]
mod v56900_tests {
    #[test]
    fn cargo_toml_version_is_56_9_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"56.9.0\""),
            "Cargo.toml version should be 56.9.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn language_power2_overview_exists() {
        let content = include_str!(
            "../../site/content/docs/language-power2-overview.mdx"
        );
        assert!(
            content.contains("Language Power 2.0"),
            "language-power2-overview.mdx should mention Language Power 2.0"
        );
        assert!(
            content.contains("bounded-generics"),
            "language-power2-overview.mdx should reference bounded-generics (v56.1-56.2)"
        );
        assert!(
            content.contains("row-polymorphism"),
            "language-power2-overview.mdx should reference row-polymorphism (v56.3)"
        );
        assert!(
            content.contains("effect-inference"),
            "language-power2-overview.mdx should reference effect-inference (v56.4)"
        );
    }
}
```

---

## Step 4: `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を更新:

```rust
// 変更前
cargo_toml.contains("version = \"56.8.0\"")
"Cargo.toml version should be 56.8.0, got: {}"

// 変更後
cargo_toml.contains("version = \"56.9.0\"")
"Cargo.toml version should be 56.9.0, got: {}"
```

---

## Step 5: clippy 確認

```bash
cargo clippy -- -D warnings
```

v56.1〜v56.8 の実装（W037・W038 lint ルール等）が `clippy` に干渉しないことを確認する。

---

## Step 6: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3248 tests passed, 0 failed を確認する。`v56900_tests` の両テストが pass することを確認する。

---

## ポスト処理

1. `CHANGELOG.md` に v56.9.0 エントリ追加
2. `versions/current.md` を v56.9.0 / 3248 tests に更新
3. `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.9.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.9.0 実績欄も COMPLETE に更新

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `language-power2-overview.mdx` のパスが `language/` 以下と混同される | `site/content/docs/` 直下（サブディレクトリなし） |
| `include_str!` パス誤り | `../../site/content/docs/language-power2-overview.mdx` — ビルドエラーで即判明 |
| `"bounded-generics"` アサーションが overview に含まれない | 概要ページ作成時に明示的にリンクを含める |
| clippy が新規 lint ルール（W037・W038）で自プロジェクトコードに警告を出す | v56.5〜v56.7 の実装済み Rust コード自体は問題ないはず。万一 warning が出た場合は確認して対応する |
