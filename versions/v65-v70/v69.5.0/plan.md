# v69.5.0 Plan — E2E デモの動作確認

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

## 実装ステップ

### Step 1: 事前確認

1. `cargo test --bin fav -- --test-threads=8` でベース 3545 tests passed, 0 failed を確認
2. `infra/e2e-demo/ai-etl/src/pipeline.fav` を開き `bind` と `<-` が両方含まれることを確認
3. `infra/e2e-demo/ai-etl/workers.yaml` を開き以下を確認:
   - `workers:` 直下に `- host:` で始まるリストエントリが 4 件あること
   - `name:` フィールドは存在しないため、カウントは `- host:` 行を使う
4. `driver.rs` の既存 `v69100_tests` モジュールに `ai_etl_demo_pipeline_uses_bind_arrow_syntax` / `ai_etl_demo_workers_yaml_has_four_workers` が存在しないことを確認（重複防止）

### Step 2: driver.rs にテスト追加

`driver.rs` の `#[cfg(test)]` モジュール末尾に以下を追加:

```rust
#[test]
fn ai_etl_demo_pipeline_uses_bind_arrow_syntax() {
    let src = include_str!("../../infra/e2e-demo/ai-etl/src/pipeline.fav");
    assert!(src.contains("bind"), "pipeline.fav should use bind keyword");
    assert!(src.contains("<-"), "pipeline.fav should use bind x <- expr syntax (not bind x = expr)");
}

#[test]
fn ai_etl_demo_workers_yaml_has_four_workers() {
    let yaml = include_str!("../../infra/e2e-demo/ai-etl/workers.yaml");
    // workers.yaml の構造: "- host:" 行が各ワーカーエントリの先頭
    // name: フィールドは存在しないため host: でカウント
    let count = yaml.lines().filter(|l| l.trim_start().starts_with("- host:")).count();
    assert_eq!(count, 4, "workers.yaml should define exactly 4 workers, got {}", count);
}
```

### Step 3: ビルド・テスト確認

1. `cargo build 2>&1 | grep "^error"` — エラーゼロ確認
2. `cargo test --bin fav -- --test-threads=8` — 3547 tests passed, 0 failed 確認

### Step 4: ドキュメント・ステータス更新

1. `roadmap-v69.1-v70.0.md` のテスト数推移テーブルに v69.5.0 の新規行を追加して `3547`（+2）を記入
2. `roadmap-v69.1-v70.0.md` の v69.5.0「状態」列を「未着手」→「完了」に変更
3. `roadmap-v69.1-v70.0.md` の v69.1.0 セクションのサンプルコード中 `bind x = expr` を `bind x <- expr` に修正（不正構文の修正）
3. `versions/current.md` の「進行中バージョン」を `v69.5.0` に更新
4. 本 `tasks.md` を COMPLETE に更新

## 依存関係

- `infra/e2e-demo/ai-etl/src/pipeline.fav` が存在し `bind x <- expr` 構文を使用していること（v69.1.0 で確認済み）
- `infra/e2e-demo/ai-etl/workers.yaml` が存在し `- host:` エントリを 4 件持つこと（v69.1.0 で確認済み）
