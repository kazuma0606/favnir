# v68.9.0 実装計画

## Step 1: `site/content/docs/runtime/distributed.mdx` 新規作成

`site/content/docs/runtime/` ディレクトリが存在するか確認後、以下の内容で作成する。
存在しない場合は `mkdir -p site/content/docs/runtime/` を実行してから作成する。

MDX ファイルの内容（コードブロック内の bash 例は 4 スペースインデントで記述して MDX パースエラーを回避する）:

    # Distributed Favnir

    Favnir v68.x では、AI パイプラインを複数ノードに分散実行するための機能を提供します。

    ## 分散実行（v68.1）

    `--cluster workers.yaml` フラグでワーカーノードを定義し、`par` ステージを複数マシンに分散します。

        fav run pipeline.fav --cluster workers.yaml --partition-by "row_id % 4"

    ## チェックポイント（v68.2）

    `--checkpoint <dir>` フラグでステージ完了後に状態を保存し、失敗時は `--resume` で再開します。

    ## Kubernetes 対応（v68.3）

    `fav deploy --target kubernetes` で Pipeline CRD マニフェストを生成します。

    ## リトライポリシー（v68.4）

    `--retry-policy` フラグで ExponentialBackoff / LinearBackoff / フォールバック戦略を設定します。

    ## 分散キャッシュ（v68.5）

    `--distributed-cache <redis_url>` フラグで Redis バックエンドの分散キャッシュを有効化します。

    ## コスト見積もり（v68.6）

    `fav cost-estimate pipeline.fav --provider aws --scale 1M-rows` でパイプライン実行コストを見積もります。

    ## AI ルーティング（v68.7）

    `fav ai-routing pipeline.fav --env dev` で環境別 LLM / VectorDB プロバイダーを確認します。

    ## 分散トレーシング（v68.8）

    `--otel-endpoint <url>` フラグで OpenTelemetry Collector にトレースを送信します。

**キーワード確認**: `"--cluster"` ✓（Step 1 の「分散実行」セクション）

## Step 2: `driver.rs` — `v68900_tests` 追加

挿入位置: `// -- v68800_tests (v68.8.0) -- Distributed Observability --` の直前
（注意: driver.rs のテストブロックは降順配置〔新しいものが上〕）

```rust
// -- v68900_tests (v68.9.0) -- 安定化・コードフリーズ --
#[cfg(test)]
mod v68900_tests {
    #[test]
    fn distributed_all_stable() {
        // v68.1: cluster（partition_by = "row_id % 4" はロードマップ例示の標準値）
        let cluster = crate::cluster::cmd_cluster_run("pipeline.fav", "workers.yaml", "row_id % 4");
        assert!(cluster.contains("--cluster"), "v68.1 --cluster should be stable");
        // v68.2: checkpoint（resume_file = "" は初回実行モード）
        let ckpt = crate::checkpoint::cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "");
        assert!(ckpt.contains("--checkpoint"), "v68.2 --checkpoint should be stable");
        // v68.5: dist_cache
        let cache = crate::dist_cache::cmd_distributed_cache("pipeline.fav", "redis://localhost:6379");
        assert!(cache.contains("--distributed-cache"), "v68.5 --distributed-cache should be stable");
        // v68.8: dist_otel
        let otel = crate::dist_otel::cmd_dist_otel("pipeline.fav", "http://tempo:4317");
        assert!(otel.contains("--otel-endpoint"), "v68.8 --otel-endpoint should be stable");
    }

    #[test]
    fn distributed_docs_complete() {
        let docs = include_str!("../../site/content/docs/runtime/distributed.mdx");
        assert!(docs.contains("--cluster"), "distributed.mdx should contain '--cluster'");
    }
}
```

- `cargo build` でエラーなし（Step 2 完了後）

## 注意事項

- `fav/src/main.rs` の変更は不要（安定化フェーズ）
- 新規 `.rs` モジュールは不要（既存モジュールの統合確認のみ）
- `distributed_all_stable` は 4 モジュールを直接呼び出す統合テスト（cluster / checkpoint / dist_cache / dist_otel）
- `distributed_docs_complete` は `include_str!` で MDX ファイルの存在と内容を確認する
- `include_str!` のパスは `fav/src/driver.rs` からの相対パス: `../../site/content/docs/runtime/distributed.mdx`
- Step 1 完了後に `cargo test --bin fav v68900_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
