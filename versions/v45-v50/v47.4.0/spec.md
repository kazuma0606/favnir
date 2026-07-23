# Spec: v47.4.0 — `String` 拡充

## 概要

`String.trim_start` / `String.trim_end` / `String.repeat` / `String.pad_left` / `String.pad_right` は
vm.rs・checker.rs に実装済み（v47.0.0 以前の過去バージョンにて実装完了済み）。
本バージョンのスコープは `driver.rs` へのテスト追加のみ。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `String.pad_left` | ✅ line 10672 | ✅ line 6062 | テストなし |
| `String.pad_right` | ✅ line 10703 | ✅ line 6062 | テストなし |
| `String.repeat` | ✅ line 10883 | ✅ line 6079 | テストなし |
| `String.trim_start` | ✅ line 10927 | ✅ line 6085 | テストなし |
| `String.trim_end` | ✅ line 10935 | ✅ line 6085 | テストなし |

---

## 解決策

`driver.rs` に `v474000_tests` モジュールを追加し、ロードマップ指定の 3 件で動作を確認する。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `string_pad_left` | `String.pad_left("42", 6, "0")` → `"000042"` |
| `string_trim_start` | `String.trim_start("  hello  ")` → `"hello  "` |
| `string_repeat` | `String.repeat("ab", 3)` → `"ababab"` |

### テストコード

```favnir
// string_pad_left
// pad_left("42", 6, "0") = "000042"
fn main() -> Bool {
  bind result <- String.pad_left("42", 6, "0")
  result == "000042"
}

// string_trim_start
// trim_start("  hello  ") = "hello  " (先頭空白のみ除去)
fn main() -> Bool {
  bind result <- String.trim_start("  hello  ")
  result == "hello  "
}

// string_repeat
// repeat("ab", 3) = "ababab"
fn main() -> Bool {
  bind result <- String.repeat("ab", 3)
  result == "ababab"
}
```

### 注意事項

- `String.pad_left(str, width, fill)` — 引数: 文字列・幅（Int）・埋め文字（String）
- `String.pad_left` は fill が空文字の場合エラー、width <= 現在長の場合は元文字列をそのまま返す
- `String.trim_start` は先頭の空白のみ除去（末尾の空白は残る）
- `String.repeat(str, n)` — n=0 で空文字、n<0 はエラー

---

## 完了条件

- `cargo test` 3027 passed, 0 failed（3024 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.4.0"`
- `CHANGELOG.md` に v47.4.0 エントリ追加
- `versions/current.md` を v47.4.0（3027 tests）に更新、進行中バージョンを `v47.5.0` に更新
- `tasks.md` を COMPLETE に更新
