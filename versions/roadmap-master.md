# Favnir Master Roadmap — v17.0 〜 v20.0

Date: 2026-06-14
Status: 計画中（v16.0.0 完了時点）

---

## 背景と方針

v16.0.0「Production Multi-Cloud」の宣言をもって、Favnir はクラウド統合の第一段階を完了した。
AWS / Azure / GCP / Snowflake / Kafka にアクセスでき、型安全なパイプラインを書ける。

しかし「できること」と「書きたくなること」は別問題である。
データエンジニアが毎日使う言語として選ばれるためには、**言語自体の体験品質**を上げなければならない。

v17〜v20 では「クラウドに繋がる」ことより「**言語として成熟する**」ことを優先する。
各マイルストーンは独立したテーマを持ち、前のマイルストーンの上に積み上がる。

```
v17.0 — Language Ergonomics   : 「書きたくなる」
v18.0 — Language Power        : 「表現できる」
v19.0 — Type System Maturity  : 「信頼できる」
v20.0 — Production Performance: 「本番で速い」
```

この順序には理由がある。
まず**日常の書き心地**を改善しなければ、どれだけ強力な型システムがあっても使われない。
書き心地が良くなってから**表現力**を広げ、表現力が広がってから**型安全性の深化**が意味を持つ。
最後に**パフォーマンス**を最適化する（早すぎる最適化を避ける）。

---

## v17.0 — Language Ergonomics

**テーマ**: 「書きたくなる言語」
**期間**: v16.1〜v16.8 → v17.0 マイルストーン宣言

### なぜ最初か

現状の最大の摩擦点は「書くのが面倒」なことである。
`String.concat` の連鎖、全フィールドの書き直し、不親切なエラーメッセージ——
これらはデータエンジニアが毎日感じる小さなストレスの積み重ねであり、
他の言語（Python / TypeScript）に戻る最大の理由になる。

「言語を選ぶ理由」の 1 位はほぼ常に**書き心地**である。

### 対象ユーザーへの価値

- 同じロジックをより少ないコードで書ける
- エラーを見て「すぐ直せる」と感じる
- 初めて書くコードでも「なんとなく動く」感覚

### 成功基準

- 既存のパイプラインコードが平均 30% 短くなる
- エラー発生時に修正候補が提示される
- `String.concat` の使用頻度が激減する

---

### v16.1 — エラーメッセージ品質向上

**最重要改善項目**。言語の第一印象はエラーメッセージで決まる。

#### 現状の問題

```
[E0001] undefined variable: user_id
```

情報が少なすぎる。「何が」はわかるが「なぜ」「どう直すか」がない。

#### 目標形式

```
[E0001] undefined variable: user_id
 --> src/pipeline.fav:12:5
  |
12 |   transform(user_id, name)
  |             ^^^^^^^ この変数は未定義です
  |
  = ヒント: `userId` (line 8) の typo ではないですか？
  = 参照: https://favnir.dev/errors/E0001
```

#### 実装内容

- `Span` に行番号・列番号・ソース行テキストを付与（AST 全体に伝播）
- エラー表示エンジン（`format_diagnostic` を `rustc` スタイルに刷新）
- **Levenshtein 距離**による「typo 候補」提示（変数名・関数名・型名）
- 全エラーコード（E0001〜E0320）に `hint` / `note` / `help` フィールド追加
- エラーコード別ドキュメント URL（`https://favnir.dev/errors/Exxxx`）
- **複数エラーの同時報告**（現状は最初のエラーで止まる場合がある）

#### 影響範囲

`checker.rs` / `driver.rs` / `fmt.rs`（エラー表示部分の全面改修）
`self/checker.fav`（セルフホスト checker のエラー情報付与）

---

### v16.2 — 文字列補間（String Interpolation）

データエンジニアが最もよく書くコードのひとつが「動的な文字列の組み立て」である。

#### 構文

```fav
// f"..." プレフィックス
let name  = "Alice"
let count = 42
let msg   = f"Hello, {name}! You have {count} items."

// 式も埋め込める
let sql = f"SELECT * FROM {table} WHERE created_at > '{format_date(since)}'"

// 複数行
let report = f"""
  User: {user.name}
  Email: {user.email}
  Items: {List.length(items)}
"""
```

#### 実装内容

- レキサー: `f"..."` トークン → `FString(Vec<FStringPart>)` に分解
  - `FStringPart::Literal(String)` — 静的テキスト
  - `FStringPart::Expr(Expr)` — `{...}` 内の式
- パーサー: `FString` → `Expr::FString(Vec<FStringPart>)` AST ノード
- 型チェッカー: 各 `Expr` 部分に `to_string` 変換を暗黙適用
- コンパイラ: `FString` → `String.concat` の連鎖にデシュガー
- `Display` 実装がある型（String / Int / Float / Bool）は自動変換
- その他の型はコンパイルエラー（`!Display` 制約）

#### 破壊的変更

なし。既存コードに影響しない。

---

### v16.3 — レコード更新構文（Record Spread / Update）

データ変換パイプラインの核心操作「1 フィールドだけ変えたいのに全部書く」問題を解決する。

#### 構文

```fav
// スプレッド構文
let updated = { ...row, status: "processed" }

// 複数フィールド更新
let enriched = { ...row, status: "ok", processed_at: now() }

// ネストしたレコードへの更新
let fixed = { ...user, address: { ...user.address, zip: "100-0001" } }

// パイプラインでの自然な使用
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({ ...row, score: compute_score(row), enriched_at: now() })
}
```

#### 実装内容

- パーサー: `{ ...expr, key: val, ... }` 構文をレコードリテラルの拡張として追加
- AST: `Expr::RecordUpdate { base: Box<Expr>, fields: Vec<(String, Expr)> }`
- 型チェッカー: `base` の型を解決し、更新フィールドの型整合性を検査
  - 存在しないフィールドへの更新 → E0xxx（新規エラーコード）
  - 型不一致 → E0009 拡張
- コンパイラ: `base` の各フィールドを個別に取り出し + 上書きフィールドで再構築
- `self/compiler.fav` / `self/checker.fav` にも対応追加

---

### v16.4 — 標準ライブラリ拡充（List / String / DateTime / Math）

現在の stdlib はデータ処理には薄い。最頻使用パターンを網羅する。

#### List 拡充

```fav
List.group_by(rows, |r| r.category)       // Map<String, List<Row>>
List.sort_by(rows, |r| r.amount)           // 昇順ソート
List.sort_by_desc(rows, |r| r.created_at) // 降順ソート
List.flatten(nested)                       // List<List<T>> -> List<T>
List.flat_map(rows, |r| expand(r))         // map + flatten
List.zip(as, bs)                           // List<Pair<A, B>>
List.zip_with(as, bs, |a, b| merge(a, b)) // List<C>
List.distinct(rows)                        // 重複除去
List.distinct_by(rows, |r| r.id)          // キー指定重複除去
List.chunk(rows, 100)                      // List<List<T>>（バッチ分割）
List.take(rows, 10)                        // 先頭 N 件
List.drop(rows, 10)                        // 先頭 N 件を除く
List.count_where(rows, |r| r.active)       // 条件付き件数
List.sum_by(rows, |r| r.amount)            // Float 合計
List.max_by(rows, |r| r.score)             // 最大値要素
List.min_by(rows, |r| r.score)             // 最小値要素
List.unzip(pairs)                          // Pair<List<A>, List<B>>
```

#### String 拡充

```fav
String.split(s, ",")                      // List<String>
String.split_once(s, "=")                 // Option<Pair<String, String>>
String.trim(s)                            // 両端空白除去
String.trim_start(s) / String.trim_end(s)
String.replace(s, "old", "new")           // 全置換
String.replace_first(s, "old", "new")     // 先頭のみ置換
String.starts_with(s, prefix)             // Bool
String.ends_with(s, suffix)               // Bool
String.pad_left(s, 10, "0")              // ゼロ埋め
String.pad_right(s, 20, " ")
String.repeat(s, n)                       // "abc" * 3 = "abcabcabc"
String.to_upper(s) / String.to_lower(s)
String.char_at(s, i)                      // Option<String>
String.lines(s)                           // List<String>（改行で分割）
String.is_empty(s)                        // Bool
String.format_int(n, 3, "0")             // "007"
String.format_float(f, 2)                // "3.14"
```

#### DateTime（新モジュール）

```fav
// 現在時刻
let now = DateTime.now()           // DateTime
let ts  = DateTime.now_unix()      // Int（Unix timestamp）

// 文字列との変換
DateTime.parse("2026-06-14T12:00:00Z")  // Result<DateTime, String>
DateTime.format(dt, "YYYY-MM-DD")       // String
DateTime.format_iso(dt)                 // "2026-06-14T12:00:00Z"

// 演算
DateTime.add_days(dt, 7)
DateTime.add_hours(dt, 24)
DateTime.diff_days(from, to)           // Int
DateTime.diff_seconds(from, to)        // Int

// 比較
DateTime.before(a, b) / DateTime.after(a, b)
DateTime.between(dt, from, to)         // Bool
```

#### Math（拡充）

```fav
Math.abs(n)
Math.round(f) / Math.ceil(f) / Math.floor(f)
Math.round_to(f, 2)    // 小数点以下 2 桁
Math.min(a, b) / Math.max(a, b)
Math.clamp(v, lo, hi)
Math.sqrt(f) / Math.pow(base, exp)
Math.log(f) / Math.log2(f) / Math.log10(f)
```

---

### v16.5 — 型エイリアス（Type Alias）

現在の `type Name(Inner)` は**名目型ラッパー**（newtype）。
それとは別に、**構造的同一性を保つ別名**が必要。

#### 構文

```fav
// 型エイリアス（構造的に同一）
alias Timestamp  = String
alias JsonStr    = String
alias RowId      = Int
alias Email      = String

// ジェネリック型エイリアス
alias Result2<T> = Result<T, String>   // よく使うエラー型の省略
alias Rows<T>    = List<T>

// 使用例
fn parse_timestamp(s: Timestamp) -> Result2<DateTime> {
  DateTime.parse(s)
}

// 関数シグネチャが読みやすくなる
fn process(id: RowId, email: Email) -> Result2<JsonStr> { ... }
```

#### 名目型との使い分け

| | `alias` | `type Name(Inner)` |
|---|---|---|
| 互換性 | 元の型と交換可能 | 区別される（型安全） |
| ユースケース | ドキュメント目的 | ビジネスルール強制 |
| where 制約 | 不可 | 可能 |

---

### v16.6 — モジュールシステム強化

現在の `use module.fn` は 1 関数ずつのインポートのみ。

#### 新構文

```fav
// 選択インポート
use utils.{ format_date, parse_csv, validate_email }

// ワイルドカードインポート
use helpers.*

// エイリアスインポート
use very.long.module.name as m
use json as J

// re-export（ライブラリ開発向け）
pub use internal.helper_fn
```

#### モジュールの可視性

```fav
// public: 外部から use できる
public fn exported() -> String { ... }

// 省略（デフォルト）: モジュール内のみ
fn internal_helper() -> String { ... }
```

---

### v16.7 — `assert_eq` + fav test 成熟

v15.3.0 で `fav test` DSL の基礎を作った。実用に耐えるレベルに引き上げる。

#### 新プリミティブ

```fav
// 値の等値比較（現在は assert_true(a == b) で代用）
assert_eq(actual, expected)

// 浮動小数点の近似比較
assert_approx_eq(actual, expected, epsilon)

// リストの要素比較
assert_contains(list, element)
assert_length(list, n)

// 文字列の部分一致
assert_str_contains(s, substring)
assert_str_starts_with(s, prefix)

// エラー内容の検証
assert_err_eq(result, "expected error message")

// スナップショットテスト
assert_snapshot(value, "snapshot_name")
```

#### テストの構造化

```fav
// テストグループ
test_group "transform pipeline" {
  test "trims whitespace" {
    let row = { name: "  Alice  " }
    assert_eq(transform(row).name, "Alice")
  }

  test "handles empty string" {
    let row = { name: "" }
    assert_err(transform(row))
  }
}
```

---

### v16.8 — パイプライン `tap` オペレータ

デバッグ・ロギング・モニタリングのための中間観察演算子。

#### 構文

```fav
// tap: 値を観察して素通し（副作用のみ）
seq Pipeline =
  Load
  |> tap(|rows| io.println(f"Loaded: {List.length(rows)} rows"))
  |> Transform
  |> tap(|result| metrics.record("transformed", result.count))
  |> Save

// inspect: デバッグ出力（--debug フラグ時のみ実行）
seq Pipeline =
  Load |> inspect |> Transform |> inspect |> Save
```

#### 実装内容

- `tap(fn: T -> Unit) -> (T -> T)` として標準ライブラリに追加
- `inspect` は `tap(|v| Debug.show_raw(v))` の糖衣構文
- `--no-tap` フラグで本番実行時に tap をスキップ可能

---

### v17.0 — Language Ergonomics マイルストーン宣言

**完了条件:**
1. 全エラーコードに hint / note を付与
2. 既存のサンプルコードで `String.concat` の連鎖が消えている
3. `{ ...row, field: value }` 構文が動作する
4. `List.group_by` / `DateTime.now` 等の新 stdlib が使える
5. `alias` が動作する
6. `use module.{ fn1, fn2 }` が動作する
7. `assert_eq` が動作する
8. `|> tap(fn)` が動作する

---

---

## v18.0 — Language Power

**テーマ**: 「表現できる言語」
**期間**: v17.1〜v17.8 → v18.0 マイルストーン宣言

### なぜ Ergonomics の次か

書き心地が良くなった後、次の壁は「**言いたいことを言えない**」という表現力の問題である。
ジェネリクスに制約が付けられない、パターンが書けない、コレクションが不自由——
これらは「できなくはないが迂回が必要」な問題で、コードを汚くする。

### 対象ユーザーへの価値

- 汎用的なライブラリ関数を書ける
- 複雑なデータ構造を自然に扱える
- パイプライン中間での型安全な変換を自由に記述できる

---

### v17.1 — 境界付きジェネリクス（Bounded Generics）

```fav
// T with Interface 構文
fn max<T with Ord>(a: T, b: T) -> T {
  if a > b { a } else { b }
}

fn serialize<T with Serialize>(val: T) -> String {
  Json.stringify_raw(val)
}

fn sort<T with Ord>(list: List<T>) -> List<T> {
  List.sort_by(list, |x| x)
}

// 複数制約
fn sort_and_serialize<T with Ord with Serialize>(items: List<T>) -> List<String> {
  List.map(List.sort_by(items, |x| x), serialize)
}
```

#### 組み込み Interface

| Interface | 意味 | 自動実装 |
|---|---|---|
| `Ord` | 順序比較（`<` `>` `<=` `>=`） | Int / Float / String |
| `Eq` | 等値比較（`==` `!=`） | 全プリミティブ型 |
| `Serialize` | JSON シリアライズ | `#[derive(Serialize)]` |
| `Display` | 文字列表現（f-string 補間） | String / Int / Float / Bool |
| `Hash` | ハッシュ値計算 | Int / String |
| `Clone` | 値の複製 | 全値型（デフォルト） |

---

### v17.2 — パターンマッチ拡張

#### or-pattern

```fav
match status {
  "active" | "pending" => process(row)
  "deleted" | "archived" => skip(row)
  _ => Result.err(f"unknown status: {status}")
}
```

#### バインディングパターン

```fav
match event {
  Event.Created({ id: i, name: n }) => handle_created(i, n)
  Event.Updated({ id: i, ..rest }) => handle_updated(i, rest)
  Event.Deleted(id) if id > 0      => handle_deleted(id)
}
```

#### リストパターン

```fav
match rows {
  [] => Result.err("empty")
  [first] => process_single(first)
  [head, ..tail] => process_many(head, tail)
}
```

---

### v17.3 — コレクション内包表記

```fav
// リスト内包（Python の list comprehension に相当）
bind doubled <- [x * 2 | x <- numbers]
bind evens   <- [x     | x <- numbers, x % 2 == 0]
bind pairs   <- [Pair(a, b) | a <- as, b <- bs]

// マップ内包
bind counts <- { k: List.length(v) | (k, v) <- Map.entries(grouped) }

// Result 内包（エラー伝播付き）
bind results <- [? transform(row) | row <- rows]
// いずれか失敗したら全体が Result.err に
```

---

### v17.4 — `let` バインディング除去（誤実装の修正）

v17.4.0 で誤って追加した `let` キーワードを除去する。
`bind x <- expr` はもともと Result / 非 Result どちらの値でも使えるため、`let` は不要だった。

```fav
// 修正後（bind で統一）
fn process(row: Row) -> Result<Output, String> {
  bind trimmed <- String.trim(row.name)
  bind score   <- compute_score(row)
  bind result  <- validate({ ...row, name: trimmed, score: score })
  Result.ok(result)
}
```

---

### v17.5 — REPL 品質向上

`fav repl`（v9.10.0 実装）をデータ探索ツールとして使えるレベルに引き上げる。

```
favnir> :help                              # コマンド一覧
favnir> :doc List.group_by                 # ドキュメント参照
favnir> :type List.map                     # 型シグネチャ表示
favnir> :load src/pipeline.fav             # ファイルをロード
favnir> :save session.fav                  # セッションを保存
favnir> :history                           # コマンド履歴表示
favnir> :reset                             # 環境リセット

# 複数行入力（:paste モード）
favnir> :paste
... fn double(x: Int) -> Int {
...   x * 2
... }
... (:end で終了)
favnir> double(21)
42
```

#### タブ補完

- 変数名・関数名・モジュール名の補完
- `:` コマンドの補完
- レコードのフィールド名補完（`row.<Tab>` で候補表示）

---

### v17.6 — `fav bench`（マイクロベンチマーク）

データエンジニアはパイプラインの性能を計測したい。

```fav
bench "transform 10k rows" {
  bind rows <- generate_test_rows(10_000)
  Transform(rows)
}

bench "json parse" {
  Json.parse(large_json_fixture)
}
```

```bash
fav bench src/pipeline.fav
# 出力:
# transform 10k rows:  avg 12.3ms  min 11.8ms  max 15.2ms  (100 runs)
# json parse:          avg  1.2ms  min  1.1ms  max  1.8ms  (100 runs)
```

---

### v17.7 — プロパティベーステスト（forall）

```fav
// forall: ランダム入力での性質確認
test "round-trip: serialize then parse" {
  forall row: Row {
    bind serialized <- serialize(row)
    bind parsed     <- deserialize(serialized)
    assert_eq(parsed, row)
  }
}

test "sort is idempotent" {
  forall rows: List<Int> {
    bind sorted <- List.sort_by(rows, |x| x)
    assert_eq(sorted, List.sort_by(sorted, |x| x))
  }
}
```

---

### v17.8 — パッケージシステム成熟（rune registry v2）

```toml
# fav.toml
[dependencies]
csv         = "^2.0.0"
bigquery    = "^1.0.0"
my-company/etl-utils = "^0.5.0"    # 社内パッケージ
```

```bash
fav add csv              # 最新版を追加
fav add csv@2.1.0        # バージョン指定
fav update               # 全パッケージ更新
fav publish              # rune registry に公開
```

#### 機能

- セマンティックバージョニング（`^` / `~` / `=` / `*`）
- ロックファイル（`fav.lock`）
- プライベートレジストリ対応（`[registry] url = "..."` in fav.toml）
- `fav new --template bigquery-pipeline` でテンプレートから作成

---

### v18.0 — Language Power マイルストーン宣言

**完了条件:**
1. `fn f<T with Ord>(...)` が動作する
2. or-pattern / list-pattern が動作する
3. `[x * 2 | x <- list]` 内包表記が動作する
4. `bind x <- non_result_expr` で非 Result 値を束縛できる（既存動作の確認）
5. REPL のタブ補完・`:doc` が動作する
6. `fav bench` が動作する
7. `forall` プロパティテストが動作する
8. `fav add` / `fav publish` が動作する

---

---

## v19.0 — Type System Maturity

**テーマ**: 「信頼できる言語」
**期間**: v18.1〜v18.8 → v19.0 マイルストーン宣言

### なぜ Language Power の次か

表現力が広がった後、次の課題は「**型システムが現実のデータに追いつく**」ことである。
現実のデータエンジニアリングでは：
- スキーマが実行時に変わる（BigQuery のカラム追加など）
- データソースによって型が違う（CSV は全て String、DB は型あり）
- エフェクトが自動的に推論されてほしい

これらを型レベルで解決することで「型チェッカーがデータパイプラインの設計図になる」。

---

### v18.1 — エフェクト推論（Effect Inference）

現在は全てのエフェクトを明示しなければならない。
自動推論することで記述量を減らし、追加忘れのエラーも消える。

```fav
// 現在: エフェクトを手動で宣言
fn load_users() -> Result<List<User>, String> !Db !IO {
  bind rows <- Postgres.query_raw(...)
  bind _    <- IO.println(...)
  Result.ok(rows)
}

// 推論後: エフェクト宣言が不要（型チェッカーが推論）
fn load_users() -> Result<List<User>, String> {
  bind rows <- Postgres.query_raw(...)  // !Db を自動推論
  bind _    <- IO.println(...)          // !IO を自動推論
  Result.ok(rows)
}

// stage でも同様
stage LoadUsers -> List<User> {  // !Db !IO が推論される
  ...
}
```

#### 明示宣言のユースケース（保持）

```fav
// インターフェースの実装では明示が必要（契約として）
interface Loader {
  fn load() -> Result<List<Row>, String> !Db
}

// 副作用なしを保証したい場合も明示
fn pure_transform(row: Row) -> Row /* エフェクトなし = 純粋 */ {
  { ...row, score: compute(row) }
}
```

---

### v18.2 — 行多相（Row Polymorphism）

「このフィールドを持つ任意のレコード型」を表現できる。
データパイプラインでの部分的な変換を型安全に書ける。

```fav
// { id: Int, ...rest } を持つ任意のレコードを受け取れる
fn add_timestamp<R with { id: Int }>(row: R) -> { ...R, timestamp: String } {
  { ...row, timestamp: DateTime.format_iso(DateTime.now()) }
}

// 使用例
let user_with_ts   = add_timestamp(User { id: 1, name: "Alice" })
let order_with_ts  = add_timestamp(Order { id: 42, amount: 100.0 })
// 両方型安全
```

---

### v18.3 — 改良版 `where` 制約（Refinement Types）

現在の `where` は型定義時のバリデーションのみ。
関数の引数レベルでも使えるようにする。

```fav
// 型定義での where（現在）
type Amount(Float) where { self > 0.0 }

// 関数引数での where（新規）
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
  a / b
}

fn process(rows: List<Row> where { List.length(rows) > 0 }) -> Result<Summary, String> {
  ...
}

// コンパイル時チェック（リテラル値の場合）
divide(10, 0)   // E0xxx: 制約違反（b != 0 が成立しない）
divide(10, 2)   // OK

// 実行時チェック（変数の場合）
divide(a, b)    // b の値が不明なため実行時にアサーション挿入
```

---

### v18.4 — スキーマ型（Schema Types）

BigQuery / Snowflake / Postgres のスキーマを型として直接扱う。

```fav
// BigQuery スキーマから型を生成（コンパイル時）
type UsersRow = schema "bigquery:my-project.my_dataset.users"
// → { id: Int, name: String, email: String, created_at: String }

// Postgres テーブルから型を生成
type OrderRow = schema "postgres:orders"
// → { id: Int, user_id: Int, amount: Float, status: String }

// JSON Schema から型を生成
type EventPayload = schema "file:schemas/event.json"

// fav.toml での事前定義
[[schema]]
name = "UsersRow"
source = "bigquery:my-project.my_dataset.users"
```

#### `fav infer` との統合

```bash
fav infer --from bigquery --table users --emit-type
# → type UsersRow = { id: Int, name: String, ... } を src/types.fav に出力
```

---

### v18.5 — 線形型（Linear Types）によるリソース安全性

接続・ファイルハンドル・トランザクションを「使い忘れ」「二重クローズ」から守る。

```fav
// Connection は linear: 必ず 1 回だけ使われる
fn with_connection<T>(f: Connection -o Result<T, String>) -> Result<T, String> !Db {
  bind conn   <- Postgres.connect()    // conn: Connection (linear)
  bind result <- f(conn)               // conn は f に「移動」
  // conn は f の中でクローズされる（忘れるとコンパイルエラー）
  result
}

// トランザクション
fn transact<T>(f: Tx -o Result<T, String>) -> Result<T, String> !Db {
  bind tx <- Postgres.begin()
  match f(tx) {
    Result.ok(v)  => { Postgres.commit(tx); Result.ok(v) }
    Result.err(e) => { Postgres.rollback(tx); Result.err(e) }
  }
}
```

`-o` は「linear arrow」（線形関数型）。引数を必ずちょうど 1 回使う。

---

### v18.6 — 共変・反変アノテーション（Variance）

ジェネリクスの型安全性を完全にする。

```fav
// covariant（+）: Producer パターン
interface Source<+T> {
  fn next() -> Option<T>
}

// contravariant（-）: Consumer パターン
interface Sink<-T> {
  fn write(val: T) -> Result<Unit, String>
}

// invariant（デフォルト）
interface Transform<T> {
  fn apply(val: T) -> T
}
```

---

### v18.7 — 型レベル定数（Const Generics）

コンパイル時に値を型パラメータとして渡す。

```fav
// リストのサイズを型で表現
type FixedList<T, const N: Int> = ...

fn take_n<T, const N: Int>(list: List<T>) -> FixedList<T, N> { ... }

// バッチサイズを型で保証
stage ProcessBatch<const BATCH_SIZE: Int>(rows: FixedList<Row, BATCH_SIZE>) -> List<Output> {
  ...
}
```

---

### v18.8 — 型駆動 API 生成

型定義から自動的に REST / GraphQL API スキーマを生成する。

```fav
// #[api] アノテーション
#[api(method = "GET", path = "/users/:id")]
fn get_user(id: Int) -> Result<User, String> !Db { ... }

#[api(method = "POST", path = "/orders")]
fn create_order(req: CreateOrderRequest) -> Result<Order, String> !Db { ... }
```

```bash
fav generate api --format openapi    # OpenAPI 3.0 仕様書生成
fav generate api --format graphql    # GraphQL スキーマ生成
fav serve                            # HTTP サーバー起動（開発用）
```

---

### v19.0 — Type System Maturity マイルストーン宣言

**完了条件:**
1. エフェクトが自動推論される
2. 行多相レコード関数が動作する
3. 関数引数の `where` 制約が動作する
4. `schema "bigquery:..."` から型が生成される
5. Linear type による接続安全性が動作する
6. `fav generate api` が動作する

---

---

## v20.0 — Production Performance

**テーマ**: 「本番で速い言語」
**期間**: v19.1〜v19.8 → v20.0 マイルストーン宣言

### なぜ最後か

パフォーマンス最適化は**何を最適化すべきか**がわかってから行うものである。
v17〜v19 で言語が成熟し、実際のユーザーがどのパイプラインを書くかが見えてきてから
ボトルネックを特定・解消する。「早すぎる最適化は諸悪の根源」。

### 目標

- 10GB CSV の処理が現在比 10x 高速化
- メモリ使用量を現在比 50% 削減
- コールドスタート（Lambda 起動）が 100ms 以下

---

### v19.1 — 遅延評価パイプライン（Lazy / Streaming）

現在: パイプラインは全ステージで全データをメモリに乗せる（eager evaluation）
目標: ストリーミング評価でメモリフットプリントを最小化

```fav
// #[streaming] でパイプラインをストリーミング評価に切り替え
#[streaming(chunk_size = 1000)]
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb

// 内部的に:
// 1. LoadCsv が 1000 行ずつ生成
// 2. Transform が 1000 行ずつ処理
// 3. WriteToDb が 1000 行ずつ書き込み
// 全データをメモリに乗せない
```

---

### v19.2 — AOT コンパイル（Cranelift バックエンド）

現在: バイトコード VM（インタープリタ実行）
目標: ネイティブバイナリ生成

```bash
# ネイティブバイナリとしてビルド
fav build --target native src/pipeline.fav -o pipeline

# 実行
./pipeline
```

- **Cranelift** を AOT バックエンドとして採用（wasmtime と同じ基盤）
- `wasm-encoder` → `cranelift-codegen` への並行実装
- VM との互換性維持（`--target vm` で従来通り動作）

---

### v19.3 — インクリメンタルコンパイル

現在: 毎回全ファイルを再コンパイル
目標: 変更ファイルのみ再コンパイル

```
# コンパイルキャッシュ
~/.fav/cache/
  <project-hash>/
    <file-hash>.ir     # IR キャッシュ
    <file-hash>.types  # 型情報キャッシュ
```

- AST / IR を `~/.fav/cache/` にキャッシュ
- ファイル変更チェック（mtime + content hash）
- 依存グラフ追跡（A が B を use していたら B の変更で A も再コンパイル）

---

### v19.4 — 並列コンパイル

現在: シングルスレッドコンパイル
目標: ファイル単位で並列コンパイル

```
フェーズ 1: 全ファイルの AST 生成（並列）
フェーズ 2: 型チェック（依存関係に従いトポロジカルソート後並列）
フェーズ 3: IR 生成（並列）
フェーズ 4: リンク（シングルスレッド）
```

---

### v19.5 — メモリレイアウト最適化

現在: `Value` enum は全バリアントが最大サイズを占める
目標: データ指向レイアウト（列指向 / Arena アロケーション）

```
現在: Vec<Value> = [variant_tag(1byte) + data(24bytes)] × N
最適: 列指向ストレージ
  - names:  Vec<String>   (連続メモリ)
  - amounts: Vec<f64>      (SIMD フレンドリー)
  - flags:  Vec<bool>      (ビットベクトル)
```

- Arrow 形式（`arrow` crate）との直接統合
- `stage` の出力が Arrow RecordBatch として格納される
- Parquet への書き込みがゼロコピーで可能に

---

### v19.6 — WASM 最適化

現在: WASM 出力が大きく、Playground の初期ロードが遅い
目標: WASM サイズ 50% 削減、初期実行 100ms 以下

- デッドコード除去（使われていない stdlib を除外）
- `wasm-opt`（Binaryen）による最適化パス統合
- WASM コンポーネントモデル対応（`wasm:io` / `wasm:http` インターフェース）

---

### v19.7 — `fav run --compile-cache`

本番環境（Lambda / ECS）での高速起動。

```bash
# 事前コンパイル
fav compile src/pipeline.fav -o pipeline.favc

# キャッシュ済みアーティファクトで起動（コンパイル不要）
fav run --precompiled pipeline.favc
# 起動時間: ~5ms（現在: ~200ms）
```

---

### v19.8 — プロファイリング強化

現在の `fav profile` は stage レベルの時間計測のみ。
関数レベル・行レベルのフレームグラフを生成できるようにする。

```bash
fav run --profile=flamegraph src/pipeline.fav
# → flamegraph.svg を生成（`inferno` / `pprof` 互換）
```

---

### v20.0 — Production Performance マイルストーン宣言

**完了条件:**
1. `#[streaming]` パイプラインが動作し、10GB CSV を定常メモリで処理できる
2. `fav build --target native` でネイティブバイナリが生成される
3. 2 回目以降のコンパイルがインクリメンタルで高速化される
4. Arrow 形式でのデータ交換が動作する
5. `fav run --precompiled` が動作する
6. WASM サイズが v16.0.0 比 50% 削減される

---

---

## 全体ロードマップ一覧

```
v16.0.0  Production Multi-Cloud 宣言（完了）
│
├── v16.1  エラーメッセージ品質向上
├── v16.2  文字列補間（f"..."）
├── v16.3  レコード更新構文（{ ...row, field: val }）
├── v16.4  stdlib 拡充（List / String / DateTime / Math）
├── v16.5  型エイリアス（alias）
├── v16.6  モジュールシステム強化
├── v16.7  assert_eq + fav test 成熟
├── v16.8  tap / inspect オペレータ
▼
v17.0.0  Language Ergonomics 宣言
│
├── v17.1  境界付きジェネリクス（T with Ord）
├── v17.2  パターンマッチ拡張（or-pattern / list-pattern）
├── v17.3  コレクション内包表記
├── v17.4  let バインディング除去（誤実装の修正）
├── v17.5  REPL 品質向上
├── v17.6  fav bench
├── v17.7  プロパティベーステスト（forall）
├── v17.8  パッケージシステム成熟（fav add / fav publish）
▼
v18.0.0  Language Power 宣言
│
├── v18.1  エフェクト推論
├── v18.2  行多相（Row Polymorphism）
├── v18.3  改良版 where 制約（Refinement Types）
├── v18.4  スキーマ型（schema "bigquery:..."）
├── v18.5  線形型（リソース安全性）
├── v18.6  共変・反変アノテーション
├── v18.7  型レベル定数（Const Generics）
├── v18.8  型駆動 API 生成（fav generate / fav serve）
▼
v19.0.0  Type System Maturity 宣言
│
├── v19.1  遅延評価パイプライン（#[streaming]）
├── v19.2  AOT コンパイル（Cranelift バックエンド）
├── v19.3  インクリメンタルコンパイル
├── v19.4  並列コンパイル
├── v19.5  メモリレイアウト最適化（Arrow 統合）
├── v19.6  WASM 最適化
├── v19.7  fav run --precompiled
├── v19.8  プロファイリング強化（flamegraph）
▼
v20.0.0  Production Performance 宣言
```

---

## 設計原則（全バージョン共通）

1. **後方互換性**: 既存のパイプラインコードは常に動作する。構文の追加はあっても既存構文の削除はしない。
2. **段階的採用**: 新機能はオプトインで使える。`--legacy` のように古い書き方も残す。
3. **エラー駆動設計**: 新機能を追加するたびに、それが生むエラーメッセージを先に設計する。
4. **データエンジニア目線**: 言語理論的に美しいかより、データパイプラインを書くときに自然かを優先する。
5. **セルフホスト維持**: `compiler.fav` / `checker.fav` は常に最新の言語機能で書かれた状態を維持する。

---

## 参照

| ファイル | 目的 |
|---|---|
| `versions/roadmap/roadmap-v16.1-v17.0.md` | v17.0 詳細実装計画 |
| `versions/roadmap/roadmap-v17.1-v18.0.md` | v18.0 詳細実装計画 |
| `versions/roadmap/roadmap-v18.1-v19.0.md` | v19.0 詳細実装計画 |
| `versions/roadmap/roadmap-v19.1-v20.0.md` | v20.0 詳細実装計画 |
| `versions/roadmap/roadmap-v15.1-v16.0.md` | 前ロードマップ（形式参照） |
| `versions/roadmap-v20.1-v25.0.md` | v20.1〜v25.0 マスタースケジュール |
| `versions/roadmap/roadmap-v20.1-v21.0.md` | v21.0 Runtime Excellence 詳細計画 |
| `versions/roadmap/roadmap-v21.1-v22.0.md` | v22.0 Developer Tooling Complete 詳細計画 |
| `versions/roadmap/roadmap-v22.1-v23.0.md` | v23.0 Distributed Scale 詳細計画 |
| `versions/roadmap/roadmap-v23.1-v24.0.md` | v24.0 VM in Favnir 詳細計画 |
| `versions/roadmap/roadmap-v24.1-v25.0.md` | v25.0 Practical Self-Hosting 詳細計画 |
