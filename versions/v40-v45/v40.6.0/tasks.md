# v40.6.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2832（前バージョン 2829 + 3）
**実績テスト数**: 2832 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2829 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.5.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.6.0 を確認
- [x] `v40500_tests::cargo_toml_version_is_40_5_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44393
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40500_tests` の閉じ `}` の行番号を確認し記録: 行44413
- [x] `driver.rs` に `v40600_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `runes/kafka/kafka.fav` に `consume_windowed` が存在しないことを確認
- [x] `runes/kafka/rune.toml` が存在しないことを確認

---

## T1 — kafka.fav 更新

- [x] ヘッダーコメントを `// runes/kafka/kafka.fav — Kafka Rune (v40.6.0)` に更新（`v25.7.0` → `v40.6.0`）
- [x] `create_topic` 関数の直後に `consume_windowed` スタブを追加（コメント + v40.6.0 スタブ注釈付き）

---

## T1b — redis.fav 更新（spec-reviewer [MED] 対応）

- [x] ヘッダーコメントを `// runes/redis/redis.fav — Redis Rune (v40.6.0)` に更新（`v25.3.0` → `v40.6.0`）
- [x] `subscribe_once` 関数の直後に `consume_windowed` スタブを追加（`// ── Streams` セクション付き）

---

## T2 — kafka rune.toml 新規作成

- [x] `runes/kafka/rune.toml` を新規作成
  - `name = "kafka"`、`version = "40.6.0"`、`entry = "kafka.fav"`
  - `description` に全 public 関数・型をリストアップ

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.5.0"` → `"40.6.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.6.0]` エントリを `[v40.5.0]` の直後に追加

---

## T5 — driver.rs 更新

- [x] `v40500_tests::cargo_toml_version_is_40_5_0` をスタブ化
- [x] `v40600_tests` モジュール（3 テスト）を末尾に追加
  - `cargo_toml_version_is_40_6_0`（NOTE コメント付き）
  - `changelog_has_v40_6_0`（spec-reviewer [LOW] 対応で追加）
  - `kafka_fav_has_consume_windowed`

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2832 を確認（実績: 2832）
- [x] `v40600_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.6.0（最新安定版）・v40.7.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.6.0 を完了済みにマーク
- [x] `versions/v40-v45/v40.6.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**spec-reviewer 指摘（実装前対応）:**
- [MED] Redis 持ち越し根拠不十分 → `runes/redis/redis.fav` にも `consume_windowed` スタブを追加（T1b）
- [LOW] `changelog_has_v40_6_0` テスト省略 → 3 テスト構成（2832）に変更（spec も更新）
- [LOW] plan.md に T7 対応ステップ欠落 → 実装時に対応（tasks.md 記録）

**code-reviewer 指摘（実装後対応）:**
- [MED] `runes/redis/rune.toml` version が `1.0.0` のまま → `40.6.0` に更新 ✅
- [LOW] `runes/redis/rune.toml` description に `consume_windowed` 未記載 → 全関数・型を列挙した description に更新 ✅
- [LOW] `runes/kafka/kafka.fav` の `KafkaConn` 型に `public` キーワードなし → `public type KafkaConn(String)` に修正 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（3 件 → 全対応）
- [x] code-reviewer 指摘対応済み（実施済み）
