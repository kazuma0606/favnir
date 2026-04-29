# Favnir v0.2.0 実装計画

更新日: 2026-04-27

---

## Phase 1: Lexer / Parser

### 新トークン・キーワード

| 追加 | 内容 |
|---|---|
| `TokenKind::Emit` | `emit` キーワード |
| `TokenKind::Db` | `Db` 効果名（`Pure`/`Io` と同じ扱い） |
| `TokenKind::Network` | `Network` 効果名 |
| `Pipe` のある `<` | `Emit<UserCreated>` の `<` は既存の `LAngle` で対応可 |

`Emit<T>` の `<` / `>` は `LAngle` / `RAngle` トークンを再利用。

### effect 注釈のパース変更

```
effect_ann  = ("!" effect_term)+       // 複数 effect
effect_term = "Pure" | "Io" | "Db" | "Network" | "Emit" "<" IDENT ">"
```

`parse_effect_ann()` をループで複数 effect を収集するよう変更する。
AST の `effect: Option<Effect>` を `effects: Vec<Effect>` に変更する。

### AST 変更

```rust
// Effect の拡張
pub enum Effect {
    Pure,
    Io,
    Db,
    Network,
    Emit(String),      // Emit<EventType>
    EmitUnion(Vec<String>),  // Emit<A | B> (合成後)
}

// FnDef / TrfDef のフィールド変更
pub effects: Vec<Effect>,   // Option<Effect> → Vec<Effect>

// 新規: レコード構築式
Expr::RecordConstruct(String, Vec<(String, Expr)>, Span)

// 新規: emit 式
Expr::Emit(Box<Expr>, Span)
```

### 式のパース変更

`parse_primary_expr` に次の分岐を追加:

1. 大文字始まり IDENT + `{` → レコード構築式
2. `emit` キーワード → `Expr::Emit(parse_expr())`

`block` 内のステートメントで `emit` も `stmt` として扱う。

---

## Phase 2: 型チェック

### Effect 型の拡張

```rust
// checker.rs の Type に変更なし; Effect は ast.rs に統一
// compose_effects を Vec<Effect> ベースに変更

fn compose_effects(a: &[Effect], b: &[Effect]) -> Vec<Effect>
```

合成ルール:
- `Pure` は無視（identity）
- 同じ effect は重複排除
- `Emit<A>` + `Emit<B>` = `Emit<A | B>` → `EmitUnion(["A", "B"])` に変換

### 型チェック追加

| タスク | 内容 |
|---|---|
| レコード構築式 | 型名が定義済みか確認; フィールド名・型の一致確認 |
| `emit` 式 | 引数の型を推論; `Emit<T>` を現在スコープの effect として記録 |
| `Db.*` 呼び出し | `!Db` のない `trf`/`fn` 内での使用を E007 として警告 |
| `Http.*` 呼び出し | `!Network` のない `trf`/`fn` 内での使用を E008 として警告 |

### `flw` の effect 推論改善

`check_flw_def` でステップごとの effect を和で合成し、
結果の effect セットを `Trf` 型の effect として設定する。

---

## Phase 3: インタープリタ

### Value の追加

```rust
// 既存の Map<String, Value> で Record は表現済み
// 変更なし
```

### eval.rs 変更

#### レコード構築式の評価

```rust
Expr::RecordConstruct(type_name, fields, span) => {
    let mut map = HashMap::new();
    for (name, expr) in fields {
        map.insert(name.clone(), eval_expr(expr, env)?);
    }
    Ok(Value::Record(map))
}
```

#### `emit` 式の評価

```rust
Expr::Emit(inner, _) => {
    let val = eval_expr(inner, env)?;
    // インタープリタのイベントログに追加
    interpreter.emit_log.push(val);
    Ok(Value::Unit)
}
```

`Interpreter` 構造体に `emit_log: Vec<Value>` を追加。
評価関数が `&mut Interpreter` を受け取る形に変更、または
`emit_log` を `Rc<RefCell<Vec<Value>>>` でクロージャから共有する。

### Db 組み込み (rusqlite)

`eval_builtin` に `"Db"` ネームスペースを追加:

```rust
("Db", "query") => {
    // sql = args[0] (String)
    // params = args[1..] (Vec<Value>)
    // db_conn.prepare(sql).query_map(params, |row| ...) -> List<Map<String,String>>
}
("Db", "query_one") => { ... }  // -> Map<String,String>?
("Db", "execute")   => { ... }  // -> Int
```

`Interpreter` に `db_conn: Option<rusqlite::Connection>` を追加。
`--db` フラグで渡した接続文字列から起動時に接続する。

### Network 組み込み (ureq)

```rust
("Http", "get")  => { ureq::get(&url).call()?.into_string()? }
("Http", "post") => { ureq::post(&url).send_string(&body)?.into_string()? }
```

`Result<String, Error>` を Favnir の `Variant("ok"/"err", ...)` にマップする。

### Map 組み込みの追加

```rust
("Map", "get")    => Map<K,V>, K -> V?
("Map", "set")    => Map<K,V>, K, V -> Map<K,V>
("Map", "keys")   => Map<K,V> -> List<K>
("Map", "values") => Map<K,V> -> List<V>
```

---

## Phase 4: CLI

### `fav run --db <url>` フラグ

`main.rs` の `cmd_run` を `--db` オプションに対応させる:

```rust
// args parsing: fav run [--db <url>] <file>
```

`rusqlite::Connection::open(path)` または `Connection::open_in_memory()` で接続し、
`Interpreter::run_with_db(program, conn)` に渡す。

### `fav explain <file>` コマンド

```rust
fn cmd_explain(path: &str) {
    // parse + check
    // 各 Item の名前・型・effect を表示
    for item in &program.items {
        println!("{:<24} : {}", name, signature);
    }
}
```

出力フォーマット:

```
fn main                : Unit              !Io !Db
trf CreateUser         : UserInput -> Int  !Db !Emit<UserCreated>
flw Onboard            : UserInput -> Int  !Db !Emit<UserCreated>
```

---

## Phase 5: 依存クレートの追加

`Cargo.toml` に追加:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
ureq     = "2"
```

`bundled` feature で SQLite を静的リンクし、外部ライブラリ不要にする。

---

## Phase 6: サンプルと動作確認

`examples/users.fav` — User CRUD (SQLite, Emit)

完了条件:

```
fav run --db sqlite://:memory: examples/users.fav
fav check examples/users.fav
fav explain examples/users.fav
```

---

## 設計メモ

### `emit` の実装方針

クロージャがイベントログを共有するために `Rc<RefCell<Vec<Value>>>` を使う。
`register_fn_def` / `register_trf_def` はすでに env を Rc で共有しているので、
emit_log も同じパターンで共有できる。

### `Db` 接続の共有

`Connection` は `Send` でないため `Rc<RefCell<Connection>>` で共有する。
（v0.6.0 以降で bytecode + VM に移行する際に再設計）

### 型なし行マッピング

`Db.query` の戻り値は `Map<String, String>`（全値を文字列変換）。
型付きマッピング（`Db.query<User>`）は v0.4.0 でジェネリクスと同時に追加する。
