# v68.7.0 タスクリスト

Status: COMPLETE
Version: 68.7.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3531
Target tests: 3533

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3531 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/ai_routing.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod cost_estimate;` が存在することを確認（`mod ai_routing;` の挿入位置）
- [x] `driver.rs` に `v68600_tests` が存在することを確認（`v68700_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68700_tests` を `v68600_tests` の直前に挿入する
- [x] `driver.rs` に `v68700_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68600_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `cost_estimate_ai_pipeline`, `cost_optimize_batch_size`
- [x] `versions/current.md` の「進行中バージョン」が `v68.6.0` であることを確認

---

## T1: `fav/src/ai_routing.rs` 新規作成

- [x] `fav/src/ai_routing.rs` を新規作成
  - [x] `pub fn cmd_ai_routing(src: &str, env: &str) -> String` を追加
    - [x] `"[ai]"` / `"llm_provider"` / `"--env"` を含む出力（`multi_cloud_ai_routing` テスト要件）
    - [x] `"ollama-local"` / `"mock"` / `"in-memory"` を含む出力（`ai_provider_local_fallback` テスト要件）
    - [x] 出力末尾は `[stub] Would apply AI routing (source: <src>)`（実際のルーティングなし）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod ai_routing;` を mod 宣言部（`mod cost_estimate;` の直後）に追加
- [x] `Some("ai-routing")` アームをサブコマンド群に追加
  - [x] `--env` の次引数を `env` として取得（省略時は `"prod"`）
  - [x] `env_idx` を `HashSet` に収集してインデックスベースで `src` 候補から除外（v68.6.0 の [MED] 修正パターン踏襲）
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", ai_routing::cmd_ai_routing(src, env))`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68700_tests` 追加

- [x] `// -- v68600_tests (v68.6.0) -- Cost-Aware Scheduling --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `multi_cloud_ai_routing`: 各キーワードを個別 `assert!` で検証（`"[ai]"` / `"llm_provider"` / `"--env"`）
  - [x] `ai_provider_local_fallback`: 各キーワードを個別 `assert!` で検証（`"ollama-local"` / `"mock"` / `"in-memory"`）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68700_tests` で 2 件 PASS
  - [x] `multi_cloud_ai_routing` PASS
  - [x] `ai_provider_local_fallback` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3533 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.7.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.7.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- `fav.toml` の `[ai]` セクション実際のパース（`toml.rs` 拡張）: 将来フェーズ
- `LLMProvider` interface 実装（anthropic / openai / ollama / mock）: 将来フェーズ
- `VectorDBProvider` interface 実装（pinecone / qdrant / pgvector / in-memory）: 将来フェーズ
- `fav run --env dev` との統合（実際のプロバイダー切り替え）: 将来フェーズ
- コスト追跡: 本番プロバイダーのみコスト計算（dev/test は $0）: 将来フェーズ

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [MED] | `main.rs` `Some("ai-routing")` L3366 | `--env` 省略時デフォルトが `"prod"` は本番誤適用リスクあり。CLI 通例では `"dev"` がデフォルト | `unwrap_or("dev")` に変更し、コメント「省略時は dev（本番誤適用防止）」を追記 |
| [LOW] | `main.rs` `Some("ai-routing")` | `--env` フラグ自身のインデックスを `skip_indices` に追加していない（`cost-estimate` アームとスタイル不一致）。実害なし | 記録のみ（フラグ自体は `starts_with('-')` で除外済みのため修正不要） |
| [LOW] | `ai_routing.rs` | `env` が行7・行11の2箇所に展開されており、将来の変更時に分散に注意 | 記録のみ（現時点では機能上の問題なし） |
