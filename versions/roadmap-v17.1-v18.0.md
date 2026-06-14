# Roadmap v17.1.0 〜 v18.0.0 — Language Power

Date: 2026-06-14

## 目標

v17.0.0「Language Ergonomics」で「書きたくなる言語」への転換を果たした。
次の課題は「**言いたいことを言えない**」という表現力の問題を解消することである。

ジェネリクスに制約が付けられない、パターンが書けない、コレクションが不自由——
これらは「できなくはないが迂回が必要」な問題で、コードを複雑にする。
v18.0.0「Language Power」では、この表現力の壁を取り除く。

- v17.1: 境界付きジェネリクス `fn max<T with Ord>(a: T, b: T) -> T` で汎用関数を書けるようにする
- v17.2: or-pattern / list-pattern でパターンマッチを完全に
- v17.3: コレクション内包表記 `[x * 2 | x <- nums]` で変換を簡潔に
- v17.4: `let` バインディングで非 Result 値の束縛を自然に
- v17.5: REPL をデータ探索ツールとして使えるレベルに引き上げる
- v17.6: `fav bench` でパイプラインのマイクロベンチマークを取れるようにする
- v17.7: `forall` プロパティベーステストで堅牢性を高める
- v17.8: パッケージシステム成熟（`fav add` / `fav publish`）でエコシステムを育てる
- v18.0: Language Power マイルストーン宣言

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| 境界付きジェネリクス構文 | `fn f<T with Interface>(...)` — `where` 節ではなくインラインの `with` キーワード |
| 組み込み Interface | `Ord` / `Eq` / `Serialize` / `Display` / `Hash` / `Clone`。既存の `interface` 機構を流用 |
| or-pattern 構文 | `"a" \| "b" => ...`（縦棒区切り）|
| list-pattern 構文 | `[]` / `[x]` / `[head, ..tail]`（Haskell / Elixir に近い形） |
| 内包表記区切り文字 | `[expr \| x <- src, guard]`（縦棒 + ガード条件） |
| `let` キーワード | 関数ボディ内の非 Result 束縛のみ対応。トップレベルでは使用不可 |
| `forall` の実装 | Rust の `proptest` クレートをバックエンドに使用。100 ケースをデフォルト試行数 |
| パッケージレジストリ | `https://registry.favnir.dev`（既存 rune-registry Lambda を v2 に拡張） |
| `fav.lock` 形式 | TOML（cargo.lock に近い形式） |
| `fav bench` の統計 | 平均・中央値・最小・最大・標準偏差。デフォルト 100 runs |

---

## バージョン計画

### v17.1.0 — 境界付きジェネリクス（Bounded Generics）

**テーマ**: 汎用ライブラリ関数を書けるようにする。
現在のジェネリクス（`fn f<T>(x: T) -> T`）は制約がなく、`T` に対して何もできない。
`T with Ord` 制約を付けることで `a < b` のような演算が可能になる。

**構文:**

```fav
// 単一制約
fn max<T with Ord>(a: T, b: T) -> T {
  if a > b { a } else { b }
}

fn serialize<T with Serialize>(val: T) -> String {
  Json.stringify_raw(val)
}

// 複数制約（with を連ねる）
fn sort_and_serialize<T with Ord with Serialize>(items: List<T>) -> List<String> {
  List.map(List.sort_by(items, |x| x), |x| serialize(x))
}

// パイプラインの stage にも適用
stage Rank<T with Ord>(rows: List<T>) -> List<T> {
  Result.ok(List.sort_by(rows, |x| x))
}
```

**組み込み Interface（自動実装）:**

| Interface | 意味 | 自動実装される型 |
|---|---|---|
| `Ord` | 順序比較（`<` `>` `<=` `>=`） | Int / Float / String |
| `Eq` | 等値比較（`==` `!=`） | 全プリミティブ型 + レコード |
| `Serialize` | JSON シリアライズ | 全レコード型（フィールドが全て Serialize を満たす場合） |
| `Display` | 文字列表現（f-string 補間） | String / Int / Float / Bool |
| `Hash` | ハッシュ値計算 | Int / String |
| `Clone` | 値の複製 | 全値型（デフォルト） |

**カスタム Interface との連携:**

```fav
// v9.12.0 で実装済みの interface 機構と組み合わせ可能
interface Scored {
  fn score(self) -> Float
}

fn top_n<T with Scored with Ord>(items: List<T>, n: Int) -> List<T> {
  let ranked = List.sort_by_desc(items, |x| x.score())
  List.take(ranked, n)
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `GenericParam { name: String, bounds: Vec<String> }` に拡張
  - 現在の `Vec<String>` (型パラメータ名のみ) を `Vec<GenericParam>` に変更

- `fav/src/frontend/parser.rs`:
  - `parse_generic_params`: `<T with Ord with Serialize>` を `GenericParam` として解析
  - `with` キーワード追加

- `fav/src/middle/checker.rs`:
  - `check_bounded_call`: ジェネリクス関数呼び出し時に型引数が各 bound を満たすか検査
  - 組み込み bound の自動実装テーブル追加
  - E0325: `型名 does not implement Interface名` エラー追加

- `self/checker.fav`:
  - `check_bounded_generics` 関数を Favnir 実装に追加

- テスト: `v171000_tests`（5件）:
  - `version_is_17_1_0`
  - `bounded_generic_ord`（`max<T with Ord>` が Int / Float / String で動作）
  - `bounded_generic_serialize`（`serialize<T with Serialize>` が動作）
  - `bounded_generic_violation`（`Ord` を満たさない型で E0325）
  - `bounded_generic_multi`（複数 bound が動作）

**完了条件（PASS=5）:**
1. `fn max<T with Ord>(a: T, b: T) -> T` が Int / Float / String で動作する
2. `fn f<T with Serialize>(v: T)` が全レコード型で動作する
3. `T with Ord with Serialize` の複数制約が動作する
4. bound を満たさない型を渡すと E0325 でエラーになる
5. カスタム `interface` との組み合わせが動作する

---

### v17.2.0 — パターンマッチ拡張

**テーマ**: `match` 式でデータの形を完全に記述できるようにする。
or-pattern / バインディングパターン / guard / list-pattern の 4 種類を追加する。

**or-pattern（縦棒で複数パターンを OR）:**

```fav
match status {
  "active" | "pending" => process(row)
  "deleted" | "archived" | "cancelled" => skip(row)
  _ => Result.err(f"unknown status: {status}")
}

// バリアントにも適用
match event {
  Event.Created(x) | Event.Updated(x) => handle_upsert(x)
  Event.Deleted(id)                   => handle_delete(id)
}
```

**バインディングパターン（レコードの分解）:**

```fav
match event {
  Event.Created({ id: i, name: n }) => handle_created(i, n)
  Event.Updated({ id: i, ..rest }) => handle_updated(i, rest)
  Event.Deleted(id) => handle_deleted(id)
}

// guard 条件（if）
match row {
  { amount: a } if a > 1000.0 => high_value(row)
  { amount: a } if a > 0.0   => normal(row)
  _                           => Result.err("negative amount")
}
```

**list-pattern（リストの先頭・末尾分解）:**

```fav
match rows {
  []              => Result.err("empty list")
  [single]        => process_single(single)
  [head, ..tail]  => process_many(head, tail)
}

// 複数先頭要素
match rows {
  [a, b, ..rest] => compare_and_process(a, b, rest)
  _              => Result.err("need at least 2 rows")
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Pattern::Or(Vec<Pattern>)` 追加
  - `Pattern::RecordDestructure { fields: Vec<(String, Pattern)>, rest: Option<String> }` 追加
  - `Pattern::List { head: Vec<Pattern>, tail: Option<String> }` 追加
  - `MatchArm.guard: Option<Expr>` フィールド追加

- `fav/src/frontend/parser.rs`:
  - `parse_pattern`: Or / RecordDestructure / List / Guard を解析
  - `|` をパターン内の OR 演算子として認識（式の `|` と区別）

- `fav/src/middle/checker.rs`:
  - `check_pattern_exhaustiveness`: Or / List パターンを含む網羅性チェック拡張
  - バインディング変数の型推論

- `fav/src/middle/compiler.rs`:
  - `compile_or_pattern`: 各パターンを順次試行する opcode 列生成
  - `compile_list_pattern`: `GetLength` + 分岐 + `GetIndex` opcode 列生成

- テスト: `v172000_tests`（5件）:
  - `version_is_17_2_0`
  - `or_pattern_string`（`"a" | "b" => ...` が動作）
  - `binding_pattern_record`（`{ id: i, name: n }` 分解が動作）
  - `list_pattern_head_tail`（`[head, ..tail]` が動作）
  - `match_guard`（`if guard` 条件が動作）

**完了条件（PASS=5）:**
1. `"a" | "b" => ...` or-pattern が動作する
2. `{ id: i, name: n }` レコード分解バインディングが動作する
3. `[head, ..tail]` list-pattern が動作する
4. `if guard` 条件が動作する
5. or-pattern / list-pattern を含む match の網羅性チェックが正しく動作する

---

### v17.3.0 — コレクション内包表記

**テーマ**: リスト変換・フィルタリングを `List.map` / `List.filter` のネストなしで書けるようにする。
Python の list comprehension に近い構文を Favnir に導入する。

**構文:**

```fav
// 基本: map
let doubled = [x * 2 | x <- numbers]

// フィルタ付き
let evens = [x | x <- numbers, x % 2 == 0]

// 変換 + フィルタ
let valid_emails = [String.trim(s) | s <- raw_emails, String.contains(s, "@")]

// 複数ソース（直積）
let pairs = [Pair(a, b) | a <- as, b <- bs]

// ネストしたリストの平坦化
let flat = [item | row <- matrix, item <- row]

// マップ内包
let counts = { k: List.length(v) | (k, v) <- Map.entries(grouped) }

// Result 内包（エラー伝播）
// いずれかが err なら全体が err になる
let results: Result<List<Output>, String> = [? transform(row) | row <- rows]
```

**Before / After 比較:**

```fav
// Before（v17.2 まで）
let valid_names =
  List.map(
    List.filter(rows, |r| String.length(String.trim(r.name)) > 0),
    |r| String.trim(r.name)
  )

// After（v17.3 以降）
let valid_names = [String.trim(r.name) | r <- rows, String.length(String.trim(r.name)) > 0]
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Expr::ListComp { expr: Box<Expr>, clauses: Vec<CompClause> }` 追加
  - `Expr::MapComp { key: Box<Expr>, val: Box<Expr>, clauses: Vec<CompClause> }` 追加
  - `Expr::ResultComp { expr: Box<Expr>, clauses: Vec<CompClause> }` 追加（`[? ...]` 形式）
  - `CompClause::For { pat: Pattern, src: Box<Expr> }` / `CompClause::Guard(Box<Expr>)` 追加

- `fav/src/frontend/parser.rs`:
  - `parse_list_comp`: `[expr | ...]` の内包表記パース
  - `|` を内包表記内の区切り文字として認識

- `fav/src/middle/checker.rs`:
  - `check_list_comp`: ソース型からパターン変数の型を推論
  - `check_result_comp`: `[? expr | ...]` の Result 型チェック

- `fav/src/middle/compiler.rs`:
  - `compile_list_comp`: `List.filter` + `List.map` への展開
  - `compile_result_comp`: `List.filter_map` + エラー伝播 opcode へ展開

- テスト: `v173000_tests`（5件）:
  - `version_is_17_3_0`
  - `list_comp_map`（`[x * 2 | x <- ns]` が動作）
  - `list_comp_filter`（guard 条件付きが動作）
  - `list_comp_multi_source`（複数ソースの直積が動作）
  - `result_comp_propagation`（`[? f(x) | x <- xs]` のエラー伝播が動作）

**完了条件（PASS=5）:**
1. `[x * 2 | x <- numbers]` が `List.map` 相当の結果を返す
2. `[x | x <- numbers, x > 0]` が `List.filter` 相当の結果を返す
3. 複数ソース `[Pair(a, b) | a <- as, b <- bs]` が動作する
4. `[? transform(row) | row <- rows]` のエラー伝播が動作する
5. マップ内包 `{ k: v | (k, v) <- ... }` が動作する

---

### v17.4.0 — `let` バインディング（非 Result 文脈）

**テーマ**: 関数ボディ内で非 Result 値に名前を付ける自然な構文を追加する。
現在は `bind x <- Result.ok(expr)` という不自然な迂回が必要だった。

**現状の問題と解決:**

```fav
// 現状: 非 Result 値を束縛するには Result.ok でラップが必要
fn process(row: Row) -> Result<Output, String> {
  bind trimmed_name <- Result.ok(String.trim(row.name))   // ← 無駄なラップ
  bind score <- Result.ok(compute_score(row))              // ← 無駄なラップ
  bind result <- validate({ ...row, name: trimmed_name, score: score })
  Result.ok(result)
}

// 提案: let で自然に束縛（Result でない値はそのまま束縛）
fn process(row: Row) -> Result<Output, String> {
  let trimmed = String.trim(row.name)
  let score   = compute_score(row)
  bind result <- validate({ ...row, name: trimmed, score: score })
  Result.ok(result)
}
```

**`let` の使用ルール:**

```fav
// OK: 非 Result 値の束縛
fn f() -> Result<Int, String> {
  let x = 42                        // 非 Result — OK
  let name = String.trim("  hi  ")  // 非 Result — OK
  bind r <- some_db_call()          // Result — bind を使う
  Result.ok(x + r)
}

// エラー: Result 値に let を使った場合
fn g() -> Result<Int, String> {
  let r = some_db_call()   // E0326: use `bind` for Result values
  Result.ok(r)
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Stmt::Let { name: String, expr: Box<Expr> }` 追加（`Stmt::Bind` とは別）

- `fav/src/frontend/parser.rs`:
  - `let name = expr` の構文追加（`bind` とは独立したキーワード）

- `fav/src/middle/checker.rs`:
  - `check_let_stmt`: `expr` の型が `Result<_, _>` でないことを確認
  - `Result<_, _>` 型に `let` を使うと E0326 エラー

- `fav/src/middle/compiler.rs`:
  - `Stmt::Let` → 単純な値 push + local 変数登録（`bind` の Result チェック opcode なし）

- `self/compiler.fav` / `self/checker.fav`:
  - `let` 文の解析・型チェックを Favnir 実装に追加

- テスト: `v174000_tests`（5件）:
  - `version_is_17_4_0`
  - `let_binding_basic`（`let x = 42` で束縛後に使用）
  - `let_binding_string`（`let name = String.trim(s)` が動作）
  - `let_with_record_spread`（`let updated = { ...row, x: val }` が動作）
  - `let_result_type_error`（Result 値に let → E0326）

**完了条件（PASS=5）:**
1. `let x = non_result_expr` で変数を束縛できる
2. `let` の後に `bind` / `chain` / `Result.ok` で使える
3. `let name = String.trim(s)` のような stdlib 呼び出しが動作する
4. `let updated = { ...row, field: val }` でレコードスプレッドと組み合わせられる
5. `let r = result_fn()` で E0326 が出る（Result には `bind` を使うよう指示）

---

### v17.5.0 — REPL 品質向上

**テーマ**: `fav repl`（v9.10.0 実装）をデータ探索ツールとして使えるレベルに引き上げる。
`:doc` / `:load` / `:paste` / タブ補完を追加し、Jupyter ノートブックの代替として使えるようにする。

**追加コマンド:**

```
favnir> :help                              # コマンド一覧表示
favnir> :doc List.group_by                 # 関数ドキュメント表示
favnir> :type List.map                     # 型シグネチャ表示
favnir> :load src/pipeline.fav             # ファイルをロードして定義を REPL に取り込む
favnir> :save session.fav                  # 現在のセッション（定義・変数）をファイルに保存
favnir> :history                           # コマンド履歴表示（上下矢印キーでも辿れる）
favnir> :reset                             # 環境リセット（変数・定義をクリア）
favnir> :env                               # 現在定義されている変数・関数の一覧

# 複数行入力（:paste モード）
favnir> :paste
... fn double(x: Int) -> Int {
...   x * 2
... }
... :end
defined fn double: Int -> Int

favnir> double(21)
42
```

**タブ補完:**

```
favnir> List.<Tab>
List.map   List.filter  List.group_by  List.chunk  List.sort_by  ...

favnir> row.<Tab>
row.id   row.name   row.email   row.created_at   ...   # レコードフィールド補完

favnir> :d<Tab>
:doc
```

**実装内容:**

- `fav/src/driver.rs`:
  - `cmd_repl` 拡張: `:doc` / `:load` / `:save` / `:history` / `:paste` コマンド実装
  - `rustyline` の `Completer` トレイトを実装してタブ補完を追加
  - `:doc` は `site/content/docs/` の MDX ファイルから概要行を抽出して表示

- `fav/src/repl/completer.rs`（新規）:
  - `FavCompleter`: モジュール名・関数名・`:` コマンドの補完ロジック
  - レコード型推論結果からフィールド補完

- テスト: `v175000_tests`（5件）:
  - `version_is_17_5_0`
  - `repl_doc_command`（`:doc List.map` が出力を返す）
  - `repl_type_command`（`:type List.map` が型シグネチャを返す）
  - `repl_load_file`（`:load` でファイルの定義を REPL に取り込める）
  - `repl_paste_mode`（`:paste` ... `:end` で複数行定義ができる）

**完了条件（PASS=5）:**
1. `:doc FunctionName` でドキュメントが表示される
2. `:type FunctionName` で型シグネチャが表示される
3. `:load file.fav` でファイルの定義が REPL に取り込まれる
4. `:paste` ... `:end` で複数行の関数定義が動作する
5. タブ補完でモジュール名・関数名が補完される

---

### v17.6.0 — `fav bench`（マイクロベンチマーク）

**テーマ**: データエンジニアがパイプラインの性能を計測・比較できるようにする。
`bench "..." { ... }` ブロックで定義し、`fav bench` で実行・統計を出力する。

**構文:**

```fav
bench "transform 10k rows" {
  let rows = generate_test_rows(10_000)
  Transform(rows)
}

bench "json parse large" {
  Json.parse(large_json_fixture)
}

bench "bigquery query" {
  bigquery.query("my-project", "my_dataset", "SELECT COUNT(*) FROM users")
}
```

**出力形式:**

```
fav bench src/pipeline.fav

running 3 benchmarks

  transform 10k rows:  avg  12.3ms  p50  11.9ms  p95  14.8ms  min  11.2ms  max  21.4ms  (100 runs)
  json parse large:    avg   1.2ms  p50   1.1ms  p95   1.5ms  min   1.0ms  max   2.1ms  (100 runs)
  bigquery query:      avg 245.0ms  p50 241.0ms  p95 280.0ms  min 230.0ms  max 310.0ms  ( 10 runs)

bench result: 3 benchmarks completed.
```

**オプション:**

```bash
fav bench src/pipeline.fav                  # デフォルト（100 runs）
fav bench src/pipeline.fav --runs 1000      # runs 数指定
fav bench src/pipeline.fav --warmup 10      # ウォームアップ回数
fav bench src/pipeline.fav --filter "json"  # ベンチ名でフィルタ
fav bench src/pipeline.fav --json           # JSON 形式で出力
```

**実装内容:**

- `fav/src/ast.rs`:
  - `TopLevel::BenchDef { name: String, body: Vec<Stmt> }` 追加

- `fav/src/frontend/parser.rs`:
  - `bench "name" { ... }` 構文追加（`test "..."` と同じパターン）

- `fav/src/driver.rs`:
  - `cmd_bench(path: &str, opts: BenchOpts)` 実装
  - `std::time::Instant` で各 run の時間計測
  - 統計計算（avg / p50 / p95 / min / max / stddev）
  - `--runs` / `--warmup` / `--filter` / `--json` オプション処理

- テスト: `v176000_tests`（5件）:
  - `version_is_17_6_0`
  - `bench_def_parses`（`bench "..." { }` が AST として解析される）
  - `bench_runs_and_reports`（`fav bench` が統計を出力する）
  - `bench_filter_option`（`--filter` でベンチを絞り込める）
  - `bench_json_output`（`--json` フラグで JSON 形式出力）

**完了条件（PASS=5）:**
1. `bench "name" { ... }` が AST として解析される
2. `fav bench` で各ベンチが実行され、avg / p50 / p95 / min / max が出力される
3. `--runs N` でベンチ回数を変更できる
4. `--filter "keyword"` でベンチを絞り込める
5. `--json` フラグで JSON 形式の出力が得られる

---

### v17.7.0 — プロパティベーステスト（`forall`）

**テーマ**: `fav test` に「任意の入力で性質が成立する」ことを確認する機能を追加する。
テストケースを手書きするのでなく、型から自動生成した入力で検証する。

**構文:**

```fav
// forall: 型を指定すると入力を自動生成して検証
test "round-trip: serialize then parse" {
  forall row: UserRow {
    bind serialized <- Json.stringify(row)
    bind parsed     <- Json.parse(serialized)
    assert_eq(parsed, row)
  }
}

test "sort is idempotent" {
  forall rows: List<Int> {
    let sorted = List.sort_by(rows, |x| x)
    assert_eq(sorted, List.sort_by(sorted, |x| x))
  }
}

test "trim is idempotent" {
  forall s: String {
    let trimmed = String.trim(s)
    assert_eq(trimmed, String.trim(trimmed))
  }
}

// 制約付き生成（where で入力の前提条件を指定）
test "divide is safe" {
  forall a: Int, b: Int where { b != 0 } {
    let result = a / b
    assert_true(result * b == a || (a % b) != 0)  // 整数除算の性質
  }
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Stmt::Forall { vars: Vec<(String, Type)>, guard: Option<Expr>, body: Vec<Stmt> }` 追加

- `fav/src/frontend/parser.rs`:
  - `forall var: Type, ... where { guard } { body }` の構文追加

- `fav/src/backend/vm.rs`:
  - `ForallGen` opcode: 型から値を自動生成する VM ランタイム
  - `Int`: ランダム符号付き整数（±10^6 の範囲 + エッジケース 0/1/-1/MAX/MIN）
  - `Float`: ランダム浮動小数点（NaN / Inf 除外）
  - `String`: ランダム ASCII 文字列（空文字・空白のみ・長文字を含む）
  - `List<T>`: 空・1 要素・ランダム長リストの組み合わせ
  - `Bool`: `true` / `false` 交互

- `fav/src/driver.rs`:
  - `cmd_test` で `Forall` を処理: デフォルト 100 ケース試行
  - 失敗時に「反例」を縮小（shrinking）して報告
  - `--cases N` オプションで試行数変更

- テスト: `v177000_tests`（5件）:
  - `version_is_17_7_0`
  - `forall_int_parses`（`forall n: Int { ... }` が解析される）
  - `forall_string_idempotent`（`trim` の冪等性が 100 ケースでパス）
  - `forall_finds_counterexample`（意図的に失敗するプロパティで反例を発見）
  - `forall_with_guard`（`where { b != 0 }` 制約付き生成が動作）

**完了条件（PASS=5）:**
1. `forall n: Int { ... }` が 100 ランダムケースで実行される
2. `forall s: String { ... }` で空文字・長文字を含むケースが生成される
3. `forall rows: List<Int> { ... }` で空リスト・長リストが生成される
4. 失敗ケースで反例（shrinking された最小反例）が報告される
5. `where { guard }` で入力の前提条件を絞り込める

---

### v17.8.0 — パッケージシステム成熟（rune registry v2）

**テーマ**: Rune を「パッケージ」として `fav.toml` で依存管理できるようにする。
`fav add` / `fav update` / `fav publish` で Cargo ライクなエコシステムを育てる。

**fav.toml 依存管理:**

```toml
[dependencies]
csv         = "^2.0.0"
bigquery    = "^1.0.0"
my-company/etl-utils = "^0.5.0"    # スコープ付き（社内パッケージ）

[dev-dependencies]
test-fixtures = "^1.0.0"

[registry]
url = "https://registry.favnir.dev"   # デフォルト（省略可）
# プライベートレジストリも可
# url = "https://registry.mycompany.com"
```

**CLI コマンド:**

```bash
fav add csv                  # 最新版を追加（fav.toml と fav.lock を更新）
fav add csv@2.1.0            # バージョン指定
fav add --dev test-fixtures  # dev-dependency として追加
fav update                   # 全パッケージを semver 範囲内で更新
fav update csv               # 特定パッケージのみ更新
fav remove csv               # 依存から削除
fav publish                  # 現在のパッケージを registry に公開
fav publish --dry-run        # 公開内容の確認（実際には公開しない）
fav new --template bigquery-pipeline  # テンプレートから新規作成
```

**fav.lock 形式:**

```toml
# このファイルは fav が自動生成します。手動編集しないでください。
[[package]]
name = "csv"
version = "2.1.0"
checksum = "sha256:abc123..."
source = "registry:https://registry.favnir.dev"

[[package]]
name = "bigquery"
version = "1.0.3"
checksum = "sha256:def456..."
source = "registry:https://registry.favnir.dev"
```

**実装内容:**

- `fav/src/driver.rs`:
  - `cmd_add(name: &str, version: Option<&str>, dev: bool)` 実装
  - `cmd_update(name: Option<&str>)` 実装
  - `cmd_remove(name: &str)` 実装
  - `cmd_publish(dry_run: bool)` 実装
  - semver 解析（`^` / `~` / `=` / `*` のサポート）

- `fav/src/registry/`（新規ディレクトリ）:
  - `client.rs`: registry API クライアント（`GET /packages/{name}`, `POST /packages`）
  - `resolver.rs`: 依存解決（SAT solver ライクな制約解消）
  - `lockfile.rs`: `fav.lock` の読み書き

- `fav/src/toml.rs`:
  - `[dependencies]` / `[dev-dependencies]` / `[registry]` セクション追加

- `infra/registry-v2/`:
  - Lambda 関数の v2 拡張（パッケージ検索・バージョン管理 API 追加）

- テスト: `v178000_tests`（5件）:
  - `version_is_17_8_0`
  - `fav_toml_dependencies_parse`（`[dependencies]` セクションが解析される）
  - `fav_lock_generates`（`fav.lock` が生成される）
  - `semver_caret_resolve`（`^2.0.0` が `2.x.x` を解決する）
  - `cmd_add_updates_toml`（`fav add csv` が `fav.toml` を更新する）

**完了条件（PASS=5）:**
1. `fav.toml` の `[dependencies]` セクションが解析される
2. `fav.lock` が自動生成される
3. `^` / `~` / `=` の semver 範囲が正しく解決される
4. `fav add csv` で `fav.toml` と `fav.lock` が更新される
5. `fav publish` で registry API にパッケージが公開される（dry-run でテスト）

---

### v18.0.0 — Language Power マイルストーン宣言

**テーマ**: v17.x シリーズの集大成。「表現できる言語」への転換を宣言する。

**実装内容:**

- `Cargo.toml`: バージョンを `18.0.0` に更新

- `CHANGELOG.md`: v17.1.0〜v17.8.0 の全エントリ追加

- `README.md`:
  - 「現在の状態」を v18.0.0 に更新
  - Language Power 達成を記載（bounded generics / pattern matching / list comprehension / package system）
  - バージョン履歴表に v17.1.0〜v18.0.0 エントリ追加

- `site/content/docs/`:
  - `language/generics.mdx` 新規作成（境界付きジェネリクスガイド）
  - `language/patterns.mdx` 新規作成（パターンマッチ全集）
  - `language/comprehensions.mdx` 新規作成
  - `language/let-binding.mdx` 新規作成
  - `language/property-testing.mdx` 新規作成
  - `packages/getting-started.mdx` 新規作成（`fav add` ガイド）
  - `packages/publishing.mdx` 新規作成（`fav publish` ガイド）

- テスト: `v180000_tests`（5件）:
  - `version_is_18_0_0`
  - `changelog_has_v17_entries`（CHANGELOG に v17.x エントリが含まれる）
  - `readme_mentions_bounded_generics`（README に bounded generics が記載されている）
  - `readme_mentions_package_system`（README に package system が記載されている）
  - `docs_generics_exists`（`site/content/docs/language/generics.mdx` が存在する）

**完了条件:**

| 確認項目 | 状態 |
|---|---|
| `fn f<T with Ord>(...)` が動作する | [ ] |
| or-pattern / list-pattern が動作する | [ ] |
| `[x * 2 \| x <- list]` 内包表記が動作する | [ ] |
| `let x = value` が関数ボディで使える | [ ] |
| REPL タブ補完・`:doc` が動作する | [ ] |
| `fav bench` が動作する | [ ] |
| `forall` プロパティテストが動作する | [ ] |
| `fav add` / `fav publish` が動作する | [ ] |
| `cargo test v180000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 依存関係

```
v17.0.0（Language Ergonomics）✅
    |
    v17.1.0（境界付きジェネリクス）← 後続全バージョンで活用
    |
    v17.2.0（パターンマッチ拡張）   v17.3.0（内包表記）    ← 並列実施可能
    |                               |
    v17.4.0（let バインディング）← 内包表記の guard と組み合わせ効果大
    |
    v17.5.0（REPL 品質向上）     v17.6.0（fav bench）      ← 並列実施可能
    |                               |
    v17.7.0（forall テスト）← let + パターンマッチに依存
    |
    v17.8.0（パッケージシステム）← 独立して先行可能
    |
    v18.0.0（マイルストーン）
```

v17.1.0（bounded generics）は最優先（後続全バージョンの基盤）。
v17.2.0 と v17.3.0 は独立して並列実施可能。
v17.8.0（パッケージシステム）は他と独立しているため先行実施も可能。

---

## 新規 Cargo 依存（予定）

| Crate | 用途 | 追加バージョン |
|---|---|---|
| `proptest 1.x` | `forall` プロパティテストの入力生成 | v17.7.0 |
| `rustyline 14.x` | REPL タブ補完・履歴（既存依存の拡張） | v17.5.0 |
| `semver 1.x` | パッケージバージョン解決 | v17.8.0 |
| その他 | なし（既存依存内で対応） | — |

---

## 実装ノート

- **境界付きジェネリクスの単相化戦略**: v17.1 では型消去（type erasure）方式で実装（コンパイル時に型引数を除去し、実行時に Interface メソッドをディスパッチ）。v19.x の型成熟後に単相化（monomorphization）へ移行する選択肢を残す。
- **or-pattern と `|` の曖昧性**: パーサーで `match` アームの先頭文脈と式文脈を区別することで解決。`match { Pattern | Pattern => ... }` の `|` はパターン OR、式中の `|` はビット OR（現状）または内包表記区切り。
- **list-pattern の `..tail` 構文**: `..tail` は残余バインディング（rest binding）。`[a, b, ..rest]` で先頭 2 要素 + 残りリストに分解できる。
- **内包表記のデシュガー**: コンパイラが `List.filter` + `List.map` に展開する。VM に新 opcode は不要。
- **`let` と `bind` の共存**: 関数ボディ内で `let`（非 Result）と `bind`（Result unwrap）を混在できる。`let x = Result.ok(5)` は E0326（Result 値には `bind` を使えというエラー）。
- **`fav bench` のウォームアップ**: JIT や OS キャッシュの影響を除くため、デフォルト 5 ウォームアップ実行後に計測開始。
- **`forall` の shrinking**: 失敗した入力を縮小（整数なら 0 に近づける、リストなら要素を減らす）して最小反例を報告する。`proptest` の shrinking アルゴリズムを活用。
- **パッケージシステムの依存解決**: v17.8 では greedy resolution（最新版を優先）で実装。完全な SAT ベースの解決は v19.x 以降。
- **`fav publish` の認証**: CLI でのトークン認証（`fav login` コマンド追加）。トークンは `~/.fav/credentials` に保存。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/roadmap-master.md` | v17.0〜v20.0 の全体戦略 |
| `versions/roadmap-v16.1-v17.0.md` | 直前ロードマップ（形式参照） |
| `fav/src/middle/checker.rs` | 型チェッカー（bounded generics 追加対象） |
| `fav/src/frontend/parser.rs` | パーサー（パターン拡張・内包表記追加対象） |
| `fav/src/backend/vm.rs` | VM（forall ジェネレータ追加対象） |
| `fav/src/driver.rs` | CLI コマンド（bench / repl 拡張対象） |
| `site/content/docs/language/` | ドキュメント追加対象 |
| `infra/registry-v2/` | パッケージレジストリ v2 対象 |
