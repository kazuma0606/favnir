# Spec: v47.1.0 — `List.zip` / `List.chunk`

## 概要

`List.zip` / `List.chunk` の動作確認テストを `driver.rs` に追加する。
各 primitive は vm.rs・checker.rs に実装済みのため、本バージョンのスコープはテスト追加のみ。
（`List.zip_with` も vm.rs 実装済みだが、テスト対象は本バージョンでは対象外。）

---

## 問題

`List.zip` / `List.chunk` は VM primitive・型チェックともに実装済みだが、
`driver.rs` の Rust テストが存在せず、リグレッションを検出できない。

---

## 解決策

`driver.rs` に `v471000_tests` モジュールを追加し、以下 2 件のテストで動作を確認する。

### テスト 1: `list_zip_pairs`

```favnir
fn main() -> Bool {
  bind names  <- List.from(["alice", "bob"])
  bind scores <- List.from([90, 80])
  bind pairs  <- List.zip(names, scores)
  List.length(pairs) == 2
}
```

- `List.zip(xs, ys)` は `{first: T, second: U}` の Record リストを返す
- `List.length(pairs) == 2` で 2 ペア生成されることを確認
- 注意: `List.first` は `Option<T>` を返すため、Record フィールドアクセスは Option 展開が必要。
  本テストでは長さチェックで十分なため、その複雑さを避ける。

### テスト 2: `list_chunk_batches`

```favnir
fn main() -> Bool {
  bind data    <- List.from([1, 2, 3, 4, 5])
  bind batches <- List.chunk(data, 2)
  List.length(batches) == 3
}
```

- `List.chunk(xs, n)` は `List<List<T>>` を返す
- 要素 5 件をサイズ 2 でチャンクすると 3 バッチ（[1,2] / [3,4] / [5]）になることを確認

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `list_zip_pairs` | `List.zip(names, scores)` → `List.length(pairs) == 2` |
| `list_chunk_batches` | `List.chunk(data, 2)` → `List.length(batches) == 3` |

---

## 完了条件

- `cargo test` 3018 passed, 0 failed（3016 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.1.0"`
- `CHANGELOG.md` に v47.1.0 エントリ追加
- `versions/current.md` を v47.1.0（3018 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## 注記: ロードマップとのテスト数差異

ロードマップ `roadmap-v47.1-v48.0.md` の推定値は「3013」だが、
v47.0.0 の実績（3016 tests）に基づき本 spec では 3016 + 2 = 3018 を正とする。
