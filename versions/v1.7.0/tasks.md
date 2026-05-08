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

- [ ] `Cargo.toml` の `version` を `"1.7.0"` に更新
- [ ] `Cargo.toml` に `walkdir = "2"` を追加（まだなければ）
- [ ] `main.rs` の HELP テキストを `v1.7.0` に更新
- [ ] `cargo build` が通ること

---

## Phase 1: `Task<T>` 非同期基盤

### 1-1: `ast.rs` の変更

- [ ] `FnDef` に `is_async: bool` フィールドを追加
- [ ] `TrfDef` に `is_async: bool` フィールドを追加
- [ ] 既存コードが `is_async: false` でコンパイルされること

### 1-2: `lexer.rs` の変更

- [ ] `TokenKind::Async` を追加
- [ ] `"async"` → `TokenKind::Async` のキーワードマッピングを追加
- [ ] テスト: `async fn` を含むソースが `Async Fn` トークン列を返す

### 1-3: `parser.rs` の変更

- [ ] `parse_fn_def` の先頭で `TokenKind::Async` を消費し `is_async` を設定
- [ ] `parse_trf_def` の先頭で `TokenKind::Async` を消費し `is_async` を設定
- [ ] パーサーテスト: `async fn f() -> String { "x" }` がパースできる
- [ ] パーサーテスト: `async trf Foo: Int -> String { |x| Int.show.show(x) }` がパースできる

### 1-4: `checker.rs` の変更

- [ ] `Type::Task(Box<Type>)` バリアントを追加
- [ ] `async fn` の戻り型を `Task<T>` に自動ラップする処理を追加
- [ ] `bind` で `Task<T>` の rhs を `T` として束縛する処理を追加
- [ ] `Task` ネームスペースをビルトインとして登録（`Task.run`, `Task.map`, `Task.and_then`）
- [ ] テスト: `async fn f() -> String` の型が `() -> Task<String>` になる
- [ ] テスト: `bind x <- async_fn()` で `x` の型が `String` になる

### 1-5: `compiler.rs` の変更

- [ ] `async fn` の本体を `IRExpr::Closure` でラップして `Task` コンストラクタ呼び出しに lowering
- [ ] `Task.run` のビルトイン呼び出しを IRExpr として生成

### 1-6: `vm.rs` の変更

- [ ] `VMValue::Task(Arc<dyn Fn() -> VMValue + Send + Sync>)` を追加
- [ ] `vm_call_builtin` に `Task.run` / `Task.map` / `Task.and_then` を追加
  - [ ] `Task.run(t)`: `VMValue::Task` のクロージャを即時実行
  - [ ] `Task.map(t, f)`: 新しい `VMValue::Task` を返す（`f` を合成）
  - [ ] `Task.and_then(t, f)`: フラット化した `VMValue::Task` を返す
- [ ] `bind` で `VMValue::Task` を自動解除する処理を追加
- [ ] E058: `Task.run` に非 Task 値を渡した場合のランタイムエラー

### 1-7: テスト

- [ ] テスト: `task_async_fn_returns_task_type` — `async fn` の型が `Task<T>` になる
- [ ] テスト: `task_bind_unwraps_task` — `bind x <- async_fn()` で Task が解除される
- [ ] テスト: `task_run_executes_immediately` — `Task.run(t)` が即時実行する
- [ ] テスト: `task_map_transforms_value` — `Task.map(t, f)` が値を変換する
- [ ] テスト: `task_and_then_chains` — `Task.and_then(t, f)` で Task をチェーンできる
- [ ] `cargo test` が全通過すること

---

## Phase 2: 型エイリアス

### 2-1: `ast.rs` の変更

- [ ] `TypeDefBody` enum に `Alias(TypeExpr)` バリアントを追加
- [ ] `Pattern::span()` 等で `TypeDefBody::Alias` のマッチ抜けがないことを確認

### 2-2: `parser.rs` の変更

- [ ] `parse_type_def` で `type Name = TypeExpr` のとき `TypeDefBody::Alias` を返す
  - [ ] `{` が続く場合は既存の Record/Sum パース
  - [ ] `=` が続いた後 `{` がない場合はエイリアスパース
- [ ] パーサーテスト: `type UserId = Int` がパースできる
- [ ] パーサーテスト: `type UserList = List<User>` がパースできる
- [ ] パーサーテスト: `type Pair<A, B> = { first: A, second: B }` がパースできる

### 2-3: `checker.rs` の変更

- [ ] `type_aliases: HashMap<String, Type>` を `CheckCtx` に追加
- [ ] `register_type_alias` を実装（E059: 未定義参照先）
- [ ] 循環チェック（E060）を `resolve_alias_with_cycle_check` で実装
- [ ] `resolve_type` でエイリアスを展開する処理を追加
- [ ] テスト: `type_alias_simple` — `type UserId = Int` で型検査が通る
- [ ] テスト: `type_alias_compatible_with_target` — `UserId` と `Int` の互換性
- [ ] テスト: `type_alias_generic` — `type Pair<A,B> = { first: A, second: B }` が動く
- [ ] テスト: `type_alias_e059_unknown_target` — 未定義型で E059 が発生する
- [ ] テスト: `type_alias_e060_circular` — 循環エイリアスで E060 が発生する

### 2-4: `compiler.rs` の変更

- [ ] `TypeDefBody::Alias` のケースを `compile_type_def` に追加（IR には出力しない）
- [ ] エイリアス型の参照がコンパイル時に展開されること

### 2-5: テスト

- [ ] テスト: `type_alias_exec_compatible` — `UserId` 型の値が `Int` を受け取る関数に渡せる
- [ ] テスト: `type_alias_in_record_field` — レコードフィールドの型にエイリアスが使える
- [ ] `cargo test` が全通過すること

---

## Phase 3: `fav test --coverage`

### 3-1: `ir.rs` の変更

- [ ] `IRStmt::TrackLine(u32)` バリアントを追加
- [ ] `collect_binds_in_stmt` / その他の IRStmt マッチ箇所に `TrackLine` ケースを追加

### 3-2: `compiler.rs` の変更

- [ ] `CompileCtx` に `coverage_mode: bool` フィールドを追加
- [ ] `compile_stmt` で `coverage_mode` が `true` の場合、各文の前に `IRStmt::TrackLine(line)` を挿入

### 3-3: `codegen.rs` の変更

- [ ] `Opcode::TrackLine(u32)` を追加（または IRStmt::TrackLine を直接 VM で処理）
- [ ] `IRStmt::TrackLine(n)` を `Opcode::TrackLine(n)` に変換

### 3-4: `vm.rs` の変更

- [ ] `VM` に `coverage: Option<HashSet<u32>>` フィールドを追加
- [ ] `pub fn enable_coverage(&mut self)` を実装
- [ ] `pub fn take_coverage(&mut self) -> HashSet<u32>` を実装
- [ ] `Opcode::TrackLine(n)` の実行ハンドラを追加

### 3-5: `driver.rs` の変更

- [ ] `cmd_test` シグネチャに `coverage: bool` を追加
- [ ] `coverage == true` の場合:
  - [ ] コンパイル時に `coverage_mode: true` を渡す
  - [ ] VM の `enable_coverage()` を呼び出す
  - [ ] テスト完了後に `take_coverage()` でデータを収集
  - [ ] `format_coverage_report(file_path, source, &executed)` を呼び出して出力
- [ ] `format_coverage_report` を実装
  - [ ] カバレッジ率を `X / Y (Z%)` 形式で出力
  - [ ] 未カバー行番号を列挙
- [ ] `is_executable_line(source, line) -> bool` を実装
- [ ] `main.rs` の `test` コマンドに `--coverage` フラグを追加

### 3-6: テスト

- [ ] テスト: `coverage_tracks_executed_lines` — 実行された行が coverage set に含まれる
- [ ] テスト: `coverage_excludes_unexecuted_branches` — 未実行分岐が coverage set に含まれない
- [ ] テスト: `coverage_report_format` — レポートが `X / Y (Z%)` 形式で出力される
- [ ] `cargo test` が全通過すること

---

## Phase 4: `fav watch` 複数ディレクトリ対応

### 4-1: `driver.rs` の変更

- [ ] `cmd_watch` シグネチャに `extra_dirs: &[&str]` と `debounce_ms: u64` を追加
- [ ] `collect_watch_paths_from_dir(dir: &str) -> Vec<PathBuf>` を実装
  - [ ] 再帰的に `.fav` ファイルを収集（`collect_fav_files_recursive`）
- [ ] `cmd_watch` で `extra_dirs` の各ディレクトリからパスを追加収集
- [ ] デバウンス時間を `debounce_ms` で可変にする

### 4-2: `main.rs` の変更

- [ ] `watch` コマンドに `--dir <path>` フラグを追加（複数指定可能）
- [ ] `watch` コマンドに `--debounce <ms>` フラグを追加
- [ ] 収集した `dirs` と `debounce_ms` を `cmd_watch` に渡す

### 4-3: テスト

- [ ] テスト: `watch_collect_paths_from_dirs` — `--dir` で指定したディレクトリの .fav が収集される
- [ ] テスト: `watch_collect_paths_multiple_dirs` — 複数 `--dir` 指定で両ディレクトリが含まれる
- [ ] `cargo test` が全通過すること

---

## Phase 5: テスト・ドキュメント

### 5-1: example ファイルの追加

- [ ] `examples/async_demo.fav` を作成
  - [ ] `async fn` + `bind Task` の基本パターンを示す
  - [ ] `fav run` / `fav check` でエラーなしを確認
- [ ] `examples/type_alias_demo.fav` を作成
  - [ ] `UserId`, `UserName`, `UserList` などのドメイン型エイリアスを示す
  - [ ] `fav run` でエラーなしを確認

### 5-2: langspec.md の作成

- [ ] `versions/v1.7.0/langspec.md` を新規作成
  - [ ] `Task<T>` 型と `async fn` / `async trf` 構文
  - [ ] `bind` による `Task<T>` の暗黙解除
  - [ ] `Task.run` / `Task.map` / `Task.and_then`
  - [ ] E057 / E058 エラーコード
  - [ ] 型エイリアス構文（単純 / ジェネリック）
  - [ ] 型エイリアスのセマンティクス（完全互換）
  - [ ] E059 / E060 エラーコード
  - [ ] `fav test --coverage` 出力フォーマット
  - [ ] `fav watch --dir` / `--debounce` フラグ

### 5-3: README.md の更新

- [ ] v1.7.0 セクションを追加（Task<T> / 型エイリアス / coverage / watch 複数ディレクトリ）

### 5-4: 全体確認

- [ ] `cargo build` で Rust コンパイラ警告ゼロ
- [ ] `cargo test` 全テスト通過（v1.6.0 継承 483 + 新規テスト）
- [ ] `async fn f() -> String` の型が `Task<String>` になる
- [ ] `bind x <- async_fn()` で `x` が `String` 型になる
- [ ] `type UserId = Int` が型検査・実行を通る
- [ ] `fav test --coverage` がカバレッジ率を出力する
- [ ] `fav watch --dir src --dir tests` が両ディレクトリを監視する
- [ ] `Cargo.toml` バージョンが `"1.7.0"`

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `async fn` が `Task<T>` 型を持ち、`bind` で解除できる
- [ ] `Task.run` / `Task.map` / `Task.and_then` が動作する
- [ ] `type UserId = Int` が型互換として扱われる
- [ ] 循環エイリアスで E060 が発生する
- [ ] `fav test --coverage` でカバレッジが出力される
- [ ] `fav watch --dir` で複数ディレクトリが監視できる
- [ ] `fav watch --debounce` でデバウンス時間が変更できる
- [ ] v1.6.0 の全テストが引き続き通る
- [ ] `Cargo.toml` バージョンが `"1.7.0"`

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
