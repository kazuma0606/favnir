# ForgeScript Syntax Notes

更新日: 2026-04-26

## 目的

このメモは、Favnir の構文設計を進めるために、現行 ForgeScript の構文と設計意図を整理したもの。

ここでは「現行 ForgeScript が何を中核構文として持っているか」を簡潔にまとめる。  
Favnir 側でそのまま継承するもの、置き換えるもの、分離するものを見比べるための土台とする。

## 構文の中心

現行 ForgeScript の中心は次の 4 つ。

- `job`
- `event`
- `app`
- DI / handler 系の宣言

関数や式は一般的な構文を持つが、言語の顔として強いのは上の宣言群。

## `job`

`job` は実行単位。

```forge
job ImportUsers {
    input path: string
    input dry_run: bool = true

    run {
        let rows = csv.read(path)
        let result = UserSchema.validate_all(rows)
        return result.summary()
    }
}
```

要点:

- `input` で外部から入る値を宣言する
- `run {}` が本体
- CLI、テスト、イベント連携から呼ばれる
- `forge job <name>` で直接実行できる

## `input`

`input` は `job` の引数宣言。

```forge
input path: string
input dry_run: bool = true
input notifier: Notifier
```

役割:

- primitive 型なら CLI オプションから入る
- 一部の値は `app.forge` の `provide` から入る
- service 的な依存は `wire` / `container bind` から供給される

つまり `input` は、CLI 引数と DI 注入の両方の入口になっている。

## `run`

`run {}` は `job` の本体。

```forge
run {
    let rows = csv.read(path)
    if dry_run {
        return rows.count()
    }
    db.insert_many("users", rows)
}
```

特徴:

- 手続き的に書ける
- `emit` を呼べる
- `return` を持てる
- 現状は `let` ベースでローカル束縛する

## `event`

`event` はイベント型の宣言。

```forge
event RowInvalid {
    row: int
    field: string
    message: string
}
```

役割:

- `emit` されるデータ構造
- handler の受け口になる
- event log や `forge explain` の対象になる

## `emit`

`emit` はイベント発火。

```forge
emit RowInvalid {
    row: 42,
    field: "email",
    message: "invalid email",
}
```

役割:

- `job` 実行中にイベントを記録・通知する
- `@on(Event)` handler に流れる
- run log / event log に残る

## `app`

`app` は composition root。

```forge
app Production {
    load validators/*
    load jobs/*
    load events/*
    load handlers/*

    provide db = Crucible::connect(env("DATABASE_URL"))

    container {
        bind Notifier to SlackNotifier::new(env("SLACK_TOKEN"))
    }

    wire ImportUsers {
        notifier: Notifier
    }
}
```

役割:

- モジュールをまとめてロードする
- infrastructure 値を `provide` する
- pluggable service を `container` で定義する
- 特定 `job` への配線を `wire` で定義する

## `load`

`load` は glob でファイル群をロードする。

```forge
load validators/*
load jobs/*
load events/*
load handlers/*
```

役割:

- `app.forge` からアプリ全体を組み立てる
- ディレクトリ単位で構成を明示する

## `provide`

`provide` は infrastructure 値の注入。

```forge
provide db    = Crucible::connect(env("DATABASE_URL"))
provide queue = EventQueue::new()
```

役割:

- app 全体で使う値を宣言する
- `job input` に自動供給される
- DB 接続や event queue のような値に向いている

## `container { bind X to Y }`

`container` は pluggable service のバインドを持つ。

```forge
container {
    bind Notifier to SlackNotifier::new(env("SLACK_TOKEN"))
    bind Report   to HtmlReport::new("target/report.html")
}
```

役割:

- trait / interface 的な抽象に対する実装を束縛する
- app ごとに差し替え可能にする
- `wire` と組み合わせて `job input` に注入する

## `wire`

`wire` は `job` ごとの依存配線。

```forge
wire ImportUsers {
    notifier: Notifier
    report: Report
}
```

役割:

- `job input` に何を差し込むかを定義する
- `provide` が infrastructure 注入なのに対し、`wire` は service 注入寄り

## `@service`

`@service` は handler や service オブジェクトを表す注釈。

```forge
@service
struct RowInvalidReportHandler {
    report: Report
}
```

役割:

- DI 対象の service を明示する
- `@on(Event)` と組み合わせて handler を定義する

## `@on(Event)`

`@on(Event)` はイベントハンドラ宣言。

```forge
@service
struct RowInvalidReportHandler {
    report: Report
}

impl RowInvalidReportHandler {
    @on(RowInvalid)
    fn handle(self, e: RowInvalid) -> unit! {
        self.report.add_error(e.row, e.message)
    }
}
```

役割:

- 特定イベントを受ける handler を定義する
- `emit` と対になる構文

## `fn`

通常の関数定義。

```forge
fn normalize_email(value: string) -> string {
    value.trim().lower()
}
```

現状の ForgeScript では、`fn` はあるが言語の顔は `job` や `event` 側に寄っている。

## `let`

現状のローカル束縛は `let`。

```forge
let rows = csv.read(path)
let result = UserSchema.validate_all(rows)
```

Favnir と比較すると:

- ForgeScript は現状 `let` を使う
- 再代入や mutable の扱いは、Favnir ほど厳格に切っていない

## `match`

分岐には `match` を使える。

```forge
match UserSchema.validate_all(rows) {
    ok(valid) => db.insert_many("users", valid)
    err(errors) => return err(errors)
}
```

役割:

- validator 結果の分岐
- result 的な値の分岐

## pipe `|>`

パイプ演算子は既に存在する。

```forge
rows |> validate(UserSchema)
```

ただし現状の位置づけは:

- 書きやすくするための構文
- 型付き合成を中核にしたものではない
- Favnir の `stage / flow` ほど意味論の中心ではない

## テスト

テストは文字列名付きブロック。

```forge
test "ImportUsers rejects invalid users" {
    ...
}
```

派生構想として次が議論されていた。

- `expect_event(...)`
- `expect_snapshot(...)`

## CLI と構文の結び付き

ForgeScript は CLI と構文が強く結び付いている。

- `forge job <name>`
- `forge explain`
- `forge explain --json`
- `forge test`

つまり、構文は単なる記述形式ではなく、CLI から直接扱われるアプリケーション単位になっている。

## ForgeScript の特徴を一言でいうと

ForgeScript は、

> job / event / app / DI を中心にした、アプリケーション構成言語

と整理できる。

## Favnir と比べた差分

大きな差分は次の通り。

- ForgeScript は `job` と `app` が中心
- Favnir は `stage` と `flow` が中心
- ForgeScript の pipe は補助構文
- Favnir の pipe は型付き合成の中核
- ForgeScript は `let` ベース
- Favnir は `bind <-` ベース
- ForgeScript は DI / handler / app 構成が前面
- Favnir は immutable / effect / typed pipeline が前面

## 何を継承できるか

Favnir へ継承候補になりうるもの:

- `event`
- `emit`
- `match`
- 一部の `app` / capability 構想
- CLI と構文の強い接続

そのまま継承しにくいもの:

- `job` を言語の最上位に置くこと
- `container bind` を OOP / trait 的に使うこと
- `let` ベースのローカル束縛
- pipe を補助記法に留めること

## 仮の結論

ForgeScript は「アプリケーションを組み立てる言語」として強い。  
Favnir はそこから、「型付き pipeline と effect を中心にした言語」へ重心を移す方向が自然。

したがって、ForgeScript の構文は参考にしつつも、そのまま踏襲するより、

- `event`
- `emit`
- `match`
- composition root 的な考え方

だけを選択的に継承する方がよい。
