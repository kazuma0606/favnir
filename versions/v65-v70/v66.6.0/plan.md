# v66.6.0 実装計画 — Model Serving Rune（`Rune.serve`）

Version: 66.6.0
Status: 未着手
Base tests: 3485
Target tests: 3487

---

## 実装ステップ

### Step 1: 新規 Rune ファイル作成

1. `runes/serve/rune.toml`（entry / effects = [] / [dependencies] 形式）
2. `runes/serve/serve.fav`（serve_stage / serve_pipeline / with_rate_limit / openapi_schema、ModelServingInterface コメント付き）

### Step 2: `driver.rs` テスト追加

- `// -- v66500_tests (v66.5.0)` コメントの直前に `v66600_tests` を挿入
- 2 テスト関数:
  - `model_serve_endpoint_type`（serve.fav の serve_stage / serve_pipeline / ModelServingInterface 検証）
  - `model_serve_schema_validation`（with_rate_limit / openapi_schema 検証）

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66600_tests
cargo test -j 8 -- --test-threads=8
```

---

## 関数一覧

| Rune | 関数 | 戻り値 | 備考 |
|---|---|---|---|
| serve | `serve_stage(stage_name, port)` | `""` | 単一ステージ公開スタブ |
| serve | `serve_pipeline(pipeline_name, port)` | `""` | パイプライン全体公開スタブ |
| serve | `with_rate_limit(rps)` | `""` | レート制限設定スタブ |
| serve | `openapi_schema(stage_name)` | `""` | OpenAPI スキーマ生成スタブ |

---

## `driver.rs` 挿入コード

```rust
// -- v66600_tests (v66.6.0) -- Model Serving Rune --
#[cfg(test)]
mod v66600_tests {
    #[test]
    fn model_serve_endpoint_type() {
        let serve = include_str!("../../runes/serve/serve.fav");
        assert!(
            serve.contains("fn serve_stage("),
            "serve.fav should define serve_stage"
        );
        assert!(
            serve.contains("fn serve_pipeline("),
            "serve.fav should define serve_pipeline"
        );
        assert!(
            serve.contains("ModelServingInterface"),
            "serve.fav should reference ModelServingInterface"
        );
    }

    #[test]
    fn model_serve_schema_validation() {
        let serve = include_str!("../../runes/serve/serve.fav");
        assert!(
            serve.contains("fn with_rate_limit("),
            "serve.fav should define with_rate_limit"
        );
        assert!(
            serve.contains("fn openapi_schema("),
            "serve.fav should define openapi_schema"
        );
    }
}
```

---

## リスク・注意点

- `ModelServingInterface` はコメント行にのみ存在するため、serve.fav のヘッダーコメントを変更・削除してはならない。やむを得ず変更する場合は `driver.rs` の `model_serve_endpoint_type` 内の `assert!(serve.contains("ModelServingInterface"), ...)` も同時に更新すること
- 新規 Rune は `public fn` 形式でスタブを統一（pinecone.fav の `fn Namespace.method` 形式とは異なる）
- 実際の HTTP サーバー起動は将来フェーズ（スタブのみ）

## 非スコープ

- 実際の HTTP サーバー起動 — 将来フェーズ
- JSON シリアライズ/デシリアライズ自動生成 — 将来フェーズ
- `fav serve` コマンド実装 — 将来フェーズ
- ヘルスチェックエンドポイント（`GET /health`）実装 — 将来フェーズ
- `rune.toml` の `effects` 更新 — 本番実装時（将来フェーズ）
