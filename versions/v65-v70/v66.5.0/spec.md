# v66.5.0 Spec — Streaming Inference Stage

Version: 66.5.0
Status: 未着手
Base tests: 3483
Target tests: 3485

---

## 概要

リアルタイムスコアリングパイプラインを型安全に実装する Rune を提供する。
Kafka ストリーム + ML モデル推論を組み合わせ、バックプレッシャー制御で無限ストリームを安全に処理する。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.5.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップの利用例では `Vec<Float>[1536]` 等の次元型パラメータや
> `Rune.ml.load` / `Rune.ml.predict` 等の ML 実行を使用しているが、型システムへの登録および
> 実際の推論実行は将来フェーズ。本バージョンでは `List<Float>` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3483 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（v66.0.0 宣言時に `"66.0.0"` に設定済み。v66.x sub-version では更新しない。v67.0.0 宣言時に `"67.0.0"` に更新する）
- `runes/inference/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66400_tests` が存在することを確認（`v66500_tests` の挿入位置）
- `driver.rs` に `v66500_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66400_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `vector_db_upsert_query`, `vector_db_type_safe_dim`
- `versions/current.md` の「進行中バージョン」が `v66.4.0` であることを確認

---

## 実装スコープ

### 1. `runes/inference/rune.toml` — 新規作成

```toml
[rune]
name        = "inference"
version     = "0.1.0"
description = "Inference Rune for Favnir — streaming ML inference: batch inference, backpressure control, SLA monitoring, stateful scoring"
entry       = "inference.fav"
effects     = []

[dependencies]
```

### 2. `runes/inference/inference.fav` — 新規作成スタブ

```favnir
// inference Rune — ストリーミング ML 推論
// inference_batch, stream_with_backpressure, stream_with_sla, stateful_score
//
// NOTE: List<Float> は Vec<Float>[N] の将来フェーズ登録までのプレースホルダー。
//       StreamingInferenceInterface — ストリーミング推論統一インターフェース（将来フェーズ）
//       include_str! テストのみ（型チェックエラーは無視する）。

// バッチ推論を実行する
public fn inference_batch(embeddings: List<List<Float>>, model: String, batch_size: Int) -> List<String> {
    []
}

// バックプレッシャー制御付きストリーミング推論
public fn stream_with_backpressure(stream: String, model: String, buffer_size: Int) -> List<String> {
    []
}

// レイテンシ SLA 付きストリーミング推論
public fn stream_with_sla(stream: String, model: String, max_latency_ms: Int) -> List<String> {
    []
}

// セッション単位の状態を保持するスコアリング
public fn stateful_score(session_id: String, embedding: List<Float>, model: String) -> String {
    ""
}
```

### 3. `driver.rs` — `v66500_tests` 追加

挿入位置: `// -- v66400_tests (v66.4.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/inference/rune.toml` が存在する
- `runes/inference/inference.fav` が存在し以下を含む:
  - `fn inference_batch(` — バッチ推論
  - `fn stream_with_backpressure(` — バックプレッシャー制御
  - `fn stream_with_sla(` — SLA 付きストリーミング
  - `fn stateful_score(` — 状態付きスコアリング
  - ヘッダーコメントに `StreamingInferenceInterface` を含む（**この文字列はコメント行に固定配置。削除・変更した場合は `streaming_inference_pipeline` テストのアサーションも連動更新すること**）
- `cargo test --bin fav v66500_tests` で 2 件 PASS
  - `streaming_inference_pipeline` PASS
  - `streaming_backpressure_ai` PASS
- `cargo test -j 8 -- --test-threads=8` で 3485 tests passed, 0 failed

---

## 非スコープ

- `Vec<Float>[N]` 次元型パラメータの型システム登録 — 将来フェーズ
- `Rune.ml.load` / `Rune.ml.predict` 実際の推論実行 — 将来フェーズ（スタブのみ）
- Kafka ストリーム実接続 — 将来フェーズ
- `rune.toml` の `effects` 更新 — 本番 API 呼び出し実装時に追加（将来フェーズ）
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/inference/inference.fav"` → 新規ファイル

### `contains` 判定の設計方針

- `inference.contains("fn inference_batch(")` — `public fn inference_batch(` にマッチ
- `inference.contains("fn stream_with_backpressure(")` — `public fn stream_with_backpressure(` にマッチ
- `inference.contains("StreamingInferenceInterface")` — ヘッダーコメントでマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること
- `inference.contains("fn stream_with_sla(")` — `public fn stream_with_sla(` にマッチ
- `inference.contains("fn stateful_score(")` — `public fn stateful_score(` にマッチ

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### 新規 Rune の rune.toml フォーマット

- `entry = "ファイル名.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める（依存なしの場合も空セクションとして明示。`runes/embed/rune.toml` と同一フォーマット）
