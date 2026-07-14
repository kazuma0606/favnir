# v40.1.0 実装計画

## 概要

Streaming Foundations スプリント第 1 版。`runes/stream/` に tumbling_window / sliding_window スタブを追加し、driver.rs テストで存在を検証する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2814 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.0.0` であることを確認

### Step 2 — rune.toml 作成
`runes/stream/rune.toml` を作成する。

```toml
[rune]
name = "stream"
version = "0.1.0"
description = "Stream windowing utilities for Favnir pipelines"
```

### Step 3 — stream.fav 作成
`runes/stream/stream.fav` を作成する。
`tumbling_window` / `sliding_window` の関数スタブを実装。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.0.0"` を `"40.1.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.1.0]` エントリを `[v40.0.0]` の直後に追加。

### Step 6 — driver.rs 更新
1. `v40000_tests::cargo_toml_version_is_40_0_0` をスタブ化
2. `v40100_tests` モジュール（3 テスト）を末尾近くに追加

### Step 7 — cargo test 実行
`cargo test` で 2817 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（rune.toml）
  └→ Step 3（stream.fav）
       └→ Step 6（driver.rs — include_str! 参照）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — version チェック）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog チェック）
            └→ Step 7（cargo test）
```

Step 2〜5 は並列実施可能。Step 6 は Step 2〜5 完了後。Step 7 は Step 6 完了後。

---

## リスクと注意点

- `include_str!("../../runes/stream/stream.fav")` は `fav/src/driver.rs` からの相対パス
- stream.fav のジェネリクス `A` は現時点でコンパイルエラーになる可能性あり → 型注釈を簡略化してスタブとして扱う
- v40000_tests のスタブ化を忘れると version assertion が失敗する
