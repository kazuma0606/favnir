# v40.3.0 実装計画

## 概要

Streaming Foundations スプリント第 3 版。`runes/stream/stream.fav` に `Event<T>` 型定義を追加し、driver.rs テストで `Event` / `timestamp` キーワードの存在を検証する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2820 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.2.0` であることを確認
- `v40200_tests::cargo_toml_version_is_40_2_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録

### Step 2 — stream.fav に Event<T> 型定義追加
`runes/stream/stream.fav` に `Event<T>` 型定義を追加。ヘッダーコメントも v40.3.0 に更新。

### Step 3 — rune.toml バージョン bump
`runes/stream/rune.toml` の `version = "40.2.0"` → `"40.3.0"` に変更。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.2.0"` → `"40.3.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.3.0]` エントリを `[v40.2.0]` の直後に追加。

### Step 6 — driver.rs 更新
1. `v40200_tests::cargo_toml_version_is_40_2_0` をスタブ化
2. `v40300_tests` モジュール（3 テスト）を末尾に追加

### Step 7 — cargo test 実行
`cargo test` で 2823 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（stream.fav — Event<T> 型定義）
       └→ Step 6（driver.rs — stream_fav_has_event_type）
  └→ Step 3（rune.toml — rune.toml version は T0 手動確認）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_3_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v40_3_0）
            └→ Step 7（cargo test）
```

Step 2〜5 は並列実施可能。Step 6 は Step 2〜5 完了後。Step 7 は Step 6 完了後。

---

## リスクと注意点

- `type Event<T> = { ... }` 構文が Favnir パーサーで通るかを確認する。パーサーエラーになる場合はコメント形式（`// type Event<T> = ...`）でスタブとして記述し、TODO を明記する
- `v40200_tests` のスタブ化を忘れると version assertion が失敗する
