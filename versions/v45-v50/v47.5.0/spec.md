# Spec: v47.5.0 — `Float` / `Int` 拡充

## 概要

`Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` を
vm.rs（primitive 実装）・checker.rs（型シグネチャ登録）・driver.rs（テスト）に追加する。

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `Float.round(f, n)` | 未実装 | 未登録 | テストなし |
| `Float.clamp(f, lo, hi)` | 未実装 | 未登録 | テストなし |
| `Float.abs(f)` | 未実装 | 未登録 | テストなし |
| `Int.to_hex(n)` | 未実装 | 未登録 | テストなし |
| `Int.abs(n)` | 未実装 | 未登録 | テストなし |

---

## 解決策

vm.rs・checker.rs に 5 primitive を実装し、driver.rs に `v475000_tests` を追加する。

---

## 関数仕様

### `Float.round(f: Float, n: Int) -> Float`
- 小数点以下 n 桁に丸める（`(f * 10^n).round() / 10^n`）
- n=0 は整数丸め、n<0 は Rust `powi` の挙動に従う（負の指数は 10^(-n) 倍の丸め）
- `|>` パイプライン使用時: `3.14159 |> Float.round(2)` — `|>` は左辺を第1引数として渡すため f が自動補完される

### `Float.clamp(f: Float, lo: Float, hi: Float) -> Float`
- lo ≤ hi を前提。f < lo → lo、f > hi → hi、それ以外 → f

### `Float.abs(f: Float) -> Float`
- 絶対値（`f.abs()`）

### `Int.to_hex(n: Int) -> String`
- 10 進整数を小文字 16 進文字列に変換（255 → "ff"）
- 負の整数は符号なしビット列として出力（Rust `format!("{:x}", n)` と同様）

### `Int.abs(n: Int) -> Int`
- 絶対値（`n.abs()`）

---

## テスト（+3）

ロードマップ指定の 3 件のみ追加。`Float.abs` / `Int.abs` は実装するがテストは本バージョンではスコープ外（単純な stdlib 関数のため、呼び出し側テストで間接的に検証済みと見なす）。

| テスト名 | 内容 |
|---|---|
| `float_round` | `Float.round(3.14159, 2)` → `3.14` |
| `float_clamp` | `Float.clamp(150.0, 0.0, 100.0)` → `100.0` |
| `int_to_hex` | `Int.to_hex(255)` → `"ff"` |

### 型チェックのスコープ

引数型の静的チェックは本バージョンのスコープ外。誤った型の引数（例: `Float.round("x", 2)`）は checker を通過し、VM で実行時エラーとして検出される。これは既存の `String.pad_left` 等と同じポリシー。

### テストコード

```favnir
// float_round
fn main() -> Bool {
  bind result <- Float.round(3.14159, 2)
  result == 3.14
}

// float_clamp
fn main() -> Bool {
  bind result <- Float.clamp(150.0, 0.0, 100.0)
  result == 100.0
}

// int_to_hex
fn main() -> Bool {
  bind result <- Int.to_hex(255)
  result == "ff"
}
```

---

## 完了条件

- `cargo test` 3030 passed, 0 failed（3027 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.5.0"`
- `CHANGELOG.md` に v47.5.0 エントリ追加
- `versions/current.md` を v47.5.0（3030 tests）に更新、進行中バージョンを `v47.6.0` に更新
- `tasks.md` を COMPLETE に更新
