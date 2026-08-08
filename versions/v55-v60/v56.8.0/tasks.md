# Tasks — v56.8.0 — ドキュメントサイト Language Power 2.0 記事

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.8.0 セクションを確認
- [x] ベーステスト数 3243（v56.7.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.7.0` であることを確認（更新前）
- [x] `site/content/docs/language/bounded-generics.mdx` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/language/row-polymorphism.mdx` が存在することを確認（更新対象）
- [x] `site/content/docs/language/effect-inference.mdx` が存在することを確認（更新対象）
- [x] `v56800_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.7.0"` を期待していることを確認（更新対象）
- [x] `site/content/docs/language/row-polymorphism.mdx` が `"fn get_name<r>"` を含まないことを確認（テスト偽陽性防止）
- [x] `site/content/docs/language/effect-inference.mdx` が `"inlay"` を含まないことを確認（テスト偽陽性防止）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.8.0` に更新
- [x] T2: `site/content/docs/language/bounded-generics.mdx` を新規作成
  - [x] 概要セクション（v56.1.0 / v56.2.0 の正式化）
  - [x] `where T: Interface` 構文（`T with Interface` との等価性）
  - [x] 複数 constraint（`T with Ord with Serialize`）
  - [x] `Serializable` カスタム interface 例
  - [x] stdlib 使用例（`List.sort`、`List.max`）
  - [x] E0422 エラー（constraint 違反）
  - [x] E0423 エラー（coherence 違反 / 重複 `impl`）
  - [x] `generics.mdx` への参照リンク
- [x] T3: `site/content/docs/language/row-polymorphism.mdx` を更新
  - [x] 「行変数の明示（v56.3.0）」セクションを末尾に追記
  - [x] `fn get_name<r>(record: { name: String | r }) -> String` 例を含める
  - [x] 「LSP ホバー表示」サブセクションを追記
- [x] T4: `site/content/docs/language/effect-inference.mdx` を更新
  - [x] 「エディタ統合（v56.4.0）」セクションを末尾に追記
  - [x] inlay hints 例（`/*!Kafka !Snowflake*/`）を含める
  - [x] `fav check --show-types` の出力例を含める
- [x] T5: `fav/src/driver.rs` — `v56800_tests` モジュールを `v56700_tests` の直前に追加
  - [x] `docs_bounded_generics_page_exists`: `bounded-generics.mdx` が `Serializable` と `E0422` を含む
  - [x] `docs_row_poly_page_exists`: `row-polymorphism.mdx` が `fn get_name<r>` を含む
  - [x] `docs_effect_inference_updated`: `effect-inference.mdx` が `inlay` を含む
- [x] T6: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.7.0"` → `"56.8.0"` に更新
  - [x] failure メッセージも `"should be 56.8.0"` に更新
  - [x] モジュール名 `v56300_tests` / 関数名は変更しない（慣例）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3246 tests passed, 0 failed**）
  - [x] `v56800_tests::docs_bounded_generics_page_exists` ok
  - [x] `v56800_tests::docs_row_poly_page_exists` ok
  - [x] `v56800_tests::docs_effect_inference_updated` ok
  - [x] 既存 3243 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v56.8.0 エントリを追加
- [x] T11: `versions/current.md` を v56.8.0 / 3246 tests に更新
- [x] T12: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.8.0 実績を COMPLETE に更新
  - [x] `3243 + 3 = 3246 tests passed, 0 failed（2026-07-26）` を追記
- [x] T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.8.0 実績欄も COMPLETE に更新

---

## 完了確認

- [x] `docs_bounded_generics_page_exists` pass
- [x] `docs_row_poly_page_exists` pass
- [x] `docs_effect_inference_updated` pass
- [x] **3246 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `bounded-generics.mdx` が新規作成されている
- [x] `bounded-generics.mdx` に `Serializable`・`E0422`・`E0423` が含まれている
- [x] `row-polymorphism.mdx` に `{ field: Type | r }` 行変数記法が追記されている
- [x] `effect-inference.mdx` に `/*!Kafka !Snowflake*/` inlay hints セクションが追記されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.8.0"` になっている
- [x] `CHANGELOG.md` に v56.8.0 エントリが追加されている
- [x] `versions/current.md` が v56.8.0 / 3246 tests を反映
- [x] T12 / T13 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `include_str!` のパス: `driver.rs`（`fav/src/driver.rs`）から `../../` で 2 段上がると
  プロジェクトルート（`fav/` の親）に到達する。そこから `site/content/docs/language/xxx.mdx`
- `docs_row_poly_page_exists` は `"fn get_name<r>"` で検索 — v56.3.0 追記前の既存ファイルには存在しない
- `docs_bounded_generics_page_exists` は `"Serializable"` と `"E0422"` の 2 assert で内容充実度を確認
- `docs_effect_inference_updated` は `"inlay"` で検索 — v56.4.0 追記前の既存ファイルには存在しない
- `bounded-generics.mdx` の `E0423`（coherence 違反）は記述するが、テストアサーションは `E0422` のみ
