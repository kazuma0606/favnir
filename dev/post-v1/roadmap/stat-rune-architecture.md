# Stat Rune Architecture: Synthesis and Inference

日付: 2026-05-03

## 概要

`stat` ルーンは、Favnir における数学的・統計的処理の中心的役割を担う。
単なる「乱数生成器」ではなく、型（State）と不変条件（Invariant）を軸とした **「双方向の統計エンジン」** として設計する。

1.  **生成 (Synthesis/Forward)**: 型と Invariant から、数学的性質を満たすデータを生成する。
2.  **検証・推論 (Verification/Inference/Backward)**: 実データと Invariant を照合し、データの性状や品質を統計的に推論する。

> **統合方針（2026-05-03 決定）**:
> `random` / `sample` を独立ルーンとする案は採用しない。
> プリミティブ生成から型駆動生成まで、すべて `Stat.*` の名前空間に統合する。
> 使用する側は `Stat` 一つを `use` するだけでよい。

---

## API 全体構造

```
Stat
 ├─ 0. プリミティブ生成  (Stat.int / Stat.float / Stat.bool / Stat.string / Stat.choice)
 ├─ 1a. 分布駆動生成     (Stat.normal / Stat.uniform / Stat.t / ...)
 ├─ 1b. 型駆動生成       (Stat.one<T> / Stat.list<T> / Stat.rows<T>)
 ├─ 1c. シミュレーション  (Stat.simulate<T>)
 ├─ 2a. プロファイリング  (Stat.profile<T>)
 ├─ 2b. サンプリング      (Stat.sample / Stat.sample_outliers)
 └─ 共通: seed 制御       (Stat.seed / seed: パラメータ)
```

---

## 0. プリミティブ生成（旧 `random` 相当）

低レベルな決定論的乱数生成。型を意識せず、単純な値が欲しい場合に使う。

```fav
use stat as Stat

-- 整数・実数・真偽値
bind n    <- Stat.int(min: 0, max: 100, seed: 42)    -- Int
bind f    <- Stat.float(min: 0.0, max: 1.0, seed: 42) -- Float
bind flag <- Stat.bool(seed: 42)                       -- Bool

-- 文字列（指定長のランダム ASCII 文字列）
bind s <- Stat.string(len: 8, seed: 42)   -- String

-- リストからランダム選択
bind item <- Stat.choice(["a", "b", "c"], seed: 42)  -- String

-- シード固定のジェネレータ（複数値を順次生成）
bind gen   <- Stat.generator(seed: 42)
bind n1    <- gen.next_int(0, 100)
bind n2    <- gen.next_int(0, 100)
```

**使い所**: テスト用の単純な値、ブレークポイント確認、ID 生成など。

---

## 1. 生成 (Synthesis): 期待を形にする

### 1-1. 分布駆動生成 (Distribution-driven)

統計的分布に基づいて数値を生成する。

```fav
bind x  <- Stat.normal(mean: 0.0, stddev: 1.0, seed: 42)      -- Float
bind y  <- Stat.uniform(min: 0.0, max: 10.0, seed: 42)         -- Float
bind xs <- Stat.normal_list(mean: 0.0, stddev: 1.0, n: 1000, seed: 42)

-- カスタム型に適用（Field<T> を明示的に渡す）
bind vals <- Stat.normal_field(mean: Complex.zero, stddev: 1.0, field: Complex.field, n: 100, seed: 42)
```

将来的には `Stat.t`, `Stat.chi_square`, `Stat.poisson` 等の分布も追加予定。

### 1-2. 型駆動生成 (Type-driven / Forward)

型定義と `invariant` から「もっともらしい」データを自動生成する。
生成には `Gen` interface が必要（`impl Gen` で自動合成可能）。

```fav
type UserRow with Gen, Show, Eq {
    name:  NonEmptyString
    email: Email          -- @を含む、255文字以内の invariant が自動的に守られる
    age:   PosInt         -- 0以上の invariant が自動的に守られる
}

-- 1件生成
bind user <- Stat.one<UserRow>(seed: 42)

-- N件のリスト生成
bind users <- Stat.list<UserRow>(1000, seed: 42)

-- CSV 行として生成（Csv interface も必要）
bind rows <- Stat.rows<UserRow>(500, seed: 7)
```

`invariant` を満たさない値は生成されない。
型の制約がそのままテストデータの品質保証になる。

### 1-3. シミュレーション (Simulation)

モデルに基づいた大規模・条件付きデータ生成。

```fav
-- シナリオを定義して生成（モンテカルロ法など）
bind rows <- Stat.simulate<UserRow>(
    count: 10_000,
    model: UserRowScenario,   -- 生成ルールを持つ値
    seed: 42
)

-- 異常値を意図的に混入させたデータ
bind dirty_rows <- Stat.simulate<UserRow>(
    count: 1000,
    model: UserRowScenario,
    noise: 0.05,   -- 5% の割合で invariant 違反データを混入
    seed: 99
)
```

**使い所**: パイプラインのストレステスト、稀事象の再現、バリデーションロジックの網羅テスト。

---

## 2. 検証・推論 (Inference): 現実を測る

実データを型（Invariant）という「物差し」で測り、その性状を明らかにする。

### 2-1. 統計的プロファイリング (Statistical Profiling)

```fav
bind report <- real_data |> Stat.profile<UserRow>

-- report: ProfileReport
-- {
--   total: Int
--   per_field: Map<String, FieldProfile>
--     -- FieldProfile: { null_rate, invariant_pass_rate, min, max, mean, stddev }
--   outliers: List<UserRow>
-- }
```

**Invariant をセンサーとして活用**: 各 invariant の適合率、外れ値の分布、欠損率を集計。
「このパイプラインは Email 型を前提にしているが、実データの 3% がその invariant を満たさない」
といった傾向のズレを自動で報告する。

### 2-2. データの性状推論

```fav
-- 型の期待値と実データのズレを「距離」として算出
bind drift <- Stat.drift<UserRow>(expected: Stat.profile(Stat.list<UserRow>(1000, seed: 0)), actual: real_data)
-- drift.score が大きいほど、実データが型の想定から遠い
```

### 2-3. インテリジェント・サンプリング

```fav
-- 大規模データから N 件をランダムにサンプリング
bind sample <- real_data |> Stat.sample(n: 100, seed: 42)

-- Invariant を満たさない「問題のある」データだけを抽出
bind outliers <- real_data |> Stat.sample_outliers<UserRow>(n: 50)

-- 分布の端（上位/下位 5%）を重点的に抽出
bind edge_cases <- real_data |> Stat.sample_edges<UserRow>(n: 50, ratio: 0.05)
```

---

## 3. `Gen` interface: 型駆動生成の仕組み

`Stat.one<T>` / `Stat.list<T>` が動作するには、型 `T` が `Gen` interface を実装している必要がある。

```fav
interface Gen {
    gen: Int? -> Self    -- seed を受け取り値を生成
}

-- 自動合成（全フィールドが Gen を持てば合成可能）
impl Gen for UserRow

-- 手書き実装（カスタムロジックが必要な場合）
impl Gen for Priority {
    gen = |seed| {
        bind n <- Stat.int(0, 3, seed: seed)
        match n {
            0 => Priority.Low
            1 => Priority.Medium
            _ => Priority.High
        }
    }
}
```

`impl Gen`（本体なし）は「全フィールドが Gen を持つ」場合のみ有効。
フィールドに `Gen` を持たない型が含まれる場合、コンパイルエラーになる。

---

## 4. なぜ `stat` が Favnir の核心なのか

Favnir は「正しさ」の言語であるが、現実のデータは常に「不確実性」を伴う。
`stat` ルーンはこの両者を繋ぐ架け橋となる。

| フェーズ | `stat` の役割 |
|---|---|
| **開発時** | `Stat.one<T>` / `Stat.list<T>` でパイプラインをテスト |
| **QA 時** | `Stat.simulate` で異常値・稀事象を再現 |
| **実行前** | `fav check --sample N` = `Stat.profile` で実データの前提確認 |
| **実行時** | `Stat.profile` / `Stat.drift` で前提（Invariant）の継続監視 |
| **最適化時** | Lineage + `Stat.profile` でボトルネックや誤差伝播を特定 |

---

## Core API まとめ

```fav
-- プリミティブ
Stat.int(min, max, seed?)         -> Int
Stat.float(min, max, seed?)       -> Float
Stat.bool(seed?)                  -> Bool
Stat.string(len, seed?)           -> String
Stat.choice(list, seed?)          -> T
Stat.generator(seed?)             -> Generator

-- 分布
Stat.normal(mean, stddev, seed?)          -> Float
Stat.normal_list(mean, stddev, n, seed?)  -> List<Float>
Stat.uniform(min, max, seed?)             -> Float

-- 型駆動
Stat.one<T>(seed?)                -> T         -- requires Gen
Stat.list<T>(n, seed?)            -> List<T>   -- requires Gen
Stat.rows<T>(n, seed?)            -> List<T>   -- requires Gen + Csv
Stat.simulate<T>(count, model, seed?, noise?) -> List<T>

-- サンプリング
Stat.sample<T>(n, seed?)          -> List<T>   -- (stage: List<T> -> List<T>)
Stat.sample_outliers<T>(n)        -> List<T>   -- (stage: List<T> -> List<T>)
Stat.sample_edges<T>(n, ratio)    -> List<T>

-- 検証・推論
Stat.profile<T>(data)             -> ProfileReport
Stat.drift<T>(expected, actual)   -> DriftReport
```

---

## 結論

`stat` ルーンは、Favnir の **SSS+V (State-Stage-Sequence + Visualise)** アーキテクチャにおける **「数学的な目」** である。

プリミティブ生成から型駆動合成、統計的推論まで `Stat.*` 一つに統合することで:
- 開発者は `use stat as Stat` だけ覚えればよい
- テスト・開発・監視のすべてで同じ API を使い回せる
- `invariant` と `Gen` interface が「正しいデータしか生成されない」を保証する
