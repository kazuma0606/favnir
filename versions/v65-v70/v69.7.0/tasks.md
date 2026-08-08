# v69.7.0 タスクリスト

Status: COMPLETE
Version: 69.7.0
Note: ドキュメントのレビュー・校正・内部リンク確認 — overview.mdx にリファレンスリンク追加 + driver.rs に 2 テスト
Base tests: 3549
Target tests: 3551（+2）

---

## T0: 事前確認

- [x] `cargo test --bin fav -- --test-threads=8` でベース 3549 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `versions/current.md` の「進行中バージョン」が `v69.6.0` であることを確認
- [x] `site/content/docs/intelligent-etl/overview.mdx` に `"reference/math-runes"` が含まれないことを確認（重複防止）
- [x] `driver.rs` に `v69600_tests` が存在することを確認（挿入先の確認）
- [x] `driver.rs` に `v69700_tests` が存在しないことを確認（重複防止）
- [x] `reference/math-runes.mdx` に `## Rune.linalg`〜`## Rune.ml` の 7 見出しが揃っていることを確認

---

## T1: `overview.mdx` — リファレンスリンク追加

- [x] `site/content/docs/intelligent-etl/overview.mdx` の「次のステップ」セクション末尾に 2 行追加
  - [x] `- [Math Rune リファレンス](./reference/math-runes) — linalg / stats / autodiff / optim / numeric / timeseries / ml API` を追加
  - [x] `- [AI Rune リファレンス](./reference/ai-runes) — embed / llm / pinecone / qdrant / pgvector / weaviate / featurestore API` を追加

---

## T2: `driver.rs` — テスト追加

テストモジュールは降順（最新が先頭）。v69700 を v69600_tests の直前に挿入する。

- [x] `v69600_tests` の直前に `v69700_tests` モジュールを追加（挿入後: v69700 → v69600 → v69500 → ...）
  - [x] `intelligent_etl_overview_links_to_reference_pages` テストを追加
    - [x] `include_str!("../../site/content/docs/intelligent-etl/overview.mdx")`
    - [x] `src.contains("reference/math-runes")` アサート
    - [x] `src.contains("reference/ai-runes")` アサート
  - [x] `intelligent_etl_math_runes_has_seven_namespaces` テストを追加
    - [x] `include_str!("../../site/content/docs/intelligent-etl/reference/math-runes.mdx")`
    - [x] `## Rune.linalg` / `## Rune.stats` / `## Rune.autodiff` / `## Rune.optim` / `## Rune.numeric` / `## Rune.timeseries` / `## Rune.ml` の 7 アサート

---

## T3: ビルド・テスト確認

- [x] `cargo build 2>&1 | grep "^error"` — エラーゼロを確認
- [x] `cargo test --bin fav -- --test-threads=8` で **3551 tests passed, 0 failed** を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.7.0 行を確定（3551、+2）
- [x] `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.7.0「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を `v69.6.0` から `v69.7.0` に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

---

## 設計上の意図的省略

- 文章表現の大幅な改訂: 将来フェーズ（本 sub-version は内部リンク補完のみ）
- 新規ドキュメントページの作成: 将来フェーズ
