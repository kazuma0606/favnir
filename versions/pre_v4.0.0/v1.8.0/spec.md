# Favnir v1.8.0 仕様書 — `Task<T>` 非同期完成 + Coverage 強化 + `fav bench`

作成日: 2026-05-09

> **テーマ**: v1.7.0 で同期ベースとして確立した `Task<T>` を拡張し、
> 並列実行 API・`async fn main()` サポート・`chain` との統合を追加する。
> さらに `fav test --coverage` を関数単位レポートに強化し、
> `fav bench` 簡易ベンチマーク機能を新設する。
>
> **前提**: v1.7.0 完了（498 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.8.0` がビルドされ HELP テキストに反映される |
| 1 | `Task` 並列 API | `Task.all` / `Task.race` / `Task.timeout` が型チェック・実行できる |
| 2 | `async fn main()` | async なエントリポイントを宣言でき、自動実行される |
| 3 | `chain` + `Task<T>` 統合 | `chain x <- async_fn()` で `Task<T>!` が一括処理される |
| 4 | Coverage 強化 | 関数単位カバレッジが `fav test --coverage` に表示される |
| 5 | `fav bench` | `bench "desc" { ... }` ブロックが計測・レポートされる |
| 6 | テスト・ドキュメント | 全テスト通過、langspec.md 更新 |

### v1.8.0 の位置付け

`roadmap-v2.md` の v1.7.0 テーマ（`Task<T>` 非同期モデルの完成形）のうち、
v1.7.0 で先送りした項目を本バージョンで完結させる。

**v1.8.0 でも tokio / async-std は導入しない。**
`Task.all` / `Task.race` / `Task.timeout` は**同期透明ラッパー**として実装する。
真の並列実行（OS スレッド・tokio ランタイム）は v1.9.0 以降の検討対象とする。

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.8.0"`
- `main.rs`: HELP テキスト `v1.8.0`

---

## 3. Phase 1 — `Task` 並列 API

### 3-1. 設計方針

v1.7.0 の同期透明モデルを維持したまま API 面を拡張する。
v1.8.0 での実体はすべて**即時評価**であり、実行順序は引数の評価順と同一。

将来（v1.9.0+）に tokio バックエンドを差し込んだとき、呼び出し側コードは無変更で
真の並列実行に切り替わる設計にする。

### 3-2. 追加 API

#### `Task.all`

```favnir
// 型: List<Task<T>> -> Task<List<T>>
bind results <- Task.all([fetch_a(), fetch_b(), fetch_c()])
// results: List<String>  （各 Task の結果リスト）
```

- 全 Task を実行し、結果を **List** として返す。
- v1.8.0 では左から順に同期実行。

#### `Task.race`

```favnir
// 型: List<Task<T>> -> Task<T>
bind first <- Task.race([fetch_fast(), fetch_slow()])
// first: String  （最初の結果。v1.8.0 では先頭 Task の結果）
```

- v1.8.0 では先頭要素を実行して返す（真の競合は v1.9.0+）。

#### `Task.timeout`

```favnir
// 型: Task<T> -> Int -> Task<Option<T>>
bind result <- Task.timeout(fetch_data(), 5000)
// result: Option<String>  （v1.8.0 では常に Some(value)、タイムアウト未実装）
```

- v1.8.0 ではタイムアウトせず常に `Some(value)` を返す（真のタイムアウトは v1.9.0+）。
- 型面は v1.9.0 と互換になるよう設計する。

### 3-3. 型システムへの変更

```
Task.all   : List<Task<T>> -> Task<List<T>>
Task.race  : List<Task<T>> -> Task<T>
Task.timeout : Task<T> -> Int -> Task<Option<T>>
```

- `checker.rs` の `"Task"` ビルトイン登録に上記シグネチャを追加。
- `vm.rs` の `vm_call_builtin` に `"Task.all"` / `"Task.race"` / `"Task.timeout"` ハンドラを追加。

### 3-4. エラーコード

| コード | 条件 |
|---|---|
| E061 | `Task.all` / `Task.race` に空リストを渡した |
| E062 | `Task.timeout` の第2引数が非 Int |

---

## 4. Phase 2 — `async fn main()`

### 4-1. 設計

v1.7.0 では `main()` は同期 (`() -> Unit !Io`) のみ許可。
v1.8.0 では `async fn main() -> Unit !Io` を受け付ける。

エントリポイント検出ロジック:

```
main の型が Task<Unit> → Task を自動実行して Unit を得る
main の型が Unit      → 従来どおり直接実行
それ以外              → E003 (main の型エラー)
```

### 4-2. 変更箇所

- `driver.rs`: `exec_artifact_main` でリターン値が `VMValue::Task` の場合、`Task.run` 相当の自動解除を行う。
- `checker.rs`: `async fn main()` の型 `() -> Task<Unit>` をエントリポイントとして受理するよう `ensure_valid_main` を拡張。

### 4-3. CLI への影響

```
fav run examples/async_main.fav   // async fn main() も動く
fav build examples/async_main.fav // 同様
```

---

## 5. Phase 3 — `chain` + `Task<T>` 統合

### 5-1. 設計

v1.5.0 以降、`chain` は `Result<T, E>` と `Option<T>` のエラー伝播に使用してきた。
v1.8.0 では `Task<Result<T, E>>` 型（Task と Result の合成）を `chain` でシームレスに扱えるようにする。

```favnir
// Task<String!> を chain で処理
async fn fetch_text(url: String) -> String! !Network {
    Http.get(url)
}

fn process() -> String! !Network {
    chain body <- fetch_text("https://example.com")
    String.trim(body)
}
```

### 5-2. 型検査規則

- `chain x <- expr` の expr 型が `Task<Result<T, E>>` の場合:
  1. `Task` を解除 → `Result<T, E>`
  2. `Result` を chain → 失敗時は関数の `T!` 型でショートサーキット
  3. 成功時: `x: T`

```
Task<T!>  の chain: x: T  (失敗時は chain_escape)
Task<T?>  の chain: x: T  (None 時は chain_escape)
```

- checker.rs の `check_chain_stmt` に Task ラッパー剥がし処理を追加。

### 5-3. エラーコード

| コード | 条件 |
|---|---|
| E063 | `chain` の対象が `Task<T>` でも `Result` でも `Option` でもない |

---

## 6. Phase 4 — Coverage 強化

### 6-1. 関数単位レポート

v1.7.0 ではファイル単位の集計のみ。v1.8.0 では関数ごとに分解して表示する。

```
coverage: examples/math.fav
  lines covered: 18 / 22 (81.8%)
  uncovered:     lines 14, 17, 20, 21

  fn add          3 / 3 (100%)
  fn subtract     3 / 3 (100%)
  fn multiply     2 / 4  (50%)  uncovered: 14, 17
  fn divide       2 / 4  (50%)  uncovered: 20, 21
```

### 6-2. 実装方針

- `IRFnDef` に `fn_name: String` フィールドを確認（既存）。
- `TrackLine` を関数ごとに収集: `HashMap<String, HashSet<u32>>` で `(fn_name → covered_lines)` を管理。
- `format_coverage_report` を `format_coverage_report_by_fn` に拡張。

### 6-3. `--coverage-report <dir>` フラグ（stub）

```
fav test --coverage-report coverage/
```

- `coverage/index.txt` にテキスト形式のレポートを出力（HTML は v1.9.0 以降）。
- ディレクトリが存在しなければ作成する。

### 6-4. CLI 変更

```
main.rs: test コマンドに --coverage-report <dir> フラグを追加
driver.rs: cmd_test に coverage_report_dir: Option<&str> を追加
```

---

## 7. Phase 5 — `fav bench`

### 7-1. 概要

`bench` ブロックで計測単位を定義し、`fav bench` で実行して結果を表示する。

```favnir
// math.bench.fav
bench "add two numbers" {
    add(1, 2)
}

bench "string concat 100 times" {
    List.range(0, 100)
        |> List.fold("", |acc, _| acc ++ "x")
}
```

```
$ fav bench examples/math.bench.fav

running 2 benchmarks in examples/math.bench.fav
  bench  add two numbers          1.2 µs/iter  (100 iterations)
  bench  string concat 100 times  38.4 µs/iter (100 iterations)
```

### 7-2. 構文

- `bench "description" { body }` をトップレベル item として追加。
- `*.bench.fav` ファイルを自動探索（`.fav` 内の `bench` ブロックも可）。
- AST: `Item::BenchDef(BenchDef)` を追加。

```rust
pub struct BenchDef {
    pub description: String,
    pub body: Expr,
    pub span: Span,
}
```

### 7-3. 実行

- デフォルト 100 イテレーション（`--iters N` で変更可能）。
- `std::time::Instant` でウォールクロック計測。
- 最小・最大・平均を計算して表示。

### 7-4. CLI

```
fav bench                          // カレントディレクトリの *.bench.fav を実行
fav bench examples/math.bench.fav  // 指定ファイルを実行
fav bench --iters 1000             // イテレーション数を指定
fav bench --filter "keyword"       // ベンチ名フィルタ
```

### 7-5. `driver.rs` の変更

- `cmd_bench(file: Option<&str>, filter: Option<&str>, iters: u64)` を追加。
- `collect_bench_cases(program)` でベンチブロックを収集。
- 各ベンチを `iters` 回実行してタイミングを計測。
- `format_bench_results(results)` で表示。

### 7-6. エラーコード

| コード | 条件 |
|---|---|
| E064 | `bench` ブロック内で I/O 以外の副作用 effect が使われた |

---

## 8. Phase 6 — テスト・ドキュメント

### 8-1. テスト要件

#### Task 並列 API

| テスト名 | 検証内容 |
|---|---|
| `task_all_collects_results` | `Task.all([t1, t2])` が全結果をリストで返す |
| `task_all_empty_list_error` | `Task.all([])` で E061 |
| `task_race_returns_first` | `Task.race([t1, t2])` が先頭 Task の結果を返す |
| `task_timeout_returns_some` | `Task.timeout(t, 1000)` が `Some(value)` を返す（v1.8.0 同期） |

#### `async fn main()`

| テスト名 | 検証内容 |
|---|---|
| `async_main_executes_correctly` | `async fn main() -> Unit !Io` がエラーなく実行される |
| `async_main_task_auto_resolved` | main の Task<Unit> が自動解除される |

#### `chain` + Task<T>

| テスト名 | 検証内容 |
|---|---|
| `chain_task_result_unwraps_both` | `chain x <- Task<T!>` で Task と Result が両方解除される |
| `chain_task_option_unwraps_both` | `chain x <- Task<T?>` で Task と Option が両方解除される |

#### Coverage 強化

| テスト名 | 検証内容 |
|---|---|
| `coverage_report_by_fn` | 関数名がレポートに含まれる |
| `coverage_report_dir_creates_file` | `--coverage-report` でファイルが生成される |

#### `fav bench`

| テスト名 | 検証内容 |
|---|---|
| `bench_collect_bench_cases` | `bench` ブロックが正しく収集される |
| `bench_runs_and_reports_timing` | ベンチが実行され `µs/iter` 形式で結果が出る |
| `bench_filter_skips_non_matching` | `--filter` でマッチしないベンチがスキップされる |

### 8-2. example ファイル

- `examples/task_parallel_demo.fav` — `Task.all` + `Task.race` の利用例
- `examples/async_main_demo.fav` — `async fn main()` の最小例
- `examples/math.bench.fav` — `bench` ブロックの基本例

### 8-3. ドキュメント更新

- `versions/v1.8.0/langspec.md` を新規作成
- `README.md` に v1.8.0 セクションを追加

---

## 9. 完了条件（Done Definition）

- [x] `Task.all([t1, t2])` が型チェックを通り実行できる
- [x] `Task.race([t1, t2])` が型チェックを通り実行できる
- [x] `Task.timeout(t, 5000)` が型チェックを通り `Option<T>` を返す
- [x] `async fn main() -> Unit !Io` が `fav run` で実行できる
- [x] `chain x <- async_fn()` で `Task<T!>` が一括処理される
- [x] `fav test --coverage` の出力に関数名と行カバレッジが含まれる
- [x] `fav test --coverage-report <dir>` でテキストレポートが書き出される
- [x] `fav bench` で `bench` ブロックが計測・表示される
- [x] v1.7.0 の全テスト（498）が引き続き通る
- [x] `cargo build` で警告ゼロ
- [x] `Cargo.toml` バージョンが `"1.8.0"`

---

## 10. 先送り一覧（v1.8.0 では対応しない）

| 制約 | バージョン |
|---|---|
| tokio / async-std による真の並列実行 | v1.9.0 |
| `Task.timeout` の実際のタイムアウト動作 | v1.9.0 |
| `Task.race` の真の並列競合 | v1.9.0 |
| `fav bench --coverage` 連携 | v1.9.0 以降 |
| Coverage の HTML 出力 | v1.9.0 以降 |
| `bench` 内での `async fn` 計測 | v1.9.0 以降 |
| `fav migrate` (v1.x → v2.0.0 変換) | v2.0.0 |
| `trf` → `stage` / `flw` → `seq` リネーム | v2.0.0 |
| セルフホスト（パーサー Favnir 移植） | v2.0.0 |
