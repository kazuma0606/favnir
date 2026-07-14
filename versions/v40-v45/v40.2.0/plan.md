# v40.2.0 実装計画

## 概要

Streaming Foundations スプリント第 2 版。`runes/stream/stream.fav` に `session_window` スタブを追加し、driver.rs テストで存在を検証する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2817 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.1.0` であることを確認
- `v40100_tests::cargo_toml_version_is_40_1_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録

### Step 2 — stream.fav に session_window 追記
`runes/stream/stream.fav` の末尾に `session_window` スタブ関数を追加。

### Step 3 — rune.toml バージョン bump
`runes/stream/rune.toml` の `version = "40.1.0"` → `"40.2.0"` に変更。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.1.0"` → `"40.2.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.2.0]` エントリを `[v40.1.0]` の直後に追加。

### Step 6 — driver.rs 更新
1. `v40100_tests::cargo_toml_version_is_40_1_0` をスタブ化
2. `v40200_tests` モジュール（3 テスト）を末尾に追加

### Step 7 — cargo test 実行
`cargo test` で 2820 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（stream.fav）
       └→ Step 6（driver.rs — stream_rune_has_session_window）
  └→ Step 3（rune.toml）
       └→ Step 6（driver.rs — rune.toml version は T0 手動確認）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_2_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v40_2_0）
            └→ Step 7（cargo test）
```

Step 2〜5 は並列実施可能。Step 6 は Step 2〜5 完了後。Step 7 は Step 6 完了後。

---

## リスクと注意点

- `include_str!("../../runes/stream/stream.fav")` は `fav/src/driver.rs` からの相対パス（v40.1.0 で確認済み）
- `v40100_tests` のスタブ化を忘れると version assertion が失敗する
- `rune.toml` の version bump も忘れずに行うこと（spec 完了条件 #2）
