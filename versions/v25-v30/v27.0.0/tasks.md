# v27.0.0 タスクリスト — Streaming Native マイルストーン宣言

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.9.0`、`cargo test --bin fav` で 2112 件 PASS、5 本の Streaming Rune（kinesis/nats/rabbitmq/sqs/pulsar）と 3 本の E2E デモが存在すること確認。`README.md` に `v27.0` が未記載であること確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.0.0"` に bump | [x] |
| T2 | `MILESTONE.md` に "Streaming Native Milestone" セクション追加 | [x] |
| T3 | `README.md` に v27.0 マイルストーンを追記（`v25.0` 付近に追加） | [x] |
| T4 | `CHANGELOG.md` 更新: 先頭に `[v27.0.0]` エントリ追加 | [x] |
| T5 | `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日追記 | [x] |
| T6 | `site/content/docs/streaming-native.mdx` 新規作成 | [x] |
| T7 | `benchmarks/v27.0.0.json` 新規作成（test_count: 2120） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v270000_tests`（8 件）を `v269000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v270000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8.6 | `cargo test streaming --bin fav` — 既存テストがリグレッションしないこと確認（ロードマップ要件: `#[streaming]` バックプレッシャー対応の検証） | [x] |
| T9 | `cargo test --bin fav` — 2120 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前）| [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.0.0"` であること
- [x] `MILESTONE.md` に `"Streaming Native"` が含まれること
- [x] `README.md` に `"v27.0"` が含まれること
- [x] `CHANGELOG.md` に `[v27.0.0]` エントリが存在すること
- [x] `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日が記載されること
- [x] `site/content/docs/streaming-native.mdx` が存在すること
- [x] `benchmarks/v27.0.0.json` が存在すること（test_count: 2120）
- [x] `v270000_tests` 8 件すべて PASS
- [x] `cargo test streaming --bin fav` でリグレッションなし
- [x] 総テスト数 ≥ 2120 件

---

## メモ

### v25.0.0 との対応（マイルストーン宣言の先例）

v25.0.0 タスクとの比較:

| v25.0.0 | v27.0.0 |
|---|---|
| `MILESTONE.md`「Practical Self-Hosting」 | `MILESTONE.md`「Streaming Native」 |
| `README.md` に `v25.0` 追記 | `README.md` に `v27.0` 追記 |
| `versions/roadmap-v20.1-v25.0.md` 更新 | `versions/roadmap/roadmap-v26.1-v27.0.md` 更新 |
| `site/content/docs/v1-release.mdx` | `site/content/docs/streaming-native.mdx` |

### README.md の更新対象箇所

`v25.0` という文字列の付近に `v27.0` を追記する。
`readme_mentions_v27` テストは `"v27.0"` を含むかどうかを確認するため、
`v27.0.0` や `v27.0 —` など `v27.0` を含む形であれば正常に通る。

### `e2e_demos_all_present` テストの前提

`examples/streaming/README.md` が存在し、かつ以下 3 つを含む必要がある:
- `"kafka_to_elasticsearch"`
- `"kinesis_to_s3"`
- `"nats_to_postgres"`

v26.7.0 で作成した README.md にこれらが記載されていることを T0 で事前確認する。

### テスト数の計算

2112（v26.9.0 修正後）+ 8（v270000_tests）= 2120

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] MILESTONE.md / streaming-native.mdx に pulsar の `!AWS` 暫定エフェクト注記が未記載 | MILESTONE.md の pulsar 行・streaming-native.mdx の Rune 表に暫定注記追加 |
| [MED] examples/streaming/README.md のサービス一覧に pulsar 行が欠落 | pulsar（6650 / 8080）行を追加 |
| [LOW] v270000_tests に rabbitmq / sqs 確認テストが欠落（kinesis/nats/pulsar のみ） | `streaming_rune_rabbitmq_has_publish` / `streaming_rune_sqs_has_send_message` を追加（テスト数 2120 → 2122、benchmarks 更新）|
