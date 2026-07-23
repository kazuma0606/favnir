# Spec: v47.3.0 — `List.scan` / `List.take_while` / `List.drop_while`

## 概要

`List.scan` / `List.take_while` / `List.drop_while` は vm.rs・checker.rs に実装済み。
本バージョンのスコープは `driver.rs` へのテスト追加のみ。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `List.take_while` | ✅ line 3389 | ✅ line 5992 | テストなし |
| `List.drop_while` | ✅ line 3422 | ✅ line 5992 | テストなし |
| `List.scan` | ✅ line 3466 | ✅ line 6005 | テストなし |

---

## 解決策

`driver.rs` に `v473000_tests` モジュールを追加し、3 件のテストで動作を確認する。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `list_scan_cumulative` | `List.scan(range(1,4), 0, \|acc, x\| acc + x)` → `length == 4` |
| `list_take_while` | `List.take_while(range(1,6), \|x\| x < 3)` → `length == 2` |
| `list_drop_while` | `List.drop_while(range(1,6), \|x\| x < 3)` → `length == 3` |

### テストコード

```favnir
// list_scan_cumulative
// scan([1,2,3], 0, +) = [0, 1, 3, 6] — init値を含む length 4
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind totals <- List.scan(xs, 0, |acc, x| acc + x)
  List.length(totals) == 4
}

// list_take_while
// take_while([1,2,3,4,5], x<3) = [1, 2] — length 2
fn main() -> Bool {
  bind xs     <- List.range(1, 6)
  bind result <- List.take_while(xs, |x| x < 3)
  List.length(result) == 2
}

// list_drop_while
// drop_while([1,2,3,4,5], x<3) = [3, 4, 5] — length 3
fn main() -> Bool {
  bind xs     <- List.range(1, 6)
  bind result <- List.drop_while(xs, |x| x < 3)
  List.length(result) == 3
}
```

### 注意事項

- `List.scan(list, init, func)` — 引数順: リスト先・init 2番目・関数 3番目
- `List.scan` の戻り値は初期値を含む（要素数 n のリストに対して n+1 要素を返す）
- `List.take_while(list, func)` / `List.drop_while(list, func)` — リスト先・関数後
- `List.range(1, 6)` = `[1, 2, 3, 4, 5]`（exclusive end、5 要素）
- `List.range(1, 4)` = `[1, 2, 3]`（exclusive end、3 要素）

---

## 完了条件

- `cargo test` 3024 passed, 0 failed（3021 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.3.0"`
- `CHANGELOG.md` に v47.3.0 エントリ追加
- `versions/current.md` を v47.3.0（3024 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## 注記: ロードマップとのテスト数差異

ロードマップ `roadmap-v47.1-v48.0.md` の推定値は「3019」だが、
v47.2.0 の実績（3021 tests）に基づき本 spec では 3021 + 3 = 3024 を正とする。
