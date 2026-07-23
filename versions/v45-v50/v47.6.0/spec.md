# Spec: v47.6.0 — `Option` 拡充

## 概要

`Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` は
vm.rs・checker.rs に実装済み（過去バージョン）。
本バージョンのスコープは `driver.rs` へのテスト追加のみ。

> **ロードマップ表現の注記**: `roadmap-v47.1-v48.0.md` の v47.6.0 は「VM primitive として追加」と記述しているが、
> 実態はすでに実装済み。本バージョンではテスト追加のみを行う。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `Option.map(opt, f)` | ✅ line 4194 | ✅ line 6108 | テストなし |
| `Option.unwrap_or(opt, default)` | ✅ line 4259 | ✅ line 6115 | テストなし |
| `Option.and_then(opt, f)` | ✅ line 4221 | ✅ line 6119 | テストなし |
| `Option.is_some(opt)` | ✅ line 4317 | ✅ line 6134 | テストなし |
| `Option.is_none(opt)` | ✅ line 4335 | ✅ line 6134 | テストなし |

---

## 解決策

`driver.rs` に `v476000_tests` モジュールを追加し、ロードマップ指定の 3 件で動作を確認する。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `option_map` | `Option.map(Option.some(5), \|n\| n * 2)` → `Option.some(10)` と等価 |
| `option_unwrap_or` | `Option.unwrap_or(Option.none(), "default")` → `"default"` |
| `option_and_then` | `Option.and_then(Option.some(5), \|n\| Option.some(n + 1))` → `some(6)` と等価 |

### テストコード（Bool を返す形式）

```favnir
// option_map: some(5) を map して some(10) になるか確認
fn main() -> Bool {
  bind opt    <- Option.some(5)
  bind result <- Option.map(opt, |n| n * 2)
  Option.unwrap_or(result, 0) == 10
}

// option_unwrap_or: none() のデフォルト値を確認
fn main() -> Bool {
  bind opt    <- Option.none()
  bind result <- Option.unwrap_or(opt, "default")
  result == "default"
}

// option_and_then: some(5) → some(6) のチェーン確認
fn main() -> Bool {
  bind opt    <- Option.some(5)
  bind result <- Option.and_then(opt, |n| Option.some(n + 1))
  Option.unwrap_or(result, 0) == 6
}
```

### 注意事項

- `Option.map` の結果を `==` で直接比較するのではなく、`Option.unwrap_or` で値を取り出して比較する
- `Option.none()` は型パラメータ不要（checker.rs が `Type::Option(Type::Unknown)` を返す）
- `Option.and_then` の closure は `Option<T>` を返す必要がある
- `Option.unwrap_or` は `none` の場合にデフォルト値をそのまま返す（monadic bind ではなく純粋な値束縛）。`bind result <- Option.unwrap_or(opt, "default")` の `result` は `String` 型になる

---

## 完了条件

- `cargo test` 3033 passed, 0 failed（3030 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.6.0"`
- `CHANGELOG.md` に v47.6.0 エントリ追加
- `versions/current.md` を v47.6.0（3033 tests）に更新、進行中バージョンを `v47.7.0` に更新
- `tasks.md` を COMPLETE に更新
