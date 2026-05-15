# Favnir v3.5.0 Specification

## Theme: `gen` rune — 型駆動データ生成

v3.5.0 は Favnir の型定義から合成データを自動生成する `gen` rune を追加する。

**「型定義そのものがデータ仕様書になる」** という考え方を実現する。
`fav infer`（v3.4.0）で生成した型定義を渡すだけで、
実データなしに PoC 用の合成データを即生成できる。

`gen` rune は `runes/gen/gen.fav` として **Favnir で実装** する。
新しい Rust モジュールは追加しない — 必要な低レベルプリミティブは
VM ビルトイン（`Gen.*`）として最小限追加する。

---

## 1. 新規型

### 1.1 `GenProfile`

```favnir
type GenProfile = {
    total:   Int
    valid:   Int
    invalid: Int
    rate:    Float
}
```

`gen.profile` の戻り値。型スキーマに対するデータの適合率を表す。

---

## 2. 新規 VM プリミティブ

### 2.1 `Random.seed`

```
Random.seed(n: Int) -> Unit
```

グローバルな RNG シードを固定する。`Random.seed(42)` の後に行う全ての
`Random.*` / `Gen.*` 呼び出しは deterministic になる。

### 2.2 `Gen` VM プリミティブ

| プリミティブ | シグネチャ | 説明 |
|-------------|-----------|------|
| `Gen.string_val` | `Int -> String !Random` | ランダムな英数字列（指定長） |
| `Gen.one_raw` | `String -> Map<String, String> !Random` | 型名からランダム行を1件生成 |
| `Gen.list_raw` | `(String, Int) -> List<Map<String, String>> !Random` | 型名から N 件生成 |
| `Gen.simulate_raw` | `(String, Int, Float) -> List<Map<String, String>> !Random` | ノイズ混入生成 |
| `Gen.profile_raw` | `(String, List<Map<String, String>>) -> GenProfile` | 型適合率を計測 |

`Gen.one_raw(type_name)`:
1. `impl Gen for T` があればその `generate` メソッドを呼ぶ
2. なければ type_metas（コンパイル時に埋め込まれた型スキーマ）から各フィールドの型を読み、
   適切なランダム値を生成して `Map<String, String>` として返す

型別の自動生成ルール:

| Favnir 型 | 生成値 |
|----------|--------|
| `Int` | `Random.int(-1000, 1000)` |
| `Float` | `Random.float()` |
| `Bool` | `Random.int(0, 1) == 1` → `"true"` / `"false"` |
| `String` | `Gen.string_val(8)` — 8文字の英数字列 |
| `Option<T>` | 50% の確率で空文字列、50% で T の値 |

`Gen.simulate_raw(type_name, n, noise)`:
- `noise` は 0.0〜1.0 の汚損率（例: `0.1` → 10% のフィールドを意図的に壊す）
- 破損パターン: 空文字列・型不一致値（"NaN" / "NULL"）・範囲外整数 など
- seed は呼び出し前に `Random.seed` で設定する

`Gen.profile_raw(type_name, data)`:
- 各行の各フィールドが型スキーマに適合するかチェック
- `Int` フィールド: `parse::<i64>()` が成功するか
- `Float` フィールド: `parse::<f64>()` が成功するか
- `Bool` フィールド: `"true"` / `"false"` か
- `String` フィールド: 常に valid
- `Option<T>` フィールド: 空文字列または T に適合する値

---

## 3. `Gen` interface

`runes/gen/gen.fav` 内で定義する:

```favnir
// Gen interface — generate = Unit -> Self !Random
interface Gen {
    generate: Unit -> Map<String, String> !Random
}
```

> `Self` の代わりに `Map<String, String>` を使う。ユーザーは `Schema.adapt_one` で
> 型付きレコードに変換する（v3.2.0 の資産を再利用）。

---

## 4. `runes/gen/gen.fav` — 公開 API

```favnir
// 低レベル生成
public fn int_val(min: Int, max: Int) -> Int !Random {
    Random.int(min, max)
}
public fn float_val() -> Float !Random {
    Random.float()
}
public fn bool_val() -> Bool !Random {
    Random.int(0, 1) == 1
}
public fn string_val(len: Int) -> String !Random {
    Gen.string_val(len)
}

// リストからランダム選択
public fn choice(items: List<String>) -> Option<String> !Random {
    if List.length(items) == 0 {
        Option.none()
    } else {
        bind idx <- Random.int(0, List.length(items) - 1)
        List.nth(items, idx)
    }
}

// 型名から生成
public fn one(type_name: String) -> Map<String, String> !Random {
    Gen.one_raw(type_name)
}
public fn list(type_name: String, n: Int, seed: Int)
    -> List<Map<String, String>> !Random {
    Random.seed(seed)
    Gen.list_raw(type_name, n)
}

// ノイズ混入生成
public fn simulate(type_name: String, n: Int, noise: Float, seed: Int)
    -> List<Map<String, String>> !Random {
    Random.seed(seed)
    Gen.simulate_raw(type_name, n, noise)
}

// データプロファイリング
public fn profile(type_name: String, data: List<Map<String, String>>)
    -> GenProfile {
    Gen.profile_raw(type_name, data)
}
```

---

## 5. `impl Gen for T` — カスタム生成ロジック

ユーザーは `impl Gen for T` でビジネスルールを反映した生成ロジックを定義できる:

```favnir
import rune "gen"

type UserRow = {
    id:     Int
    name:   String
    email:  String
    age:    Int
    region: String
}

impl Gen for UserRow {
    generate = |_| {
        bind age    <- gen.int_val(18, 80)
        bind region <- Option.unwrap_or(gen.choice(["EU", "US", "JP", "APAC"]), "EU")
        bind name   <- Option.unwrap_or(gen.choice(["Alice", "Bob", "Carol", "Dave"]), "Alice")
        {
            "id"     : Int.to_string(gen.int_val(1, 999999))
            "name"   : name
            "email"  : $"{name}@example.com"
            "age"    : Int.to_string(age)
            "region" : region
        }
    }
}
```

`impl Gen for T` がある場合、`gen.one("UserRow")` はその `generate` メソッドを呼ぶ。

---

## 6. `fav check --sample N`

```bash
fav check pipeline.fav --sample 100
```

1. `pipeline.fav` の `stage`/`seq` ブロックが受け取る最初の型を推定
2. `Gen.list_raw(type_name, N)` で N 件の合成データを生成
3. パイプラインを合成データで試し実行
4. 型エラー・ランタイムエラーがあれば報告する

→ 実データなしでパイプラインの型安全性を確認できる。

```
$ fav check pipeline.fav --sample 100
Generating 100 synthetic rows for type 'UserRow'...
Running pipeline with synthetic data...
  ok: all 100 rows processed without errors
  (use --sample to test with real data for integration verification)
```

---

## 7. 利用例

### 基本的な合成データ生成

```favnir
import rune "gen"

type User = {
    id:   Int
    name: String
    age:  Int
}

public fn main() -> Unit !Io !Random {
    bind users <- gen.list("User", 5, 42)
    for row in users {
        IO.println($"id={Map.get(row, \"id\")} name={Map.get(row, \"name\")}")
    }
}
```

### ノイズ混入でクレンジングパイプライン検証

```favnir
import rune "gen"
import rune "csv"

public fn main() -> Unit !Io !Random {
    // 1000件のうち10%が汚損
    bind dirty <- gen.simulate("User", 1000, 0.1, 42)

    // プロファイルして汚損率確認
    bind report <- gen.profile("User", dirty)
    IO.println($"Valid: {report.valid}/{report.total} ({report.rate})")

    // 型適合する行だけ抽出（クレンジング）
    bind clean <- gen.profile("User", dirty)  // 実際には Schema.adapt でフィルタ
    IO.println($"Clean rate: {clean.rate}")
}
```

### `fav infer` → `gen` の連携

```bash
# Step 1: 既存 CSV から型定義を生成
fav infer customers.csv --out schema/customer.fav

# Step 2: 型定義を編集して impl Gen を追加

# Step 3: 合成データで PoC を実行
fav run poc_pipeline.fav  # gen.list("Customer", 1000, seed: 42) を内部で使用
```

---

## 8. 完了条件

- `gen.one("T")` で全フィールドが型スキーマを持つ型のランダム行を生成できる
- `gen.list("T", N, seed)` で seed 固定の deterministic な大量生成ができる
- `impl Gen for T` でカスタム生成ロジックを定義できる
- `gen.choice(items)` でリストからランダム選択できる
- `gen.simulate("T", N, noise, seed)` でノイズ混入データを生成できる
- `gen.profile("T", data)` で GenProfile を返す
- `runes/gen/gen.fav` は Rust コードを一行も含まない
- `Random.seed(n)` で deterministic な実行ができる
- `fav check --sample N` でパイプライン型安全性を合成データで確認できる
- 既存テストが全て通る

---

## 9. 非ゴール（v3.5.0 スコープ外）

- `Gen.one<T>()` の型パラメータ構文（現在は文字列 `"T"` で指定）
- `Gen.seed` をネストした関数スコープで管理する PRNG スタック
- Faker ライブラリ相当の名前・住所・メール生成（`Gen.string_val` で代替）
- `Gen.shrink<T>` — プロパティベーステスト用の縮小戦略
- `fav check --sample N` でのカバレッジレポート統合
