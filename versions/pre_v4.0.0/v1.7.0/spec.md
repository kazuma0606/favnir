# Favnir v1.7.0 仕様書 — `Task<T>` 非同期基盤 + テストカバレッジ + 型エイリアス

作成日: 2026-05-08

> **テーマ**: 非同期計算の型基盤（`Task<T>`）・テストカバレッジ計測・型エイリアス・
> `fav watch` 複数ディレクトリ対応により、言語の表現力と開発ループをさらに強化する。
>
> **前提**: v1.6.0 完了（483 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.7.0` がビルドされ HELP テキストに反映される |
| 1 | `Task<T>` 非同期基盤 | `Task<T>` 型が定義でき、`async fn` が `Task<T>` を返し、同期ランタイムで実行できる |
| 2 | 型エイリアス | `type Alias = ExistingType` が型検査・実行に通る |
| 3 | `fav test --coverage` | テスト実行後にライン カバレッジが出力される |
| 4 | `fav watch` 複数ディレクトリ対応 | `--dir` フラグで複数ディレクトリを監視できる |
| 5 | テスト・ドキュメント | 全テスト通過、langspec.md 更新 |

### v1.7.0 の位置付け

`roadmap-v2.md` の v1.7.0 テーマ（`Task<T>` 非同期モデル）を軸にしながら、
v1.6.0 から先送りにした `fav test --coverage` と `fav watch` 複数ディレクトリも
同バージョンで完結させる。

**非同期ランタイム（tokio / async-std）は導入しない。**
v1.7.0 では `Task<T>` を型として定義し、`async fn` の構文解析・型検査・**同期実行**
（即時評価クロージャ相当）を実装する。非同期 I/O と並列実行は v1.8.0 に持ち越す。

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.7.0"`
- `main.rs`: HELP テキスト `v1.7.0`

---

## 3. Phase 1 — `Task<T>` 非同期基盤

### 3-1. 設計方針（v1.7.0 スコープ）

```
v1.7.0: 型・構文・同期実行（await なし、async runtime なし）
v1.8.0: tokio 統合・真の並列実行
```

`Task<T>` は「値を遅延評価するコンテナ型」として実装する。
同期 `bind` で解除でき、`chain` での伝播もサポートする。

### 3-2. 構文

```fav
// async fn: 戻り値の型が Task<T> になる
async fn fetch_user(id: Int) -> String !Io {
    IO.println("fetching...");
    "user_" ++ Int.show.show(id)
}

// bind で Task<T> を解除（v1.7.0 では同期実行）
bind name <- fetch_user(1)

// async trf
async trf ParseRow: String -> Int {
    |s| s |> String.to_int |> Option.unwrap(0)
}

// Task.run — 同期コンテキストから Task を強制実行
bind val <- Task.run(fetch_user(42))
```

### 3-3. 型

```
Task<T>          : 非同期計算コンテナ
Task.run<T>      : Task<T> -> T            (同期実行)
Task.map<T,U>    : Task<T> -> (T -> U) -> Task<U>
Task.and_then<T,U>: Task<T> -> (T -> Task<U>) -> Task<U>
```

### 3-4. AST の変更

```rust
// ast.rs
// FnDef に async フラグを追加
pub struct FnDef {
    pub is_async: bool,  // 追加
    ...
}

// 同様に TrfDef に is_async フラグを追加
```

`async fn` の戻り型は `Task<T>` に自動ラップされる（型検査で処理）。

### 3-5. 型システムの変更

```rust
// checker.rs
// Task<T> の型表現
Type::Task(Box<Type>)  // または Type::Named("Task", [T]) でも可

// async fn check: 戻り型 T → Task<T> に自動昇格
// bind Task<T> = expr → T を束縛（Task を解除）
```

### 3-6. VM の変更

```rust
// v1.7.0 での Task<T> の実体 = クロージャ（即時実行）
// VMValue::Task(Box<dyn FnOnce() -> VMValue>)
// Task.run → クロージャを即時実行
// bind Task<T> → Task.run と同等（暗黙解除）
```

### 3-7. エラーコード

| コード | 条件 |
|---|---|
| E057 | `async fn` の本体内で `Task<T>` の暗黙解除（bind）が非 async コンテキストで行われた |
| E058 | `Task.run` に `Task<T>` 以外の値を渡した |

### 3-8. 制限（v1.7.0）

- `Task.all` / `Task.race` / `Task.timeout` は未実装（v1.8.0）
- `async fn main()` のランタイム起動は未実装（main は同期のみ）
- 真の並列実行なし（即時評価）
- `!Network` 等との統合は v1.8.0 以降

---

## 4. Phase 2 — 型エイリアス

### 4-1. 構文

```fav
// 単純エイリアス
type UserId = Int
type UserName = String
type UserList = List<User>

// ジェネリックエイリアス
type Pair<A, B> = { first: A, second: B }

// エイリアスの利用
fn greet(id: UserId, name: UserName) -> String {
    $"User #{Int.show.show(id)}: {name}"
}
```

### 4-2. セマンティクス

- 型エイリアスは **型の別名**。`UserId` と `Int` は完全に互換。
- 既存レコード型宣言（`type User { name: String }`）とは異なる。
- ジェネリックエイリアスはパラメータ数が一致していれば適用可能。

### 4-3. AST の変更

```rust
// ast.rs の TypeDef を拡張（または TypeAlias を追加）
pub enum TypeDefBody {
    Record(Vec<FieldDef>),    // 既存
    Sum(Vec<VariantDef>),     // 既存
    Alias(Type),              // 追加: type Alias = T
}
```

### 4-4. 型検査規則

- エイリアス先の型が未定義 → E059
- エイリアスの循環参照 → E060
- 型エイリアスは checker の型解決フェーズで展開（`resolve_type_alias`）

### 4-5. エラーコード

| コード | 条件 |
|---|---|
| E059 | 型エイリアスの参照先が未定義 |
| E060 | 型エイリアスが循環している |

---

## 5. Phase 3 — `fav test --coverage`

### 5-1. 概要

テスト実行後に、ソースファイルのどの行が実行されたかを報告する。

```
$ fav test examples/math.test.fav --coverage

running 5 tests in examples/math.test.fav
  PASS  addition works        (0.1ms)
  PASS  subtraction works     (0.1ms)
  ...

test result: 5 passed; 0 failed; 0 filtered; finished in 0.6ms

coverage: examples/math.fav
  lines covered: 12 / 15 (80.0%)
  uncovered:     lines 8, 12, 14
```

### 5-2. 実装方針

カバレッジは **IR レベルのライン追跡**で実装する:

1. コンパイル時: `IRStmt::TrackLine(u32)` 文を各 IR 文の前に挿入
2. VM: `coverage_lines: Option<HashSet<u32>>` を保持
3. 各 `IRStmt::TrackLine(n)` 実行時に `n` を set に追加
4. テスト完了後: ソースファイルの実行可能行数と比較してレポート生成

### 5-3. CLI

```
fav test                             // カバレッジなし（デフォルト）
fav test --coverage                  // カバレッジを stdout に出力
fav test --coverage-report coverage/ // coverage/ ディレクトリに HTML 出力（stub）
```

### 5-4. 実行可能行数の計算

- コメント行・空行・型定義行は「実行可能行数」に含めない
- `fn` / `trf` 本体の各文の行番号を収集してトータルとする
- v1.7.0 では `.fav` ファイル単位でのみ報告（関数単位は v1.8.0 以降）

### 5-5. VM への変更

```rust
// vm.rs
pub struct VM {
    ...
    coverage: Option<HashSet<u32>>,  // 追加
}

pub fn set_coverage_tracking(enable: bool) { ... }
pub fn get_coverage() -> HashSet<u32> { ... }
```

### 5-6. IR の変更

```rust
// ir.rs
pub enum IRStmt {
    ...
    TrackLine(u32),   // 追加: カバレッジ追跡用
}
```

### 5-7. コンパイラの変更

```rust
// compiler.rs
// compile_stmt で各文の前に TrackLine(stmt.span.line) を挿入
// --coverage フラグが有効な場合のみ挿入（デフォルト off）
```

---

## 6. Phase 4 — `fav watch` 複数ディレクトリ対応

### 6-1. CLI

```
fav watch                            // カレントディレクトリの .fav ファイルを監視
fav watch --dir src                  // src/ ディレクトリを監視
fav watch --dir src --dir tests      // 複数ディレクトリを監視
fav watch --dir src --cmd test       // test コマンドで監視
fav watch --debounce 200             // デバウンス時間を ms 単位で指定（デフォルト 80ms）
```

### 6-2. 実装変更点

`cmd_watch` の `collect_watch_paths` を以下のように拡張:

```rust
pub fn cmd_watch(file: Option<&str>, cmd: &str, dirs: &[&str], debounce_ms: u64) {
    // dirs が空なら従来動作（fav.toml の src / カレントディレクトリ）
    // dirs が非空なら各ディレクトリを再帰的に監視
}

pub fn collect_watch_paths_from_dirs(dirs: &[PathBuf]) -> Vec<PathBuf> {
    // 各ディレクトリを walkdir で再帰探索して .fav ファイルを収集
}
```

- `walkdir` クレートを使用（または標準の `std::fs::read_dir` を再帰化）
- `.fav` 拡張子のファイルのみ収集
- `--debounce` でデバウンス時間を可変にする

### 6-3. Cargo.toml の変更

```toml
walkdir = "2"  # 追加（既存なら不要）
```

---

## 7. Phase 5 — テスト・ドキュメント

### 7-1. テスト要件

#### `Task<T>` テスト

| テスト名 | 検証内容 |
|---|---|
| `task_async_fn_returns_task_type` | `async fn` の戻り型が `Task<T>` になる |
| `task_bind_unwraps_task` | `bind x <- async_fn()` で `Task<T>` が解除される |
| `task_run_executes_immediately` | `Task.run(t)` が即時実行される |
| `task_map_transforms_value` | `Task.map(t, f)` が値を変換する |
| `task_e057_async_bind_outside_context` | 非 async コンテキストで Task を直接 bind すると E057 |

#### 型エイリアステスト

| テスト名 | 検証内容 |
|---|---|
| `type_alias_simple` | `type UserId = Int` が型検査を通る |
| `type_alias_compatible_with_target` | `UserId` と `Int` が互換 |
| `type_alias_generic` | `type Pair<A,B> = { first: A, second: B }` が動く |
| `type_alias_e059_unknown_target` | 未定義型のエイリアスで E059 |
| `type_alias_e060_circular` | 循環エイリアスで E060 |

#### カバレッジテスト

| テスト名 | 検証内容 |
|---|---|
| `coverage_tracks_executed_lines` | 実行された行が coverage set に含まれる |
| `coverage_excludes_unexecuted_branches` | 未実行の分岐が coverage set に含まれない |
| `coverage_report_format` | カバレッジレポートが `X / Y (Z%)` 形式で出力される |

#### `fav watch` 拡張テスト

| テスト名 | 検証内容 |
|---|---|
| `watch_collect_paths_from_dirs` | `--dir` 指定時に指定ディレクトリの .fav が収集される |
| `watch_collect_paths_multiple_dirs` | 複数 `--dir` 指定時に両方のディレクトリが監視対象になる |

### 7-2. example ファイル

- `examples/async_demo.fav` — `async fn` + `bind Task` の基本パターン
- `examples/type_alias_demo.fav` — `UserId`, `UserList` などのドメイン型エイリアス

### 7-3. ドキュメント更新

- `versions/v1.7.0/langspec.md` を新規作成
  - `Task<T>` 型と `async fn` 構文
  - `bind` による `Task<T>` の暗黙解除
  - `Task.run` / `Task.map` / `Task.and_then`
  - E057 / E058 エラーコード
  - 型エイリアス構文とセマンティクス
  - E059 / E060 エラーコード
  - `fav test --coverage` 出力フォーマット
  - `fav watch --dir` / `--debounce` フラグ
- `README.md` に v1.7.0 セクション追加

---

## 8. 完了条件（Done Definition）

- [x] `async fn f() -> String` が `Task<String>` を返す型を持つ
- [x] `bind x <- async_fn()` で `Task<T>` が解除できる
- [x] `Task.run(t)` が即時実行できる
- [x] `type UserId = Int` が型検査・実行に通る
- [x] `UserId` と `Int` が型互換
- [x] `fav test --coverage` が実行後にカバレッジ率を出力する
- [x] `fav watch --dir src --dir tests` が両ディレクトリを監視する
- [x] `fav watch --debounce 200` でデバウンス時間が変更できる
- [x] v1.6.0 の全テスト（483）が引き続き通る
- [x] `cargo build` で警告ゼロ
- [x] `Cargo.toml` バージョンが `"1.7.0"`

---

## 9. 先送り一覧（v1.7.0 では対応しない）

| 制約 | バージョン |
|---|---|
| `Task.all` / `Task.race` / `Task.timeout`（並列実行 API） | v1.8.0 |
| `async fn main()` のランタイム起動（tokio 統合） | v1.8.0 |
| 真の非同期 I/O（tokio / async-std） | v1.8.0 |
| `fav test --coverage-report` の HTML 出力 | v1.8.0 以降 |
| カバレッジの関数単位レポート | v1.8.0 以降 |
| 型エイリアスの `type` 引数付き展開の完全サポート | v1.8.0 以降 |
| `fav bench`（簡易ベンチマーク） | v1.8.0 以降 |
| `fav migrate`（v1.x → v2.0.0 キーワードリネーム） | v2.0.0 |
| `trf` → `stage` / `flw` → `seq` リネーム | v2.0.0 |
| セルフホスト（パーサー Favnir 移植） | v2.0.0 |
