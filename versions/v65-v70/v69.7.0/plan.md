# v69.7.0 Plan — ドキュメントのレビュー・校正・内部リンク確認

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

## 実装ステップ

### Step 1: 事前確認

1. `cargo test --bin fav -- --test-threads=8` でベース 3549 tests passed, 0 failed を確認
2. `site/content/docs/intelligent-etl/overview.mdx` の「次のステップ」セクションに `reference/math-runes` が含まれないことを確認（重複防止）
3. `driver.rs` に `v69700_tests` が存在しないことを確認

### Step 2: `overview.mdx` にリファレンスリンクを追加

`site/content/docs/intelligent-etl/overview.mdx` の「次のステップ」セクション末尾（現在は 5 行）に以下の 2 行を追記する:

```markdown
- [Math Rune リファレンス](./reference/math-runes) — linalg / stats / autodiff / optim / numeric / timeseries / ml API
- [AI Rune リファレンス](./reference/ai-runes) — embed / llm / pinecone / qdrant / pgvector / weaviate / featurestore API
```

現在の「次のステップ」セクション（57〜61行）:
```markdown
- [クイックスタート](./quickstart) — 15 分で動かす
- [Math Foundation](./math-foundation) — linalg / stats / autodiff の使い方
- [AI Stages](./ai-stages) — embed / llm / vectordb ステージの構築
- [デバッグガイド](./debugging) — fav debug / fav viz の使い方
- [分散実行](./distributed) — クラスタ・K8s でのスケールアウト
```

### Step 3: `driver.rs` にテスト追加

テストモジュールは**降順（最新が先頭）**で並んでいる（v69600 → v69500 → v69200 → ...）。
v69700 を v69600_tests の直前に挿入する。

```rust
// -- v69700_tests (v69.7.0) -- ドキュメント校正・内部リンク確認 --
#[cfg(test)]
mod v69700_tests {
    #[test]
    fn intelligent_etl_overview_links_to_reference_pages() {
        let src = include_str!("../../site/content/docs/intelligent-etl/overview.mdx");
        assert!(src.contains("reference/math-runes"), "overview.mdx should link to reference/math-runes");
        assert!(src.contains("reference/ai-runes"), "overview.mdx should link to reference/ai-runes");
    }

    #[test]
    fn intelligent_etl_math_runes_has_seven_namespaces() {
        let src = include_str!("../../site/content/docs/intelligent-etl/reference/math-runes.mdx");
        assert!(src.contains("## Rune.linalg"), "math-runes.mdx should contain Rune.linalg section");
        assert!(src.contains("## Rune.stats"), "math-runes.mdx should contain Rune.stats section");
        assert!(src.contains("## Rune.autodiff"), "math-runes.mdx should contain Rune.autodiff section");
        assert!(src.contains("## Rune.optim"), "math-runes.mdx should contain Rune.optim section");
        assert!(src.contains("## Rune.numeric"), "math-runes.mdx should contain Rune.numeric section");
        assert!(src.contains("## Rune.timeseries"), "math-runes.mdx should contain Rune.timeseries section");
        assert!(src.contains("## Rune.ml"), "math-runes.mdx should contain Rune.ml section");
    }
}
```

### Step 4: ビルド・テスト確認

1. `cargo build 2>&1 | grep "^error"` — エラーゼロ確認
2. `cargo test --bin fav -- --test-threads=8` — 3551 tests passed, 0 failed 確認

### Step 5: ドキュメント・ステータス更新

1. `roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.7.0 行を確定（3551、+2）
2. `roadmap-v69.1-v70.0.md` の v69.7.0 状態列を「完了」に変更
3. `versions/current.md` の「進行中バージョン」を `v69.7.0` に更新
4. 本 `tasks.md` を COMPLETE に更新

## 依存関係

- Step 2（overview.mdx 更新）は Step 3（テスト追加）の前に完了する必要がある（include_str! でファイルを読み込むため）
