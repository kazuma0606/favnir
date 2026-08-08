# v69.3.0 実装計画 — ドキュメントサイト「Intelligent ETL ガイド」

Status: DRAFT
Version: 69.3.0

---

## 実装ステップ

### Step 1: ディレクトリ構造の作成

```
site/content/docs/intelligent-etl/
├── overview.mdx
├── quickstart.mdx
├── math-foundation.mdx
├── ai-stages.mdx
├── debugging.mdx
├── distributed.mdx
└── reference/
    ├── math-runes.mdx
    └── ai-runes.mdx
```

### Step 2: 各 MDX ファイルの作成

以下の順序で作成する（依存関係の浅いものから）:

1. `overview.mdx` — ビジョン・全体像
2. `quickstart.mdx` — 15 分チュートリアル（`infra/e2e-demo/ai-etl/` を参照）
3. `math-foundation.mdx` — Rune.linalg / Rune.stats / Rune.autodiff
4. `ai-stages.mdx` — Rune.embed / Rune.llm / VectorDB Rune
5. `debugging.mdx` — fav debug / fav viz / fav suggest
6. `distributed.mdx` — --cluster / --checkpoint / kubernetes / --distributed-cache / --otel-endpoint
7. `reference/math-runes.mdx` — Math Rune API リファレンス
8. `reference/ai-runes.mdx` — AI Rune API リファレンス

### Step 3: テスト実行（確認のみ）

```bash
cargo test -j 8 -- --test-threads=8  # 3545 tests PASS（変化なし）
```

MDX ファイルのみの変更のため Rust テスト数は 3545 のまま変化しない。
テスト数が増減した場合は Rust ファイルへの意図しない変更がないか確認すること。

`reference/` サブディレクトリは Step 2 の 7 番目ファイルを作成する前に作成すること。

---

## MDX 作成時の注意点

- 先頭に `import` 行を書かない（MDX acorn エラー回避）
- コードブロックはフェンス記法のみ（インデント記法不可）
- Favnir コードサンプルの `bind` 構文は `bind x <- expr`（`= expr` 不可）
- `Vec<Float>[1536]` は使わない（未定義型。`List<Float>` を使う）

---

## sub-version ポリシー

v69.x では `Cargo.toml` / `CHANGELOG.md` は変更しない。v70.0.0 宣言時に一括更新する。
