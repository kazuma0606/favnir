# Favnir v0.2.0 タスク一覧

更新日: 2026-04-27 (全タスク完了)

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Lexer / Parser

### Lexer

- [x] 1-1: `emit` キーワードを `TokenKind::Emit` として追加する
- [x] 1-2: `Db`, `Network` を effect キーワードとして追加する (Ident として contextual parse)
- [x] 1-3: Lexer の単体テストを更新する

### AST

- [x] 1-4: `Effect` enum を拡張する (`Db`, `Network`, `Emit(String)`, `EmitUnion(Vec<String>)`)
- [x] 1-5: `FnDef` / `TrfDef` の `effect: Option<Effect>` を `effects: Vec<Effect>` に変更する
- [x] 1-6: `Expr::RecordConstruct(String, Vec<(String, Expr)>, Span)` を追加する
- [x] 1-7: `Expr::EmitExpr(Box<Expr>, Span)` を追加する

### Parser

- [x] 1-8: `parse_effect_ann()` を複数 effect のループ形式に変更する (`!Db !Emit<UserCreated>`)
- [x] 1-9: `Emit<IDENT>` の effect term パースを実装する
- [x] 1-10: `parse_primary_expr` に大文字始まり IDENT + `{` でレコード構築式をパースする分岐を追加する
- [x] 1-11: `parse_primary_expr` に `emit expr` のパースを追加する
- [x] 1-12: `block` の `stmt` パースで `emit` 式を扱えるようにする
- [x] 1-13: Parser の単体テストを追加する (effect 複数、レコード構築式、emit)

---

## Phase 2: 型チェック

- [x] 2-1: `compose_effects(a: &[Effect], b: &[Effect]) -> Vec<Effect>` を実装する
- [x] 2-2: `Emit<A> + Emit<B> = Emit<A | B>` の合成ロジックを実装する
- [x] 2-3: `check_flw_def` を `Vec<Effect>` ベースの合成に対応させる (Type::Trf → Vec<Effect>)
- [x] 2-4: レコード構築式 `Expr::RecordConstruct` の型チェックを実装する (型名の存在確認)
- [x] 2-5: `Expr::EmitExpr` の型チェックを実装する (Unit を返す)
- [x] 2-6: `Db.*` 呼び出しを `!Db` のない関数内で使った場合に E007 を報告する
- [x] 2-7: `Http.*` 呼び出しを `!Network` のない関数内で使った場合に E008 を報告する
- [x] 2-8: `emit` を `!Emit<T>` のない関数内で使った場合に E009 を報告する
- [x] 2-9: `fav explain` 用に型・effect の文字列表現を生成するヘルパーを実装する (main.rs)
- [x] 2-10: 型チェックの単体テストを追加する

---

## Phase 3: インタープリタ

### レコード構築式

- [x] 3-1: `Expr::RecordConstruct` の評価を実装する (フィールドを評価して `Value::Record` を生成)

### emit 評価

- [x] 3-2: emit_log を `thread_local! { RefCell<Vec<Value>> }` で実装する
- [x] 3-3: `Expr::EmitExpr` の評価を実装する (値を emit_log に追加し Unit を返す)
- [x] 3-4: `Emit.log()` 組み込みを実装する (emit_log のスナップショットを返す)

### Db 組み込み (rusqlite)

- [x] 3-5: `Cargo.toml` に `rusqlite = { version = "0.31", features = ["bundled"] }` を追加する
- [x] 3-6: グローバル `OnceLock<Mutex<Connection>>` で Db 接続を管理する (thread_local 不要)
- [x] 3-7: `Db.execute(sql, args...)` を実装する (変更行数を返す)
- [x] 3-8: `Db.query(sql, args...)` を実装する (`List<Map<String, String>>` を返す)
- [x] 3-9: `Db.query_one(sql, args...)` を実装する (`Map<String, String>?` を返す)
- [x] 3-10: Favnir 型 → SQLite 型のバインド変換を実装する (Int/Float/String/Bool/Unit)
- [x] 3-11: SQLite 列値 → `String` の変換を実装する

### Network 組み込み (ureq)

- [x] 3-12: `Cargo.toml` に `ureq = "2"` を追加する
- [x] 3-13: `Http.get(url)` を実装する (`String!` = `Variant("ok"/"err", ...)` を返す)
- [x] 3-14: `Http.post(url, body)` を実装する

### Map 組み込みの追加

- [x] 3-15: `Map.get(map, key)` を実装する (`V?` を返す)
- [x] 3-16: `Map.set(map, key, value)` を実装する (新しい `Map` を返す)
- [x] 3-17: `Map.keys(map)` を実装する (`List<K>` を返す)
- [x] 3-18: `Map.values(map)` を実装する (`List<V>` を返す)

### Debug 組み込みの追加

- [x] 3-19: `Debug.show(value)` を実装する (任意の値を文字列表現に変換する)

### インタープリタ単体テスト

- [x] 3-20: レコード構築式の評価テストを書く
- [x] 3-21: `emit` の評価テストを書く (stub)
- [x] 3-22: `Db.execute` / `Db.query` / `Db.query_one` の評価テストを書く (インメモリ SQLite)
- [x] 3-23: `Http.get` のテストは外部依存のためスキップ (実装は完了)
- [x] 3-24: `Map.get` / `Map.set` / `Map.keys` / `Map.values` / `Debug.show` のテストを書く

---

## Phase 4: CLI

- [x] 4-1: `fav run` に `--db <url>` フラグを追加する
- [x] 4-2: `--db` フラグの接続文字列から `rusqlite::Connection` を生成して `Interpreter` に渡す
- [x] 4-3: `--db` 省略時はインメモリ DB (`:memory:`) をデフォルトとする
- [x] 4-4: `fav explain <file>` コマンドを実装する
- [x] 4-5: `fav explain` の出力フォーマットを実装する (名前・型・effect を整列表示)
- [x] 4-6: `fav help` のヘルプテキストを更新する

---

## Phase 5: サンプルと動作確認

- [x] 5-1: `examples/users.fav` を書く (User CRUD + Emit)
- [x] 5-2: `fav run --db :memory: examples/users.fav` が動くことを確認する
- [x] 5-3: `fav check examples/users.fav` が型エラーなく通ることを確認する
- [x] 5-4: `fav explain examples/users.fav` の出力を確認する
- [x] 5-5: `examples/effect_errors.fav` で E007/E008/E009 が正しく報告されることを確認する

---

## ドキュメント

- [x] 6-1: `README.md` に v0.2.0 の使い方 (`--db`, `fav explain`) を追記する
- [x] 6-2: `examples/users.fav` にコメントを書く
