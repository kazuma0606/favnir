# Favnir Algebraic Structures & Mathematical Integrity

日付: 2026-05-02

## 概要

Favnir は、データ解析の正当性を数学的基盤（Algebraic Integrity）の上に構築する。
抽象化システム（`interface`）を利用して、群、環、体などの代数構造を定義し、演算子と結びつけることで、厳密かつ汎用的な計算モデルを提供する。

---

## 1. 数学的な演算子の解放 (Operator Overloading)

特定の代数インターフェースを `impl` することで、対応する演算子が利用可能になる。

| インターフェース | 必要メソッド | 演算子 | 性質 |
|:---|:---|:---|:---|
| `Semigroup` | `combine` | `+` | 結合法則 |
| `Monoid` | `empty` | (zero) | 単位元の存在 |
| `Group` | `inverse` | `-` (unary) | 逆元の存在 |
| `Ring` | `multiply` | `*` | 分配法則 |
| `Field` | `divide` | `/` | 除法 (戻り値は `T!`) |

### 例: 複素数の実装
```fav
type Complex { re: Float, im: Float }

impl Ring for Complex {
    combine  = |a, b| Complex { re: a.re + b.re, im: a.im + b.im }
    multiply = |a, b| Complex { 
        re: a.re * b.re - a.im * b.im,
        im: a.re * b.im + a.im * b.re 
    }
}
```

---

## 2. ジェネリックな統計・最適化アルゴリズム

アルゴリズムを具体型（`Float` 等）ではなく `interface` に対して記述することで、再利用性を極大化する。

Favnir は「暗黙の型クラス解決」を行わない。代わりに、`interface` の実装値を**明示的な引数**として渡すことで、どの演算規則を使うかをコードから明確に読み取れるようにする。

```fav
-- Field<T> の実装値を明示的に受け取ることで、加重平均をジェネリックに記述
stage WeightedAverage<T>(items: List<(T, Float)>, field: Field<T>) -> T! = |items, field| {
    bind sum_val <- items |> List.map(|(v, w)| field.multiply(v, w)) |> List.fold(field.empty, field.combine)
    bind sum_w   <- items |> List.map(|(_, w)| w) |> List.fold(0.0, `+`)
    field.divide(sum_val, sum_w)
}

-- 呼び出し側: どの Field 実装を使うかが一目でわかる
bind result <- WeightedAverage(data, Complex.field)
```

### なぜ `where T: Field` ではないのか

Haskell の型クラスや Rust のトレイト境界（`where T: Field`）は、コンパイラが実装を暗黙的に解決する。
Favnir はこれを採用しない。理由：

1. **可読性**: `field.combine` が何を呼んでいるかが呼び出しコードから明確。
2. **AI フレンドリー**: 生成 AI がコードを読む際、依存している演算規則の出所が明示されている。
3. **CoC 一貫性**: 型安全性のための情報は型シグネチャに集約する。暗黙の解決は「隠れた依存」を生む。

---

## 3. 微分可能なパイプライン (Differentiable Programming) [仮説段階]

> **注意**: このセクションは設計仮説であり、v2.0.0 以降の研究課題である。

### 背景にある仮説

`stage` の連鎖（`seq`）が以下の条件を満たす場合、数学的に自動微分が可能になる：

1. 各 `stage` の入出力型が `Field` の演算で構成されている（加算・乗算で閉じている）
2. 型の不連続な変換（例: `Float -> Bool` の分岐）が存在しない
3. `stage` の本体が副作用のない純粋な計算である（`!Pure`）

これらの条件下では、`seq` 全体を一つの微分可能な関数と見なすことができ、**勾配（gradient）を自動計算して入力パラメータを最適化**できる。

### 設計方向

```fav
-- 各 stage が Float -> Float の滑らかな変換であれば...
seq SmoothTransform: Float -> Float !Pure {
    stage Normalize: Float -> Float = |x| (x - mean) / std
    stage Scale:     Float -> Float = |x| x * weight
    stage Shift:     Float -> Float = |x| x + bias
}

-- ...seq 全体を最適化の対象にできる
bind optimized <- Optimizer.minimize(SmoothTransform, loss_fn, learning_rate: 0.01)
```

### `interface Differentiable`

```fav
interface Differentiable<T> {
    -- stage を微分した勾配関数を返す
    grad: (T -> T) -> (T -> T)
}
```

### 現時点での限界と課題

| 課題 | 現状 |
|:---|:---|
| `match` / 条件分岐を含む stage | 微分不可（コンパイラが W005 を出す予定） |
| `!Db` / `!Io` などの副作用を持つ stage | 微分対象から除外される |
| `String` 型を含む中間型 | `Field` を持たないため微分不可 |
| 具体的な自動微分アルゴリズム | 前向き微分 (Forward AD) vs 逆向き微分 (Reverse AD) の選択が未定 |

コンパイラは「この `seq` は微分可能か」を静的にチェックし、不可能な場合は早期エラーを報告する設計を目指す。

---

## 4. メリット：数学と実務の融合

1.  **自動並列化**: `Monoid`（結合法則）が保証されていれば、大規模データの集計を安全に分割・並列実行できる。
2.  **型安全な最適化**: 「滑らかでない（微分不可能）」な箇所をコンパイラが指摘し、適切な最適化手法を提案する。
3.  **高信頼な疑似データ生成**: `Field` の性質を利用して、任意のカスタム型に対して正規分布などの統計的サンプリング（`Stat.sample`）を自動適用できる。
