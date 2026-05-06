# Favnir Abstract Flow Design

日付: 2026-05-01

## 概要

`abstract seq` はパイプライン構造そのものを抽象化する仕組み。

`interface` が「型の振る舞い」を抽象化するのに対して、
`abstract seq` は「パイプラインのステージ構成」を抽象化する。

これは他の言語にはない Favnir 固有の抽象化概念である。

---

## 宣言

```fav
abstract seq DataPipeline<Row> {
    parse:    String    -> List<Row>!
    validate: Row       -> Row!
    save:     List<Row> -> Int     !Db
}
```

フィールドが型を持つレコード宣言と同じ構造。
ただし値ではなく `stage` の型シグネチャを宣言する。

型パラメータ `<Row>` はスロット間で共有される。

---

## 束縛（instantiation）

### トップレベル

```fav
seq UserImport = DataPipeline<UserRow> {
    parse    <- ParseUserCsv      -- String -> List<UserRow>!  ✓
    validate <- ValidateUser      -- UserRow -> UserRow!        ✓
    save     <- SaveUsers         -- List<UserRow> -> Int !Db  ✓
}

seq OrderImport = DataPipeline<OrderRow> {
    parse    <- ParseOrderCsv
    validate <- ValidateOrder
    save     <- SaveOrders
}
```

`DataPipeline<UserRow>` を `bind`-スタイルのブロックで埋めることで、
完全に型付けされた具体的な `seq` が生成される。

### 型不一致はコンパイルエラー

```fav
seq BadImport = DataPipeline<UserRow> {
    parse    <- ParseUserCsv
    validate <- ValidateOrder     -- OrderRow -> OrderRow! ✗ UserRow 期待
    save     <- SaveUsers
}
-- E001: validate slot expects (UserRow -> UserRow!), got (OrderRow -> OrderRow!)
```

---

## 式レベルの束縛（動的注入）

`bind` で書けるため、関数引数から `stage` を注入できる。
これにより本番・テスト・プロファイル別の実装差し替えが型安全になる。

```fav
fn make_import(save: List<UserRow> -> Int !Db) -> seq String -> Int !Db {
    bind pipeline <- DataPipeline<UserRow> {
        parse    <- ParseUserCsv
        validate <- ValidateUser
        save     <- save          -- 引数から注入
    }
    pipeline
}

-- 本番
bind prod_import <- make_import(SaveUsersPostgres)

-- テスト
bind test_import <- make_import(SaveUsersMock)
```

型は全てコンパイル時に検証される。
実行時の差し替えではなく、束縛時点での型検査。

---

## 部分束縛

全スロットを一度に埋める必要はない。
段階的に束縛できる。

```fav
-- parse だけ先に決める（部分束縛）
bind base_pipeline <- DataPipeline<UserRow> {
    parse <- ParseUserCsv
}
-- 型: PartialFlw<DataPipeline<UserRow>, { validate save }>
-- validate と save が未束縛であることを型が記録する

-- 残りを埋めて完成
bind full_pipeline <- base_pipeline {
    validate <- ValidateUser
    save     <- SaveUsers
}
-- 型: seq String -> Int !Db  （具体的な seq に解決される）
```

未束縛スロットが残る `PartialFlw` は `fav check` で未完成として報告される。
完全束縛された時点で初めて実行可能な `seq` になる。

---

## レコード構築との対称性

Favnir の構築パターンが `bind X <- Template { slot <- value }` で統一される。

```fav
-- 値の構築（レコード）
bind user <- User {
    name  <- "Alice"
    email <- "alice@example.com"
}

-- パイプラインの構築（abstract seq）
bind import_flow <- DataPipeline<UserRow> {
    parse    <- ParseUserCsv
    validate <- ValidateUser
    save     <- SaveUsers
}
```

型・パイプラインのどちらも `bind` で安全に組み立てる一貫した語彙になる。

---

## effect の推論

束縛が完成した時点で、`seq` の effect は各スロットの effect の合成として推論される。

```fav
abstract seq DataPipeline<Row> {
    parse:    String    -> List<Row>!       -- effect なし
    validate: Row       -> Row!             -- effect なし
    enrich:   Row       -> Row   !Network  -- !Network
    save:     List<Row> -> Int   !Db       -- !Db
}

seq UserImport = DataPipeline<UserRow> {
    parse    <- ParseUserCsv
    validate <- ValidateUser
    enrich   <- EnrichFromApi
    save     <- SaveUsers
}
-- UserImport の推論された effect: !Network !Db
```

スロットに渡した `stage` の effect が自動的に集約される。
宣言時の effect と実際の effect が合わない場合はコンパイルエラー。

---

## `fav explain` での表示

```
seq UserImport  (DataPipeline<UserRow>)
  parse    : String -> List<UserRow>!    ← ParseUserCsv
  validate : UserRow -> UserRow!         ← ValidateUser
  save     : List<UserRow> -> Int !Db   ← SaveUsers

  resolved : String -> Int !Db
  effects  : Db
```

抽象テンプレート名と具体バインディングの両方が表示される。
「何の形で作られたか」と「何が差し込まれているか」が一度に把握できる。

---

## `fav graph` での表示

```
DataPipeline<UserRow>
  [parse]    → ParseUserCsv    → [validate] → ValidateUser
  [validate] → ValidateUser    → [save]     → SaveUsers
  [save]     → SaveUsers       → (Int !Db)
```

抽象スロットノード（`[parse]` など）と具体ノード（`ParseUserCsv` など）を区別して描画する。

---

## 他の抽象化との比較

| 概念 | 対象 | 目的 |
|---|---|---|
| `interface` | 型 | 型が持てる操作の契約 |
| `abstract type` | 型 | 具体型のテンプレート |
| `abstract stage` | 変換単体 | シグネチャだけ、実装は別に提供 |
| `abstract seq` | パイプライン構造 | ステージ構成そのものの抽象化 |

`abstract seq` だけが「パイプラインの形」を抽象化する。
他の言語のテンプレートメソッドパターンに近いが、
型パラメータと `bind` による明示的な段階的束縛が Favnir 固有の特徴。

---

## ユースケース

### 1. ETL パイプラインのテンプレート化

```fav
abstract seq ETL<In, Out> {
    extract:   Source   -> List<In>!  !Network
    transform: In       -> Out!
    load:      List<Out> -> Int       !Db
}

seq UserETL = ETL<RawUser, UserRow> {
    extract   <- FetchUsersFromApi
    transform <- NormalizeUser
    load      <- SaveUsers
}
```

### 2. マルチバックエンド対応

```fav
abstract seq StoragePipeline<Row> {
    validate: Row -> Row!
    save:     Row -> Int !Db
}

-- Postgres バックエンド
bind pg_pipeline <- StoragePipeline<UserRow> {
    validate <- ValidateUser
    save     <- SaveUsersPg
}

-- BigQuery バックエンド
bind bq_pipeline <- StoragePipeline<UserRow> {
    validate <- ValidateUser
    save     <- SaveUsersBq
}
```

### 3. テスト用パイプライン構築

```fav
test "import pipeline validates correctly" {
    bind pipeline <- DataPipeline<UserRow> {
        parse    <- ParseUserCsv
        validate <- ValidateUser
        save     <- SaveUsersMock    -- テスト用実装
    }
    bind users <- Stat.list<UserRow>(100, seed: 42)
    bind result <- pipeline(Csv.encode(users))
    assert_eq(result, 100)
}
```

---

## 実装上の考慮

### 型レベルの表現

- `abstract seq Name<Params> { slots }` → 型システム上は「スロット付き関数型のレコード」として扱う
- 各スロットは `stage` 型 (`A -> B !Fx`) を値として持つ
- 完全束縛後は通常の `seq` 型 (`A -> Z !Fx1 !Fx2 ...`) に解決される

### PartialFlw の扱い

- 部分束縛中の型は `PartialFlw<Template, { remaining_slots }>` として表現
- `fav check` は未束縛スロットを警告またはエラーとして報告
- `fav run` / `fav build` は完全束縛のみ受け付ける

### セルフホスト適性

- スロットの型検査は `stage` の型検査と同じロジックで実装できる
- `bind` 構文を再利用するため、パーサー拡張は最小限
- explain / graph は既存の IR 解析パスに「テンプレート由来」フラグを追加するだけで対応可能

---

## 一言でいうと

`abstract seq` は:

> パイプラインの「形」を宣言し、`bind` で型安全に「中身」を埋める仕組み

`bind` による束縛はレコード構築と同じ語彙で表現されるため、
Favnir の一貫性を保ちながらパイプライン抽象化を実現する。
