# Favnir v9.7.5 Spec

Date: 2026-06-02
Theme: 名目型ラッパー完成 — `where` バリデーター + `with` 解析 + E0013

---

## 概要

v9.7.0 で導入した `type Name(Inner)` 名目型ラッパーを実際に使えるレベルに仕上げる。

主な追加機能:

1. **`where` バリデーター** — `type Percent(Float) where |v| v >= 0.0 && v <= 100.0`
   コンストラクタが `Percent(x) -> Result<Float, String>` になり、述語が偽の場合はエラーを返す。

2. **`with` 解析** — `type UserId(Int) with Serialize, Deserialize`
   `with` 節を `with_impls` に保存する（自動合成は v9.8.0 で実装）。

3. **E0013 — QuestionOutsideResult** — `expr?` が Result 以外の式（Option 等）に適用された場合に
   checker.fav がコンパイルエラーを返す。

4. **checker.fav: `where` あり型のコンストラクタ型を更新** — `has_where: true` のとき、
   コンストラクタを `Inner -> Result<Name, String>` として型環境に登録する。

---

## 機能詳細

### 1. `where` バリデーター

#### 構文

```favnir
type Percent(Float) where |v| v >= 0.0 && v <= 100.0
type Username(String) where |s| String.length(s) > 0 && String.length(s) <= 32
type PositiveInt(Int) where |n| n > 0
```

#### 実行時動作（compiler.fav が生成するコード）

`where` 節を持つラッパー型は、通常のバリアントコンストラクタの代わりに
**バリデーター関数**を生成する:

```favnir
// type Percent(Float) where |v| v >= 0.0 && v <= 100.0 → 以下を生成:
fn Percent(v: Float) -> Result<Float, String> {
    if v >= 0.0 && v <= 100.0 {
        Result.ok(v)
    } else {
        Result.err("Percent: validation failed")
    }
}
```

コンストラクタ関数名は型名と同じ（`Percent`）。VM の `fn_idx_by_name` が
`looks_like_variant_ctor` より先に検索されるため、この関数が優先される。

#### 使用例

```favnir
type Percent(Float) where |v| v >= 0.0 && v <= 100.0

fn apply_discount(price: Float, pct: Float) -> Result<Float, String> {
    match Percent(pct) {
        Err(e) => Result.err(e)
        Ok(p)  => Result.ok(price * p / 100.0)
    }
}

public fn main() -> String {
    match apply_discount(1000.0, 20.0) {
        Ok(discounted) => Float.to_string(discounted)
        Err(e)         => e
    }
}
// → "200.0"

match Percent(150.0) {
    Err(msg) => msg  // → "Percent: validation failed"
    Ok(p)    => Float.to_string(p)
}
```

#### `where` なし型との違い

| 型定義 | コンストラクタ動作 | 戻り値型 |
|---|---|---|
| `type UserId(Int)` | VM が自動でバリアント化 | `UserId`（= `Int` at runtime） |
| `type Percent(Float) where ...` | compiler.fav が検証関数を生成 | `Result<Float, String>` |

#### 内部表現（実行時）

`where` あり・なしどちらの場合も、実行時の値はそのまま内部型の値（boxing なし）。
ラッパー型による区別は型チェック時のみ行われる。

---

### 2. `with` 節の解析

```favnir
type UserId(Int) with Serialize, Deserialize
type Email(String) with Serialize
type Percent(Float) where |v| v >= 0.0 && v <= 100.0 with Serialize
```

v9.7.5 では構文解析のみ実装し、`with_impls` に文字列リストとして保存する。
自動合成（`fn UserId_to_json(...)` 等の生成）は **v9.8.0** で実装。

`with Iface` が未知のインターフェース名でもエラーにしない（警告のみ / 将来の E0011 用）。

---

### 3. E0013 — QuestionOutsideResult

```favnir
fn try_double(n: Int) -> Result<Int, String> { Result.ok(n * 2) }
fn get_option() -> Option<Int> { Option.some(42) }

fn main_ok() -> Int {
    try_double(5)?   // OK — inner type infers to "Result..."
}

fn main_bad() -> Int {
    get_option()?    // E0013: ? requires a Result expression, got Option<Int>
}
```

**実装方針（checker.fav）:**

`infer_hm` の `EQuestion(inner)` ケースで:
1. `inner` の型を推論 (`infer_expr(inner, env)`)
2. 推論型が `"Result"` で始まらない場合 → `Result.err("E0013: ...")`
3. 始まる場合 → `Result.ok("Unknown")`（アンラップ後の型は現状 Unknown）

---

### 4. checker.fav: `has_where` コンストラクタ型

`collect_variant_constructors` の `IWrapper` ハンドラを更新:

```
has_where = false  →  env_insert(env, Name, "Inner -> Name")
has_where = true   →  env_insert(env, Name, "Inner -> Result<Name, String>")
```

これにより、`where` あり型のコンストラクタ呼び出しは
`Result<Name, String>` を返すと型チェッカーが認識する。

---

## スコープ外（延期）

- `T!` 型後置 — エフェクト注釈 `!IO` との構文的曖昧性を解決してから実装（v9.8.0）
- `with` 自動合成（`Serialize`/`Deserialize`/`Show`/`Eq`）— v9.8.0
- E0010 (WrapperTypeMismatch) — checker.fav の型推論強化後に実装
- E0011 (UnknownInterface) — v9.8.0 で `with` 合成と同時に実装

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `type Name(Inner) where \|v\| pred` で `Name(valid)` → `Ok(inner)` | |
| `type Name(Inner) where \|v\| pred` で `Name(invalid)` → `Err("Name: validation failed")` | |
| `type Name(Inner) with Iface` が parse エラーなし | |
| `expr?` on Option → E0013 | |
| checker.fav で `where` あり型のコンストラクタが `Result` 型として認識される | |
| self-check 通過 | |
| bootstrap 維持 | |
| 全テスト通過（目標: 1205 件以上） | |
