# Favnir v2.8.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.8.0"` に変更。
`src/main.rs` の HELP テキストを `v2.8.0` に更新。

---

## Phase 1 — `rand` クレートの追加

### `Cargo.toml`

```toml
[dependencies]
# 既存...
rand = "0.8"
```

`Random.int` / `Random.float` の実装に使用する。

---

## Phase 2 — チェッカー拡張（`!Random` エフェクト）

### `src/middle/checker.rs`

#### 2-1. `BUILTIN_EFFECTS` に `"Random"` を追加

```rust
const BUILTIN_EFFECTS: &[&str] = &[
    "Pure", "Io", "Db", "Network", "File", "Trace", "Emit",
    "Random",  // 追加
];
```

これにより `fn f() -> Int !Random` の宣言が型チェックエラーにならない。

#### 2-2. グローバル名前空間に `"Random"` を追加

```rust
for ns in &[
    "Math", "List", "String", "Option", "Result", "Db", "Http", "Map",
    "Debug", "Emit", "Util", "Trace", "File", "Json", "Csv", "Task",
    "Random",  // 追加
] {
    self.env.define(ns.to_string(), Type::Named(ns.to_string(), vec![]));
}
```

これにより `Random.int(...)` の呼び出しが "Unknown identifier" エラーにならない。

---

## Phase 3 — VM 拡張（`Random.int` / `Random.float`）

### `src/backend/vm.rs`

既存の `vm_call_builtin` に 2 つのハンドラを追加する。

```rust
"Random.int" => {
    let min = args
        .pop_front()
        .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
    let max = args
        .pop_front()
        .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
    match (min, max) {
        (VMValue::Int(lo), VMValue::Int(hi)) => {
            use rand::Rng;
            let n = rand::thread_rng().gen_range(lo..=hi);
            Ok(VMValue::Int(n))
        }
        _ => Err("Random.int requires (Int, Int)".to_string()),
    }
}

"Random.float" => {
    use rand::Rng;
    let f: f64 = rand::thread_rng().gen();
    Ok(VMValue::Float(f))
}
```

---

## Phase 4 — `runes/stat/stat.fav` の実装

### 4-1. 基本ジェネレータ

```favnir
// デフォルト範囲での乱数生成
public fn sample_int() -> Int !Random = Random.int(0, 100)
public fn sample_float() -> Float !Random = Random.float()
public fn sample_bool() -> Bool !Random = Random.int(0, 1) == 1
```

### 4-2. 一様分布（カリー化）

```favnir
// uniform(min)(max): [min, max] の範囲で乱数を生成
public fn uniform(min: Int) -> Int -> Int !Random = |max| {
    Random.int(min, max)
}
```

### 4-3. リストからの選択

```favnir
// String リストからランダムに 1 要素を選ぶ
// 空リストの場合は "" を返す
public fn choice_str(xs: List<String>) -> String !Random = {
    bind i <- Random.int(0, List.length(xs) - 1)
    Option.unwrap_or(List.first(List.drop(xs, i)), "")
}

// Int リストからランダムに 1 要素を選ぶ
// 空リストの場合は 0 を返す
public fn choice_int(xs: List<Int>) -> Int !Random = {
    bind i <- Random.int(0, List.length(xs) - 1)
    Option.unwrap_or(List.first(List.drop(xs, i)), 0)
}
```

### 4-4. リスト生成

```favnir
// n 個のランダムな Int のリストを生成（各要素は [0, 100]）
public fn list_int(n: Int) -> List<Int> !Random = {
    List.map(List.range(0, n), |_| Random.int(0, 100))
}

// n 個のランダムな Float のリストを生成（各要素は [0.0, 1.0)）
public fn list_float(n: Int) -> List<Float> !Random = {
    List.map(List.range(0, n), |_| Random.float())
}
```

### 4-5. 簡易プロファイル

```favnir
// Int リストの簡易統計
public type ProfileReport = {
    total: Int
    min_v: Int
    max_v: Int
}

public fn profile_int(xs: List<Int>) -> ProfileReport = {
    bind total <- List.length(xs)
    bind min_v <- Option.unwrap_or(List.fold(xs, Option.none(), |acc, x|
        match acc {
            Some(cur) => if x < cur { Option.some(x) } else { Option.some(cur) }
            None      => Option.some(x)
        }
    ), 0)
    bind max_v <- Option.unwrap_or(List.fold(xs, Option.none(), |acc, x|
        match acc {
            Some(cur) => if x > cur { Option.some(x) } else { Option.some(cur) }
            None      => Option.some(x)
        }
    ), 0)
    ProfileReport { total: total  min_v: min_v  max_v: max_v }
}
```

> `profile_int` が複雑な場合は `total: List.length(xs)  min_v: 0  max_v: 0` の
> スタブ実装に留めてよい。テストは `total` のみを検証する。

---

## Phase 5 — `runes/stat/stat.test.fav` の作成

決定論的な境界値と長さを検証するテストスイート。

```favnir
// スタンドアロン形式: 型・関数をインラインで定義してテストする

// --- uniform ---
test "uniform min==max always returns that value" {
    bind n <- uniform(7)(7)
    assert_eq(n, 7)
}

test "uniform(0)(0) returns 0" {
    bind n <- uniform(0)(0)
    assert_eq(n, 0)
}

// --- choice_str ---
test "choice_str single element" {
    bind s <- choice_str(collect { yield "only"; })
    assert_eq(s, "only")
}

// --- choice_int ---
test "choice_int single element" {
    bind n <- choice_int(collect { yield 42; })
    assert_eq(n, 42)
}

// --- list_int ---
test "list_int returns correct length" {
    bind xs <- list_int(5)
    assert_eq(List.length(xs), 5)
}

test "list_int length 0 returns empty list" {
    bind xs <- list_int(0)
    assert_eq(List.length(xs), 0)
}

// --- list_float ---
test "list_float returns correct length" {
    bind xs <- list_float(3)
    assert_eq(List.length(xs), 3)
}

// --- profile_int ---
test "profile_int total count" {
    bind xs <- list_int(4)
    bind report <- profile_int(xs)
    assert_eq(report.total, 4)
}

test "profile_int empty list" {
    bind report <- profile_int(collect { })
    assert_eq(report.total, 0)
}
```

---

## Phase 6 — examples/stat_demo の作成

### `fav/examples/stat_demo/fav.toml`

```toml
[rune]
name    = "stat_demo"
version = "0.1.0"
src     = "src"

[runes]
path = "../../../runes"
```

### `fav/examples/stat_demo/src/main.fav`

```favnir
import rune "stat"

public fn main() -> Unit !Io !Random {
    bind n <- stat.sample_int()
    IO.println($"sample_int: {n}");

    bind f <- stat.sample_float()
    IO.println($"sample_float: {Debug.show(f)}");

    bind b <- stat.sample_bool()
    IO.println($"sample_bool: {Debug.show(b)}");

    bind u <- stat.uniform(1)(10)
    IO.println($"uniform(1)(10): {u}");

    bind color <- stat.choice_str(collect {
        yield "red";
        yield "green";
        yield "blue";
    })
    IO.println($"choice_str: {color}");

    bind xs <- stat.list_int(5)
    IO.println($"list_int(5): {Debug.show(xs)}");

    bind report <- stat.profile_int(xs)
    IO.println($"profile total={report.total} min={report.min_v} max={report.max_v}")
}
```

---

## Phase 7 — Rust 統合テスト（src/driver.rs）

```rust
// Random.int の境界値
#[test]
fn stat_rune_random_int_min_equals_max() {
    // Random.int(7, 7) は常に 7 を返す
}

// uniform(5)(5) == 5
#[test]
fn stat_rune_uniform_deterministic() {
    // stat.uniform(5)(5) == 5
}

// choice_str 単一要素
#[test]
fn stat_rune_choice_str_single() {
    // stat.choice_str(["only"]) == "only"
}

// list_int 長さ検証
#[test]
fn stat_rune_list_int_length() {
    // stat.list_int(4) の長さが 4
}

// profile_int total
#[test]
fn stat_rune_profile_int_total() {
    // profile_int([1,2,3]) の total が 3
}

// sample_bool は Bool を返す
#[test]
fn stat_rune_sample_bool_returns_bool() {
    // stat.sample_bool() の戻り値が Bool
}
```

---

## Phase 8 — ドキュメント・最終確認

- `versions/v2.8.0/langspec.md` を作成
  - `Random.int` / `Random.float` の仕様
  - `!Random` エフェクトの説明
  - 各 stat 関数の API ドキュメント
  - 使い方サンプル
  - 互換性
- `cargo build` 警告ゼロを確認
- `cargo test` 全テスト通過を確認（目標 617 → 625 程度）

---

## テスト数の見込み

v2.7.0 ベースライン: 617

- checker.rs Random 効果テスト: +1
- vm.rs Random.int / Random.float テスト: +2
- driver.rs stat 統合テスト: +6
- 目標: **626**（+9 程度）

---

## 注意点

### `rand::thread_rng()` のスレッド安全性

`rand::thread_rng()` はスレッドローカルのため安全。
並列テスト実行中でも独立した RNG インスタンスを使う。

### `list_int` / `list_float` のエフェクト伝播

`List.map(List.range(0, n), |_| Random.int(0, 100))` の戻り型は
`List<Int>` だが、ラムダ内で `!Random` エフェクトを使う。
チェッカーがエフェクトを正しく伝播させるかを確認する。
問題がある場合は `List.fold` + `List.concat` パターンに切り替える。

### `profile_int` の実装複雑度

`min_v` / `max_v` の計算に `List.fold` + `Option` パターンが必要。
実装が難しい場合はスタブ（`min_v: 0  max_v: 0`）を使い、
`total` だけを正確に返す実装でよい。

### `choose_str` / `choice_int` の空リスト

`Random.int(0, -1)` を呼ぶと `min > max` になる可能性がある。
実装では `List.length(xs) == 0` の場合にデフォルト値を返すガードを入れる。

```favnir
public fn choice_str(xs: List<String>) -> String !Random = {
    if List.length(xs) == 0 {
        ""
    } else {
        bind i <- Random.int(0, List.length(xs) - 1)
        Option.unwrap_or(List.first(List.drop(xs, i)), "")
    }
}
```
