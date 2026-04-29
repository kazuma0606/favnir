# Favnir ロードマップ

更新日: 2026-04-29

v0.1.0 から v1.0.0 までの実装方針の概要。
各バージョンは前のバージョンの完了を前提にして進める。

---

## v0.1.0 — コア言語 + 純粋インタープリタ

**テーマ**: 最小限の言語で pipeline が動くこと

### 含むもの

- 基本型 (`Bool`, `Int`, `Float`, `String`, `Unit`, `List<T>`, `Map<K,V>`)
- ユーザー定義型 (`type` による record / sum)
- 束縛 (`bind <-`)、パターン分解
- 関数 (`fn`)、クロージャ
- 変換 (`trf`)、フロー (`flw`)
- パイプライン演算子 (`|>`)
- パターンマッチ (`match`)
- 条件式 (`if`)
- effect 注釈の構文・型検査 (`Pure`, `Io`)
- tree-walking インタープリタ
- CLI: `fav run`, `fav check`
- 最小組み込み関数 (IO, List, String, Option, Result)

### 完了条件

- `fav run` で `trf` + `flw` + `match` が動く
- `fav check` で型エラーを位置情報付きで報告できる

---

## v0.2.0 — effect システムの完成

**テーマ**: 副作用を持つ処理を型安全に実行できること

### 追加するもの

- `Db` effect の実行対応 — SQLite + SQL 埋め込み (SQLx スタイル)
  - `Db.query`, `Db.query_one`, `Db.execute` (パラメータは `?` バインド)
  - `fav run --db sqlite://./app.db` フラグで接続設定
  - ORM なし・スキーマ管理なし; SQL を文字列で直接記述する
- `Emit<Event>` effect の実行対応 (インメモリ event bus)
  - `emit TypeName { field: expr }` 構文の追加
  - `Emit<A> + Emit<B> = Emit<A | B>` の合成を型検査に組み込む
- `Network` effect の実行対応 (HTTP GET/POST の最小対応)
- レコード構築式の追加 (`TypeName { field: expr, ... }`)
- 多重 effect 注釈 (`!Db !Emit<UserCreated>`)
- `fav explain <file>` の初期実装 (各 `trf`/`flw` の type + effect を表示)

### 完了条件

- `fav run --db sqlite://:memory: examples/users.fav` で User CRUD が動く
- `trf CreateUser: UserInput -> Int !Db !Emit<UserCreated>` が型チェックを通る
- `emit UserCreated { ... }` が動き、`Emit<UserCreated>` が型に現れる
- `Http.get(url)` が `String!` を返して動く
- `fav explain` で `flw` の effect chain が表示される

---

## v0.3.0 — モジュールシステム ✓ 完了: 2026-04-28

**テーマ**: 複数ファイルで構成された Favnir プロジェクトを扱えること

### 追加するもの

- `namespace` によるトップレベル名前空間
- `use` による import
- file-based module の解決 (ファイルパス → module path)
- `rune` の最小実装 (公開単位・`pub` による可視性制御)
- `fav.toml` によるプロジェクト設定
- 名前解決エラーの改善

### 完了条件

- `use data.csv.parse` で別ファイルの `trf` を参照できる
- `pub trf` / `pub type` で公開・非公開を制御できる
- `rune` 単位で `fav check` が動く

---

## v0.4.0 — ジェネリクス + `cap` ✓ 完了: 2026-04-29

**テーマ**: 型引数と capability による抽象化ができること

### 追加するもの

- generic `type` (例: `type Option<T>`, `type Result<T, E>`)
- generic `fn` (例: `fn identity<T>(value: T) -> T`)
- generic `trf` (例: `trf Map<T, U>: List<T> -> List<U>`)
- 単相型推論の多相化 (Hindley-Milner の最小サブセット)
- `cap` の定義構文と値渡し
- 標準 `cap`: `Ord<T>`, `Eq<T>`, `Show<T>`
- `T?` / `T!` の internal 型への展開を generic ADT に統一

### 完了条件

- `fn map<T, U>(items: List<T>, f: T -> U) -> List<U>` が書けて動く
- `cap Ord<T>` を定義して `sort(users, User.ord)` が動く
- `Option<T>` / `Result<T, E>` が generic ADT として機能する

---

## v0.5.0 — `chain` + パターン強化 ✓ 完了: 2026-04-29

**テーマ**: ローカルな文脈付き合成と `match` の表現力向上

### 追加するもの

- `chain` 束縛の実装 (failure 伝播 + effect 蓄積)
- `chain` の型検査 (蓄積された effect の型への反映)
- `pipe match` sugar (`|> match { ... }`)
- pattern guard (`where` 句)
- `inspect` trf (pipeline 途中観測、`!Trace` effect)
- `collect / yield` 構文 (限定的な列生成)

### 完了条件

```fav
bind user <- row
chain user <- parse_user
chain user <- normalize
chain id   <- save_user
```

が動き、蓄積された effect が型に現れる。

---

## v0.6.0 — bytecode + 小型 VM ✓ 完了: 2026-04-29

**テーマ**: インタープリタから portable artifact へ

### 追加するもの

- typed IR の設計と定義
- typed IR へのコンパイル (AST → IR)
- bytecode へのコンパイル (IR → bytecode)
- 小型 VM の実装
- `.fvc` artifact フォーマット (bytecode + metadata)
- `fav build <file> -o <file.fvc>` の実装
- `fav exec <file.fvc>` の実装
- source span / type / effect 情報を artifact に残す

### 完了条件

- `fav build main.fav -o main.fvc` が動く
- `fav exec main.fvc` で実行できる
- インタープリタと bytecode 実行の出力が一致する

---

## v0.7.0 — 標準ライブラリ ✓ 完了: 2026-04-30

**テーマ**: 実用的なデータ処理ができる組み込みを整える

### 追加するもの

- `std.list` の完全な実装 (`map`, `filter`, `fold`, `flat_map`, `zip`, `sort` 等)
- `std.string` の完全な実装 (`trim`, `split`, `join`, `replace`, `starts_with` 等)
- `std.map` の実装 (`get`, `set`, `keys`, `values`, `merge` 等)
- `std.option` の実装
- `std.result` の実装
- `std.io` の拡充 (ファイル読み書き、`!File` effect)
- `std.json` の最小実装 (parse / encode)
- `std.csv` の最小実装 (parse / encode)

### 完了条件

- CSV を読んで変換して JSON に書き出す処理が標準ライブラリだけで書ける

---

## v0.8.0 — CLI + tooling

**テーマ**: 開発体験を整える

### 追加するもの

- `fav fmt` の実装 (コードフォーマッタ)
- `fav lint` の初期実装 (基本的なスタイルチェック)
- `fav test` の実装 (`test "..." { assert(...) }` 構文の実行)
- テストオプション: `--filter`, `--fail-fast`, `--trace`
- `fav explain <file>` の強化 (flow の型・effect の可視化)
- エラーメッセージの全体的な改善
- `--trace` オプション (実行トレースの表示)

### 完了条件

- `fav test` で `test` ブロックが実行できる
- `fav fmt` でコードが整形される
- `fav explain` で `flw` の入出力・effect が一覧できる

---

## v0.9.0 — WASM backend

**テーマ**: ポータビリティと sandbox 実行

### 追加するもの

- typed IR → WASM lowering の実装
- capability runtime の整備 (WASM host 呼び出し経由)
- `fav build --target wasm` の実装
- `fav exec <file.wasm>` の実装
- WASM sandbox 上での effect dispatch
- browser / edge 環境での実行検証 (最低限)

### 完了条件

- `fav build --target wasm` で `.wasm` が生成できる
- capability 経由で `Db` / `Io` が動く

---

## v1.0.0 — 安定版

**テーマ**: 仕様・ツールチェイン・セルフホストの入口を揃える

### 追加するもの

- 言語仕様の安定化 (後方互換の約束)
- self-hosting の入口 (parser または checker の一部を `.fav` で書く)
- `rune` レジストリの最小設計
- LSP の最小実装 (hover, diagnostics)
- `fav.toml` の完全な仕様
- ドキュメントの整備
- v1.0.0 リリースノートの作成

### 完了条件

- Favnir で書かれた `.fav` コードが `fav run` で動く
- 言語仕様を破壊する変更を以降は入れない
- 他者が使い始められる状態になっている

---

## バージョンと機能の対応表

| バージョン | テーマ | 主な追加 |
|---|---|---|
| v0.1.0 | コア + インタープリタ | `trf`, `flw`, `bind`, `match`, `Pure/Io` |
| v0.2.0 | effect 完成 | `Db`(SQLite+SQL埋込), `Emit<E>`, `Network`, `fav explain` |
| v0.3.0 | モジュール | `namespace`, `use`, `rune`, `fav.toml` |
| v0.4.0 | ジェネリクス + `cap` | generic型, `cap`, 多相推論 |
| v0.5.0 | `chain` + パターン強化 | `chain`, `pipe match`, pattern guard, `inspect` |
| v0.6.0 | bytecode + VM | typed IR, bytecode, `fav build/exec` |
| v0.7.0 | 標準ライブラリ | std.list/string/map/json/csv |
| v0.8.0 | CLI + tooling | `fav fmt/lint/test/explain` |
| v0.9.0 | WASM | WASM backend, capability runtime |
| v1.0.0 | 安定版 | 仕様安定, self-hosting 入口, LSP |
