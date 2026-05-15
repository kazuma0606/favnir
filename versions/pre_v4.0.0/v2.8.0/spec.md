# Favnir v2.8.0 Language Specification

作成日: 2026-05-13

---

## テーマ

型駆動の乱数生成・統計ルーンを Favnir で実装する。

v2.8.0 では VM 側に最小限の 2 プリミティブ（`Random.int` / `Random.float`）を追加し、
その上の全ロジックを `runes/stat/stat.fav` に純粋な Favnir で書く。

---

## 1. VM プリミティブ（Rust 側、最小限）

### 1-1. `Random.int(min, max) -> Int !Random`

```
Random.int(0, 100)  // 0 以上 100 以下のランダムな Int を返す（両端含む）
```

- 引数: `min: Int`, `max: Int`（`min <= max` を前提）
- 戻り値: `[min, max]` の範囲の `Int`
- エフェクト: `!Random`
- 実装: `rand` クレートの `gen_range(min..=max)`

### 1-2. `Random.float() -> Float !Random`

```
Random.float()  // 0.0 以上 1.0 未満のランダムな Float を返す
```

- 引数: なし（Unit）
- 戻り値: `[0.0, 1.0)` の `Float`
- エフェクト: `!Random`
- 実装: `rand` クレートの `gen::<f64>()`

### 1-3. `effect Random` の追加

- チェッカーの `BUILTIN_EFFECTS` リストに `"Random"` を追加
- チェッカーのグローバル名前空間に `"Random"` を追加
- これにより `fn f() -> Int !Random = ...` が型チェックを通る

---

## 2. stat rune の API

### 2-1. 基本ジェネレータ

```favnir
// デフォルト範囲でのランダム生成
public fn sample_int() -> Int !Random
// → Random.int(0, 100) を呼ぶ

public fn sample_float() -> Float !Random
// → Random.float() を呼ぶ

public fn sample_bool() -> Bool !Random
// → Random.int(0, 1) == 1 を呼ぶ
```

### 2-2. パラメータ付き分布

```favnir
// [min, max] の範囲で一様乱数を生成（カリー化）
public fn uniform(min: Int) -> Int -> Int !Random

// 使い方:
// bind n <- uniform(1)(10)   // 1 以上 10 以下
```

### 2-3. リストからのランダム選択

```favnir
// String リストからランダムに 1 要素を選ぶ
public fn choice_str(xs: List<String>) -> String !Random

// Int リストからランダムに 1 要素を選ぶ
public fn choice_int(xs: List<Int>) -> Int !Random

// 使い方:
// bind color <- choice_str(collect { yield "red"; yield "green"; yield "blue"; })
```

### 2-4. リスト生成

```favnir
// n 個のランダムな Int のリストを生成（範囲 [0, 100]）
public fn list_int(n: Int) -> List<Int> !Random

// n 個のランダムな Float のリストを生成（範囲 [0.0, 1.0)）
public fn list_float(n: Int) -> List<Float> !Random

// 使い方:
// bind samples <- list_int(10)   // 10 個の Int リスト
```

### 2-5. 簡易プロファイル

```favnir
// リストの基本統計を返す
public type ProfileReport = {
    total: Int    // 要素数
    min_v: Int    // 最小値（Int リスト用）
    max_v: Int    // 最大値（Int リスト用）
}

// Int リストの簡易プロファイル
public fn profile_int(xs: List<Int>) -> ProfileReport
```

---

## 3. stat rune のディレクトリ構成

```
runes/
  fav.toml                 ← 既存（v2.7.0 で作成）
  validate/                ← 既存
  stat/
    stat.fav               ← 実装（Rust コードなし）
    stat.test.fav          ← テストスイート
```

---

## 4. テスト戦略

### 決定論的テスト

ランダム性があるため、テストは正確な値ではなく**範囲・長さ・境界値**で検証する。

```favnir
// uniform(5)(5) は必ず 5 を返す（min == max）
test "uniform min==max is deterministic" {
    bind n <- uniform(5)(5)
    assert_eq(n, 5)
}

// choice_str(["only"]) は必ず "only" を返す（単要素リスト）
test "choice_str single element" {
    bind s <- choice_str(collect { yield "only"; })
    assert_eq(s, "only")
}

// list_int(7) の長さは 7
test "list_int returns correct length" {
    bind xs <- list_int(7)
    assert_eq(List.length(xs), 7)
}
```

### Rust 統合テスト

- `Random.int(5, 5)` → 5（決定論的）
- `Random.int(0, 10)` の結果が `[0, 10]` の範囲内
- `Random.float()` の結果が `[0.0, 1.0)` の範囲内
- `list_int(3)` で長さ 3 のリストが返る

---

## 5. `Cargo.toml` への依存追加

```toml
rand = "0.8"
```

- `Random.int` / `Random.float` の実装に使用
- VM 側でのみ使用（stat.fav の実装に Rust コードはない）

---

## 6. 互換性

- 既存のテストに影響しない（Random は新規追加のみ）
- v2.7.0 の validate rune は変更しない
- `!Random` は新しい組み込みエフェクトとして追加（既存エフェクトに影響なし）
- `Cargo.toml` への `rand` 追加以外の Rust 変更は vm.rs と checker.rs のみ
