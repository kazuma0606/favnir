# Favnir v2.9.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.9.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.9.0` に更新
- [x] `src/main.rs`: `print_welcome()` の `"Favnir v2.8.0"` を `"Favnir v2.9.0"` に更新

---

## Phase 1 — E067 解消 (`collect` 内 `for`)

### `src/middle/checker.rs`

- [x] `check_stmt` の `Stmt::ForIn` ハンドラから E067 ガードを削除する
  ```rust
  // 削除する:
  // if self.in_collect {
  //     self.type_error("E067", "`for` inside `collect` block is not supported in v1.9.0", ...);
  //     return;
  // }
  ```
- [x] `collect_yield_types(&mut self, stmts: &[Stmt]) -> Vec<Type>` ヘルパーを追加する
  - `Stmt::Yield` → `check_expr` して型を収集
  - `Stmt::ForIn` → イテレータの型を確認し、for ボディを再帰スキャン
  - その他のステートメント → `check_stmt` で通常チェック
- [x] `Expr::Collect` ハンドラを `collect_yield_types` を使うよう更新する
  - 変更前: `for stmt in &block.stmts { if Yield { push } else { check_stmt } }`
  - 変更後: `let yield_tys = self.collect_yield_types(&block.stmts);`
- [x] 既存テスト `for_in_in_collect_e067` を削除する
  - `E067` を期待するテストは不要になる
- [x] `collect_for_in_allowed` テストを追加する（E067 が出ないことを確認）

---

## Phase 2 — `Type::Stream(Box<Type>)` 追加

### `src/middle/checker.rs`

- [x] `Type` enum に `Stream(Box<Type>)` を追加する
  - コメント: `/// Stream<T> lazy sequence (v2.9.0)`
- [x] `is_compatible` に `(Type::Stream(a), Type::Stream(b)) => a.is_compatible(b)` を追加
- [x] `display` に `Type::Stream(t) => format!("Stream<{}>", t.display())` を追加
- [x] `apply` に `Type::Stream(t) => Type::Stream(Box::new(self.apply(t)))` を追加
- [x] `occurs` に `Type::Stream(t) => occurs(var, t)` を追加
- [x] `unify` に `(Type::Stream(a), Type::Stream(b)) => unify(a, b)` を追加
- [x] `substitute_self_in_type` に `Type::Stream(t) => ...` を追加
- [x] その他 `Type` の exhaustive match に `Stream` ケースを追加
  - `is_type_implementing`、`resolve_field_access_type` 等
- [x] `parse_type_expr` に `"Stream"` のケースを追加（`Task` の直後）:
  ```rust
  "Stream" => Type::Stream(Box::new(
      resolved_args.into_iter().next().unwrap_or(Type::Unknown),
  )),
  ```
- [x] グローバル名前空間リストに `"Stream"` を追加:
  ```rust
  "Math", "List", "String", "Option", "Result", "Db", "Http", "Map", "Debug", "Emit",
  "Util", "Trace", "File", "Json", "Csv", "Task", "Random", "Stream",
  ```
- [x] `resolve_field_access_type` に `("Stream", method)` ケースを追加:
  - `("Stream", "from")` → `Some(Type::Unknown)`
  - `("Stream", "of")` → `Some(Type::Stream(Box::new(Type::Unknown)))`
  - `("Stream", "map")` → `Some(Type::Stream(Box::new(Type::Unknown)))`
  - `("Stream", "filter")` → `Some(Type::Stream(Box::new(Type::Unknown)))`
  - `("Stream", "take")` → `Some(Type::Stream(Box::new(Type::Unknown)))`
  - `("Stream", "collect")` → `Some(Type::List(Box::new(Type::Unknown)))`
- [x] `Stream<T>` 型チェックテストを追加する（2件）

---

## Phase 3 — `VMValue::Stream` と VM ハンドラ追加

### `src/backend/vm.rs`

- [x] `VMStream` enum を `VMValue` の前に定義する:
  ```rust
  #[derive(Debug, Clone)]
  pub enum VMStream {
      Gen { seed: VMValue, next_fn: VMValue },
      Of(Vec<VMValue>),
      Map { inner: Box<VMStream>, map_fn: VMValue },
      Filter { inner: Box<VMStream>, pred_fn: VMValue },
      Take { inner: Box<VMStream>, n: i64 },
  }
  ```
- [x] `VMValue` enum に `Stream(Box<VMStream>)` を追加する
- [x] `vmvalue_type_name` に `VMValue::Stream(_) => "Stream"` を追加
- [x] `vmvalue_repr` に `VMValue::Stream(_) => "<stream>".to_string()` を追加
- [x] その他 `VMValue` の exhaustive match に `Stream` ケースを追加
  - `PartialEq` / `Display` 等があれば追加
- [x] `VM::call_builtin` に `"Stream.from"` ハンドラを追加
  - 引数: seed, next_fn
  - 戻り値: `VMValue::Stream(Box::new(VMStream::Gen { seed, next_fn }))`
- [x] `VM::call_builtin` に `"Stream.of"` ハンドラを追加
  - 引数: `VMValue::List(items)`
  - 戻り値: `VMValue::Stream(Box::new(VMStream::Of(items)))`
- [x] `VM::call_builtin` に `"Stream.map"` ハンドラを追加
  - 引数: stream, map_fn
  - 戻り値: `VMValue::Stream(Box::new(VMStream::Map { ... }))`
- [x] `VM::call_builtin` に `"Stream.filter"` ハンドラを追加
  - 引数: stream, pred_fn
  - 戻り値: `VMValue::Stream(Box::new(VMStream::Filter { ... }))`
- [x] `VM::call_builtin` に `"Stream.take"` ハンドラを追加
  - 引数: stream, n: Int
  - 戻り値: `VMValue::Stream(Box::new(VMStream::Take { ... }))`
- [x] `VM::call_builtin` に `"Stream.collect"` ハンドラを追加
  - 引数: stream
  - `self.materialize_stream(artifact, *inner)` を呼び出す
  - 戻り値: `VMValue::List(items)`
- [x] `VM::materialize_stream` ヘルパーを追加:
  - `VMStream::Gen` → エラー（"`use Stream.take first`"）
  - `VMStream::Of(items)` → `Ok(items)`
  - `VMStream::Map` → inner を materialize して各要素に `call_value`
  - `VMStream::Filter` → inner を materialize して pred で絞り込み
  - `VMStream::Take` → inner が `Gen` なら seed を n 回展開、それ以外は materialize して truncate

---

## Phase 4 — コンパイラ グローバル登録

### `src/middle/compiler.rs`

- [x] 第 1 グローバル登録ループ（Phase 0 前半、`std_state_defs` の前）に `"Stream"` を追加
- [x] 第 2 グローバル登録ループ（`ctx.next_global_idx = ...` の前）に `"Stream"` を追加

---

## Phase 5 — テスト追加

### `src/driver.rs`

- [x] `collect_for_in_yield_all` テストを追加
  - `collect { for x in List.range(0, 5) { yield x; } }` → `[0,1,2,3,4]`
- [x] `collect_for_in_yield_filtered` テストを追加
  - `collect { for x in List.range(0, 6) { if x % 2 == 0 { yield x; } } }` → `[0,2,4]`
- [x] `collect_for_in_yield_transformed` テストを追加
  - `collect { for x in List.range(0, 5) { yield x * 2; } }` → `[0,2,4,6,8]`
- [x] `stream_from_take_collect` テストを追加
  - `Stream.from(0, |n| n + 1)` → `Stream.take(s, 5)` → `Stream.collect` → `[0,1,2,3,4]`
- [x] `stream_of_collect` テストを追加
  - `Stream.of(list)` → `Stream.collect` → 元のリスト
- [x] `stream_map_collect` テストを追加
  - `Stream.map(stream, |x| x * 2)` → `Stream.collect` → 各要素 2 倍
- [x] `stream_filter_collect` テストを追加
  - `Stream.filter(stream, |x| x % 2 == 0)` → `Stream.collect` → 偶数のみ
- [x] `stream_take_limits_length` テストを追加
  - `Stream.take(stream, 3)` → `Stream.collect` → 3件
- [x] `stream_of_map_filter_pipeline` テストを追加
  - `Stream.of(list)` → `Stream.map` → `Stream.filter` → `Stream.collect`
- [x] `stream_collect_infinite_errors` テストを追加
  - `Stream.from(0, |n| n + 1)` を `Stream.take` なしで `Stream.collect` → ランタイムエラー
  - `exec_source_expect_error` パターンで検証

---

## Phase 6 — examples/stream_demo 作成

- [x] `fav/examples/stream_demo/` ディレクトリを作成
- [x] `fav/examples/stream_demo/fav.toml` を作成
  - `[rune] name = "stream_demo"  version = "0.1.0"  src = "src"`
- [x] `fav/examples/stream_demo/src/main.fav` を作成
  - `Stream.from`, `Stream.take`, `Stream.collect` のデモ
  - `Stream.of`, `Stream.filter`, `Stream.map` のデモ
  - `collect { for x in list { yield ...; } }` のデモ

---

## Phase 7 — ドキュメント・最終確認

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.8.0: 625 → 目標 637）

### ドキュメント作成

- [x] `versions/v2.9.0/langspec.md` を作成
  - E067 解消 / `collect { for ... { yield ... } }` の仕様
  - `Stream<T>` 型と `Stream.*` 関数の API ドキュメント
  - `materialize` のタイミング（`Stream.collect` で実体化）
  - 無限ストリームのエラー処理
  - 互換性（既存テスト影響なし）

---

## 完了条件チェック

- [x] `Cargo.toml` バージョンが `"2.9.0"`
- [x] E067 テストが削除されている
- [x] `collect { for x in list { yield x; } }` が型チェックを通り正しく動作する
- [x] `collect { for x in list { if cond { yield x; } } }` が動作する
- [x] `Type::Stream(Box<Type>)` が checker.rs に追加されている
- [x] `Stream<T>` がソースコードで型アノテーションとして使える
- [x] `Stream.from(0, |n| n + 1)` が `VMValue::Stream` を返す
- [x] `Stream.take(s, 5)` が `VMValue::Stream` を返す
- [x] `Stream.collect(s)` が `VMValue::List` を返す
- [x] `Stream.of(list)` が `VMValue::Stream` を返す
- [x] `Stream.map(s, f)` が `VMValue::Stream` を返す
- [x] `Stream.filter(s, pred)` が `VMValue::Stream` を返す
- [x] 無限ストリームの `Stream.collect` がランタイムエラーになる
- [x] `cargo build` 警告ゼロ
- [x] `cargo test` 全テスト通過（目標 637）
- [x] `versions/v2.9.0/langspec.md` 作成済み
