# v69.8.0 Plan — パフォーマンス回帰テスト

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

## 実装ステップ

### Step 1: 事前確認

1. `cargo test --bin fav -- --test-threads=8` でベース 3551 tests passed, 0 failed を確認
2. `benchmarks/compare/` ディレクトリが存在することを確認（新規作成先の確認）
3. `benchmarks/compare/v69-baseline.md` が存在しないことを確認
4. `site/content/docs/runtime/benchmarks.mdx` に `"Intelligent ETL"` が含まれないことを確認（重複防止）
5. `driver.rs` に `v69700_tests` が存在することを確認（挿入先の確認）
6. `driver.rs` に `v69800_tests` が存在しないことを確認（重複防止）

### Step 2: `benchmarks/compare/v69-baseline.md` を新規作成

以下の内容で作成する:
- ファイルタイトル: `# Performance Baseline: v65.0 → v69.x`
- 比較環境セクション
- コンパイル時間比較テーブル（v65.0 vs v69.x）
- VM 実行時間（bench-results.json 実測値: mean_ms=0.191、p99_ms=0.200）
- AOT 実行時間（bench-results.json 実測値: mean_ms=0.532、p99_ms=0.576）
- AI ステージスループット（mock モードでの参考値）
- "v65.0" と "v69" の両キーワードを含むこと

### Step 3: `site/content/docs/runtime/benchmarks.mdx` に Intelligent ETL セクション追加

既存の末尾行 `For tuning your own pipelines, see the [Performance Tuning Guide](./performance).` の直後に以下のセクションを追加:
- 見出し: `## Intelligent ETL パフォーマンス`
- AI ETL パイプライン（embed + llm + vectordb）のスループット参考値テーブル
- "Intelligent ETL" キーワードを含む

### Step 4: `driver.rs` にテスト追加

テストモジュールは**降順（最新が先頭）**で並んでいる（v69700 → v69600 → ...）。
v69800 を v69700_tests の直前に挿入する。

```rust
// -- v69800_tests (v69.8.0) -- パフォーマンス回帰テスト --
#[cfg(test)]
mod v69800_tests {
    #[test]
    fn benchmark_compare_has_v69_baseline() {
        let src = include_str!("../../benchmarks/compare/v69-baseline.md");
        assert!(src.contains("v65.0"), "v69-baseline.md should reference v65.0 baseline");
        assert!(src.contains("v69"), "v69-baseline.md should reference v69 results");
    }

    #[test]
    fn site_benchmarks_covers_intelligent_etl() {
        let src = include_str!("../../site/content/docs/runtime/benchmarks.mdx");
        assert!(src.contains("Intelligent ETL"), "benchmarks.mdx should cover Intelligent ETL performance");
    }
}
```

### Step 5: ビルド・テスト確認

1. `cargo build 2>&1 | grep "^error"` — エラーゼロ確認
2. `cargo test --bin fav -- --test-threads=8` — 3553 tests passed, 0 failed 確認

### Step 6: ドキュメント・ステータス更新

1. `roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.8.0 行を確定（3553、+2）
2. `roadmap-v69.1-v70.0.md` の v69.8.0 状態列を「完了」に変更
3. `versions/current.md` の「進行中バージョン」を `v69.8.0` に更新
4. 本 `tasks.md` を COMPLETE に更新

## 依存関係

- Step 2（v69-baseline.md 作成）と Step 3（benchmarks.mdx 更新）は並行して実施可能
- Step 4（テスト追加）は Step 2・3 完了後
