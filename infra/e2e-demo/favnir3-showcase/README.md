# Favnir 3.0 統合ショーケース

v75.1〜v79.8 で実装した全機能（Temporal / Provenance / Verifiable / Execution Effects）を
統合したエンドツーエンドデモ。

## 実行手順

```bash
cd infra/e2e-demo/favnir3-showcase
fav run pipeline.fav
```

## 構成

- `pipeline.fav` — 統合パイプライン（v75〜v78 全スプリント機能を統合）
- `fav.toml` — プロジェクト設定（Cached / Adaptive エフェクト設定）
- `contract.fav` — ShowcaseContract3 宣言（全スプリントコントラクト）

## スプリント対応表

| スプリント | バージョン | 機能 |
|---|---|---|
| Temporal | v75.1〜v75.9 | As-of query / SCD2 / freshness policy |
| Provenance | v76.1〜v76.9 | 来歴追跡 / OpenLineage / PII 削除 |
| Verifiable | v77.1〜v77.9 | 不変条件検証 / 反例生成 / CI 統合 |
| Execution Effects | v78.1〜v78.9 | キャッシュ / 並列実行 / 実行モード選択 |
