# Spec: v47.7.0 — `Result` 拡充

## 概要

`Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` は
vm.rs・checker.rs に実装済み（過去バージョン）。
本バージョンのスコープは `driver.rs` へのテスト追加のみ。

> **ロードマップ表現の注記**: `roadmap-v47.1-v48.0.md` の v47.7.0 は「VM primitive として追加」と記述しているが、
> 実態はすでに実装済み。本バージョンではテスト追加のみを行う。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `Result.map(r, f)` | ✅ line 4379 | ✅ line 6153 | テストなし |
| `Result.map_err(r, f)` | ✅ line 4406 | ✅ line 6176 | テストなし |
| `Result.and_then(r, f)` | ✅ line 4433 | ✅ line 6168 | テストなし |
| `Result.is_ok(r)` | ✅ line 4499 | ✅ line 6187 | テストなし |
| `Result.is_err(r)` | ✅ line 4519 | ✅ line 6187 | テストなし |

---

## 解決策

`driver.rs` に `v477000_tests` モジュールを追加し、ロードマップ指定の 3 件で動作を確認する。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `result_map` | `Result.map(Result.ok(5), \|n\| n * 2)` → unwrap_or 10 |
| `result_map_err` | `Result.map_err(Result.err("oops"), \|e\| "wrapped: " ++ e)` → `match` で `"wrapped: oops"` を直接検証 |
| `result_and_then` | `Result.and_then(Result.ok(5), \|n\| Result.ok(n + 1))` → unwrap_or 6 |

### テストコード（Bool を返す形式）

```favnir
// result_map: ok(5) を map して ok(10) になるか確認
fn main() -> Bool {
  bind r      <- Result.ok(5)
  bind result <- Result.map(r, |n| n * 2)
  Result.unwrap_or(result, 0) == 10
}

// result_map_err: err("oops") の error を変換して "wrapped: oops" になることを直接検証
fn main() -> Bool {
  bind r      <- Result.err("oops")
  bind result <- Result.map_err(r, |e| String.concat("wrapped: ", e))
  match result {
    err(e) => e == "wrapped: oops"
    ok(_)  => false
  }
}

// result_and_then: ok(5) → ok(6) のチェーン確認
fn main() -> Bool {
  bind r      <- Result.ok(5)
  bind result <- Result.and_then(r, |n| Result.ok(n + 1))
  Result.unwrap_or(result, 0) == 6
}
```

### 注意事項

- `bind` は単純代入（monadic unwrap ではない）。`Result.ok(5)` は `VMValue::Variant("ok", Some(...))` として格納される
- `Result.map` は `ok` バリアントのみ変換し、`err` はそのまま通す
- `Result.map_err` は `err` バリアントのみ変換し、`ok` はそのまま通す
- `Result.and_then` の closure は `Result<T, E>` を返す必要がある
- `Result.unwrap_or(r, default)` は `ok` の場合は中身を、`err` の場合は `default` を返す
- `result_map_err` テストは `match result { err(e) => e == "wrapped: oops" | ok(_) => false }` で変換内容を直接検証する。`vm_stdlib_tests.rs` に `match` による err payload 取り出しの先例あり（line 905〜921）

---

## 完了条件

- `cargo test` 3036 passed, 0 failed（3033 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.7.0"`
- `CHANGELOG.md` に v47.7.0 エントリ追加
- `versions/current.md` を v47.7.0（3036 tests）に更新、進行中バージョンを `v47.8.0` に更新
- `tasks.md` を COMPLETE に更新
