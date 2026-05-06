# Favnir 抽象化システム設計

日付: 2026-05-01

## 方針

Favnir の抽象化は「説明可能な抽象化」を原則とする。

- 抽象化しても `fav explain` で全体像が見える
- 暗黙解決はしない（全て明示的に束縛する）
- AI が安全に扱える（曖昧な規則を持ち込まない）

> **Note: 移行について**  
> v0.4.0 で導入された `cap` (Capability) システムは、本設計によって `interface` システムへと完全に置き換えられる。既存の `cap` キーワードおよび関連実装は、段階的に `interface` へ移行・統合される。

---

## キーワード一覧

| キーワード | 対象 | 意味 |
|---|---|---|
| `interface` | 操作の集合 | 型が持てる操作の契約（旧 `cap`） |
| `impl` | 型 × interface | interface の手書き実装 |
| `impl` (本体なし) | 型 × interface | interface の自動合成（型構造から生成） |
| `with` | 型宣言 | interface の自動合成をインラインで宣言 |
| `abstract type` | 型 | 直接インスタンス化できない型テンプレート |
| `abstract stage` | 変換 | 実装なしの変換宣言 |
| `abstract seq` | パイプライン | ステージ構成の抽象化（詳細は別ドキュメント） |
| `invariant` | 型フィールド | 型に埋め込む不変条件 |

---

## 1. `interface`（旧 `cap`）

型が「何ができるか」を宣言する compile-time の契約。
Haskell の typeclass・Rust の trait に相当するが、**暗黙解決はしない**。

### 宣言

```fav
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {           -- Eq を前提にする
    compare: Self -> Self -> Int
}
```

`Ord : Eq` は「Ord を実装するには Eq も必要」を意味する。

### 手書き実装（`impl`）

```fav
impl Show for Int {
    show = |x| Int.to_string(x)
}

impl Eq for UserRow {
    eq = |a b| a.id == b.id
}
```

### 値として渡す

interface は **値として明示的に渡す**。暗黙解決なし。

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> { ... }

bind sorted <- sort(users, User.ord)
-- User.ord は Ord<User> の実装値
```

これにより：
- 呼び出し側で何が使われているか常に明示される
- テストで差し替え可能
- AI が安全に推論できる

---

## 2. `impl`（自動合成）

型の構造から interface の実装をコンパイラが自動生成する。
本体ブロック `{ ... }` を省略した `impl` が自動合成を意味する。

### 手書き実装（本体あり）

```fav
impl Show for Int {
    show = |x| Int.to_string(x)
}
```

### 自動合成（本体なし）

```fav
impl Show, Eq, Json, Csv for UserRow
```

`UserRow` の全フィールドが `Show`・`Eq`・`Json`・`Csv` を持つ場合のみ有効。
フィールドの型が interface を満たさない場合はコンパイルエラー。

### 型宣言へのインライン（`with`）

```fav
type UserRow with Show, Eq, Json, Csv {
    name:  String
    email: String
    age:   Int
}
```

`with` はその場で `impl Show, Eq, Json, Csv for UserRow` を宣言するシンタックスシュガー。

### `Gen` interface と `stat` の接続

`Stat.one<T>()` のような型駆動生成には `Gen` interface が必要。

```fav
interface Gen {
    gen: Int? -> Self    -- seed を受け取り値を生成
}

impl Gen for UserRow   -- フィールドごとに再帰的に Gen を呼ぶ

bind user  <- Stat.one<UserRow>(seed: 42)
bind users <- Stat.list<UserRow>(100, seed: 42)
```

`impl Gen` (本体なし) が書けるのは「全フィールドが Gen を持つ型」のみ。
この制約はコンパイラが検査する。

---

## 3. `abstract type`

直接インスタンス化できない型テンプレート。
具体型が `with` で実装を提供する。

```fav
abstract type Shape {
    area:      Self -> Float
    perimeter: Self -> Float
}

type Circle with Shape {
    radius: Float
}

impl Shape for Circle {
    area      = |c| 3.14 * c.radius * c.radius
    perimeter = |c| 2.0 * 3.14 * c.radius
}
```

`abstract type` と `interface` の違い：

| | `interface` | `abstract type` |
|---|---|---|
| 用途 | 横断的な操作の契約 | 型テンプレート・継承的構造 |
| フィールド | 持てない | 持てる（将来） |
| 複数適用 | 複数 interface を同時に持てる | 1つの abstract type を継承 |

---

## 4. `abstract stage`

実装を持たない変換宣言。シグネチャだけを確定し、実装は外部から注入する。

```fav
abstract stage FetchUser:  UserId     -> User?    !Db
abstract stage SaveUser:   User       -> UserId   !Db
abstract stage NotifyUser: UserId     -> Unit     !Network
```

### 使い方

```fav
-- 具体実装
stage FetchUserPostgres: UserId -> User? !Db = |id| {
    Db.query("SELECT ...")
}

stage FetchUserMock: UserId -> User? !Db = |_| {
    Option.some(TestUser)
}

-- 注入
fn make_flow(fetch: FetchUser) -> seq UserId -> Unit {
    bind pipeline <- SomePipeline {
        fetch <- fetch
    }
    pipeline
}

bind prod_flow <- make_flow(FetchUserPostgres)
bind test_flow <- make_flow(FetchUserMock)
```

`abstract stage` は `abstract seq` のスロット型として使うことが多い。<!-- note: these are v2.0.0 keywords -->

---

## 5. `invariant`

型に不変条件を埋め込む。コンストラクタ時に自動検査される。

```fav
type Email {
    value: String
    invariant String.contains(value, "@")
    invariant String.len(value) <= 255
}

type PositiveInt {
    value: Int
    invariant value > 0
}

type AgeRange {
    value: Int
    invariant value >= 0
    invariant value <= 150
}
```

### invariant の価値

- `Email` 型を持っていればバリデーション済みが保証される
- `validate` ルーンと自然に連携（`Email` 型なら email チェック不要）
- `Stat.one<Email>()` は invariant を満たす値を生成する
- `fav explain` で型の契約として表示される

```
type Email
  value: String
  invariants:
    - String.contains(value, "@")
    - String.len(value) <= 255
```

### 実行モデル

- コンストラクタ時に検査（コンパイル時に可能なものは静的検査）
- 失敗時は `T!` を返す（`Email.new("invalid") -> Email!`）

---

## 全体像

```
抽象化の層

  パイプライン層
  └─ abstract seq    : ステージ構成の抽象化（Favnir 固有）

  変換層
  └─ abstract stage    : 実装なしの変換（注入可能なステージ）

  型層
  ├─ interface        : 型の操作契約（横断的）
  ├─ abstract type    : 型テンプレート（継承的）
  └─ invariant        : 型の不変条件（契約的）

  実装層
  ├─ impl（本体あり） : 手書き実装
  └─ impl（本体なし）/ with : 構造からの自動合成
```

各層は独立しており、組み合わせて使える：

```fav
-- interface + invariant + with の組み合わせ例

type Email with Show, Eq, Json {
    value: String
    invariant String.contains(value, "@")
    invariant String.len(value) <= 255
}

-- abstract stage + abstract seq の組み合わせ例

abstract stage ValidateEmail: String -> Email!

abstract seq SignupPipeline {
    parse:    Json         -> SignupForm!
    validate: SignupForm   -> SignupForm!
    notify:   UserId       -> Unit       !Network
    save:     SignupForm   -> UserId     !Db
}
```

---

## 明示性の原則

Favnir の抽象化は「隠蔽」ではなく「構造化」である。

- 暗黙の実装解決は行わない（全て明示的 `impl` で宣言、または `with` による自動合成）
- `fav explain` はどの実装が使われているかを常に表示できる
- `abstract` であっても `fav explain` に「このスロットには〇〇が束縛されている」と表示される
- AI にとっても、明示的な束縛は推論が容易

```
-- fav explain の出力例
seq UserImport  (SignupPipeline)
  parse    : Json -> SignupForm!         ← ParseSignupJson
  validate : SignupForm -> SignupForm!   ← ValidateSignupForm
  notify   : UserId -> Unit !Network    ← SendWelcomeEmail
  save     : SignupForm -> UserId !Db   ← SaveSignup

  resolved : Json -> UserId !Network !Db
  effects  : Network, Db
```
