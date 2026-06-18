# Favnir v13.1.0 仕様書

Date: 2026-06-09
Theme: interface 継承仕様確定 + ambient effect 禁止調査

---

## 概要

v14.0.0「能力型完成宣言」に向けた capability-context 移行の第一歩。
`interface A: B { ... }` 継承構文を言語に追加し、
ambient effect（ctx なしのエフェクト呼び出し）を警告として検出する仕組みを導入する。

この版での実装スコープ：
1. `interface` 継承構文（`: ParentName`）の parser / AST / checker 対応
2. 循環継承エラー（E0019）
3. ambient effect 警告（W008）+ `fav check --ambient` フラグ
4. `lab/audit/w008-ambient.md` 調査レポート自動生成

新しいエフェクト型や capability interface の具体的な実装（`DbRead` 等）は **v13.2.0 以降**。
この版は「基盤の仕様確定と移行コストの可視化」に集中する。

---

## 機能 1: interface 継承構文（`: ParentName`）

### 構文

```
interface CommonCtx {
  io:  Io
  env: Env
}

interface LoadCtx: CommonCtx {
  db: DbRead
}
```

`LoadCtx: CommonCtx` は「`LoadCtx` は `CommonCtx` のフィールドをすべて継承する」を意味する。
継承後の `LoadCtx` の実効フィールドは `io: Io`、`env: Env`、`db: DbRead` の 3 つ。

### AST 変更

`InterfaceDef` に `parent: Option<String>` フィールドを追加：

```rust
pub struct InterfaceDef {
    pub name:   String,
    pub parent: Option<String>,   // 新規
    pub fields: Vec<InterfaceField>,
}
```

### parser.rs 変更

`parse_interface_def` で `interface Name` の後に `: ParentName` を optional で読む：

```
interface <Name> [: <ParentName>] { <fields> }
```

### checker.rs 変更

interface のフィールドアクセス型解決時、`parent` が指定されていれば
親 interface のフィールドを再帰的にマージして解決する：

```
lookup_interface_field(name, "io") →
  LoadCtx に "io" がなければ parent である CommonCtx を探す → Io
```

### compiler.fav / checker.fav 変更

`self/compiler.fav` の `parse_interface_def` に `: ParentName` の解析を追加。
`self/checker.fav` の `infer_field_access` に親フィールド継承ロジックを追加。

### エラーメッセージ例

フィールドアクセスが解決できない場合（既存 E0021 相当）は変更なし。

---

## 機能 2: 循環継承エラー（E0019）

### 検出タイミング

`fav check` 時、interface 定義を解析した後に継承グラフを構築し、
有向グラフの閉路を検出する。

### エラー形式

```
E0019: circular interface inheritance detected
  --> pipeline.fav:5:1
   |
 5 | interface A: B { }
   | ^^^^^^^^^^^^^^^^^ A inherits B
 8 | interface B: A { }
   |              ^ B inherits A — cycle
   |
   = help: remove the circular dependency between A and B
```

### 検出アルゴリズム

DFS で親を辿り、同じノードに再到達したらエラー。
深さ上限は 16（それ以上はエラー: E0019 depth exceeded）。

---

## 機能 3: ambient effect 警告（W008）と `fav check --ambient`

### 定義

「ambient effect 呼び出し」= エフェクト付き namespace（`IO`、`Postgres`、`AWS`、
`Snowflake`、`Http`、`Grpc`、`Llm`、`Queue`、`Cache` 等）を
ctx 引数なしに直接呼び出している箇所。

```favnir
// ambient effect — W008
bind _ <- IO.println("done")
bind rows <- Postgres.query_raw(sql, params)

// ctx ベース — W008 なし（v13.2.0 以降の書き方）
bind _ <- ctx.io.println("done")
bind rows <- ctx.db.query(sql, params)
```

### W008 メッセージ形式

```
W008: ambient effect call — IO.println called without ctx argument
  --> pipeline.fav:5:12
   |
 5 | bind _ <- IO.println("done")
   |           ^^^^^^^^^^^^^^^^^^
   |
   = help: pass io capability: `ctx.io.println("done")`
   = note: ambient effects will become E0023 (error) in v14.0
```

### フラグ: `fav check --ambient`

`fav check --ambient <file>` で W008 を有効化。
通常の `fav check` では W008 は出力しない（`--ambient` フラグが必要）。

理由: 既存コードの大半が ambient effect を使っており、
通常の check で大量の W008 が出ると開発フローが中断する。
`--ambient` は移行状況の確認専用フラグ。

### 対象 namespace（W008 検出対象）

`IO` / `Postgres` / `AWS` / `Snowflake` / `Http` / `Grpc` / `Llm` /
`Queue` / `Cache` / `Slack` / `Email` / `Gen`（副作用のある関数のみ）

`Gen.uuid_raw` / `Gen.nano_id` は副作用（乱数生成）があるため対象。
`List.*` / `String.*` / `Map.*` 等の純粋関数は対象外。

### lint.rs への追加

新規関数 `check_ambient_effects(program: &Program) -> Vec<LintWarning>` を追加。
`fav lint` には含めない（`fav check --ambient` 専用）。

---

## 機能 4: W008 調査レポート（`fav check --ambient --report`）

`fav check --ambient --report <file>` で `lab/audit/w008-ambient.md` に
W008 検出結果をまとめた Markdown レポートを出力する。

```markdown
# W008 Ambient Effect Audit — 2026-06-09

## self/compiler.fav

| 行 | 呼び出し | namespace |
|---|---|---|
| 142 | IO.println | IO |
| 203 | IO.read_file_raw | IO |
...

合計: 47 件

## self/checker.fav

...
合計: 12 件

## 総計: 59 件
```

CI に組み込まず、開発者が手動で実行して移行の進捗を確認するツール。

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `interface_inheritance_parsed` | `interface LoadCtx: CommonCtx { db: DbRead }` がパースエラーなし |
| `interface_inheritance_field_access` | `LoadCtx` から `io`（親の CommonCtx フィールド）にアクセスできる |
| `e0019_circular_interface` | `A: B` + `B: A` → E0019 |
| `e0019_single_interface_no_error` | 継承なし interface → E0019 なし |
| `w008_ambient_io_println` | `IO.println(...)` が `--ambient` フラグで W008 |
| `w008_ambient_postgres_raw` | `Postgres.query_raw(...)` が W008 |
| `w008_no_flag_no_warning` | `--ambient` なしでは W008 は出ない |
| `w008_pure_list_no_warning` | `List.map(...)` は W008 の対象外 |
| `version_is_13_1_0` | `CARGO_PKG_VERSION == "13.1.0"` |

---

## 完了条件

- [ ] `interface A: B { ... }` がパース・コンパイル・型チェックできる
- [ ] 継承フィールドが `checker.rs` / `checker.fav` で正しく解決される
- [ ] E0019 循環継承が検出される
- [ ] `fav check --ambient` で W008 が出力される
- [ ] `fav check`（フラグなし）では W008 は出ない
- [ ] `lab/audit/w008-ambient.md` が `--report` フラグで生成される
- [ ] `self/compiler.fav` / `self/checker.fav` が `fav check` でエラーなし（W008 は `--ambient` 時のみ）
- [ ] `cargo test` 全通過
- [ ] CI 全 green

---

## 非目標

- `DbRead` / `DbWrite` 等の具体的な capability interface の実装（v13.2.0）
- `ctx.db.query(...)` 構文のコンパイル（v13.2.0 以降）
- ambient effect の実際のエラー化（v13.8.0 で E0023 に昇格）
- `impl A for B { ... }` の継承への対応（interface 継承と impl は独立）
- 多重継承（1 つの interface は 1 つの親のみ）
