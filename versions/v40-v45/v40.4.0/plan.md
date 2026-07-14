# v40.4.0 実装計画

## 概要

Streaming Foundations スプリント第 4 版。`runes/stream/stream.fav` に `with_late_policy` スタブを追加し、driver.rs テストで存在を検証する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2823 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.3.0` であることを確認
- `v40300_tests::cargo_toml_version_is_40_3_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録

### Step 2 — stream.fav に with_late_policy 追記
`runes/stream/stream.fav` の末尾に `with_late_policy` スタブ関数を追加。ヘッダーコメントも v40.4.0 に更新。

### Step 3 — rune.toml バージョン bump + description 更新
`runes/stream/rune.toml` の `version = "40.3.0"` → `"40.4.0"` に変更。description に `with_late_policy` を追記。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.3.0"` → `"40.4.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.4.0]` エントリを `[v40.3.0]` の直後に追加。

### Step 6 — driver.rs 更新
1. `v40300_tests::cargo_toml_version_is_40_3_0` をスタブ化
2. `v40400_tests` モジュール（3 テスト）を末尾に追加

### Step 7 — cargo test 実行
`cargo test` で 2826 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（stream.fav — with_late_policy）
       └→ Step 6（driver.rs — stream_fav_has_late_policy）
  └→ Step 3（rune.toml — version bump は T2 で実施、手動確認は tasks.md T2 参照）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_4_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v40_4_0）
            └→ Step 7（cargo test）
```

Step 2〜5 は並列実施可能。Step 6 は Step 2〜5 完了後。Step 7 は Step 6 完了後。

---

## リスクと注意点

- `with_late_policy` は引数 `tolerance`・`policy` を受け取るが、スタブでは両引数を使わず全イベント通過とする
- `v40300_tests` のスタブ化を忘れると version assertion が失敗する
