# v40.6.0 実装計画

## 概要

Streaming Foundations スプリント第 6 版。
`runes/kafka/kafka.fav` に `consume_windowed` スタブを追加し、
欠落している `rune.toml` を新規作成する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2829 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.5.0` であることを確認
- `v40500_tests::cargo_toml_version_is_40_5_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `runes/kafka/kafka.fav` に `consume_windowed` が存在しないことを確認
- `runes/kafka/rune.toml` が存在しないことを確認

### Step 2 — kafka.fav 更新
1. ヘッダーコメントを `// runes/kafka/kafka.fav — Kafka Rune (v40.6.0)` に更新
2. `create_topic` 関数の直後に `consume_windowed` スタブを追加

```
public fn consume_windowed(conn: KafkaConn, topic: String, group_id: String, window_secs: Int) -> Result<String, String> {
    Kafka.consume_batch_raw(conn, topic, window_secs)
}
```

### Step 3 — kafka rune.toml 新規作成
`runes/kafka/rune.toml` を新規作成する（redis の rune.toml と同構造）。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.5.0"` → `"40.6.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.6.0]` エントリを `[v40.5.0]` の直後に追加。

### Step 6 — driver.rs 更新
1. `v40500_tests::cargo_toml_version_is_40_5_0` をスタブ化
2. `v40600_tests` モジュール（2 テスト）を末尾に追加

### Step 7 — cargo test 実行
`cargo test` で 2831 tests / 0 failures を確認。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（kafka.fav — consume_windowed）
       └→ Step 6（driver.rs — kafka_fav_has_consume_windowed）
  └→ Step 3（rune.toml 新規作成）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_6_0）
  └→ Step 5（CHANGELOG）
            └→ Step 7（cargo test）
```

Step 2〜5 は相互に独立しており並列実施可能。

---

## リスクと注意点

- `kafka.fav` のヘッダー `(v25.7.0)` → `(v40.6.0)` の更新を忘れないこと（過去バージョンで recurring な code-reviewer 指摘）
- `consume_windowed` の第 3 引数 `group_id` は `consume_batch_raw` に渡さない（`consume_batch_raw` は `conn, topic, max_count` のシグネチャ）
- `rune.toml` の description には全 public 関数・型をリストアップすること（v40.2.0 以降の recurring 指摘）
- `v40600_tests` に `use super::*` は不要（`include_str!` のみ使用）
