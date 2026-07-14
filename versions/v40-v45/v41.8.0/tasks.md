# v41.8.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2868（前バージョン 2867 + 1）
**実績テスト数**: 2868

---

## T0 — 事前確認

- [x] `cargo test` が 2867 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.7.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.8.0 を確認
- [x] `v41700_tests::cargo_toml_version_is_41_7_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 44650
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `site/content/cookbook/refinement-types.mdx` が存在しないことを確認
- [x] `site/content/docs/language/refinement-types.mdx` が存在し行数を確認: 92 行

---

## T1 — `site/content/cookbook/refinement-types.mdx` 新規作成

- [x] frontmatter（title / description）を追加
- [x] 「問題」セクション（ドメイン制約の散在問題）を追加
- [x] 「Refinement Type Alias」セクション（`type Age = Int where |v| ...`）を追加
- [x] 「実用例: ユーザープロフィール」セクションを追加
- [x] 「W030: 冗長ガードを書かない」セクションを追加
- [x] 「Newtype との組み合わせ」セクションを追加
- [x] 「まとめ」表を追加

---

## T2 — `site/content/docs/language/refinement-types.mdx` 更新

- [x] ファイル末尾に「Type Alias Refinement（v41.1.0+）」セクションを追加（パラメータ refinement との違い表を含む）
- [x] ファイル末尾に「W030: 冗長ガード lint（v41.7.0+）」セクションを追加

---

## T3 — driver.rs テストモジュール更新

- [x] `v41700_tests::cargo_toml_version_is_41_7_0` をスタブ化（"Stubbed: version bumped to 41.8.0"）
- [x] `v41800_tests` モジュール（1 テスト）を追加:
  - `cargo_toml_version_is_41_8_0`（NOTE コメント付き）

---

## T4 — Cargo.toml バージョン bump

- [x] `version = "41.7.0"` → `"41.8.0"`

---

## T5 — CHANGELOG.md 更新

- [x] `[v41.8.0]` エントリを `[v41.7.0]` の直前に追加

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2868 を確認（2867 + 1 件）
- [x] `v41800_tests` 1 件 pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.8.0（最新安定版）・v41.9.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.8.0 を完了済みにマーク
- [x] `versions/v40-v45/v41.8.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要

---

## コードレビュー指摘と対応

### [MED] cookbook/docs のコードブロック言語識別子不統一
- **指摘**: cookbook は `favnir`、docs は `fav` で異なる
- **対応**: サイト全体の確立されたコンベンション（cookbook: `favnir`、docs: `fav`）であることを確認。変更不要 ✅

### [LOW] W030 が `&&` 複合 invariant に効かない点が未記載
- **指摘**: `type Age = Int where |v| v >= 0 && v <= 150` の両辺とも W030 対象と誤解される恐れ
- **対応**: cookbook W030 セクションに「単一比較演算 invariant のみ対象（複合は将来対応）」の Note を追加 ✅

### [LOW] docs 追加セクションが日本語（既存は英語）
- **指摘**: 既存ページは英語だが追加セクションは日本語で言語混在
- **対応**: 追加セクションを英語に統一 ✅

### [LOW] 「未対応」表現の曖昧さ
- **指摘**: 「同一型に組み合わせる」が何を指すか不明
- **対応**: cookbook の表現を「同一型に組み合わせることは未対応（将来バージョンで対応予定）」のまま維持。具体例の追加は冗長になるため現状維持

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（完了条件の自動/手動分離・記法不一致の明記・Newtype 断言の緩和・T6 テスト数表記修正）
- [x] code-reviewer 指摘対応済み（W030 限定説明の Note 追加・docs セクションを英語に統一）
