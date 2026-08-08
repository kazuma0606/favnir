# v68.8.0 タスクリスト

Status: COMPLETE
Version: 68.8.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3533
Target tests: 3535

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3533 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/dist_otel.rs` が存在しないことを確認（新規作成）
- [x] `fav/src/main.rs` に `mod ai_routing;` が存在することを確認（`mod dist_otel;` の挿入位置）
- [x] `driver.rs` に `v68700_tests` が存在することを確認（`v68800_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v68800_tests` を `v68700_tests` の直前に挿入する
- [x] `driver.rs` に `v68800_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68700_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `multi_cloud_ai_routing`, `ai_provider_local_fallback`
- [x] `versions/current.md` の「進行中バージョン」が `v68.7.0` であることを確認
- [x] `main.rs` の `Some("run")` アームに `--distributed-cache` ブランチが存在することを確認（挿入位置: その直後）
- [x] `main.rs` の `Some("run")` アームに `--env` ブランチが存在することを確認（`--otel-endpoint` ブランチの後続位置確認）

---

## T1: `fav/src/dist_otel.rs` 新規作成

- [x] `fav/src/dist_otel.rs` を新規作成
  - [x] `pub fn cmd_dist_otel(src: &str, otel_endpoint: &str) -> String` を追加
    - [x] `"--otel-endpoint"` / `"trace_id"` / `"span"` を含む出力（`distributed_otel_trace` テスト要件）
    - [x] `"LLM"` / `"VectorDB"` / `"Grafana"` を含む出力（`distributed_latency_breakdown` テスト要件）
    - [x] 出力末尾は `[stub] Would export trace to: <otel_endpoint>`（実際の送信なし）
    - [x] `format!` の `{}` 3 個と引数 3 個（otel_endpoint, otel_endpoint, otel_endpoint）が一致することを確認
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod dist_otel;` を mod 宣言部（`mod ai_routing;` の直後）に追加
- [x] `Some("run")` アームの `--distributed-cache` ブランチの直後に `--otel-endpoint` ブランチを追加
  - [x] `args.iter().any(|a| a == "--otel-endpoint")` で分岐
  - [x] `otel_endpoint` の値取得: 次引数が存在し `-` で始まらない場合のみ採用、それ以外は `eprintln!` + `process::exit(1)`
  - [x] `otel_endpoint` インデックス（i+1）を `HashSet` に収集してインデックスベースで `src` 候補から除外
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", dist_otel::cmd_dist_otel(src, otel_endpoint))` + `return;`
  - [x] 先行ブランチとの同時指定時の優先順位をコメントで明記
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68800_tests` 追加

- [x] `// -- v68700_tests (v68.7.0) -- Multi-Cloud AI Routing --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `distributed_otel_trace`: 各キーワードを個別 `assert!` で検証（`"--otel-endpoint"` / `"trace_id"` / `"span"`）
  - [x] `distributed_latency_breakdown`: 各キーワードを個別 `assert!` で検証（`"LLM"` / `"VectorDB"` / `"Grafana"`）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68800_tests` で 2 件 PASS
  - [x] `distributed_otel_trace` PASS
  - [x] `distributed_latency_breakdown` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3535 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.8.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v68.8.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際の OTel Collector への trace 送信: 将来フェーズ（v69.0.0 以降）
- 分散トレース: 各ステージを span として記録（parent/child 関係）: 将来フェーズ
- LLM span: モデル名・プロンプトトークン数・コスト・レイテンシ: 将来フェーズ
- VectorDB span: インデックス名・クエリ次元・top_k・レイテンシ: 将来フェーズ
- Grafana ダッシュボード定義（`infra/monitoring/favnir-ai-dashboard.json`）: 将来フェーズ
- Prometheus メトリクス統合（既存 v29.x の OTel Rune との連携）: 将来フェーズ

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [MED] | `dist_otel.rs` L15〜16 | `src` 引数が `format!` に渡されず未使用（Clippy `unused_variables` 警告が発生） | `[stub]` 行を `Would export trace to: {} (source: {})` に変更し `otel_endpoint, src` を渡すよう修正 |
| [LOW] | `main.rs` L433〜434 | `any()` + `position()` の二重線形走査でスタイル不統一 | `if let Some(otel_idx) = args.iter().position(...)` に変更し 1 回の走査に統一 |
