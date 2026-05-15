# Favnir v2.9.0 Language Specification

Release date: 2026-05-13

---

## Overview

v2.9.0 adds two features:

1. **`collect` 内 `for` の許可** — E067 を廃止し、`collect { for x in list { yield x; } }` が正式サポートされる
2. **`Stream<T>` 遅延シーケンス** — 遅延評価のシーケンス型と、6 つの Stream 操作 API

---

## 1. `collect` 内 `for`（E067 解消）

v1.9.0 では `collect` ブロック内の `for` は E067 エラーになっていたが、v2.9.0 で正式サポート。

### 構文

```favnir
bind xs <- collect {
    for x in some_list {
        yield x;          // 外側の collect に帰属
    }
}
```

### フィルタパターン

```favnir
bind evens <- collect {
    for x in List.range(0, 10) {
        if x % 2 == 0 {
            yield x;
        }
    }
}
// evens = [0, 2, 4, 6, 8]
```

### 変換パターン

```favnir
bind squares <- collect {
    for x in List.range(1, 6) {
        yield x * x;
    }
}
// squares = [1, 4, 9, 16, 25]
```

### 実装メモ

- VM は変更なし。`YieldValue` opcode は VM-global の `collect_frames` スタックを使うため、クロージャ内（for ボディ）から `yield` しても外側の `collect` に帰属する。
- `checker.rs` の `collect_yield_types` ヘルパーが `for` ボディ内の `yield` を再帰スキャンして型推論する。
- E067 は廃止済み。

---

## 2. `Stream<T>` 遅延シーケンス

### 型

`Stream<T>` — 遅延評価のシーケンス型。`Stream.to_list` が呼ばれたときに具体値に展開される。

型アノテーション例:
```favnir
bind s : Stream<Int> <- Stream.of(List.range(1, 10))
```

### API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `Stream.from(list)` | `List<T> -> Stream<T>` | リストから有限ストリームを生成 |
| `Stream.of(list)` | `List<T> -> Stream<T>` | `Stream.from` の別名 |
| `Stream.gen(seed, f)` | `(T, T->T) -> Stream<T>` | 無限ストリームを生成（seed から開始し f で次の値を生成） |
| `Stream.map(stream, f)` | `(Stream<T>, T->U) -> Stream<U>` | 各要素に f を適用（遅延） |
| `Stream.filter(stream, pred)` | `(Stream<T>, T->Bool) -> Stream<T>` | 条件を満たす要素のみ（遅延） |
| `Stream.take(stream, n)` | `(Stream<T>, Int) -> Stream<T>` | 最大 n 件に制限 |
| `Stream.to_list(stream)` | `Stream<T> -> List<T>` | ストリームを具体化してリストに変換 |

> **注意**: `Stream.collect` という名前は `collect` キーワードと競合するため、`Stream.to_list` を使用する。

### 使用例

#### 有限ストリーム

```favnir
bind s <- Stream.from(List.range(1, 6))
bind m <- Stream.map(s, |x| x * x)
bind xs <- Stream.to_list(m)
// xs = [1, 4, 9, 16, 25]
```

#### フィルタ

```favnir
bind s <- Stream.of(List.range(1, 11))
bind f <- Stream.filter(s, |x| x > 5)
bind xs <- Stream.to_list(f)
// xs = [6, 7, 8, 9, 10]
```

#### 無限ストリーム + take

```favnir
// 1, 2, 4, 8, 16, ... (powers of 2)
bind s <- Stream.gen(1, |x| x * 2)
bind t <- Stream.take(s, 5)
bind xs <- Stream.to_list(t)
// xs = [1, 2, 4, 8, 16]
```

#### パイプライン

```favnir
bind s <- Stream.of(List.range(1, 11))
bind m <- Stream.map(s, |x| x * x)
bind f <- Stream.filter(m, |x| x > 20)
bind xs <- Stream.to_list(f)
// xs = [25, 36, 49, 64, 81, 100]
```

### エラー

無限ストリーム（`Stream.gen`）を `Stream.take` なしで `Stream.to_list` するとランタイムエラー:

```
RuntimeError: cannot collect an infinite stream without Stream.take
```

### 実装メモ

- `VMStream` enum: `Gen / Of / Map / Filter / Take` — 操作を記録するだけで即時評価しない
- `Stream.to_list` 呼び出し時に `materialize_stream` が再帰的に展開
- `Stream.map` / `Stream.filter` はクロージャを含むため `VM::call_builtin`（`&mut self`）に実装
- `VMValue::PartialEq`: `Stream == Stream` は常に `false`（ストリームは比較不可）

---

## エラーコード

| コード | 説明 | 状態 |
|--------|------|------|
| E067 | `for` inside `collect` is not supported | **廃止**（v2.9.0） |

---

## テスト数

- v2.8.0 ベースライン: 625
- v2.9.0: **637**（+12）

| カテゴリ | 件数 |
|---------|------|
| checker: `Stream<T>` 型チェック | +2 |
| driver: `collect { for ... }` 統合テスト | +3 |
| driver: `Stream.*` 統合テスト | +7 |
