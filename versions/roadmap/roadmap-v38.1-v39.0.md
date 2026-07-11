# Roadmap v38.1.0 〜 v39.0.0 — Intelligence & Assistance

Date: 2026-07-06
Status: 骨格確定（v35.0 完了時点）、詳細は v38.0 完了後に確定

---

## 目標

v38.0「Multi-Source ETL Power」で「複数ソースを型安全につなげる」を実現した。
このフェーズは **「AI がパイプラインを補助する」** を実現する。

**前版との関係**:
- v9.6: Llm Rune 基本実装（Claude / OpenAI 統合）✓
- v38.x: Llm Rune 強化（streaming / function calling / embeddings）+ `fav suggest` / `fav generate` 追加

---

## バージョン計画

### v38.1.0 — `fav suggest` ✅

エラーコードから修正案を LLM で生成する。

**想定動作**:
```bash
$ fav check main.fav
main.fav:12:5: E0001 undefined variable `custmer_id`
$ fav suggest E0001 main.fav:12
Suggestion: Did you mean `customer_id`? (typo)
Apply fix? [y/N]
```

**実装**: `fav/src/suggest.rs` + `Some("suggest")` アーム / `ANTHROPIC_API_KEY` 使用

**完了条件（達成済み）**:
- `fav/src/suggest.rs` 新規作成（`cmd_suggest` / `builtin_hint` / `llm_suggest`）
- `main.rs` に `mod suggest;` + `Some("suggest")` ディスパッチアーム追加
- Rust テスト 3 件（meta 2 件 + 機能 1 件）
- 2744 tests passed, 0 failed

---

### v38.2.0 — `fav generate --from sql` ✅

PostgreSQL / MySQL の SELECT / JOIN / WHERE / ORDER BY を Favnir パイプラインに変換。

**完了条件（達成済み）**:
- `fav/src/generate_sql.rs` 新規作成（`sql_to_favnir` / `generate_load` / `generate_filter` / `generate_join`）
- `main.rs` の `Some("generate")` ブロックに `Some("--from")` アーム追加
- Rust テスト 6 件（meta 2 件 + functional 4 件: existence + SELECT + JOIN + WHERE）
- 2750 tests passed, 0 failed

---

### v38.3.0 — `fav generate --from csv` 強化 ✅

v10.8.0 `fav infer` を `schema` + `expect` ブロック出力に強化する。

**完了条件**: Rust テスト 4 件（2754 tests passed, 0 failed）

---

### v38.4.0 — LSP AI 補完（オプション）✅

`fav.toml` に `[lsp.ai] enabled = true` で LLM rerank を有効化。未設定時はフォールバック。

**完了条件**: 設定解析テスト 6 件（meta 2 件 + 機能 4 件）（2760 tests passed, 0 failed）

---

### v38.5.0 — `fav explain --verbose` LLM 拡張 ✅

コンテキスト付き説明と実際のコードに即した修正例を生成する。

**完了条件**: Rust テスト 5 件（meta 2 件 + 機能 3 件）（2765 tests passed, 0 failed）

---

### ✅ v38.6.0 — RAG パイプラインテンプレート

`fav new --template rag-pipeline` テンプレート追加（ingest / embed / retrieve / generate 構成）。

**完了条件**: Rust テスト 5 件（2770 tests passed, 0 failed）

---

### ✅ v38.7.0 — Llm Rune 強化

- `Llm.stream_raw` / `llm.stream` — ストリーミングレスポンス（collect-all 実装）
- `Llm.function_call_raw` / `llm.function_call` — ツール呼び出し
- `Llm.embed_raw` / `llm.embed` — Embeddings 生成（LLM_PROVIDER=openai 専用）

**完了条件**: Rust テスト 4 件（2774 tests passed, 0 failed）

---

### ✅ v38.8.0 — AI 支援 cookbook 3 本

- `site/content/cookbook/sql-to-favnir.mdx`
- `site/content/cookbook/rag-pipeline.mdx`
- `site/content/cookbook/llm-streaming.mdx`

**完了条件**: Rust テスト 3 件（`cargo_toml_version_is_38_8_0` / `changelog_has_v38_8_0` / `ai_cookbook_files_exist`）（2777 tests passed, 0 failed）

---

### ✅ v38.9.0 — v39.0 前調整・安定化

- `site/content/docs/ai-overview.mdx` — v38.x AI 支援機能概要ドキュメント新規作成

**完了条件**: Rust テスト 4 件（`cargo_toml_version_is_38_9_0` / `changelog_has_v38_9_0` / `ai_overview_doc_exists` / `suggest_rs_has_llm_suggest`）（2781 tests passed, 0 failed）

---

### ✅ v39.0.0 — Intelligence & Assistance マイルストーン宣言 ★クリーンアップ

**完了条件**:
- v38.1〜v38.9 の全機能が動作する / テスト数 2785+（v38.9.0 実績ベース）
- GitHub Issues の P1/P2 ラベル付きオープンバグが **0 件**（OSS 公開前のため対象外）
- `★クリーンアップ` 完了

**完了**: Rust テスト 4 件（2785 tests passed, 0 failed）

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v37.1-v38.0.md`
- 次サブスプリント: `versions/roadmap/roadmap-v39.1-v40.0.md`
