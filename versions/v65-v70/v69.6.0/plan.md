# v69.6.0 Plan — Playground の UI 改善・サンプル追加

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。

## 実装ステップ

### Step 1: 事前確認

1. `cargo test --bin fav -- --test-threads=8` でベース 3547 tests passed, 0 failed を確認
2. `site/content/playground/ai-examples.mdx` に "Autodiff Demo" / "GradientStep" が含まれないことを確認（重複防止）
3. `site/content/playground/etl-samples.mdx` が存在しないことを確認
4. `driver.rs` に `v69600_tests` モジュールが存在しないことを確認

### Step 2: `ai-examples.mdx` に Autodiff Demo を追加

`ai-examples.mdx` の `## コスト見積もり` 見出し直前（108行目付近）に以下のセクションを追加する。
追加するセクション見出しは `### 5. Autodiff Demo（WASM）` とする。

追加コンテンツの要素:
- `GradientStep` ステージのサンプルコード（`bind grad <- Rune.autodiff.gradient(...)` 構文）
- `ComputeJacobian` ステージのサンプルコード（`Rune.autodiff.jacobian`）
- WASM 上で pure 関数として完全動作する旨の説明

### Step 3: `etl-samples.mdx` を新規作成

`site/content/playground/etl-samples.mdx` を新規作成する。
ファイルはページタイトル `# Playground — ETL パイプライン サンプル` で始める。

含めるコンテンツ:
1. **`schema Order`** 定義（`id: String`, `amount: Float`, `status: String` の 3 フィールド）
2. **CSV フィルタリング** サンプル（`FilterActive` ステージ、`List.filter` 使用）
3. **集計パイプライン** サンプル（`SumAmounts` ステージ、`bind amounts <- List.map(...)` 使用）
4. **Full ETL Pipeline** サンプル（`LoadOrders` → `Transform` → `ETLPipeline` 定義）
5. **bind 構文の説明**（`<-` が正しく `= expr` は不正構文、pure 関数への bind も有効と明記）

### Step 4: `driver.rs` にテスト追加

テストモジュールは**降順（最新が先頭）**で並んでいる（v69500 → v69200 → v69100 → v69000）。
v69600 は v69500 より新しいため、v69500_tests の直前に挿入する（結果: v69600 → v69500 → v69200 → ...）。

追加するコード:

```rust
// -- v69600_tests (v69.6.0) -- Playground UI 改善・サンプル追加 --
#[cfg(test)]
mod v69600_tests {
    #[test]
    fn playground_has_autodiff_sample() {
        let src = include_str!("../../site/content/playground/ai-examples.mdx");
        assert!(src.contains("Autodiff Demo"), "ai-examples.mdx should contain 'Autodiff Demo' section");
        assert!(src.contains("GradientStep"), "ai-examples.mdx should contain 'GradientStep' stage");
    }

    #[test]
    fn playground_etl_samples_page_exists() {
        let src = include_str!("../../site/content/playground/etl-samples.mdx");
        assert!(src.contains("ETL"), "etl-samples.mdx should contain 'ETL'");
        assert!(src.contains("bind"), "etl-samples.mdx should use bind syntax");
        assert!(src.contains("schema Order"), "etl-samples.mdx should contain 'schema Order' definition");
    }
}
```

### Step 5: ビルド・テスト確認

1. `cargo build 2>&1 | grep "^error"` — エラーゼロ確認
2. `cargo test --bin fav -- --test-threads=8` — 3549 tests passed, 0 failed 確認

### Step 6: ドキュメント・ステータス更新

1. `roadmap-v69.1-v70.0.md` のテスト数推移テーブルの v69.6.0 行を `3549（+2）` に確定させる（すでに行は存在する）
2. `roadmap-v69.1-v70.0.md` の v69.6.0 状態列を「完了」に変更
3. `versions/current.md` の「進行中バージョン」を `v69.6.0` に更新
4. 本 `tasks.md` を COMPLETE に更新

## 依存関係

- Step 2（ai-examples.mdx 更新）と Step 3（etl-samples.mdx 作成）は並行して実施可能
- Step 4（テスト追加）は Step 2・3 完了後
