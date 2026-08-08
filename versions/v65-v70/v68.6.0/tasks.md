# v68.6.0 タスクリスト

Status: COMPLETE
Version: 68.6.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3529
Target tests: 3531

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3529 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/cost_estimate.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod dist_cache;` が存在することを確認（`mod cost_estimate;` の挿入位置）
- [x] `driver.rs` に `v68500_tests` が存在することを確認（`v68600_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68600_tests` を `v68500_tests` の直前に挿入する
- [x] `driver.rs` に `v68600_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68500_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `distributed_cache_hit_across_workers`, `distributed_cache_invalidation`
- [x] `versions/current.md` の「進行中バージョン」が `v68.5.0` であることを確認

---

## T1: `fav/src/cost_estimate.rs` 新規作成

- [x] `fav/src/cost_estimate.rs` を新規作成
  - [x] `pub fn cmd_cost_estimate(src: &str, provider: &str, scale: &str) -> String` を追加
    - [x] `"Cost Estimate"` / `"TOTAL"` / `"--scale"` を含む出力（`cost_estimate_ai_pipeline` テスト要件）
    - [x] `"Optimizations"` / `"バッチサイズ"` / `"-55%"` を含む出力（`cost_optimize_batch_size` テスト要件）
    - [x] 出力末尾は `[stub] Would calculate costs for: <src>`（実際の計算なし）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod cost_estimate;` を mod 宣言部（`mod dist_cache;` の直後）に追加
- [x] `Some("cost-estimate")` アームをサブコマンド群に追加
  - [x] `--provider` の次引数を `provider` として取得（省略時は `"aws"`）
  - [x] `--scale` の次引数を `scale` として取得（省略時は `"1M-rows"`）
  - [x] `src` 検出時に `provider` / `scale` の値を除外（`a.as_str() != provider && a.as_str() != scale`）
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", cost_estimate::cmd_cost_estimate(src, provider, scale))`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68600_tests` 追加

- [x] `// -- v68500_tests (v68.5.0) -- Distributed Incremental Cache --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `cost_estimate_ai_pipeline`: 各キーワードを個別 `assert!` で検証（`"Cost Estimate"` / `"TOTAL"` / `"--scale"`）
  - [x] `cost_optimize_batch_size`: 各キーワードを個別 `assert!` で検証（`"Optimizations"` / `"バッチサイズ"` / `"-55%"`）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68600_tests` で 2 件 PASS
  - [x] `cost_estimate_ai_pipeline` PASS
  - [x] `cost_optimize_batch_size` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3531 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.6.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.6.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際のプロバイダー API 料金テーブル取得（OpenAI / Anthropic / Cohere）: 将来フェーズ
- ベクトル DB コスト計算（Pinecone / Weaviate / pgvector）: 将来フェーズ
- コンピュートコスト計算（AWS ECS / Lambda / GCP Cloud Run）: 将来フェーズ
- 最適化提案の実際のロジック（バッチサイズ自動調整・Spot 活用）: 将来フェーズ
- `--provider gcp` / `--provider azure` の実際の料金差分: 将来フェーズ

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [MED] | `main.rs` `Some("cost-estimate")` | src 検出でファイル名が `provider`/`scale` 値と偶然一致した場合に誤検出するリスク（値比較方式の限界） | フラグ値インデックスを `HashSet` に収集してインデックスベースで除外する方式に変更 |
| [LOW] | `driver.rs` L49110 | 旧 `cmd_cost_estimate(provider) -> i32` と新 `cost_estimate::cmd_cost_estimate` が同名並立 | 旧関数にコメントを追加（v59300_tests 参照のため残存。将来削除計画を明示） |

## 保守注記

- `"-55%"` はスタブ実装のハードコード値。将来この値を変更する場合は `cost_estimate.rs`・`plan.md`・`driver.rs` の三箇所を同期すること。
