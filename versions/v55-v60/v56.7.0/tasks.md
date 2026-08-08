# Tasks — v56.7.0 — モジュール名前空間（qualified imports）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.7.0 セクションを確認
- [x] ベーステスト数 3240（v56.6.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.6.0` であることを確認（更新前）
- [x] `ImportDecl` に `is_wildcard` フィールドが存在しないことを確認（新規追加対象）
- [x] `parse_import_decl` が `peek2()` を使用していないことを確認（追加対象）
- [x] `process_imports` の `ImportDecl` destructure が `..` を使わず全フィールドを列挙していることを確認
- [x] `v56700_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.6.0"` を期待していることを確認（更新対象）
- [x] W038 が `lint.rs` に存在しないことを確認（新規追加対象）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.7.0` に更新
- [x] T2: `fav/src/ast.rs` — `ImportDecl` に `is_wildcard: bool` フィールド追加
  - [x] `kind: ImportKind` の後、`span: Span` の前に `is_wildcard: bool,` を挿入
  - [x] コメント: `// v56.7.0: import "path" as alias.*`
- [x] T3: `fav/src/frontend/parser.rs` — `as alias.*` パース追加
  - [x] `parse_import_decl` の alias 解析直後に `is_wildcard` 判定を追加
  - [x] `peek() == Dot && peek2() == Some(Star)` の場合のみ `.*` と認識
  - [x] 両トークン（`.` と `*`）を `advance()` で消費
  - [x] `Ok(Item::ImportDecl { ..., is_wildcard, ... })` に追加
- [x] T4: `fav/src/middle/checker.rs` — `process_imports` の destructure 修正
  - [x] `Item::ImportDecl { path, alias, is_rune, is_public, kind: _, span }` に `is_wildcard: _,` を追加
- [x] T5: `fav/src/fmt.rs` — ワイルドカードインポートフォーマット
  - [x] `Item::ImportDecl` アームに `is_wildcard` バインディングを追加
  - [x] alias の `format!` を `match alias { Some(a) if *is_wildcard => ..., ... }` に変更
- [x] T6: `fav/src/lint.rs` — W038 追加
  - [x] `check_w038_wildcard_import_collision` 関数を追加（`check_w035_legacy_import_rune` の直前）
  - [x] `lint_program` 内の W037 呼び出し直後に `check_w038_wildcard_import_collision(program, &mut errors)` を追加
- [x] T7: `fav/src/driver.rs` — `v56700_tests` モジュールを `v56600_tests` の直前に追加
  - [x] `qualified_import_deep_access`: `import "./stages" as stages` が `is_wildcard: false` で解析される
  - [x] `wildcard_import_expands`: `import "./validate" as v.*` が `is_wildcard: true` で解析される
  - [x] `w038_wildcard_import_collision_warning`: ワイルドカードインポート 2 件に W038 が発行される
- [x] T8: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.6.0"` → `"56.7.0"` に更新
  - [x] モジュール名 `v56300_tests` / 関数名 は変更しない（慣例）

---

## テスト・検証

- [x] T9: `cargo build` でコンパイルエラーがないことを確認
- [x] T10: `cargo test` 全通過（**3243 tests passed, 0 failed**）
  - [x] `v56700_tests::qualified_import_deep_access` ok
  - [x] `v56700_tests::wildcard_import_expands` ok
  - [x] `v56700_tests::w038_wildcard_import_collision_warning` ok
  - [x] 既存 3240 件全通過
- [x] T11: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T12: `CHANGELOG.md` に v56.7.0 エントリを追加
- [x] T13: `versions/current.md` を v56.7.0 / 3243 tests に更新
- [x] T14: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.7.0 実績を COMPLETE に更新
  - [x] `3240 + 3 = 3243 tests passed, 0 failed（2026-07-26）` を追記
  - [x] v56.7.0 セクションの「resolver で正式サポート」という記述を訂正済み（spec 作成時に更新）
- [x] T15: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.7.0 実績欄も COMPLETE に更新
  - [x] 「resolver で正式サポート」の文言をパース確認テストで代替と訂正済み

---

## 完了確認

- [x] `qualified_import_deep_access` pass
- [x] `wildcard_import_expands` pass
- [x] `w038_wildcard_import_collision_warning` pass
- [x] **3243 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `ImportDecl` に `is_wildcard: bool` フィールドが追加されている
- [x] `import "./path" as alias.*` が `is_wildcard: true` としてパースされる
- [x] `import "./path" as alias` が `is_wildcard: false` としてパースされる（既存動作）
- [x] `fmt.rs` が `is_wildcard: true` の場合に `as alias.*` を出力する
- [x] `checker.rs:process_imports` の `ImportDecl` destructure に `is_wildcard: _` が追加されている
- [x] `check_w038_wildcard_import_collision` が `lint_program` に統合されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.7.0"` になっている
- [x] `CHANGELOG.md` に v56.7.0 エントリが追加されている
- [x] `versions/current.md` が v56.7.0 / 3243 tests を反映
- [x] T14 / T15 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `peek2()` は `self.tokens.get(self.pos + 1).map(|t| &t.kind)` で `Option<&TokenKind>` を返す既存ヘルパー
- `alias.is_some()` ガードで短絡評価 — alias なしで `.*` チェックをスキップ
- `is_wildcard` フィールド追加で `..` を使わない `checker.rs:process_imports` のみ明示的修正が必要だった
- W038 は `check_w035_legacy_import_rune` の直前（ファイル末尾付近）に配置
- wildcard 名前注入（スコープ展開）は resolver 必要のため v57.0 以降に委譲
