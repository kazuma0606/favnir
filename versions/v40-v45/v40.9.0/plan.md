# v40.9.0 実装計画

## 概要

Streaming Foundations スプリント第 9 版（コードフリーズ版）。
新規機能追加なし。`site/content/docs/streaming-foundations.mdx` を 1 件新規作成し、
v40.x で追加したストリーミング機能の概観ドキュメントを整備する。
Rust コードの変更は Cargo.toml バージョン bump と driver.rs テスト更新のみ。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2838 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.8.0` であることを確認
- `v40800_tests::cargo_toml_version_is_40_8_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `site/content/docs/streaming-foundations.mdx` が存在しないことを確認
- `driver.rs` に `v40900_tests` モジュールが存在しないことを確認

### Step 2 — streaming-foundations.mdx 作成
フロントマター（title / description）+ ウィンドウ関数一覧 + イベント型説明 + 関連 cookbook リンクで構成。
`streaming-foundations` という文字列を含めること（v41.0.0 テストへの対応）。

### Step 3 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.8.0"` → `"40.9.0"` に変更。

### Step 4 — CHANGELOG.md 更新
`[v40.9.0]` エントリを `[v40.8.0]` の直後に追加。

### Step 5 — driver.rs テストモジュール更新
1. `v40800_tests::cargo_toml_version_is_40_8_0` をスタブ化
2. `v40900_tests` モジュール（2 テスト）を末尾に追加（`use super::*` 不要）

### Step 6 — cargo test 実行
`cargo test` で 2840 tests / 0 failures を確認。

### Step 7 — バージョン管理ドキュメント更新
`versions/current.md`・ロードマップ完了マーク・`tasks.md` COMPLETE 更新。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（streaming-foundations.mdx）
  └→ Step 3（Cargo.toml）
       └→ Step 5（driver.rs — cargo_toml_version_is_40_9_0）
  └→ Step 4（CHANGELOG）
       └→ Step 5（driver.rs — changelog_has_v40_9_0）
            └→ Step 6（cargo test）
                 └→ Step 7（docs 更新）
```

Step 2〜4 は相互に独立しており並列実施可能。

---

## リスクと注意点

- `include_str!("../../site/content/docs/streaming-foundations.mdx")` のパスは driver.rs から見て `fav/src/` 起点（`../` で `fav/`、`../../` で `favnir/` ルート → `../../site/` = `favnir/site/`）。ただし v40.9.0 ではこの `include_str!` テストは作成しない。
- `streaming-foundations.mdx` に `streaming-foundations` という文字列を含めること — v41.0.0 の `streaming_foundations_doc_exists` テストが `site/content/docs/streaming-foundations.mdx` を `include_str!` で読み込み、この文字列を検証する予定。
- `v40900_tests` は `include_str!` のみ使用のため `use super::*` 不要。
