# Favnir v6.5.0 実装計画 — サイトドキュメント補完

作成日: 2026-05-27

---

## 実装順序

```
Phase A (pipeline.mdx)
  → Phase B (schema.mdx)
    → Phase C (infer.mdx)
      → Phase D (rune-cli.mdx 更新)
        → Phase E (検証)
```

各フェーズはドキュメント作成のみ。コード変更なし。

---

## Phase A — `language/pipeline.mdx`

### 変更ファイル

- `site/content/docs/language/pipeline.mdx` (新規作成)

### フロントマター

```mdx
---
title: "パイプライン（stage / seq）"
order: 6
category: "言語仕様"
description: "stage・seq・|> によるデータパイプラインの定義"
---
```

### 注意点

- `abstract stage` / `abstract seq` の構文は `compiler.fav` で v6.3.0 に対応済み
- `fav explain` の出力フォーマットは実装の出力と一致させること
- `|>` のエフェクト合成ルール（左右のエフェクトを union する）を明記

---

## Phase B — `language/schema.mdx`

### 変更ファイル

- `site/content/docs/language/schema.mdx` (新規作成)

### フロントマター

```mdx
---
title: "スキーマと制約"
order: 7
category: "言語仕様"
description: "schemas/*.yaml による型制約の宣言と fav build --schema"
---
```

### 注意点

- 制約は `checker.rs` で実装済みのものに限定して記載
- `T.validate` は v6.6.0 で完全実装のため「preview」扱いで記載
- YAML のインデントを正確に（スペース 2 個）

---

## Phase C — `stdlib/infer.mdx`

### 変更ファイル

- `site/content/docs/stdlib/infer.mdx` (新規作成)

### フロントマター

```mdx
---
title: "fav infer"
order: 6
category: "標準ライブラリ"
description: "CSV・DB・Proto から Favnir 型定義を自動生成する"
---
```

### 注意点

- `fav infer` は CLI コマンドだが、型生成という性質上 stdlib カテゴリに配置
- `--db` のサポート対象 DB（PostgreSQL / SQLite / DuckDB）を明記
- 生成型はそのままコードに貼って使える形式で出力されることを示す

---

## Phase D — `rune-cli.mdx` 更新

### 変更ファイル

- `site/content/docs/rune-cli.mdx` (末尾に追記)

### 追記箇所

ファイル末尾に `## fav deploy` と `## fav build --schema` のセクションを追加。
既存の `rune` コマンド解説セクションには手を加えない。

---

## Phase E — 検証

- 全 MDX ファイルのコードブロックが正しい Favnir 構文であること
- `rune-cli.mdx` の既存部分を壊していないこと
- サイトの order/category が既存ページと整合していること

### 既存 order 確認

| ファイル | category | order |
|---------|---------|-------|
| language/types.mdx | 言語仕様 | 1 |
| language/effects.mdx | 言語仕様 | 2 |
| language/pattern-matching.mdx | 言語仕様 | 3 |
| language/runes.mdx | 言語仕様 | 4 |
| language/testing.mdx | 言語仕様 | 5 |
| **language/pipeline.mdx** | **言語仕様** | **6** |
| **language/schema.mdx** | **言語仕様** | **7** |
| stdlib/io.mdx | 標準ライブラリ | 1 |
| stdlib/list.mdx | 標準ライブラリ | 2 |
| stdlib/map.mdx | 標準ライブラリ | 3 |
| stdlib/result.mdx | 標準ライブラリ | 4 |
| stdlib/option.mdx | 標準ライブラリ | 5 |
| **stdlib/infer.mdx** | **標準ライブラリ** | **6** |

---

## リスク・注意

| リスク | 対策 |
|-------|------|
| コード例の構文エラー | `fav check` で各サンプルを検証してから記載 |
| `abstract seq` の正確な構文 | `compiler.fav` または既存テストで確認 |
| `T.validate` の API が未確定 | v6.5.0 では「preview」と明示、API 確定は v6.6.0 |
| `fav deploy --target ecs` が未実装 | v6.7.0 で実装予定と明記、`--dry-run` のみ記載 |
