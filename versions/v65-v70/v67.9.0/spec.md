# v67.9.0 — 安定化・コードフリーズ（Developer Intelligence 前調整）

Date: 2026-08-06
Status: 未着手
Sprint: Developer Intelligence（v67.1〜v68.0）

---

## 概要

v67.1〜v67.8 で実装した全機能（デバッガ・可視化・AI 提案・テストツール群）の統合確認を行い、
v68.0.0 宣言に向けてコードフリーズする。
MDX ドキュメント `developer-intelligence.mdx` を作成し、機能一覧を公開する。

## スコープ

### IN スコープ

- `site/content/docs/tools/developer-intelligence.mdx` — 新規作成
  - 内容: v67.1〜v67.8 の全機能（`fav debug` / `fav viz` / `fav suggest` / `fav simulate` / `Rune.proptest` / `fav profile --interactive` / `fav doc --math`）の概要
  - `"fav debug"` を含むこと（`debug_viz_suggest_docs_complete` テストが要求）
- `fav/src/driver.rs` — `v67900_tests` 追加（2 件）

### OUT スコープ

- 各コマンドの動作変更・バグ修正（コードフリーズのためコード変更なし）
- `Cargo.toml` / `CHANGELOG.md` の更新（v68.0.0 宣言時に一括）
- v67.1〜v67.8 のソースコード修正
- `Rune.proptest` 構文の型チェック通過確認: v67.6.0 で `proptest.rs` を実装済み・テスト 2 件 PASS 済みのため、本バージョンでは追加確認不要（コードフリーズ対象外）

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `dev_intelligence_all_stable` | `debug.rs` / `viz.rs` / `suggest.rs` / `simulate.rs` の各ソースファイルが存在し、コマンド実行に必要な関数・定数を含む |
| `debug_viz_suggest_docs_complete` | `site/content/docs/tools/developer-intelligence.mdx` が存在し `"fav debug"` を含む |

ベーステスト: 3513 → 目標: **3515**

## `developer-intelligence.mdx` の要件

- `"fav debug"` を含む（必須、テスト要件）
- v67.1〜v67.8 の機能（debug / viz / suggest / simulate / proptest / profile --interactive / doc --math）をそれぞれ 1 段落ずつ紹介
- コードサンプルは `bash` フェンスコードブロックで記述
