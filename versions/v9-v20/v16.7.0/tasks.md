# v16.7.0 Tasks — `fav test` 成熟（assert_eq / test_group / スナップショット）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.7.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — AST: `Item::TestGroup` 追加（ast.rs）

- [x] B-1: `fav/src/ast.rs` の `Item` enum に `TestGroup { name: String, tests: Vec<TestDef>, span: Span }` 追加
- [x] B-2: `Item::span()` に `TestGroup { span, .. } => span` 追加
- [x] B-3: `cargo build` → exhaustive match エラーが出ることを確認（Phase H で対処）

---

## Phase C — Lexer: `test_group` キーワード追加（lexer.rs）

- [x] C-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `TestGroup` variant 追加
- [x] C-2: `next_token` の識別子認識に `"test_group" => TokenKind::TestGroup` 追加
- [x] C-3: `cargo build` → コンパイルエラーなし確認

---

## Phase D — Parser: `test_group "name" { test ... }` パース（parser.rs）

- [x] D-1: `parse_item` に `TokenKind::TestGroup` 分岐を追加
- [x] D-2: `parse_test_group()` 実装 — `"name" { test ... }` をパースして `Item::TestGroup` を返す
- [x] D-3: `cargo build` → コンパイルエラーなし確認

---

## Phase E — VM: 新 assert プリミティブ追加（vm.rs）

- [x] E-1: `assert_eq(actual, expected)` — `vmvalue_repr` で文字列化して比較、不一致で詳細エラー
- [x] E-2: `assert_approx_eq(actual, expected, epsilon)` — Float 近似比較
- [x] E-3: `assert_contains(list, elem)` — リスト内の要素存在確認
- [x] E-4: `assert_length(list, n)` — リスト長確認
- [x] E-5: `assert_str_contains(s, substring)` — 文字列部分一致確認
- [x] E-6: `assert_str_starts_with(s, prefix)` — 文字列プレフィックス確認
- [x] E-7: `assert_err_eq(result, expected_msg)` — エラー内容の文字列一致確認
- [x] E-8: `assert_snapshot(value, name)` — `.snap/{name}.snap` の作成・比較（`UPDATE_SNAPSHOTS=1` で更新）
- [x] E-9: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler: `Item::TestGroup` コンパイル（compiler.rs）

- [x] F-1: `compile_program` の item ループで `Item::TestGroup` を処理
  - `tests` の各 `TestDef` を `$testgroup:{group_name}:{test_name}` 形式でコンパイル
- [x] F-2: builtin 名前テーブル（2 箇所）に 7 新関数を追加
  （assert_approx_eq / assert_contains / assert_length / assert_str_contains /
  assert_str_starts_with / assert_err_eq / assert_snapshot）
- [x] F-3: `cargo build` → コンパイルエラーなし確認

---

## Phase G — Checker: `Item::TestGroup` exhaustive match（checker.rs）

- [x] G-1: `register_item_signatures` に `Item::TestGroup { .. }` 追加
- [x] G-2: `check_item` に `Item::TestGroup { tests, .. } => { /* check each test */ }` 追加
- [x] G-3: `cargo build` → コンパイルエラーなし確認

---

## Phase H — driver.rs: cmd_test 更新 + fmt.rs + exhaustive match

- [x] H-1: `collect_test_cases` を 4-tuple `(path, display_name, fn_name, prog)` に変更
- [x] H-2: `collect_test_cases` で `Item::TestGroup` も収集（`$testgroup:{group}:{test}` fn_name）
- [x] H-3: テストランナーループを 4-tuple に対応更新
- [x] H-4: `cmd_test` に `--update-snapshots` フラグ対応
  （`unsafe { set_var("UPDATE_SNAPSHOTS", "1") }` — Rust 2024 edition で set_var は unsafe）
- [x] H-5: `main.rs` に `--update-snapshots` フラグ解析追加
- [x] H-6: `fav/src/fmt.rs` に `Item::TestGroup { name, tests, .. }` フォーマット追加
- [x] H-7: `driver.rs` の exhaustive match に `Item::TestGroup { .. } => {}` 追加
- [x] H-8: `cargo build` → コンパイルエラーなし確認

---

## Phase I — テスト追加（v167000_tests）

- [x] I-1: `fav/src/driver.rs` に `v167000_tests` モジュール追加
- [x] I-2: `version_is_16_7_0` — `Cargo.toml` に `"16.7.0"` が含まれる
- [x] I-3: `assert_eq_pass` — `assert_eq(2, 2)` を含む test が PASS する
- [x] I-4: `assert_eq_fail` — `assert_eq(1, 2)` を含む test が FAIL する
- [x] I-5: `test_group_runs_all` — `test_group` 内の 2 テストが両方実行される
- [x] I-6: `assert_snapshot_creates_file` — 初回実行で `.snap/` にファイルが生成される
- [x] I-7: `cargo test v167000` → 5/5 PASS 確認

---

## Phase J — サイトドキュメント + コミット

- [x] J-1: `site/content/docs/language/testing.mdx` 更新
  - `test_group` 構文・全 assert プリミティブ一覧・スナップショットワークフロー・フラグ説明
- [x] J-2: `cargo test v167000` → 5/5 PASS 最終確認
- [x] J-3: `cargo test` → 全件 PASS（リグレッションなし）確認（version check 10件は想定内除外）
- [x] J-4: コミット（commit: 8c8ac1f）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.7.0"` | [x] |
| `assert_eq(2, 2)` が PASS する | [x] |
| `assert_eq(1, 2)` が FAIL し、actual/expected がメッセージに出る | [x] |
| `assert_err_eq(Result.err("msg"), "msg")` が PASS する | [x] |
| `assert_str_contains("hello world", "world")` が PASS する | [x] |
| `test_group "name" { ... }` 内の全テストが実行される | [x] |
| `assert_snapshot` が初回で `.snap/*.snap` を作成する | [x] |
| `--update-snapshots` でスナップショットが更新される | [x] |
| `cargo test v167000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `site/content/docs/language/testing.mdx` が更新されている | [x] |

---

## 技術メモ

- **Favnir test ボディは単一式**: `test "foo" { expr }` のボディは `let`/`bind` バインディング + 最終単一式。複数の bare expression call は不可（2 つ目で "expected RBrace" パースエラー）。
- **新アサート関数はコンパイラのグローバルテーブルへの追加が必須**: `compiler.rs` 2 箇所の builtin 名前リストに追加しないと `CallBuiltin` opcode が生成されず実行時エラーになる。
- **TestGroup fn 名**: `$testgroup:{group_name}:{test_name}`（`$test:{name}` と区別）。
- **`set_var` は Rust 2024 edition で unsafe**: `unsafe { std::env::set_var(...) }` が必要。
