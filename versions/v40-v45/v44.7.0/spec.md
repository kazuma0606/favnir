# v44.7.0 Spec — ドキュメントサイト Precision & Flow 概要ページ

## 概要

v44.1〜v44.6 で実装した Precision & Flow 機能群を統合解説する **ドキュメントサイトページ** `site/content/docs/precision-and-flow.mdx` を追加する。

Refinement type、CEP、Opaque type、型注釈 lineage、Back-pressure、E2E デモの 6 機能を 1 ページで網羅し、ユーザーが Precision & Flow の全体像を把握できるようにする。

---

## 成果物

### `site/content/docs/precision-and-flow.mdx`

以下のセクションを含む MDX ドキュメント:

- タイトル: `Precision & Flow`
- 概要説明（v44.x スプリントの目標）
- 各機能の説明と Favnir コードスニペット:
  - Refinement type（`type HighValue = Float where |v| v > 1000.0`）
  - CEP パターン（`cep pattern HighValueDetected { HighValue within 300 }`）
  - Opaque type（`opaque type OrderId = String`）
  - 型注釈 lineage（`bind valid: List<Float> <- events`）
  - Back-pressure（`#[max_inflight(50)] stage PolicyGate`）
  - E2E デモへのリンク（`infra/e2e-demo/precision-flow/`）

---

## テスト

`v44700_tests` 2 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_7_0` | `Cargo.toml` に `"44.7.0"` が含まれる |
| `precision_and_flow_doc_exists` | `include_str!("../../site/content/docs/precision-and-flow.mdx")` で MDX を読み込み、`"Precision & Flow"` が含まれることを確認 |

テストパターンは `v41900_tests::type_precision_doc_exists`（`v41900_tests` モジュール内）と同形式（`include_str!` + `contains` アサート）。

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2958 passed; 0 failed**（2956 + 2）
- `v44700_tests` 2 件 pass
- `site/content/docs/precision-and-flow.mdx` が存在し `"Precision & Flow"` を含む

---

## 注意事項

- テストは `include_str!("../../site/content/docs/precision-and-flow.mdx")` — コンパイル時にファイル存在確認（パス: `fav/src/` から 2 段上の `site/content/docs/`）
- `v44600_tests::cargo_toml_version_is_44_6_0` をスタブ化すること
  - スタブ化の方法: `assert!` 行のみ削除し `// Stubbed: version bumped to 44.7.0 in v44.7.0.` に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）
- ロードマップ推定（2946）は旧見積もり。実績 2956 を基準とする
