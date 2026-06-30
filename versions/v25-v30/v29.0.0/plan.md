# v29.0.0 Plan — Observability First マイルストーン宣言

## 実装順序

```
T1 Cargo.toml bump
  ↓
T2 MILESTONE.md 更新
T3 README.md 更新
T4 site/content/docs/observability-first.mdx 新規作成
T5 roadmap 完了マーク
T6 CHANGELOG.md 更新
T7 benchmarks/v29.0.0.json 新規作成
  ↓（並行可）
T8 driver.rs テスト追加
  ↓
T8.5 cargo test --bin fav v290000 — 6/6 PASS
  ↓
T9 cargo test --bin fav 全体 — 2312 PASS
  ↓
T10 tasks.md COMPLETE 更新
```

## ファイル変更一覧

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `"28.9.0"` → `"29.0.0"` |
| `MILESTONE.md` | 更新 | "Observability First" セクション追加 |
| `README.md` | 更新 | v29.0 参照追記 |
| `site/content/docs/observability-first.mdx` | 新規 | Observability First ドキュメント |
| `versions/roadmap/roadmap-v28.1-v29.0.md` | 更新 | v29.0 完了マーク追記 |
| `CHANGELOG.md` | 更新 | `[v29.0.0]` セクション追加 |
| `benchmarks/v29.0.0.json` | 新規 | `{"version": "29.0.0", "test_count": 2312}` |
| `fav/src/driver.rs` | 更新 | `v290000_tests` 6 件追加 |

## 注意事項

- 新機能追加・破壊的変更なし（マイルストーン宣言版）
- MILESTONE.md の "Observability First" セクションは v28.0.0（Data Lakehouse）セクションの**上**に追加
  （v28.0.0 が現在の先頭なので、v29.0.0 を新先頭に追加）
- `site/content/docs/observability-first.mdx` は `data-lakehouse.mdx` / `streaming-native.mdx` と同じ構成に揃える
- `driver.rs` の `v290000_tests` は `// ── v289000_tests` コメント行の**直前**に追加
