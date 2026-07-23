# Tasks: v47.9.0 — stdlib ドキュメント + v48.0 前調整

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3039 passed, 0 failed を確認
- [x] `site/content/docs/stdlib/` に `list.mdx` / `string.mdx` / `map.mdx` が存在することを確認
- [x] `site/content/docs/stdlib/float.mdx` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/stdlib/v2.mdx` が存在しないことを確認（新規作成対象）

## T1 — MDX ドキュメント作成・更新

- [x] `site/content/docs/stdlib/float.mdx` 新規作成
  - [x] `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` を記載
- [x] `site/content/docs/stdlib/v2.mdx` 新規作成
  - [x] `"Standard Library 2.0"` の文言を含める
  - [x] v47.1〜v47.8 の全追加関数を索引として列挙
- [x] `site/content/docs/stdlib/list.mdx` 更新
  - [x] `zip` / `chunk` / `flat_map` / `group_by` / `dedupe` / `scan` / `take_while` / `drop_while` を追記（zip/flat_map は既存）
- [x] `site/content/docs/stdlib/string.mdx` 更新
  - [x] `pad_left` / `trim_start` / `repeat` を確認（既存に記載済みのため追記不要）
- [x] `site/content/docs/stdlib/option.mdx` 更新
  - [x] `map` / `unwrap_or` / `and_then` / `is_some` / `is_none` を確認（既存に記載済みのため追記不要）
- [x] `site/content/docs/stdlib/result.mdx` 更新
  - [x] `map` / `map_err` / `and_then` / `is_ok` / `is_err` を確認（既存に記載済みのため追記不要）
- [x] `site/content/docs/stdlib/map.mdx` 更新
  - [x] `merge` / `filter_values` / `map_values` を追記（merge は既存、filter_values/map_values を追加）
- [x] `site/content/cookbook/stdlib-v2.mdx` 新規作成（v47 新関数サンプルパイプライン、frontmatter 必須フィールド含む）

## T2 — `driver.rs` にテスト追加・バージョン更新・完了

- [x] `v478000_tests` の直前に `v479000_tests` モジュールを追加（2 テスト）
  - [x] `stdlib_v2_doc_exists`: `float.mdx` に `"Float.round"` が含まれるか
  - [x] `stdlib_v2_overview_exists`: `v2.mdx` に `"Standard Library 2.0"` が含まれるか
- [x] `fav/Cargo.toml` version → `"47.9.0"`
- [x] `CHANGELOG.md` に v47.9.0 エントリ追加
- [x] `cargo test` 3041 passed, 0 failed（3039 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.9.0（3041 tests）に更新、進行中バージョンを `v48.0.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.9.0 完了条件テスト数（3041）を実績で確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全 `[x]`）

> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v48.0.0 マイルストーン宣言時に実施

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | cookbook サンプル更新がスコープに存在しない | `site/content/cookbook/stdlib-v2.mdx` を spec/plan/tasks に追加 |
| [HIGH] | plan.md Step 1 の MDX フェンス内に ```favnir が入れ子でネスト | インデントリスト形式に変更してフェンス入れ子を回避 |
| [MED] | Option/Result の mdx 更新がスコープから漏れている | `option.mdx` / `result.mdx` 更新を spec/plan/tasks に追加（既存に記載済みを確認） |
| [MED] | frontmatter 必須フィールド検証がテスト対象外 | spec.md に「目視確認」方針を明記 |
| [LOW] | list/string/map 更新内容のテスト検証なし | spec.md に「目視確認」方針を明記 |
