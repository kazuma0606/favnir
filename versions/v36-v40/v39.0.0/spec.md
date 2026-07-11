# v39.0.0 spec — Intelligence & Assistance マイルストーン宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.0.0 |
| テーマ | Intelligence & Assistance マイルストーン宣言・★クリーンアップ |
| 前提 | v38.9.0 COMPLETE — v39.0 前調整・安定化完了 |
| 完了条件 | `v39000_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新 |

## 背景と目的

v38.1〜v38.9 のスプリントで以下を達成した。本バージョンはこれらを統合して Intelligence & Assistance マイルストーンを正式宣言し、v39 世代に移行する。

### 達成内容

| バージョン | 内容 |
|---|---|
| v38.1.0 | `fav suggest` — エラーコードから修正案を LLM で生成（`suggest.rs` 新規作成）|
| v38.2.0 | `fav generate --from sql` — PostgreSQL / MySQL SELECT / JOIN を Favnir パイプラインに変換 |
| v38.3.0 | `fav generate --from csv` 強化 — `schema` + `expect` ブロック出力 |
| v38.4.0 | LSP AI 補完 — `fav.toml` `[lsp.ai] enabled = true` で LLM rerank 有効化 |
| v38.5.0 | `fav explain --verbose` LLM 拡張 — コンテキスト付き説明・実コードに即した修正例 |
| v38.6.0 | RAG パイプラインテンプレート — `fav new --template rag-pipeline`（ingest / embed / retrieve / generate）|
| v38.7.0 | Llm Rune 強化 — `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` |
| v38.8.0 | AI 支援 cookbook 3 本（sql-to-favnir / rag-pipeline / llm-streaming）|
| v38.9.0 | v39.0 前調整・安定化（`site/content/docs/ai-overview.mdx` 新規作成）|

## ロードマップとの差異

ロードマップの完了条件「テスト数 ≥ 2785」は v38.9.0 実績 2781 件に +4 件（本バージョン追加分）で 2785 件となり達成見込み。
ロードマップ記載の「GitHub Issues P1/P2 ラベル付きオープンバグ 0 件」条件は Favnir が OSS 公開前のため GitHub Issues が存在しない。本バージョンでは対象外とする（v36.0 / v37.0 / v38.0 と同規約）。

## 実装スコープ

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v39.0 Intelligence & Assistance 宣言セクション追加（先頭に挿入）|
| `README.md` | v39.0 マイルストーン宣言行を追加 |
| `CHANGELOG.md` | `## [v39.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v38900_tests::cargo_toml_version_is_38_9_0` スタブ化 |
| `fav/src/driver.rs` | `v39000_tests` モジュール（4 件）追加 |
| `fav/Cargo.toml` | バージョン `38.9.0` → `39.0.0` |
| ビルドキャッシュ | `cargo clean`（★クリーンアップ） |
| `versions/v36-v40/v39.0.0/tasks.md` | COMPLETE 更新 |

## v39000_tests の設計

| テスト名 | 検証内容 | `include_str!` パス |
|---|---|---|
| `cargo_toml_version_is_39_0_0` | Cargo.toml に `"39.0.0"` が含まれる | `"../Cargo.toml"` |
| `changelog_has_v39_0_0` | `CHANGELOG.md` に `[v39.0.0]` が含まれる | `"../../CHANGELOG.md"` |
| `milestone_has_intelligence_and_assistance` | `MILESTONE.md` に `"Intelligence & Assistance"` が含まれる | `"../../MILESTONE.md"` |
| `readme_mentions_intelligence_assistance` | `README.md` に `"Intelligence & Assistance"` が含まれる | `"../../README.md"` |

imports 不要（`include_str!` のみ使用）。

## 宣言文

```
fav suggest でエラーから修正案を AI が提案し、
fav generate --from sql でパイプラインを自動生成し、
fav explain --verbose でコンテキスト付き解説を受け取れる。
Llm Rune はストリーミング・function calling・Embeddings に対応し、
RAG パイプラインを fav new --template rag-pipeline で即座に生成できる。

これが Favnir v39.0 — Intelligence & Assistance の姿である。
```

## MILESTONE.md への追加内容

```
## v39.0.0 — Intelligence & Assistance（2026-07-10）

> 「`fav suggest` でエラーから修正案を AI が提案し、
>  `fav generate --from sql` でパイプラインを自動生成し、
>  `fav explain --verbose` でコンテキスト付き解説を受け取れる。
>  Llm Rune はストリーミング・function calling・Embeddings に対応し、
>  RAG パイプラインを `fav new --template rag-pipeline` で即座に生成できる。
>
>  これが Favnir v39.0 — Intelligence & Assistance の姿である。」

v39.0.0 をもって、Favnir の **Intelligence & Assistance** を正式に宣言する。

### 達成コンポーネント（v38.1〜v38.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| fav suggest | v38.1 | エラーコードから修正案を LLM で生成 |
| fav generate --from sql | v38.2 | SQL → Favnir パイプライン自動変換 |
| fav generate --from csv 強化 | v38.3 | schema + expect ブロック出力 |
| LSP AI 補完 | v38.4 | [lsp.ai] enabled = true で LLM rerank |
| fav explain --verbose | v38.5 | コンテキスト付き LLM 解説・修正例 |
| RAG テンプレート | v38.6 | fav new --template rag-pipeline |
| Llm Rune 強化 | v38.7 | stream / function_call / embed 対応 |
| AI 支援 cookbook | v38.8 | sql-to-favnir / rag-pipeline / llm-streaming |
| 安定化 | v38.9 | ai-overview.mdx ドキュメント整備 |

**宣言日**: 2026-07-10

---
```

挿入位置: `# Favnir Milestones` ヘッダの直後、`## v38.0.0` セクションの直前。

## README.md への追加行

```markdown
**v39.0（2026-07-10）で、[Intelligence & Assistance](./MILESTONE.md) マイルストーンを宣言しました。**
```

挿入位置: `**v38.0（2026-07-10）で、[Multi-Source ETL Power]...` 行の直後。

## ★クリーンアップ

v39.0.0 は x.0.0 マイルストーンのため `cargo clean` が必須（v31〜v38 の x.0.0 と同規約）。

**注意**: `cargo clean` により `fav/tmp/hello.fav` が消える可能性がある（v30.0.0 での知見）。
クリーンアップ前後で `fav/tmp/hello.fav` の存在を確認し、消失した場合は以下の内容で復元すること:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

T2 の順序: `fav/tmp/hello.fav` 存在確認 → `cargo clean` → `hello.fav` 存在確認 → `cargo test`

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.9.0 | 2781 |
| v39.0.0 追加分（v39000_tests 4 件 + v38900_tests スタブ化 0 件変化） | +4 |
| v39.0.0 期待値 | 2785 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Intelligence & Assistance"` が含まれる | `milestone_has_intelligence_and_assistance` テスト |
| 2 | `README.md` に `"Intelligence & Assistance"` が含まれる | `readme_mentions_intelligence_assistance` テスト |
| 3 | `CHANGELOG.md` に `[v39.0.0]` が含まれる | `changelog_has_v39_0_0` テスト |
| 4 | `Cargo.toml` バージョンが `39.0.0` | `cargo_toml_version_is_39_0_0` テスト |
| 5 | `cargo clean` 実施済み | T2 実行記録 |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2785） | `cargo test` 実行結果（2781 + 4 = 2785） |
