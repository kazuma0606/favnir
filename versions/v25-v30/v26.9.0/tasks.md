# v26.9.0 タスクリスト — Pulsar Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.8.0`、テスト数 2102 件、`runes/pulsar/` ディレクトリが存在しない、`vm.rs` に `Pulsar.produce_raw` がない、`docker-compose.yml` に `pulsar:` がないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.9.0"` に bump | [x] |
| T2 | `runes/pulsar/pulsar.fav` を新規作成（4 関数: produce / consume / ack / nack） | [x] |
| T3 | `fav/src/backend/vm.rs` に `Pulsar.*_raw` primitive 4 件追加（RabbitMQ ブロックの直後・Azure Blob の直前、`#[cfg]` ガード付き） | [x] |
| T4 | `examples/streaming/docker-compose.yml` に `pulsar` サービス追加（`apachepulsar/pulsar:3.2.0`、ports 6650/8080） | [x] |
| T5 | `site/content/docs/runes/pulsar.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v26.9.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v26.9.0.json` 新規作成（test_count: 2110） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v269000_tests`（8 件）を `v268000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v269000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8.6 | `cargo test pulsar --bin fav` — 6 件以上 PASS 確認（ロードマップ要件「3 件以上」超過） | [x] |
| T9 | `cargo test --bin fav` — 2110 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前）| [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.9.0"` であること
- [x] `runes/pulsar/pulsar.fav` に `fn produce(` が含まれること
- [x] `runes/pulsar/pulsar.fav` に `fn consume(` が含まれること
- [x] `runes/pulsar/pulsar.fav` に `fn ack(` が含まれること
- [x] `runes/pulsar/pulsar.fav` に `fn nack(` が含まれること
- [x] `fav/src/backend/vm.rs` に `Pulsar.produce_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Pulsar.consume_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Pulsar.ack_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Pulsar.nack_raw` が含まれること
- [x] `examples/streaming/docker-compose.yml` に `pulsar:` サービスが含まれること
- [x] `site/content/docs/runes/pulsar.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.9.0]` エントリが存在すること
- [x] `benchmarks/v26.9.0.json` が存在すること（test_count: 2110）
- [x] `v269000_tests` 8 件すべて PASS
- [x] `cargo test pulsar --bin fav` で 6 件以上 PASS（実績見込み: 6 件）
- [x] 総テスト数 ≥ 2110 件

---

## メモ

### vm.rs の挿入位置

`// ── RabbitMQ primitives (v26.3.0)` ブロック末尾の直後、
`// ── Azure Blob Storage` ブロック直前に挿入する。

### `#[cfg]` パターン（RabbitMQ と同一）

各 primitive に:
- `#[cfg(not(target_arch = "wasm32"))]` — native 実装アーム
- `#[cfg(target_arch = "wasm32")]` — `err_vm("Pulsar not supported on wasm32")` アーム

### base64 エンコード（produce_raw）

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD};
let payload_b64 = STANDARD.encode(value.as_bytes());
```

`base64` クレートは Cargo.toml に既存登録済み。

### テスト期待値（cargo test pulsar）

`cargo test pulsar --bin fav` で検出されるテスト（見込み 6 件）:
- `v269000_tests::pulsar_rune_has_produce_fn`
- `v269000_tests::pulsar_rune_has_consume_fn`
- `v269000_tests::pulsar_rune_has_ack_fn`
- `v269000_tests::pulsar_rune_has_nack_fn`
- `v269000_tests::pulsar_vm_has_produce_raw`
- `v269000_tests::pulsar_vm_has_ack_raw`

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] `!AWS` エフェクトが Pulsar（Apache 製品）に使われる意図が pulsar.fav に未コメント | `TODO(v27.x): !Pulsar エフェクトに移行予定` コメントを pulsar.fav に追加 |
| [MED] `pulsar_vm_has_consume_raw` / `pulsar_vm_has_nack_raw` テスト欠落 | driver.rs に 2 件追加（テスト数 2110 → 2112）、benchmarks 更新 |
| [LOW] docker-compose ヘルスチェックが相対パス・retries 不足 | `/pulsar/bin/pulsar-admin`（絶対パス）・`retries: 10` に修正 |
| [LOW] CHANGELOG の `#[cfg(not(wasm32))]` が誤記 | `#[cfg(not(target_arch = "wasm32"))]` に修正 |
| [LOW] pulsar.mdx の JetStream 誤記（NATS の機能） | `Persistent Topic の詳細制御` に修正 |
| [LOW] pulsar.mdx のコード例に `!AWS` 暫定注記がなかった | コード例直下に警告ノートを追加 |
