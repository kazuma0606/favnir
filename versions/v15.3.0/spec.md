# v15.3.0 Spec — `fav test` DSL（ネイティブテストフレームワーク）

Date: 2026-06-14
Branch: master

---

## テーマ

Favnir ファイル内に `test "..." { ... }` ブロックを書けるようにし、
`fav test <file>` で実行・レポートできるようにする。

現在 Rust `#[test]` で書いているパイプライン検証を **Favnir ネイティブ**に移行できる基盤を作る。
`assert_eq` / `assert_ok` / `assert_err` / `assert_true` の 4 アサーション関数を標準提供し、
`cargo test` スタイルの PASS/FAIL レポートを出力する。

---

## スコープ

### A: 構文 — `test "..." { ... }` ブロック

```fav
test "transform trims whitespace" {
  bind row <- Result.ok({ full_name: "  Alice  ", email: "A@example.com" })
  bind result <- transform_row(row)
  assert_eq(result.full_name, "Alice")
}

test "BigQueryInsert returns ok" {
  bind rows <- insert_rows()
  assert_ok(rows)
}
```

- `test` はトップレベルにのみ記述可能（fn / stage / seq の内部には不可）
- `test` ブロックは `fav run` では完全に無視される（実行対象から除外）
- `test` ブロック内では `fn` / `stage` を除く通常の Favnir 式が使用可能

### B: アサーション関数（4 種類）

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `assert_eq(a, b)` | `(T, T) -> Unit` | `a == b` でなければテスト失敗 |
| `assert_ok(r)` | `(Result<T, E>) -> T` | `Result.ok(v)` でなければ失敗、ok の場合は unwrap して返す |
| `assert_err(r)` | `(Result<T, E>) -> E` | `Result.err(e)` でなければ失敗、err の場合は unwrap して返す |
| `assert_true(b)` | `(Bool) -> Unit` | `b == true` でなければ失敗 |

失敗時は `TestFailure { test_name, message }` を VM エラーとして伝播する。
成功時は次のアサーションに進む。

### C: AST — `TopLevel::TestDef`

```rust
pub enum TopLevel {
    // ... 既存 ...
    TestDef {
        name: String,
        body: Vec<Stmt>,
        span: Span,
    },
}
```

### D: コンパイラ — TestDef の IR 生成

- `TestDef` ごとに独立した IR 関数を生成（例: `__test__transform_trims_whitespace`）
- `assert_eq` / `assert_ok` / `assert_err` / `assert_true` を VM プリミティブ呼び出しとしてコンパイル
- TestDef は通常の `IRProgram.fns` とは別に `IRProgram.tests` スライスに格納
- `fav run` 時は `tests` を無視（コンパイルするが実行しない）

### E: VM — アサーション opcode（または primitive）

既存の `assert` / `assert_eq` / `assert_ne` primitive を拡張:

| Primitive | 動作 |
|---|---|
| `assert_eq(a, b)` | `a != b` なら `panic!("assert_eq failed: left={a}, right={b}")` |
| `assert_ok(r)` | `r == err(e)` なら `panic!("assert_ok failed: got err({e})")` / ok なら v を返す |
| `assert_err(r)` | `r == ok(v)` なら `panic!("assert_err failed: got ok({v})")` / err なら e を返す |
| `assert_true(b)` | `b == false` なら `panic!("assert_true failed")` |

失敗は `panic!` → VM が `TestFailure` として捕捉 → テスト FAIL としてカウント。

### F: `cmd_test` — `fav test <file>`

```
$ fav test src/pipeline.fav

running 3 tests
test transform_trims_whitespace ... ok
test bigquery_insert_returns_ok ... FAILED
test query_returns_3_rows      ... ok

failures:
  bigquery_insert_returns_ok: assert_ok failed: got err("connection refused")

test result: FAILED. 2 passed; 1 failed
```

動作フロー:
1. `.fav` ファイルをパース → `TestDef` リストを収集
2. 各 TestDef を独立してコンパイル → VM で実行
3. 実行中に `panic!` が発生した場合を `TestFailure` として捕捉
4. 全テスト完了後に PASS/FAIL を集計して出力
5. FAIL が 1 件以上あれば exit code 1

### G: CLI — `fav test` サブコマンド

`fav --help` への追加:
```
test <file>     Run test blocks in a .fav file
```

`fav/src/driver.rs` に `cmd_test` 関数追加。

### H: サイトドキュメント

`site/content/docs/language/testing.mdx` — 新規作成:
- `test "..." { }` 構文説明
- 4 アサーション関数のリファレンス
- `fav test <file>` の実行例
- Mock（`Ctx.mock`）との組み合わせ例

### I: テスト（v153000_tests — 5 件）

1. `version_is_15_3_0`
2. `test_def_in_ast`（`ast.rs` に `TestDef` が含まれる）
3. `assert_ok_primitive_exists`（`vm.rs` に `assert_ok` primitive が含まれる）
4. `cmd_test_exists`（`driver.rs` に `cmd_test` 関数が含まれる）
5. `testing_doc_exists`（`site/content/docs/language/testing.mdx` が存在する）

---

## 完了条件

1. `cargo test v153000` → 5/5 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "15.3.0"`
4. `fav test sample.fav` で `test "..." { assert_eq(...) }` が実行・レポートされる
5. `fav run sample.fav` では `test` ブロックが無視される（実行対象外）
6. FAIL したテストの message に失敗箇所が含まれる

---

## 新規 Cargo 依存

なし（既存の VM インフラで実装する）。

---

## 既知の制約・スコープ外

- 並列テスト実行は対象外（v16.x 以降）
- テストカバレッジレポートは対象外
- Mock DSL の新規設計は対象外（`Ctx.mock` の既存機能を流用）
- `test` ブロック内での `stage` / `seq` / `par` は対象外
- ファイル横断テスト（test suite）は対象外（単一ファイルのみ）
- `fav test --watch` は対象外（`fav watch` が既存機能として存在）

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v15.3.0 セクション
- `fav/src/backend/vm.rs` — 既存の `assert` / `assert_eq` / `assert_ne` primitive
- `fav/src/frontend/parser.rs` — TopLevel パース
- `fav/src/ast.rs` — `TopLevel` enum
- `fav/src/middle/compiler.rs` — `compile_program`
