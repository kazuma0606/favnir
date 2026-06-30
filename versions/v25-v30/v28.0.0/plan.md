# v28.0.0 Plan — Data Lakehouse マイルストーン宣言

## 実装戦略

マイルストーン宣言バージョン（v25.0.0 / v27.0.0 と同パターン）。
新機能実装なし。ドキュメント更新 + driver.rs テスト追加のみ。

## フェーズ構成

### Phase 1 — MILESTONE.md 更新
`MILESTONE.md` に "Data Lakehouse" セクション追加。
- v28.0.0 宣言、完了コンポーネント表、象徴デモコード
- v28.x 残件（rusqlite 実統合 / delta-rs 実統合）記載

### Phase 2 — README.md 更新
既存の v27.0 参照の後に v28.0 "Data Lakehouse" 参照を追記。

### Phase 3 — サイトドキュメント作成
`site/content/docs/data-lakehouse.mdx` — milestone 解説ページ。

### Phase 4 — roadmap 完了マーク
`versions/roadmap/roadmap-v27.1-v28.0.md` 末尾に完了マーク追記。

### Phase 5 — CHANGELOG 更新
`CHANGELOG.md` 先頭に `[v28.0.0]` セクション追加。

### Phase 6 — Cargo.toml バージョン更新
`27.9.0` → `28.0.0`

### Phase 7 — ベンチマーク
`benchmarks/v28.0.0.json` 新規作成（test_count: 2226）。

### Phase 8 — driver.rs テスト追加
`v280000_tests` モジュール（6 件）を `driver.rs` に追加。

### Phase 8.5 — 中間テスト確認
`cargo test --bin fav v280000` で 6/6 PASS 確認。

### Phase 9 — テスト全通過確認
`cargo test --bin fav` で 2226 tests PASS 確認。

### Phase 10 — tasks.md COMPLETE 更新

## 実装順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

## 参照パターン

- v25.0.0: `MILESTONE.md` "Practical Self-Hosting" + `site/content/docs/v1-release.mdx`
- v27.0.0: `MILESTONE.md` "Streaming Native" セクション
- 本バージョンは同パターンで "Data Lakehouse" を追加
