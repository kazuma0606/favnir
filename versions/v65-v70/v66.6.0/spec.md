# v66.6.0 Spec — Model Serving Rune（`Rune.serve`）

Version: 66.6.0
Status: 未着手
Base tests: 3485
Target tests: 3487

---

## 概要

Favnir ステージをモデルサービングエンドポイントとして公開する Rune。
`fav serve` コマンドでパイプラインを HTTP API として起動できる設計の基盤を提供する。
レート制限・OpenAPI スキーマ生成を型安全に記述できる。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.6.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップでは `fav serve` コマンドの実際のHTTPサーバー起動・
> JSON シリアライズ自動生成・OpenAPI ファイル出力を示しているが、これらは将来フェーズ。
> 本バージョンでは `List<String>` / `String` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3485 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（v66.0.0 宣言時に設定済み。v66.x sub-version では更新しない。v67.0.0 宣言時に `"67.0.0"` に更新する）
- `runes/serve/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66500_tests` が存在することを確認（`v66600_tests` の挿入位置）
- `driver.rs` に `v66600_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66500_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `streaming_inference_pipeline`, `streaming_backpressure_ai`
- `versions/current.md` の「進行中バージョン」が `v66.5.0` であることを確認

---

## 実装スコープ

### 1. `runes/serve/rune.toml` — 新規作成

```toml
[rune]
name        = "serve"
version     = "0.1.0"
description = "Model Serving Rune for Favnir — expose stages and pipelines as HTTP endpoints with rate limiting and OpenAPI schema generation"
entry       = "serve.fav"
effects     = []

[dependencies]
```

### 2. `runes/serve/serve.fav` — 新規作成スタブ

```favnir
// serve Rune — モデルサービングエンドポイント
// serve_stage, serve_pipeline, with_rate_limit, openapi_schema
//
// NOTE: 実際の HTTP サーバー起動・JSON シリアライズは将来フェーズ。
//       ModelServingInterface — モデルサービング統一インターフェース（将来フェーズ）
//       include_str! テストのみ（型チェックエラーは無視する）。

// 単一ステージを HTTP エンドポイントとして公開する
public fn serve_stage(stage_name: String, port: Int) -> String {
    ""
}

// パイプライン全体を HTTP エンドポイントとして公開する
public fn serve_pipeline(pipeline_name: String, port: Int) -> String {
    ""
}

// レート制限を設定する（rps: requests per second）
public fn with_rate_limit(rps: Int) -> String {
    ""
}

// OpenAPI スキーマを生成する
public fn openapi_schema(stage_name: String) -> String {
    ""
}
```

### 3. `driver.rs` — `v66600_tests` 追加

挿入位置: `// -- v66500_tests (v66.5.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/serve/rune.toml` が存在する
- `runes/serve/serve.fav` が存在し以下を含む:
  - `fn serve_stage(` — 単一ステージ公開
  - `fn serve_pipeline(` — パイプライン全体公開
  - `fn with_rate_limit(` — レート制限設定
  - `fn openapi_schema(` — OpenAPI スキーマ生成
  - ヘッダーコメントに `ModelServingInterface` を含む（**この文字列はコメント行に固定配置。削除・変更した場合は `model_serve_endpoint_type` テストのアサーションも連動更新すること**）
- `cargo test --bin fav v66600_tests` で 2 件 PASS
  - `model_serve_endpoint_type` PASS
  - `model_serve_schema_validation` PASS
- `cargo test -j 8 -- --test-threads=8` で 3487 tests passed, 0 failed

---

## 非スコープ

- 実際の HTTP サーバー起動 — 将来フェーズ
- JSON シリアライズ/デシリアライズ自動生成 — 将来フェーズ
- `fav serve` コマンド実装 — 将来フェーズ
- ヘルスチェックエンドポイント実装 — 将来フェーズ
- `rune.toml` の `effects` 更新 — 本番 API 呼び出し実装時に追加（将来フェーズ）
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/serve/serve.fav"` → 新規ファイル

### `contains` 判定の設計方針

- `serve.contains("fn serve_stage(")` — `public fn serve_stage(` にマッチ
- `serve.contains("fn serve_pipeline(")` — `public fn serve_pipeline(` にマッチ
- `serve.contains("ModelServingInterface")` — ヘッダーコメントでマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること
- `serve.contains("fn with_rate_limit(")` — `public fn with_rate_limit(` にマッチ
- `serve.contains("fn openapi_schema(")` — `public fn openapi_schema(` にマッチ

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### 新規 Rune の rune.toml フォーマット

- `entry = "ファイル名.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める（依存なしの場合も空セクションとして明示。`runes/embed/rune.toml` と同一フォーマット）
