# Favnir Standard States & Reusable Invariants

日付: 2026-05-02

## 概要

Favnir では、`Int` や `String` といったプリミティブな型をそのまま使うのではなく、ビジネスルールに基づいた意味を持つ型（**State**）として定義することを推奨する。

いちいち `invariant` を自前で書かなくても、一般的によく使われる制約を **「標準 State（rune）」** として提供することで、開発者は「型を選ぶだけ」でバリデーションを完了できる。

---

## 1. `std.states` ルーンの提供

標準ライブラリとして `std.states` を提供し、再利用可能な Invariant 付き型を集約する。

### 基本的な数値型
- `PosInt`: 正の整数 (`> 0`)
- `NonNegInt`: 0 以上の整数 (`>= 0`)
- `Probability`: 0.0 以上 1.0 以下の実数
- `PortNumber`: 1 〜 65535 の整数

### 基本的な文字列型
- `Email`: メールアドレス形式
- `Url`: URL 形式
- `NonEmptyString`: 空文字でない文字列
- `Slug`: 英数字とハイフンのみの文字列

---

## 2. 構文案：型の合成と Invariant の適用

既存の型に制約を「その場」で追加する構文や、標準 Invariant を合成する手段を検討する。

### 案A: 型コンストラクタによる適用
```fav
use std.states.PosInt

bind count: PosInt <- 10  -- 直接指定
```

### 案B: パイプラインによる変換（バリデーション）
以前議論した「Safe Cast」と統合し、変換工程として扱う。

```fav
chain age <- input_age |> Int.to_pos  -- Result<PosInt, E> を返す
```

---

## 3. 型推論と Invariant の連携 (Human-in-the-Loop)

型ホール `_` を使った際、LSP が値のコンテキストから最適な「State（Invariant 付き型）」を提案する。

```fav
bind score: _ <- 95
```

### LSP の挙動
1. 95 は `Int` である。
2. しかし、この関数の引数名や用途から `std.states.Probability` (0-100等) や `PosInt` が適合する可能性があると AI が判断。
3. **提案**: `_` を `PosInt` に置換しますか？
4. 人間が承諾すると、コードが `bind score: PosInt <- 95` に固定される。

これにより、ただの「数値」が「意味のあるデータ」へと昇格する。

---

## 4. Invariant の合成 (Composition)

複数の Invariant を組み合わせて新しい型を定義する。

```fav
type UserAge = Int 
    with Invariant.min(0) 
    with Invariant.max(150)

-- または
type AdultAge = std.states.PosInt 
    with Invariant.min(18)
```

---

## 5. メリット：データ解析基盤としての強み

この「標準 State」群が整備されることで、以下のことが可能になる。

1. **自動ドキュメント化**: `fav explain` で「このパイプラインは `Email` を受け取り `PosInt` を出す」ことが保証されていることが一目でわかる。
2. **テストデータの自動生成**: `Stat.one<Email>()` は、わざわざ正規表現を書かなくても「正しいメールアドレス」をランダム生成できる。
3. **DB スキーマとの同期**: `PosInt` 型のフィールドは、DB 出力時に自動的に `CHECK (value > 0)` 制約として書き出される。

---

## 一言でいうと

> バリデーションコードを書く時代を終わらせ、
> 正しい「State（状態）」を選択するだけでデータが守られる世界を作る。
