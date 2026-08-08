# v66.5.0 実装計画 — Streaming Inference Stage

Version: 66.5.0
Status: 未着手
Base tests: 3483
Target tests: 3485

---

## 実装ステップ

### Step 1: 新規 Rune ファイル作成

1. `runes/inference/rune.toml`（entry / effects = [] / [dependencies] 形式）
2. `runes/inference/inference.fav`（inference_batch / stream_with_backpressure / stream_with_sla / stateful_score、StreamingInferenceInterface コメント付き）

### Step 2: `driver.rs` テスト追加

- `// -- v66400_tests (v66.4.0)` コメントの直前に `v66500_tests` を挿入
- 2 テスト関数:
  - `streaming_inference_pipeline`（inference.fav の inference_batch / stream_with_backpressure / StreamingInferenceInterface 検証）
  - `streaming_backpressure_ai`（stream_with_sla / stateful_score 検証）

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66500_tests
cargo test -j 8 -- --test-threads=8
```

---

## 関数一覧

| Rune | 関数 | 戻り値 | 備考 |
|---|---|---|---|
| inference | `inference_batch(embeddings, model, batch_size)` | `[]` | バッチ推論スタブ |
| inference | `stream_with_backpressure(stream, model, buffer_size)` | `[]` | バックプレッシャー制御スタブ |
| inference | `stream_with_sla(stream, model, max_latency_ms)` | `[]` | SLA 付きストリーミングスタブ |
| inference | `stateful_score(session_id, embedding, model)` | `""` | 状態付きスコアリングスタブ |

---

## `driver.rs` 挿入コード

```rust
// -- v66500_tests (v66.5.0) -- Streaming Inference Stage --
#[cfg(test)]
mod v66500_tests {
    #[test]
    fn streaming_inference_pipeline() {
        let inference = include_str!("../../runes/inference/inference.fav");
        assert!(
            inference.contains("fn inference_batch("),
            "inference.fav should define inference_batch"
        );
        assert!(
            inference.contains("fn stream_with_backpressure("),
            "inference.fav should define stream_with_backpressure"
        );
        assert!(
            inference.contains("StreamingInferenceInterface"),
            "inference.fav should reference StreamingInferenceInterface"
        );
    }

    #[test]
    fn streaming_backpressure_ai() {
        let inference = include_str!("../../runes/inference/inference.fav");
        assert!(
            inference.contains("fn stream_with_sla("),
            "inference.fav should define stream_with_sla"
        );
        assert!(
            inference.contains("fn stateful_score("),
            "inference.fav should define stateful_score"
        );
    }
}
```

---

## リスク・注意点

- `StreamingInferenceInterface` はコメント行にのみ存在するため、コメントを変更した場合はテストも連動更新が必要
- 新規 Rune は `public fn` 形式でスタブを統一（pinecone.fav の `fn Namespace.method` 形式とは異なる）
- `List<Float>` は `Vec<Float>[N]` の将来フェーズ登録までのプレースホルダー（型チェックエラーは無視する）
