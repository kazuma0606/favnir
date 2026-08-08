# v69.7.0 Spec — ドキュメントのレビュー・校正・内部リンク確認

## Background

v69.3.0 で Intelligent ETL ドキュメントサイト（8 ファイル）を追加した。
現状、`overview.mdx` の「次のステップ」セクションには sub-page（quickstart / ai-stages / debugging / distributed）へのリンクはあるが、
`reference/math-runes` および `reference/ai-runes` へのリンクが欠落している。
v69.7.0 では内部リンクを補完し、ドキュメント品質を検証する。

## Goals

1. `site/content/docs/intelligent-etl/overview.mdx` の「次のステップ」セクションに
   `reference/math-runes` と `reference/ai-runes` へのリンクを追加する
2. `reference/math-runes.mdx` が 7 つの Math Rune 名前空間を網羅していることをテストで確認する
   （linalg / stats / autodiff / optim / numeric / timeseries / ml）

## Out of Scope

- MDX ファイルの文章表現の大幅な改訂
- 新規ドキュメントページの作成
- `Cargo.toml` / `CHANGELOG.md` の変更（sub-version ポリシー）

## Success Criteria

- `cargo test --bin fav -- --test-threads=8` で **3551 tests passed, 0 failed**（v697 フィルタで個別確認も可）
- ベース (3549) + 2 = **3551 tests**
- `overview.mdx` に `"reference/math-runes"` と `"reference/ai-runes"` が含まれる
- `reference/math-runes.mdx` に 7 つの名前空間見出し（`## Rune.linalg`, `## Rune.stats`, `## Rune.autodiff`, `## Rune.optim`, `## Rune.numeric`, `## Rune.timeseries`, `## Rune.ml`）がすべて含まれる

## Files to Modify

- `site/content/docs/intelligent-etl/overview.mdx` — 「次のステップ」セクションにリファレンスリンクを追加
- `fav/src/driver.rs` — `v69700_tests` モジュールを追加（2 テスト、v69600_tests の直前に挿入）
- `versions/roadmap/roadmap-v69.1-v70.0.md` — v69.7.0 状態・テスト数更新

## Files NOT to Modify

- `Cargo.toml`（sub-version ポリシー）
- `CHANGELOG.md`（sub-version ポリシー）
- `reference/math-runes.mdx`（既に 7 名前空間揃っているため変更不要）
- `reference/ai-runes.mdx`（変更不要）

## Error Codes

なし

## 追加するリンク（overview.mdx）

「次のステップ」セクションの末尾に以下を追加:

```markdown
- [Math Rune リファレンス](./reference/math-runes) — linalg / stats / autodiff / optim / numeric / timeseries / ml API
- [AI Rune リファレンス](./reference/ai-runes) — embed / llm / pinecone / qdrant / pgvector / weaviate / featurestore API
```
