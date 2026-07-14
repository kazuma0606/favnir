# v43.13.0 Spec — Language Expressiveness cookbook + 安定化

## 概要

v43.1〜v43.12 で実装した型推論 6 カテゴリ（戻り値型推論・ジェネリック推論・ラムダ推論・パイプライン型伝播・構造体リテラル推論・双方向推論）と opaque type・冗長注釈 lint の成果を **ドキュメントとして整備** するコードフリーズリリース。

新規機能の追加は一切行わない。

---

## 成果物

### 1. `site/content/cookbook/type-inference-guide.mdx`

型推論の使い方を実践的なユースケースで解説する cookbook 記事。

内容:
- 戻り値型省略（W031 との関係）
- ジェネリック型引数省略（W032 との関係）
- ラムダ引数型推論（パイプライン上流からの伝播）
- opaque type によるカプセル化

### 2. `site/content/docs/language/type-inference.mdx`

言語リファレンスの `docs/language/` セクションに追加する型推論の網羅的解説。

内容:
- 型推論の仕組みと 6 カテゴリの概要
- 各カテゴリのコード例
- 制限事項と将来計画

### 3. `site/content/docs/language-expressiveness.mdx`

「Language Expressiveness」スプリント（v43.1〜v43.13）の全成果を宣言するドキュメント。v44.0.0 マイルストーン宣言の下準備。

---

## テスト

meta テスト 2 件（`include_str!` によるファイル存在確認）:

1. `type_inference_guide_mdx_exists` — `site/content/cookbook/type-inference-guide.mdx` の存在確認
2. `language_expressiveness_doc_exists` — `site/content/docs/language-expressiveness.mdx` の存在確認

`site/content/docs/language/type-inference.mdx` は上記 2 件とは別に CHANGELOG で言及。

---

## スコープ外

- 新規 Rust コード追加（フィーチャーフリーズ）
- `fav check --explain` / opaque type の checker.fav 統合（将来版）
- W033 の実装（将来版）

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2937 passed; 0 failed**
- `v431300_tests` 2 件 pass
  1. `type_inference_guide_mdx_exists`
  2. `language_expressiveness_doc_exists`
