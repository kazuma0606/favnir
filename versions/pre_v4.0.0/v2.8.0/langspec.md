# Favnir v2.8.0 Language Specification

作成日: 2026-05-13

---

## 概要

v2.8.0 では `!Random` エフェクトと `stat` rune を導入する。
`Random.int` / `Random.float` 組み込み関数により型安全な乱数生成が可能になり、
純 Favnir で書かれた `stat` rune でサンプリング・統計ユーティリティを提供する。

---

## 新機能

### 1. `!Random` エフェクト

`!Random` は乱数生成を行う関数に付与するエフェクト注釈。

```favnir
fn roll_die() -> Int !Random = Random.int(1, 6)
```

`BUILTIN_EFFECTS` に追加済み。`fav check` でエラーにならない。

---

### 2. `Random` 組み込み名前空間

#### `Random.int(min: Int, max: Int) -> Int !Random`

`[min, max]` の範囲（両端含む）で一様分布の整数乱数を返す。

```favnir
bind n <- Random.int(1, 100)   // 1 以上 100 以下の整数
```

- `min == max` の場合は常に `min` を返す
- `min > max` の場合は実行時エラー（`rand::thread_rng().gen_range` の仕様）

#### `Random.float() -> Float !Random`

`[0.0, 1.0)` の範囲で一様分布の浮動小数点乱数を返す。

```favnir
bind f <- Random.float()   // 0.0 以上 1.0 未満の Float
```

---

### 3. `stat` rune

`import rune "stat"` でインポートする純 Favnir rune。

#### 型

```favnir
type ProfileReport = {
    total: Int
    min_v: Int
    max_v: Int
}
```

#### 基本ジェネレータ

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `sample_int` | `() -> Int !Random` | `[0, 100]` のランダム整数 |
| `sample_float` | `() -> Float !Random` | `[0.0, 1.0)` のランダム Float |
| `sample_bool` | `() -> Bool !Random` | ランダムな Bool |

#### 一様分布（カリー化）

```favnir
// uniform(min)(max): [min, max] の整数乱数
stat.uniform(1)(10)   // 1 以上 10 以下
```

#### リスト選択

```favnir
// 空リストは "" / 0 を返す
stat.choice_str(xs)   // List<String> -> String !Random
stat.choice_int(xs)   // List<Int>    -> Int    !Random
```

#### リスト生成

```favnir
stat.list_int(n)      // n 個の [0,100] 整数 -> List<Int>  !Random
stat.list_float(n)    // n 個の [0,1) Float  -> List<Float> !Random
```

#### 簡易プロファイル

```favnir
bind report <- stat.profile_int(xs)
// report.total  = List.length(xs)
// report.min_v  = リスト最小値（空なら 0）
// report.max_v  = リスト最大値（空なら 0）
```

---

## 使い方

### fav.toml

```toml
[rune]
name    = "my_app"
version = "0.1.0"
src     = "src"

[runes]
path = "../../../runes"
```

### main.fav

```favnir
import rune "stat"

public fn main() -> Unit !Io !Random = {
    bind n <- stat.sample_int()
    IO.println($"sample_int: {n}")
}
```

---

## 互換性

- v2.7.0 以前のコードへの影響なし
- `"Random"` は `BUILTIN_EFFECTS` / コンパイラグローバルに追加済み
- 既存テスト 617 件はすべて維持（617 → 625）

---

## テスト数

| カテゴリ | 追加 |
|----------|------|
| checker.rs `!Random` 効果テスト | +1（既存テスト内カバー） |
| vm.rs `Random.int/float` | +2（既存テスト内カバー） |
| driver.rs stat 統合テスト | +8 |
| **合計** | **625**（v2.7.0: 617） |
