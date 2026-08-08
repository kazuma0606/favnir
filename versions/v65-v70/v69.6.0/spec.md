# v69.6.0 Spec — Playground の UI 改善・サンプル追加

## Background

v69.2.0 で `site/content/playground/ai-examples.mdx` を追加した（4 サンプル）。
`Rune.autodiff` は WASM 動作テーブルに掲載されているが、専用サンプルがなく一覧と整合しない。
v69.6.0 では Playground ページを拡充し、v70.0.0 宣言に向けた完成度を高める。

## Goals

1. `ai-examples.mdx` に 5 番目のサンプル「Autodiff Demo」を追加する
   - `Rune.autodiff` の WASM 実動作を示すサンプルコード
   - 自動微分（勾配計算）を使った数値最適化の例
2. `site/content/playground/etl-samples.mdx` を新規作成する
   - ETL パイプラインサンプル集（CSV 読込・変換・出力）
   - Playground の「ETL」タブ向けコンテンツ

## Favnir における `bind x <- expr` の使用方針

Favnir では `bind x <- expr` はエフェクトを持つ計算だけでなく、pure 関数にも使用できる。
`infra/e2e-demo/ai-etl/src/pipeline.fav` の実際のコード（line 49）でも:
```favnir
bind pairs <- List.map(articles, |a| { (a.id, a.embedding) })
```
と使われており、これは型チェッカーが受け入れる正しい構文。
etl-samples.mdx のサンプルコードでも同様に `bind x <- List.map(...)` / `bind x <- List.filter(...)` を使用してよい。

**不正構文**: `bind x = expr`（`<-` ではなく `=` を使う形式）は必ず `<-` に修正すること。

## Out of Scope

- Playground UI（JavaScript / React）の実際の変更
- WASM ビルドや実際のブラウザ動作確認
- `Cargo.toml` / `CHANGELOG.md` の変更（sub-version ポリシー）

## Success Criteria

- `cargo test --bin fav v696` で **2 tests passed, 0 failed**
- ベース (3547) + 2 = **3549 tests**
- `ai-examples.mdx` に `"Autodiff Demo"` と `"GradientStep"` が含まれる
- `etl-samples.mdx` が存在し `"ETL"` と `"bind"` と `"schema Order"` が含まれる

## Files to Modify / Create

- `site/content/playground/ai-examples.mdx` — Autodiff Demo セクションを追加（5 番目プリセット）
- `site/content/playground/etl-samples.mdx` — 新規作成
- `fav/src/driver.rs` — `v69600_tests` モジュールを追加（2 テスト、v69500_tests の直前に挿入）
- `versions/roadmap/roadmap-v69.1-v70.0.md` — v69.6.0 状態・テスト数更新

## Files NOT to Modify

- `Cargo.toml`（sub-version ポリシー）
- `CHANGELOG.md`（sub-version ポリシー）

## Error Codes

なし

## Autodiff Demo サンプルコード（概要）

```favnir
// 勾配降下法: f(x) = x^2 の最小値を探す
public stage GradientStep: Float -> Float = |x| {
    bind grad <- Rune.autodiff.gradient(|v| { v * v }, x)
    x - 0.1 * grad
}

// ヤコビアン: R^n -> R^m の微分行列
public stage ComputeJacobian: List<Float> -> List<List<Float>> = |inputs| {
    Rune.autodiff.jacobian(|xs| { List.map(xs, |x| { x * x }) }, inputs)
}
```

`Rune.autodiff.gradient` / `Rune.autodiff.jacobian` は pure 関数のため WASM で完全動作する。

## etl-samples.mdx コンテンツ概要

- CSV フィルタリング・集計・Full ETL Pipeline サンプル
- `bind x <- expr` 構文を使用（`List.map` / `List.filter` への bind も有効）
- `Rune.csv.read` を使用（Playground では mock データ返却）

## テスト数推移への影響

v69.6.0 の +2 を含め、v69.5〜v69.9 各 +2 の計 +10 が v70.0.0 の base に積み上がる。
v70.0.0 の完了条件は roadmap-v69.1-v70.0.md にて「3559（v69.9 ベース 3555 + 4）」に更新済み。
