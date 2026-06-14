# v16.7.0 Plan — `fav test` 成熟（assert_eq / test_group / スナップショット）

Date: 2026-06-14

---

## 実装フェーズ

### Phase A — Cargo バージョン更新

`fav/Cargo.toml` の `version` を `"16.7.0"` に変更。

---

### Phase B — AST: `Item::TestGroup` 追加（ast.rs）

`fav/src/ast.rs` に以下を追加：

```rust
Item::TestGroup {
    name: String,    // "transform pipeline"
    tests: Vec<TestDef>,
    span: Span,
}
```

`Item::span()` に `Item::TestGroup { span, .. } => span` を追加。

既存の `Item::TestDef` は単独テスト `test "..." { ... }` 用として維持。

---

### Phase C — Lexer: `test_group` キーワード追加（lexer.rs）

`TokenKind` enum に `TestGroup` variant を追加。
`next_token` の識別子認識に `"test_group" => TokenKind::TestGroup` を追加。

※ `assert_eq` 等の新 assert 関数はキーワードではなく通常の識別子として扱う
（既存の `assert_ok/err/true` と同じ方式）。パーサー・VM で対処。

---

### Phase D — Parser: `test_group "name" { ... }` パース（parser.rs）

`parse_item` に `TokenKind::TestGroup` 分岐を追加：

```rust
TokenKind::TestGroup => {
    let span = self.peek_span().clone();
    self.advance(); // consume 'test_group'
    let name = self.expect_string_literal()?;
    self.expect(&TokenKind::LBrace)?;
    let mut tests = Vec::new();
    while self.peek() != &TokenKind::RBrace {
        self.expect(&TokenKind::Test)?;
        tests.push(self.parse_test_def_body()?);
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(Item::TestGroup { name, tests, span })
}
```

※ `parse_test_def_body()` として既存の `TestDef` ボディパースを切り出すか、
内部で `parse_test_def()` を流用する。

---

### Phase E — VM: 新 assert プリミティブ追加（vm.rs）

`vm_call_builtin` の match アームに以下を追加：

```rust
"assert_eq" => {
    // args: [actual, expected]
    // 成功: Ok(Value::Unit)
    // 失敗: Err("assert_eq failed:\n  actual:   {a}\n  expected: {e}")
}
"assert_approx_eq" => { /* args: [actual, expected, epsilon] */ }
"assert_contains"  => { /* args: [list, elem] */ }
"assert_length"    => { /* args: [list, n] */ }
"assert_str_contains"   => { /* args: [s, substring] */ }
"assert_str_starts_with" => { /* args: [s, prefix] */ }
"assert_err_eq"    => { /* args: [result, expected_msg] */ }
"assert_snapshot"  => {
    // args: [value, name]
    // .snap/{name}.snap が無ければ作成して Ok(Unit)
    // あれば比較して一致 → Ok(Unit), 不一致 → Err(diff)
    // UPDATE_SNAPSHOTS env var が "1" なら常に上書き
}
```

`assert_snapshot` は `.snap/` ディレクトリを自動作成。
スナップショット値は `vmvalue_repr(&value)` の文字列で保存。
`--update-snapshots` は `UPDATE_SNAPSHOTS=1` 環境変数で VM に伝達する。

---

### Phase F — Compiler: `Item::TestGroup` コンパイル（compiler.rs）

`compile_program` の `for item in &program.items` ループで `Item::TestGroup` を処理：

```rust
Item::TestGroup { tests, .. } => {
    for test in tests {
        // Item::TestDef と同様にコンパイル（グループ名はメタデータ）
        // テスト関数名: "test_group:{group_name}:{test_name}" として登録
    }
}
```

※ グループ名は関数名プレフィックスとして埋め込み、`cmd_test` で抽出する。

---

### Phase G — Checker: `Item::TestGroup` + exhaustive match（checker.rs）

- `register_item_signatures` に `Item::TestGroup { tests, .. } => { ... }` 追加
  （各 `TestDef` の型チェックを `Item::TestDef` と同様に処理）
- `check_item` に `Item::TestGroup { .. } => {}` 追加（ボディは個別テストで処理済み）

---

### Phase H — driver.rs: cmd_test 更新 + fmt.rs + exhaustive match

**driver.rs:**

- `cmd_test` の `--update-snapshots` フラグ対応:
  - フラグ受取 → `std::env::set_var("UPDATE_SNAPSHOTS", "1")`
  - VM の `assert_snapshot` が env var を参照
- `cmd_test` の TestGroup 処理:
  - グループ名プレフィックス `"test_group:{name}:"` を検出
  - グループ別にテスト結果を集計してサマリー出力
  - 出力形式:
    ```
    running 3 tests in "transform pipeline"
      test trims whitespace       ... PASS
      test handles empty string   ... PASS
    test result: PASS. 3 passed; 0 failed.
    ```
- `Item::TestGroup { .. }` の exhaustive match 対応（`collect_test_defs` 等）

**fmt.rs:**
```rust
Item::TestGroup { name, tests, .. } => {
    format!("test_group \"{}\" {{ {} tests }}", name, tests.len())
}
```

---

### Phase I — テスト追加（v167000_tests）

`fav/src/driver.rs` に `v167000_tests` モジュールを追加：

```rust
#[cfg(test)]
mod v167000_tests {
    use super::*;
    use std::fs;

    #[test]
    fn version_is_16_7_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"16.7.0\""), "...");
    }

    #[test]
    fn assert_eq_pass() {
        // assert_eq(2, 2) → PASS
        let src = r#"
test "eq pass" {
  assert_eq(2, 2)
}
"#;
        // fav test の結果が PASS
    }

    #[test]
    fn assert_eq_fail() {
        // assert_eq(1, 2) → FAIL with message
    }

    #[test]
    fn test_group_runs_all() {
        // test_group "g" { test "a" { assert_true(true) } test "b" { assert_true(true) } }
        // → 2 tests run
    }

    #[test]
    fn assert_snapshot_creates_file() {
        // 初回実行で .snap/test_snap.snap が作成される
    }
}
```

---

### Phase J — サイトドキュメント + コミット

- `site/content/docs/language/testing.mdx` を新規作成（または更新）
  - `test` / `test_group` 構文
  - 全 assert プリミティブの一覧と使用例
  - スナップショットテストのワークフロー
  - `fav test --filter` / `--fail-fast` / `--update-snapshots` フラグ説明
- `cargo test v167000` → 5/5 PASS 確認
- `cargo test` → 全件 PASS（リグレッションなし）確認
- コミット

---

## リスク・考慮点

| リスク | 対策 |
|---|---|
| `test_group` キーワードが既存のテスト識別子と衝突 | `is_rune_use_pattern()` と同様、既存パースパスと分離 |
| `assert_snapshot` のファイル I/O が CI でテストを汚す | `.snap/` を `.gitignore` に追記 + テスト後にクリーンアップ |
| `Item::TestGroup` の exhaustive match 漏れ | `cargo build` でコンパイルエラーが出るため安全 |
| `--update-snapshots` の env var が並列テストに干渉 | v16.7.0 では許容（スナップショットテストは 1 件のみ） |

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version → 16.7.0 |
| `fav/src/ast.rs` | `Item::TestGroup` 追加 |
| `fav/src/frontend/lexer.rs` | `TokenKind::TestGroup` + `"test_group"` キーワード |
| `fav/src/frontend/parser.rs` | `parse_item` の `TestGroup` 分岐 |
| `fav/src/backend/vm.rs` | 8 新 assert プリミティブ |
| `fav/src/middle/compiler.rs` | `Item::TestGroup` コンパイル + exhaustive match |
| `fav/src/middle/checker.rs` | `Item::TestGroup` exhaustive match |
| `fav/src/driver.rs` | `cmd_test` 更新 + `v167000_tests` + exhaustive match |
| `fav/src/fmt.rs` | `Item::TestGroup` フォーマット + exhaustive match |
| `site/content/docs/language/testing.mdx` | 新規作成 |
