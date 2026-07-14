# v41.8.0 実装計画 — Type Precision cookbook

## 前提（確認済み）

- `site/content/cookbook/refinement-types.mdx` は存在しない → 新規作成
- `site/content/docs/language/refinement-types.mdx` は存在する（92 行）→ 末尾に追加
- 既存 cookbook の frontmatter 形式: `---\ntitle: "..."\ndescription: "..."\n---`
- v41700_tests の `cargo_toml_version_is_41_7_0` をスタブ化する
- テスト数は 2867 + 1 = 2868（ロードマップ推定と一致）

---

## 実装ステップ

### Step 1: `site/content/cookbook/refinement-types.mdx` 新規作成

```markdown
---
title: "Refinement Type でドメイン制約を型に刻む"
description: "type alias の where 節で値の不変条件を型レベルで表現する実用パターン"
---

# Refinement Type でドメイン制約を型に刻む

## 問題

データパイプラインでは「年齢は 0 以上」「金額は正」「名前は空でない」といった制約が
コード中の if ガードや assert として散在しがちです。これらの制約を型に刻むことで、
関数シグネチャが仕様書になり、冗長なバリデーションが不要になります。

## Refinement Type Alias

`where |v| pred` 節を使って型レベルの invariant を定義します:

​```favnir
type Age     = Int   where |v| v >= 0 && v <= 150
type USD     = Float where |v| v >= 0.0
type Name    = String where |v| String.length(v) > 0
type UserId  = String where |v| String.length(v) == 36  // UUID 形式
​```

## 実用例: ユーザープロフィール

​```favnir
type Age  = Int   where |v| v >= 0 && v <= 150
type Name = String where |v| String.length(v) > 0

type UserProfile = {
    name: Name
    age:  Age
    bio:  String
}

fn greet(user: UserProfile) -> String {
    String.concat("Hello, ", user.name)
    // user.name は Name 型 — String.length > 0 が保証済み
}
​```

## W030: 冗長ガードを書かない

refinement 型の invariant は型システムが保証しているため、同一条件の if ガードは不要です。
Favnir の W030 lint が検出します:

​```favnir
type PositiveInt = Int where |v| v >= 0

fn double(x: PositiveInt) -> Int {
    if x >= 0 {      // W030: redundant guard — PositiveInt invariant が既に保証している
        x * 2
    } else { 0 }
}

// 正しい書き方
fn double_ok(x: PositiveInt) -> Int {
    x * 2   // W030 なし — 不要なガードを削除
}
​```

## Newtype との組み合わせ

現時点では `type Kg(Float)` の Newtype と refinement type alias を同一型に組み合わせることは
未対応です（将来バージョンで対応予定）。別々に使い分けることができます:

​```favnir
// alias: 値の制約（refinement）
type Weight = Float where |v| v >= 0.0

// newtype: 単位の区別（算術自動委譲）
type Kg(Float)

fn total(a: Weight, b: Weight) -> Weight {
    a + b   // Float の算術演算が適用される
}
​```

## まとめ

| パターン | 用途 |
|---|---|
| `type Age = Int where \|v\| v >= 0` | 値の不変条件を型に表現 |
| W030 lint | 冗長な if ガードを検出 |
| Newtype `type Kg(Float)` | 型の意味（単位）を区別 |
```

---

### Step 2: `site/content/docs/language/refinement-types.mdx` 末尾に追加

ファイル末尾（92 行目の後）に以下を追加:

```markdown

## Type Alias Refinement（v41.1.0+）

`type` alias に `where |v| pred` 節を付けることで、値の不変条件を型レベルで定義できる:

​```fav
type PositiveInt = Int    where |v| v >= 0
type Name        = String where |v| String.length(v) > 0
​```

- `|v|` はクロージャ形式: `v` が型の値を表す変数
- 条件式には `&&` / `||` / 比較演算子を使える
- パラメータ refinement（`fn f(x: Int where { x > 0 })`）とは別機能

## W030: 冗長ガード lint（v41.7.0+）

refinement 型の invariant が既に保証している条件を if で再確認すると W030 警告が出る:

​```fav
type PositiveInt = Int where |v| v >= 0

fn double(x: PositiveInt) -> Int {
  if x >= 0 {  // W030: redundant guard
    x * 2
  } else { 0 }
}
​```

W030 を解消するには冗長な if ガードを削除する。refinement 型が invariant を保証している。
```

---

### Step 3: driver.rs テストモジュール更新

- `v41700_tests::cargo_toml_version_is_41_7_0` をスタブ化
- `v41800_tests` モジュール（1 テスト）を末尾に追加

---

### Step 4: Cargo.toml バージョン bump

`version = "41.7.0"` → `"41.8.0"`

---

### Step 5: CHANGELOG.md 更新

```markdown
## [v41.8.0] — 2026-07-11

### Added
- `site/content/cookbook/refinement-types.mdx` — Type Precision cookbook: refinement type alias + W030 lint 実用パターン
- `site/content/docs/language/refinement-types.mdx` に Type Alias Refinement（v41.1.0+）と W030 lint（v41.7.0+）セクションを追加
```

---

## 実装順序

1. `site/content/cookbook/refinement-types.mdx` 新規作成
2. `site/content/docs/language/refinement-types.mdx` 末尾に追加
3. `driver.rs` テスト追加（`v41700_tests` スタブ化 + `v41800_tests` 追加）
4. `Cargo.toml` バージョン bump（Step 3 の `cargo_toml_version_is_41_8_0` テストが通るよう bump が先でも可だが、同時実行で問題なし）
5. `CHANGELOG.md` 更新
6. `cargo test` 実行・確認
