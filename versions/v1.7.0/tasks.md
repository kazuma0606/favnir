# Favnir v1.7.0 タスク一覧 — `Task<T>` 非同期基盤 + テストカバレッジ + 型エイリアス

作成日: 2026-05-08

> **ゴール**: `Task<T>` 非同期基盤・型エイリアス・`fav test --coverage`・`fav watch` 複数ディレクトリ対応により、
> 言語の表現力と開発ループをさらに強化する。
>
> **前提**: v1.6.0 完了（483 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.7.0"` に更新
- [x] `Cargo.toml` に `walkdir = "2"` を追加（まだなければ）
- [x] `main.rs` の HELP テキストを `v1.7.0` に更新
- [x] `cargo build` が通ること

---

## Phase 1: `Task<T>` 非同期基盤

### 1-1: `ast.rs` の変更

- [x] `FnDef` に `is_async: bool` フィールドを追加
- [x] `TrfDef` に `is_async: bool` フィールドを追加
- [x] 既存コードが `is_async: false` でコンパイルされること

### 1-2: `lexer.rs` の変更

- [x] `TokenKind::Async` を追加
- [x] `"async"` → `TokenKind::Async` のキーワードマッピングを追加
- [x] テスト: `async fn` を含むソースが `Async Fn` トークン列を返す

### 1-3: `parser.rs` の変更

- [x] `parse_fn_def` の先頭で `TokenKind::Async` を消費し `is_async` を設定
- [x] `parse_trf_def` の先頭で `TokenKind::Async` を消費し `is_async` を設定
- [x] パーサーテスト: `async fn f() -> String { "x" }` がパースできる
- [x] パーサーテスト: `async trf Foo: Int -> String { |x| Int.show.show(x) }` がパースできる

### 1-4: `checker.rs` の変更

- [x] `Type::Task(Box<Type>)` バリアントを追加
- [x] `async fn` の戻り型を `Task<T>` に自動ラップする処理を追加
- [x] `bind` で `Task<T>` の rhs を `T` として束縛する処理を追加
- [x] `Task` ネームスペースをビルトインとして登録（`Task.run`, `Task.map`, `Task.and_then`）
- [x] テスト: `async fn f() -> String` の型が `() -> Task<String>` になる
- [x] テスト: `bind x <- async_fn()` で `x` の型が `String` になる

### 1-5: `compiler.rs` の変更

- [x] `async fn` の本体を `IRExpr::Closure` でラップして `Task` コンストラクタ呼び出しに lowering
- [x] `Task.run` のビルトイン呼び出しを IRExpr として生成

### 1-6: `vm.rs` の変更

- [x] `VMValue::Task(Arc<dyn Fn() -> VMValue + Send + Sync>)` を追加
- [x] `vm_call_builtin` に `Task.run` / `Task.map` / `Task.and_then` を追加
  - [x] `Task.run(t)`: `VMValue::Task` のクロージャを即時実行
  - [x] `Task.map(t, f)`: 新しい `VMValue::Task` を返す（`f` を合成）
  - [x] `Task.and_then(t, f)`: フラット化した `VMValue::Task` を返す
- [x] `bind` で `VMValue::Task` を自動解除する処理を追加
- [x] E058: `Task.run` に非 Task 値を渡した場合のランタイムエラー

### 1-7: テスト

- [x] テスト: `task_async_fn_returns_task_type` — `async fn` の型が `Task<T>` になる
- [x] テスト: `task_bind_unwraps_task` — `bind x <- async_fn()` で Task が解除される
- [x] テスト: `task_run_executes_immediately` — `Task.run(t)` が即時実行する
- [x] テスト: `task_map_transforms_value` — `Task.map(t, f)` が値を変換する
- [x] テスト: `task_and_then_chains` — `Task.and_then(t, f)` で Task をチェーンできる
- [x] `cargo test` が全通過すること

---

## Phase 2: 型エイリアス

### 2-1: `ast.rs` の変更

- [x] `TypeDefBody` enum に `Alias(TypeExpr)` バリアントを追加
- [x] `Pattern::span()` 等で `TypeDefBody::Alias` のマッチ抜けがないことを確認

### 2-2: `parser.rs` の変更

- [x] `parse_type_def` で `type Name = TypeExpr` のとき `TypeDefBody::Alias` を返す
  - [x] `{` が続く場合は既存の Record/Sum パース
  - [x] `=` が続いた後 `{` がない場合はエイリアスパース
- [x] パーサーテスト: `type UserId = Int` がパースできる
- [x] パーサーテスト: `type UserList = List<User>` がパースできる
- [x] パーサーテスト: `type Pair<A, B> = { first: A, second: B }` がパースできる

### 2-3: `checker.rs` の変更

- [x] `type_aliases: HashMap<String, Type>` を `CheckCtx` に追加
- [x] `register_type_alias` を実装（E059: 未定義参照先）
- [x] 循環チェック（E060）を `resolve_alias_with_cycle_check` で実装
- [x] `resolve_type` でエイリアスを展開する処理を追加
- [x] テスト: `type_alias_simple` — `type UserId = Int` で型検査が通る
- [x] テスト: `type_alias_compatible_with_target` — `UserId` と `Int` の互換性
- [x] テスト: `type_alias_generic` — `type Pair<A,B> = { first: A, second: B }` が動く
- [x] テスト: `type_alias_e059_unknown_target` — 未定義型で E059 が発生する
- [x] テスト: `type_alias_e060_circular` — 循環エイリアスで E060 が発生する

### 2-4: `compiler.rs` の変更

- [x] `TypeDefBody::Alias` のケースを `compile_type_def` に追加（IR には出力しない）
- [x] エイリアス型の参照がコンパイル時に展開されること

### 2-5: テスト

- [x] テスト: `type_alias_exec_compatible` — `UserId` 型の値が `Int` を受け取る関数に渡せる
- [x] テスト: `type_alias_in_record_field` — レコードフィールドの型にエイリアスが使える
- [x] `cargo test` が全通過すること

---

## Phase 3: `fav test --coverage`

### 3-1: `ir.rs` の変更

- [x] `IRStmt::TrackLine(u32)` バリアントを追加
- [x] `collect_binds_in_stmt` / その他の IRStmt マッチ箇所に `TrackLine` ケースを追加

### 3-2: `compiler.rs` の変更

- [x] `CompileCtx` に `coverage_mode: bool` フィールドを追加
- [x] `compile_stmt` で `coverage_mode` が `true` の場合、各文の前に `IRStmt::TrackLine(line)` を挿入

### 3-3: `codegen.rs` の変更

- [x] `Opcode::TrackLine(u32)` を追加（または IRStmt::TrackLine を直接 VM で処理）
- [x] `IRStmt::TrackLine(n)` を `Opcode::TrackLine(n)` に変換

### 3-4: `vm.rs` の変更

- [x] `VM` に `coverage: Option<HashSet<u32>>` フィールドを追加
- [x] `pub fn enable_coverage(&mut self)` を実装
- [x] `pub fn take_coverage(&mut self) -> HashSet<u32>` を実装
- [x] `Opcode::TrackLine(n)` の実行ハンドラを追加

### 3-5: `driver.rs` の変更

- [x] `cmd_test` シグネチャに `coverage: bool` を追加
- [x] `coverage == true` の場合:
  - [x] コンパイル時に `coverage_mode: true` を渡す
  - [x] VM の `enable_coverage()` を呼び出す
  - [x] テスト完了後に `take_coverage()` でデータを収集
  - [x] `format_coverage_report(file_path, source, &executed)` を呼び出して出力
- [x] `format_coverage_report` を実装
  - [x] カバレッジ率を `X / Y (Z%)` 形式で出力
  - [x] 未カバー行番号を列挙
- [x] `is_executable_line(source, line) -> bool` を実装
- [x] `main.rs` の `test` コマンドに `--coverage` フラグを追加

### 3-6: テスト

- [x] テスト: `coverage_tracks_executed_lines` — 実行された行が coverage set に含まれる
- [x] テスト: `coverage_excludes_unexecuted_branches` — 未実行分岐が coverage set に含まれない
- [x] テスト: `coverage_report_format` — レポートが `X / Y (Z%)` 形式で出力される
- [x] `cargo test` が全通過すること

---

## Phase 4: `fav watch` 複数ディレクトリ対応

### 4-1: `driver.rs` の変更

- [x] `cmd_watch` シグネチャに `extra_dirs: &[&str]` と `debounce_ms: u64` を追加
- [x] `collect_watch_paths_from_dir(dir: &str) -> Vec<PathBuf>` を実装
  - [x] 再帰的に `.fav` ファイルを収集（`collect_fav_files_recursive`）
- [x] `cmd_watch` で `extra_dirs` の各ディレクトリからパスを追加収集
- [x] デバウンス時間を `debounce_ms` で可変にする

### 4-2: `main.rs` の変更

- [x] `watch` コマンドに `--dir <path>` フラグを追加（複数指定可能）
- [x] `watch` コマンドに `--debounce <ms>` フラグを追加
- [x] 収集した `dirs` と `debounce_ms` を `cmd_watch` に渡す

### 4-3: テスト

- [x] テスト: `watch_collect_paths_from_dirs` — `--dir` で指定したディレクトリの .fav が収集される
- [x] テスト: `watch_collect_paths_multiple_dirs` — 複数 `--dir` 指定で両ディレクトリが含まれる
- [x] `cargo test` が全通過すること

---

## Phase 5: テスト・ドキュメント

### 5-1: example ファイルの追加

- [x] `examples/async_demo.fav` を作成
  - [x] `async fn` + `bind Task` の基本パターンを示す
  - [x] `fav run` / `fav check` でエラーなしを確認
- [x] `examples/type_alias_demo.fav` を作成
  - [x] `UserId`, `UserName`, `UserList` などのドメイン型エイリアスを示す
  - [x] `fav run` でエラーなしを確認

### 5-2: langspec.md の作成

- [x] `versions/v1.7.0/langspec.md` を新規作成
  - [x] `Task<T>` 型と `async fn` / `async trf` 構文
  - [x] `bind` による `Task<T>` の暗黙解除
  - [x] `Task.run` / `Task.map` / `Task.and_then`
  - [x] E057 / E058 エラーコード
  - [x] 型エイリアス構文（単純 / ジェネリック）
  - [x] 型エイリアスのセマンティクス（完全互換）
  - [x] E059 / E060 エラーコード
  - [x] `fav test --coverage` 出力フォーマット
  - [x] `fav watch --dir` / `--debounce` フラグ

### 5-3: README.md の更新

- [x] v1.7.0 セクションを追加（Task<T> / 型エイリアス / coverage / watch 複数ディレクトリ）

### 5-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（v1.6.0 継承 483 + 新規テスト）
- [x] `async fn f() -> String` の型が `Task<String>` になる
- [x] `bind x <- async_fn()` で `x` が `String` 型になる
- [x] `type UserId = Int` が型検査・実行を通る
- [x] `fav test --coverage` がカバレッジ率を出力する
- [x] `fav watch --dir src --dir tests` が両ディレクトリを監視する
- [x] `Cargo.toml` バージョンが `"1.7.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `async fn` が `Task<T>` 型を持ち、`bind` で解除できる
- [x] `Task.run` / `Task.map` / `Task.and_then` が動作する
- [x] `type UserId = Int` が型互換として扱われる
- [x] 循環エイリアスで E060 が発生する
- [x] `fav test --coverage` でカバレッジが出力される
- [x] `fav watch --dir` で複数ディレクトリが監視できる
- [x] `fav watch --debounce` でデバウンス時間が変更できる
- [x] v1.6.0 の全テストが引き続き通る
- [x] `Cargo.toml` バージョンが `"1.7.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| `Task.all` / `Task.race` / `Task.timeout`（並列実行 API） | v1.8.0 |
| `async fn main()` のランタイム起動（tokio 統合） | v1.8.0 |
| 真の非同期 I/O（tokio / async-std） | v1.8.0 |
| `fav test --coverage-report` の HTML 出力 | v1.8.0 以降 |
| カバレッジの関数単位レポート | v1.8.0 以降 |
| `fav bench`（簡易ベンチマーク） | v1.8.0 以降 |
| 文字列補間内の入れ子補間 | v2.0.0 以降 |
| レコード分解でのスプレッド（`{ name, ..rest }`） | v2.0.0 以降 |
| `fav migrate`（v1.x → v2.0.0 キーワードリネーム） | v2.0.0 |
| `trf` → `stage` / `flw` → `seq` リネーム | v2.0.0 |
| セルフホスト（パーサー Favnir 移植） | v2.0.0 |
