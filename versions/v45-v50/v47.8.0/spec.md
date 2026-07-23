# Spec: v47.8.0 — `Map` 拡充

## 概要

`Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` は
vm.rs・checker.rs に実装済み（過去バージョン）。
本バージョンのスコープは `driver.rs` へのテスト追加のみ。

> **ロードマップ表現の注記**: `roadmap-v47.1-v48.0.md` の v47.8.0 は「VM primitive として追加」と記述しているが、
> 実態はすでに実装済み。本バージョンではテスト追加のみを行う。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `Map.merge(base, overrides)` | ✅ line 12023 | ✅ line 6805 | テストなし |
| `Map.filter_values(map, pred)` | ✅ line 3834 | ✅ line 6809 | テストなし |
| `Map.map_values(map, f)` | ✅ line 3809 | ✅ line 6806 | テストなし |
| `Map.keys(map)` | ✅ line 11927 | ✅ line 6799 | テストなし |
| `Map.values(map)` | ✅ line 11945 | ✅ line 6800 | テストなし |

---

## 解決策

`driver.rs` に `v478000_tests` モジュールを追加し、ロードマップ指定の 3 件で動作を確認する。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `map_merge` | `Map.merge({"key":1}, {"key":2})` → 右辺優先で value == 2 |
| `map_filter_values` | `Map.filter_values({"a":1,"b":2}, \|v\| v > 1)` → size == 1 |
| `map_map_values` | `Map.map_values({"x":5}, \|v\| v * 2)` → `"x"` の value == 10 |

### テストコード（Bool を返す形式）

```favnir
// map_merge: 右辺優先 (key=1 vs key=2 → 2)
fn main() -> Bool {
  bind m1     <- Map.set((), "key", 1)
  bind m2     <- Map.set((), "key", 2)
  bind merged <- Map.merge(m1, m2)
  Option.unwrap_or(Map.get(merged, "key"), 0) == 2
}

// map_filter_values: v > 1 を満たすエントリのみ残す → size == 1
fn main() -> Bool {
  bind m        <- Map.set(Map.set((), "a", 1), "b", 2)
  bind filtered <- Map.filter_values(m, |v| v > 1)
  Map.size(filtered) == 1
}

// map_map_values: 各値を 2 倍にする
fn main() -> Bool {
  bind m      <- Map.set((), "x", 5)
  bind mapped <- Map.map_values(m, |v| v * 2)
  Option.unwrap_or(Map.get(mapped, "x"), 0) == 10
}
```

### 注意事項

- Map 構築: `Map.set((), "key", value)` — `()` が空マップ（Unit = empty Record）
- `Map.merge(base, overrides)` — 右辺優先（重複キーは overrides の値で上書き）
- `Map.get(map, key)` → `Option<V>` を返すため `Option.unwrap_or` で取り出す
- `Map.filter_values(map, pred)` — pred は `V -> Bool` を受け取るクロージャ
- `Map.map_values(map, f)` — f は `V -> W` を受け取るクロージャ
- `bind` は単純代入（monadic unwrap ではない）

---

## 完了条件

- `cargo test` 3039 passed, 0 failed（3036 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.8.0"`
- `CHANGELOG.md` に v47.8.0 エントリ追加
- `versions/current.md` を v47.8.0（3039 tests）に更新、進行中バージョンを `v47.9.0` に更新
- `tasks.md` を COMPLETE に更新（T0 事前確認の全チェックボックスを含む T0〜T2 全 `[x]`）
