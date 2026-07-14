# v44.9.0 Spec — v45.0 前調整・安定化

## 概要

v45.0 前のコードフリーズ版。新規機能追加なし。`site/content/docs/precision-and-flow-overview.mdx` を新規作成し、v44.x スプリント全体の俯瞰ドキュメントを整備する。

`precision-and-flow.mdx`（v44.7.0 で作成、機能別詳細説明）とは異なり、`precision-and-flow-overview.mdx` は v44.x スプリントのゴール・達成事項・v45.0 への道筋を 1 ページで俯瞰するエグゼクティブサマリーページとする。

---

## 成果物

### `site/content/docs/precision-and-flow-overview.mdx`

以下のセクションを含む MDX ドキュメント:

- タイトル: `Precision & Flow Overview`（`"Precision & Flow"` を含む — テストアサート条件）
- v44.x スプリントのゴール（型安全なリアルタイムパイプラインを最小注釈で記述）
- 完了バージョン一覧（v44.1〜v44.8）と各バージョンの成果
- v45.0 Precision & Flow 宣言への道筋

---

## テスト

`v44900_tests` 2 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_9_0` | `Cargo.toml` に `"44.9.0"` が含まれる |
| `precision_and_flow_overview_doc_exists` | `include_str!("../../site/content/docs/precision-and-flow-overview.mdx")` で MDX を読み込み、`"Precision & Flow"` が含まれることを確認 |

テストパターンは `v44700_tests::precision_and_flow_doc_exists` と同形式（`include_str!` + `contains` アサート）。

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2962 passed; 0 failed**（2960 + 2）
- `v44900_tests` 2 件 pass
- `site/content/docs/precision-and-flow-overview.mdx` が存在し `"Precision & Flow"` を含む

---

## 注意事項

- `precision-and-flow-overview.mdx` はロードマップ記載の「更新」対象だが、ファイルが未存在のため新規作成とする
- `precision-and-flow.mdx`（v44.7.0）との違いを明確にすること（詳細説明 vs. 俯瞰サマリー）
- コードフリーズ: 新規 Rust 機能・AST 変更・新規ヘルパー関数は追加しない
- `v44800_tests::cargo_toml_version_is_44_8_0` をスタブ化すること
  - スタブ化の方法: `assert!` 行のみ削除し `// Stubbed: version bumped to 44.9.0 in v44.9.0.` に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）
- ロードマップ推定（2950）は旧見積もり。実績 2960 を基準とする
