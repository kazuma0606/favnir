# v16.7.0 Tasks — `fav test` 成熟（assert_eq / test_group / スナップショット）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"16.7.0"` に変更
- [ ] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — AST: `Item::TestGroup` 追加（ast.rs）

- [ ] B-1: `fav/src/ast.rs` の `Item` enum に `TestGroup { name: String, tests: Vec<TestDef>, span: Span }` 追加
- [ ] B-2: `Item::span()` に `TestGroup { span, .. } => span` 追加
- [ ] B-3: `cargo build` → exhaustive match エラーが出ることを確認（Phase H で対処）

---

## Phase C — Lexer: `test_group` キーワード追加（lexer.rs）

- [ ] C-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `TestGroup` variant 追加
- [ ] C-2: `next_token` の識別子認識に `"test_group" => TokenKind::TestGroup` 追加
- [ ] C-3: `cargo build` → コンパイルエラーなし確認

---

## Phase D — Parser: `test_group "name" { test ... }` パース（parser.rs）

- [ ] D-1: `parse_item` に `TokenKind::TestGroup` 分岐を追加
- [ ] D-2: `parse_test_group_body()` 実装 — `"name" { test ... }` をパースして `Vec<TestDef>` を返す
- [ ] D-3: `Item::TestGroup { name, tests, span }` を返す
- [ ] D-4: `cargo build` → コンパイルエラーなし確認

---

## Phase E — VM: 新 assert プリミティブ追加（vm.rs）

- [ ] E-1: `assert_eq(actual, expected)` — `vmvalue_repr` で文字列化して比較、不一致で詳細エラー
- [ ] E-2: `assert_approx_eq(actual, expected, epsilon)` — Float 近似比較
- [ ] E-3: `assert_contains(list, elem)` — リスト内の要素存在確認
- [ ] E-4: `assert_length(list, n)` — リスト長確認
- [ ] E-5: `assert_str_contains(s, substring)` — 文字列部分一致確認
- [ ] E-6: `assert_str_starts_with(s, prefix)` — 文字列プレフィックス確認
- [ ] E-7: `assert_err_eq(result, expected_msg)` — エラー内容の文字列一致確認
- [ ] E-8: `assert_snapshot(value, name)` — `.snap/{name}.snap` の作成・比較（`UPDATE_SNAPSHOTS=1` で更新）
- [ ] E-9: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler: `Item::TestGroup` コンパイル（compiler.rs）

- [ ] F-1: `compile_program` の item ループで `Item::TestGroup` を処理
  - `tests` の各 `TestDef` を `Item::TestDef` と同様にコンパイル
  - テスト関数名を `"__testgroup_{group}_{test}"` 形式で登録（cmd_test での分類に使用）
- [ ] F-2: `Item::TestGroup { .. } => {}` の exhaustive match 追加（compiler.rs 内の全 match）
- [ ] F-3: `cargo build` → コンパイルエラーなし確認

---

## Phase G — Checker: `Item::TestGroup` exhaustive match（checker.rs）

- [ ] G-1: `register_item_signatures` に `Item::TestGroup { tests, .. }` 処理追加（各 TestDef の登録）
- [ ] G-2: `check_item` に `Item::TestGroup { tests, .. } => { /* check each test */ }` 追加
- [ ] G-3: `cargo build` → コンパイルエラーなし確認

---

## Phase H — driver.rs: cmd_test 更新 + fmt.rs + exhaustive match

- [ ] H-1: `cmd_test` に `--update-snapshots` フラグ対応（`UPDATE_SNAPSHOTS=1` を env に設定）
- [ ] H-2: `cmd_test` でグループ名プレフィックス `"__testgroup_"` を検出してグループ別集計
- [ ] H-3: グループ別サマリー出力 (`running N tests in "group_name"` + 個別 PASS/FAIL + サマリー)
- [ ] H-4: `fav/src/fmt.rs` に `Item::TestGroup { name, tests, .. }` フォーマット追加
- [ ] H-5: `driver.rs` の exhaustive match に `Item::TestGroup { .. } => {}` 追加（全 match 箇所）
- [ ] H-6: `cargo build` → コンパイルエラーなし確認

---

## Phase I — テスト追加（v167000_tests）

- [ ] I-1: `fav/src/driver.rs` に `v167000_tests` モジュール追加
- [ ] I-2: `version_is_16_7_0` — `Cargo.toml` に `"16.7.0"` が含まれる
- [ ] I-3: `assert_eq_pass` — `assert_eq(2, 2)` を含む test が PASS する
- [ ] I-4: `assert_eq_fail` — `assert_eq(1, 2)` を含む test が FAIL し、適切なメッセージが出る
- [ ] I-5: `test_group_runs_all` — `test_group` 内の 2 テストが両方実行される
- [ ] I-6: `assert_snapshot_creates_file` — 初回実行で `.snap/` にファイルが生成される
- [ ] I-7: `cargo test v167000` → 5/5 PASS 確認

---

## Phase J — サイトドキュメント + コミット

- [ ] J-1: `site/content/docs/language/testing.mdx` 新規作成
  - `test` / `test_group` 構文の説明と例
  - 全 assert プリミティブの一覧（assert_ok/err/true/eq/approx_eq/contains 等）
  - スナップショットテストのワークフロー（初回 → 比較 → 更新）
  - `fav test` フラグ説明（`--filter` / `--fail-fast` / `--update-snapshots`）
- [ ] J-2: `cargo test v167000` → 5/5 PASS 最終確認
- [ ] J-3: `cargo test` → 全件 PASS（リグレッションなし）確認
- [ ] J-4: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.7.0"` | [ ] |
| `assert_eq(2, 2)` が PASS する | [ ] |
| `assert_eq(1, 2)` が FAIL し、actual/expected がメッセージに出る | [ ] |
| `assert_err_eq(Result.err("msg"), "msg")` が PASS する | [ ] |
| `assert_str_contains("hello world", "world")` が PASS する | [ ] |
| `test_group "name" { ... }` 内の全テストが実行される | [ ] |
| グループ別サマリーが出力される | [ ] |
| `assert_snapshot` が初回で `.snap/*.snap` を作成する | [ ] |
| `--update-snapshots` でスナップショットが更新される | [ ] |
| `cargo test v167000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/language/testing.mdx` が存在する | [ ] |
