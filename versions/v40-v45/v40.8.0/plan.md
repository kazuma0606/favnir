# v40.8.0 実装計画

## 概要

Streaming Foundations スプリント第 8 版。
`site/content/cookbook/` に Streaming 関連 MDX を 2 件新規作成する。
Rust コードの変更は Cargo.toml バージョン bump と driver.rs テスト更新のみ。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2835 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.7.0` であることを確認
- `v40700_tests::cargo_toml_version_is_40_7_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `site/content/cookbook/window-aggregation.mdx` が存在しないことを確認
- `site/content/cookbook/kafka-streaming.mdx` が存在しないことを確認
- `driver.rs` に `v40800_tests` モジュールが存在しないことを確認

### Step 2 — window-aggregation.mdx 作成
フロントマター（title / description）+ コード例（`tumbling_window` 使用）+ 関連 Rune セクションで構成。

### Step 3 — kafka-streaming.mdx 作成
フロントマター（title / description）+ コード例（`consume_windowed` 使用）+ 関連 Rune セクションで構成。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.7.0"` → `"40.8.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.8.0]` エントリを `[v40.7.0]` の直後に追加。

### Step 6 — driver.rs テストモジュール更新
1. `v40700_tests::cargo_toml_version_is_40_7_0` をスタブ化
2. `v40800_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 不要）

### Step 7 — cargo test 実行
`cargo test` で 2838 tests / 0 failures を確認。

### Step 8 — バージョン管理ドキュメント更新
`versions/current.md`・ロードマップ完了マーク・`tasks.md` COMPLETE 更新。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（window-aggregation.mdx）
       └→ Step 6（driver.rs — cookbook_window_aggregation_exists）
  └→ Step 3（kafka-streaming.mdx）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_8_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v40_8_0）
            └→ Step 7（cargo test）
                 └→ Step 8（docs 更新）
```

Step 2〜5 は相互に独立しており並列実施可能。

---

## リスクと注意点

- `include_str!("../../site/content/cookbook/window-aggregation.mdx")` のパスは driver.rs から見て `fav/src/` 起点の相対パスであることを確認（`../` で `fav/`、`../../` で `favnir/` ルート → `../../site/` = `favnir/site/`）
- `window-aggregation.mdx` に `tumbling_window` という文字列が含まれていることを確認（テストが依存）
- `kafka-streaming.mdx` には直接テストなし — 手動確認で代替
- `v40800_tests` は `include_str!` のみ使用のため `use super::*` 不要
