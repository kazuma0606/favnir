# v16.7.0 Spec — `fav test` 成熟（assert_eq / test_group / スナップショット）

Date: 2026-06-14

---

## 概要

v15.3.0 で基礎を作った `fav test` DSL を実用レベルに引き上げる。
現状は `assert_ok` / `assert_err` / `assert_true` の 3 プリミティブのみだが、
本バージョンで以下を追加する：

1. **`assert_eq`** — 値の等値比較（最もよく使われるアサーション）
2. **`assert_approx_eq`** — 浮動小数点の近似比較
3. **`assert_contains`** — リスト内の要素存在確認
4. **`assert_length`** — リストの長さ確認
5. **`assert_str_contains`** — 文字列の部分一致確認
6. **`assert_str_starts_with`** — 文字列のプレフィックス確認
7. **`assert_err_eq`** — エラー内容の検証（エラーメッセージ文字列を比較）
8. **`assert_snapshot`** — スナップショットテスト（初回で `.snap/` に保存、以降比較）
9. **`test_group "name" { test ... }`** — テストのグループ化とグループ別サマリー
10. **`--update-snapshots`** フラグ — スナップショット強制更新

---

## 新構文

### assert_eq

```fav
test "values are equal" {
  assert_eq(String.concat("a", "b"), "ab")
  assert_eq(1 + 1, 2)
  assert_eq(List.length([1, 2, 3]), 3)
}
```

失敗時のメッセージ例:
```
FAIL test "values are equal"
  assert_eq failed:
    actual:   3
    expected: 4
```

### assert_approx_eq

```fav
test "float approximation" {
  assert_approx_eq(Math.sqrt(2.0), 1.414, 0.001)
}
```

### assert_contains / assert_length

```fav
test "list assertions" {
  let rows = [1, 2, 3]
  assert_contains(rows, 2)
  assert_length(rows, 3)
}
```

### assert_str_contains / assert_str_starts_with

```fav
test "string assertions" {
  let msg = "Hello, Alice!"
  assert_str_contains(msg, "Alice")
  assert_str_starts_with(msg, "Hello")
}
```

### assert_err_eq

```fav
test "error message check" {
  let result = Result.err("not found")
  assert_err_eq(result, "not found")
}
```

### assert_snapshot

```fav
test "snapshot test" {
  let data = { name: "Alice", count: 42 }
  assert_snapshot(data, "alice_snapshot")
  // 初回: .snap/alice_snapshot.snap を作成
  // 2 回目以降: ファイルと比較し、差分があれば FAIL
}
```

### test_group

```fav
test_group "transform pipeline" {
  test "trims whitespace" {
    let row = { name: "  Alice  " }
    assert_eq(String.trim(row.name), "Alice")
  }

  test "handles empty string" {
    let result = Result.err("empty")
    assert_err(result)
  }

  test "string contains check" {
    assert_str_contains("Hello, World!", "World")
  }
}
```

出力:
```
running 3 tests in "transform pipeline"

  test trims whitespace       ... PASS
  test handles empty string   ... PASS
  test string contains check  ... PASS

test result: PASS. 3 passed; 0 failed; 0 skipped.
```

---

## 実装設計

### ast.rs

`Item::TestGroup { name: String, tests: Vec<TestDef>, span: Span }` を追加。
既存の `Item::TestDef` は単独テスト用として維持する。

### parser.rs

- `test_group` キーワード（`TokenKind::TestGroup`）を追加
- `parse_test_group()`: `test_group "name" { test ... }` をパース → `Item::TestGroup`
- `parse_item` の `TokenKind::Test` 分岐とは別に `TokenKind::TestGroup` 分岐を追加
- 新 assert 関数はキーワードではなく通常の関数呼び出しとして扱う（既存と同じ方式）

### vm.rs（`vm_call_builtin` 拡張）

新 VM プリミティブ追加（`vm_call_builtin` の match アーム）：

| 関数名 | 引数 | 成功 | 失敗 |
|---|---|---|---|
| `assert_eq` | `(actual, expected)` | `Unit` | `Err("assert_eq failed: actual={a}, expected={e}")` |
| `assert_approx_eq` | `(actual, expected, epsilon)` | `Unit` | `Err(...)` |
| `assert_contains` | `(list, elem)` | `Unit` | `Err(...)` |
| `assert_length` | `(list, n)` | `Unit` | `Err(...)` |
| `assert_str_contains` | `(s, sub)` | `Unit` | `Err(...)` |
| `assert_str_starts_with` | `(s, prefix)` | `Unit` | `Err(...)` |
| `assert_err_eq` | `(result, msg)` | `Unit` | `Err(...)` |
| `assert_snapshot` | `(value, name)` | `Unit` | `Err(...)` |

`assert_eq` は `vmvalue_repr` で文字列化して比較する（型ごとの特殊処理不要）。

`assert_snapshot` の実装:
- `.snap/{name}.snap` ファイルが存在しない → JSON 形式で保存して PASS
- ファイルが存在する → ファイル内容と比較。一致 → PASS、不一致 → FAIL（diff 表示）
- `--update-snapshots` フラグ時: 常に上書き保存して PASS

### compiler.rs

`Item::TestGroup` のコンパイル: `tests` を個別 `Item::TestDef` と同様にコンパイル（グループメタデータは `cmd_test` で管理）。

### checker.rs

- `Item::TestGroup { .. } => {}` の exhaustive match 追加
- 新 assert 関数の引数型チェックは `check_builtin_apply` に追加

### driver.rs

- `cmd_test` を拡張:
  - `Item::TestGroup` を展開してグループ名付きでテスト実行
  - グループ別にサマリー行を出力
  - `--update-snapshots` フラグを受け取り VM へ渡す
- `AssertSnapshot` の `update_snapshots: bool` をグローバル状態（スレッドローカル or 引数）で制御

### fmt.rs

`Item::TestGroup { name, tests, .. } => format!("test_group \"{}\" {{ ... }}", name)` 追加。

---

## テスト（v167000_tests — 5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_16_7_0` | `Cargo.toml` バージョンが `16.7.0` |
| `assert_eq_pass` | `assert_eq(2, 2)` → PASS |
| `assert_eq_fail` | `assert_eq(1, 2)` → テスト FAIL（適切なメッセージ） |
| `test_group_runs_all` | `test_group` 内の全テストが実行される |
| `assert_snapshot_creates_file` | 初回 `assert_snapshot` で `.snap/` にファイル生成 |

---

## 完了条件（PASS=5）

1. `assert_eq(actual, expected)` が等値で PASS・不等値で FAIL する
2. `assert_err_eq(result, "msg")` がエラー内容を検証する
3. `test_group` でテストがグループ化され、グループ別サマリーが出る
4. `assert_snapshot` が初回で `.snap/` ファイルを生成し、2 回目から比較する
5. `--update-snapshots` でスナップショットが更新される

---

## 非対応（v16.7.0 スコープ外）

- `assert_approx_eq` / `assert_contains` / `assert_length` / `assert_str_contains` /
  `assert_str_starts_with` は VM プリミティブ追加のみ（テストは最小限）
- `bench` / `bench_group` は対象外（v18.x 以降）
- `test_group` のネスト（グループ内グループ）は対象外
- スナップショットの diff 表示は行単位の簡易実装のみ
