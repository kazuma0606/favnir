# Roadmap v16.1.0 〜 v17.0.0 — Language Ergonomics

Date: 2026-06-14

## 目標

v16.0.0「Production Multi-Cloud」宣言をもって Favnir はクラウド統合の第一段階を完了した。
次のフェーズは**言語自体の成熟**である。
データエンジニアが「書きたくなる言語」になるために、日常の書き心地を徹底的に改善する。

- v16.1: エラーメッセージを `rustc` スタイルへ刷新（typo ヒント・行番号・URL）
- v16.2: 文字列補間 `f"Hello, {name}!"` で `String.concat` 連鎖を一掃
- v16.3: レコード更新構文 `{ ...row, status: "ok" }` でフィールド全書きを排除
- v16.4: `List.group_by` / `DateTime.now` / `Math.round_to` 等の stdlib 大幅拡充
- v16.5: 型エイリアス `alias Email = String` でシグネチャの可読性向上
- v16.6: モジュールインポート強化（選択 / ワイルドカード / エイリアス）
- v16.7: `assert_eq` / `test_group` / スナップショットテストで `fav test` を実用レベルへ
- v16.8: パイプライン `tap` オペレータで中間観察・ロギングを型安全に
- v17.0: Language Ergonomics マイルストーン宣言

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| エラー表示形式 | `rustc` スタイル（`-->` ファイル・行・列、`^` アンダーライン、`= hint:`、`= help:`）|
| typo 候補提示 | Levenshtein 距離 ≤ 2 の変数名 / 関数名 / 型名を最大 3 候補表示 |
| 文字列補間構文 | `f"..."` プレフィックス（Python / Swift 方式）。`{expr}` 内に任意の式を記述可能 |
| f-string デシュガー | コンパイル時に `String.concat` 連鎖へ展開（VM 変更なし） |
| レコードスプレッド | `{ ...base, key: val }` — `base` の型を静的解決して各フィールドを取り出す |
| 型エイリアス | `alias` キーワード。構造的同一性を保つ（`type Name(Inner)` の名目型とは別物）|
| モジュール可視性 | `public fn` で外部公開。デフォルトはモジュール内のみ（現状と同じ） |
| `tap` の実行制御 | `--no-tap` フラグ指定時は tap クロージャを完全スキップ（本番パフォーマンス保護）|
| v17.0 テスト数 | 累計 1600 件以上を目標 |

---

## バージョン計画

### v16.1.0 — エラーメッセージ品質向上

**テーマ**: 言語の第一印象を決めるエラーメッセージを `rustc` スタイルに刷新する。
「何が」だけでなく「なぜ」「どこで」「どう直すか」を伝える。

**現状と目標の比較:**

```
// 現状
[E0001] undefined variable: user_id

// 目標
[E0001] undefined variable: user_id
 --> src/pipeline.fav:12:5
  |
12 |   transform(user_id, name)
  |             ^^^^^^^ この変数は未定義です
  |
  = ヒント: `userId` (line 8) の typo ではないですか？
  = 参照: https://favnir.dev/errors/E0001
```

**実装内容:**

- `fav/src/frontend/lexer.rs` / `parser.rs`:
  - `Span { file: String, line: usize, col: usize, len: usize }` 構造体追加
  - 全トークンに `Span` を付与
  - AST ノードに `Span` フィールド伝播

- `fav/src/error.rs`（新規）:
  - `Diagnostic { code: String, msg: String, span: Span, hints: Vec<String>, notes: Vec<String> }` 型
  - `format_diagnostic(diag: &Diagnostic, source: &str) -> String` — rustc スタイル表示
  - ソース行テキスト取得 + `^` アンダーライン生成
  - URL 付与: `https://favnir.dev/errors/E{code}`

- `fav/src/middle/checker.rs`:
  - 全エラー生成箇所を `Diagnostic` に統一
  - `levenshtein_candidates(name: &str, candidates: &[&str]) -> Vec<String>` 実装
    - 距離 ≤ 2 の候補を最大 3 件抽出して `hint` として付与
  - E0001〜E0320 全コードに `hint` / `note` / `help` テキスト追加

- `fav/src/driver.rs`:
  - エラー表示を `format_diagnostic` 経由に統一
  - `--no-color` フラグ追加（CI 環境向け）

- `site/content/docs/errors/` ディレクトリ:
  - `E0001.mdx` 〜 `E0020.mdx`（最頻出エラー）の詳細ページ作成
  - 各ページ: エラーの原因・よくある間違い・修正例

- テスト: `v161000_tests`（5件）:
  - `version_is_16_1_0`
  - `error_output_has_line_number`（E0001 出力に ` --> ` が含まれる）
  - `error_output_has_caret`（`^` アンダーラインが含まれる）
  - `error_output_has_hint`（hint テキストが含まれる）
  - `error_output_has_url`（`favnir.dev/errors/` が含まれる）

**完了条件（PASS=5）:**
1. `fav check` 出力に `-->` ファイル・行・列が含まれる
2. `^` アンダーラインが正しい列を指している
3. Levenshtein ≤ 2 の typo に対して候補が提示される
4. 全エラーコードに `hint` または `help` テキストが付与されている
5. エラー URL が出力に含まれる

---

### v16.2.0 — 文字列補間（String Interpolation）

**テーマ**: データエンジニアが最もよく書く「動的文字列の組み立て」を自然に書けるようにする。
`String.concat` の連鎖を `f"..."` で置き換え、コードを劇的に短縮する。

**構文:**

```fav
// 基本
let name  = "Alice"
let count = 42
let msg   = f"Hello, {name}! You have {count} items."

// 式も埋め込める
let sql = f"SELECT * FROM {table} WHERE created_at > '{format_date(since)}'"

// 関数呼び出し
let summary = f"Total: {List.length(rows)} rows, avg: {Math.round_to(avg, 2)}"

// 複数行（triple-quote）
let report = f"""
  User:  {user.name}
  Email: {user.email}
  Items: {List.length(items)}
"""
```

**Before / After 比較:**

```fav
// Before（v16.1 まで）
let msg = String.concat("Hello, ", String.concat(name, String.concat("! You have ", String.concat(String.from_int(count), " items."))))

// After（v16.2 以降）
let msg = f"Hello, {name}! You have {count} items."
```

**実装内容:**

- `fav/src/frontend/lexer.rs`:
  - `f"..."` トークンを `FStringStart` / `FStringLiteral` / `FStringExprStart` / `FStringExprEnd` / `FStringEnd` に分解
  - ネストした `{` `}` のバランスチェック
  - triple-quote `f"""..."""` 対応

- `fav/src/frontend/parser.rs`:
  - `parse_fstring()` — FString トークン列を `Expr::FString(Vec<FStringPart>)` に変換
  - `FStringPart::Literal(String)` / `FStringPart::Expr(Box<Expr>)`

- `fav/src/ast.rs`:
  - `Expr::FString(Vec<FStringPart>)` 追加
  - `FStringPart` enum 追加

- `fav/src/middle/checker.rs`:
  - `check_fstring`: 各 `Expr` 部分の型を検査
  - `Display` を持つ型（String / Int / Float / Bool）のみ補間可能
  - その他の型 → E0322: `型名 does not implement Display`
  - 型強制: `Int` / `Float` / `Bool` は自動 `to_string` 変換

- `fav/src/middle/compiler.rs`:
  - `compile_fstring`: `String.concat` 連鎖にデシュガー（VM 変更なし）
  - `FStringPart::Expr` に型変換 opcode を挿入

- `self/compiler.fav`:
  - f-string のパース・コンパイルを Favnir 実装に追加

- テスト: `v162000_tests`（5件）:
  - `version_is_16_2_0`
  - `fstring_basic_interpolation`（`f"Hello, {name}"` → 正しい文字列）
  - `fstring_int_interpolation`（Int 型の自動変換）
  - `fstring_expr_interpolation`（`f"len={List.length(xs)}"` → 正しい文字列）
  - `fstring_triple_quote`（複数行 f-string が動作する）

**完了条件（PASS=5）:**
1. `f"Hello, {name}!"` が `String.concat` 展開として動作する
2. `{Int}` / `{Float}` / `{Bool}` が自動変換される
3. `{expr}` 内に任意の式（関数呼び出し・フィールドアクセス）が書ける
4. `Display` を持たない型への補間でコンパイルエラーが出る（E0322）
5. triple-quote `f"""..."""` が動作する

---

### v16.3.0 — レコード更新構文（Record Spread / Update）

**テーマ**: データ変換パイプラインの核心操作「1 フィールドだけ変えたい」を自然に書けるようにする。
全フィールドを書き直す必要を排除し、`stage` 内の変換ロジックを簡潔にする。

**設計上の重要な制約:**

スプレッド構文は**値の組み立て方**であり、型の宣言ではない。

```fav
// ✅ 正しい使い方: スプレッドで値を組み立て、戻り型は明示的に宣言
stage Enrich(row: RawRow) -> EnrichedRow {      // ← 戻り型は名前付き型で明示
  Result.ok({ ...row, score: compute_score(row) })  // ← spread は値の組み立てのみ
}

// ❌ 禁止: 戻り型を省略してスプレッドで返す
fn enrich(row: UserRow) {                      // E0327: spread を含む戻り値には
  { ...row, score: 1.0 }                       //        明示的な戻り型宣言が必要
}
```

戻り型を省略すると呼び出し元は関数の中身を読まないと型がわからなくなる。
スプレッドは「値の作り方」を簡潔にする機能であり、戻り型は常に名前付き型で宣言する。

**値位置でのスプレッド構文:**

```fav
// 変数への代入（関数内部での中間値）
let updated = { ...row, status: "processed" }

// 複数フィールド更新
let enriched = { ...row, status: "ok", processed_at: DateTime.now_unix() }

// ネストしたレコードの更新
let fixed = { ...user, address: { ...user.address, zip: "100-0001" } }

// パイプライン stage での自然な使用（戻り型は EnrichedRow として明示）
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({ ...row, score: compute_score(row), enriched_at: DateTime.now_unix() })
}

// 条件付き更新（戻り型は OutputRow として明示）
stage Normalize(row: InputRow) -> OutputRow {
  let trimmed = String.trim(row.name)
  Result.ok({ ...row, name: trimmed, is_valid: String.length(trimmed) > 0 })
}
```

**Before / After 比較:**

```fav
// Before（v16.2 まで）
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({
    id:           row.id,
    name:         row.name,
    email:        row.email,
    created_at:   row.created_at,
    score:        compute_score(row),     // ← これだけが新規
    enriched_at:  DateTime.now_unix(),    // ← これだけが新規
  })
}

// After（v16.3 以降）
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({ ...row, score: compute_score(row), enriched_at: DateTime.now_unix() })
}
```

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `parse_record_literal` を拡張: `{ ...expr, key: val, ... }` を認識
  - `RecordEntry::Spread(Box<Expr>)` と `RecordEntry::Field(String, Box<Expr>)` を区別

- `fav/src/ast.rs`:
  - `Expr::RecordUpdate { base: Box<Expr>, updates: Vec<(String, Expr)> }` 追加
  - または `Expr::Record` の `fields` を `RecordField { key: String, value: Expr, is_spread: bool }` に拡張

- `fav/src/middle/checker.rs`:
  - `check_record_update`:
    - `base` の型を解決（`{field: Type, ...}` のレコード型が必要）
    - `base` の型が `Unknown` または未解決の型変数の場合 → E0328（型が静的に確定しない）
    - `updates` の各フィールドが `base` 型に存在することを確認 → 存在しないフィールド → E0323
    - 更新値の型が元のフィールド型と一致することを確認 → 型不一致 → E0009 拡張
  - `check_spread_return`:
    - スプレッド式を含む関数ボディで戻り型が未宣言の場合 → E0327
    - 戻り型が宣言されている場合: `{ ...row, updates }` の展開結果が宣言した型に一致するか検査

- `fav/src/middle/compiler.rs`:
  - `compile_record_update`: `base` の各フィールドを `GetField` opcode で取り出し + `updates` で上書き
  - ネストした `{ ...x.addr, zip: "..." }` も再帰的にコンパイル

- `self/compiler.fav` / `self/checker.fav`:
  - レコードスプレッド構文に対応追加

- テスト: `v163000_tests`（5件）:
  - `version_is_16_3_0`
  - `record_spread_with_explicit_return_type`（明示的戻り型あり → 正しく動作）
  - `record_spread_without_return_type_error`（戻り型なしのスプレッド返し → E0327）
  - `record_spread_type_check`（存在しないフィールドへの更新で E0323）
  - `record_spread_nested`（ネストしたレコードのスプレッド）

**完了条件（PASS=5）:**
1. 明示的な戻り型を宣言した stage で `{ ...row, field: val }` が動作する
2. 戻り型なしでスプレッドを返す関数が E0327 でブロックされる
3. `Unknown` 型の `base` に対するスプレッドが E0328 でブロックされる
4. 存在しないフィールドへの更新が E0323 を出す
5. 型不一致の更新が E0009 を出す

---

### v16.4.0 — 標準ライブラリ拡充

**テーマ**: データ処理で最頻使用するパターンを stdlib で直接サポートする。
`List.group_by` / `DateTime.now` / `Math.round_to` など、毎回自前実装していた関数を提供する。

**List 拡充:**

```fav
List.group_by(rows, |r| r.category)       // Map<String, List<Row>>
List.sort_by(rows, |r| r.amount)           // 昇順ソート
List.sort_by_desc(rows, |r| r.created_at) // 降順ソート
List.flatten(nested)                       // List<List<T>> -> List<T>
List.flat_map(rows, |r| expand(r))         // map + flatten
List.zip(as, bs)                           // List<Pair<A, B>>
List.zip_with(as, bs, |a, b| merge(a, b)) // List<C>
List.distinct(rows)                        // 重複除去（== による）
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

**String 拡充:**

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
String.repeat(s, n)                       // "abc" を n 回繰り返す
String.to_upper(s) / String.to_lower(s)
String.char_at(s, i)                      // Option<String>
String.lines(s)                           // List<String>（改行で分割）
String.is_empty(s)                        // Bool
String.format_int(n, 3, "0")             // "007"（ゼロ埋め整形）
String.format_float(f, 2)                 // "3.14"（小数点桁数指定）
```

**DateTime（新モジュール）:**

```fav
// 現在時刻
let now = DateTime.now()                   // DateTime 型
let ts  = DateTime.now_unix()              // Int（Unix timestamp 秒）

// 文字列との変換
DateTime.parse("2026-06-14T12:00:00Z")    // Result<DateTime, String>
DateTime.format(dt, "YYYY-MM-DD")          // String
DateTime.format_iso(dt)                    // "2026-06-14T12:00:00Z"

// 演算
DateTime.add_days(dt, 7)
DateTime.add_hours(dt, 24)
DateTime.diff_days(from, to)               // Int
DateTime.diff_seconds(from, to)            // Int

// 比較
DateTime.before(a, b) / DateTime.after(a, b)   // Bool
DateTime.between(dt, from, to)                  // Bool
```

**Math 拡充:**

```fav
Math.abs(n)                                // Int / Float
Math.round(f) / Math.ceil(f) / Math.floor(f)
Math.round_to(f, 2)                        // 小数点以下 2 桁
Math.min(a, b) / Math.max(a, b)
Math.clamp(v, lo, hi)
Math.sqrt(f) / Math.pow(base, exp)
Math.log(f) / Math.log2(f) / Math.log10(f)
```

**実装内容:**

- `fav/src/backend/vm.rs`:
  - 上記全関数の VM プリミティブ追加（`List.*` / `String.*` / `DateTime.*` / `Math.*`）
  - `DateTime` 内部表現: Unix timestamp（Int）+ Rust `chrono::DateTime<Utc>`
  - `chrono` crate を Cargo 依存に追加

- `fav/src/middle/compiler.rs` / `checker.rs`:
  - `DateTime` 型追加（AST `Type::DateTime`）
  - 新プリミティブの型シグネチャを `builtin_ret_ty` に追加

- `self/checker.fav`:
  - `datetime_fn` / `math_fn` スキーム追加

- `runes/stdlib/list.fav` / `string.fav` / `datetime.fav` / `math.fav`:
  - Favnir ラッパー関数（型付きインターフェース）

- テスト: `v164000_tests`（5件）:
  - `version_is_16_4_0`
  - `list_group_by_works`（`group_by` で Map が返る）
  - `list_chunk_works`（`chunk(rows, 3)` で正しく分割）
  - `string_split_works`（`split("a,b,c", ",")` → 3 要素リスト）
  - `datetime_now_unix_works`（`now_unix()` が正の Int を返す）

**完了条件（PASS=5）:**
1. `List.group_by` / `List.chunk` / `List.sort_by` が動作する
2. `String.split` / `String.trim` / `String.replace` が動作する
3. `DateTime.now` / `DateTime.parse` / `DateTime.format_iso` が動作する
4. `Math.round_to` / `Math.clamp` が動作する
5. 新 stdlib 関数のテストが全件 PASS する

---

### v16.5.0 — 型エイリアス（Type Alias）

**テーマ**: 関数シグネチャの可読性を上げる。
`alias Email = String` で「ドキュメント目的の別名」を宣言できるようにする。
既存の名目型ラッパー `type Name(Inner)` とは目的が異なる。

**構文:**

```fav
// 基本の型エイリアス
alias Timestamp  = String
alias JsonStr    = String
alias RowId      = Int
alias Email      = String

// ジェネリック型エイリアス
alias Result2<T>  = Result<T, String>
alias Rows<T>     = List<T>
alias Pipeline<A, B> = stage A -> B

// 使用例: シグネチャが読みやすくなる
fn parse_timestamp(s: Timestamp) -> Result2<DateTime> {
  DateTime.parse(s)
}

fn process(id: RowId, email: Email) -> Result2<JsonStr> {
  ...
}

// エイリアス同士は元の型と交換可能（構造的同一性）
fn send(addr: Email) -> Result<Unit, String> { ... }
let e: String = "alice@example.com"
send(e)  // OK: Email = String は同じ型
```

**`alias` vs `type` の違い:**

| | `alias Email = String` | `type Email(String)` |
|---|---|---|
| 互換性 | `String` と交換可能 | 区別される（`Email.unwrap()` が必要） |
| 目的 | ドキュメント・可読性 | ビジネスルール強制 |
| `where` バリデーター | 不可 | 可能 |

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `alias Name = Type` / `alias Name<T> = Type<T>` の構文追加
  - `TopLevel::AliasDecl { name: String, params: Vec<String>, ty: Type }` 追加

- `fav/src/ast.rs`:
  - `TopLevel::AliasDecl` 追加

- `fav/src/middle/checker.rs`:
  - `resolve_alias`: 型チェック時にエイリアスを展開（型検査は展開後の型で行う）
  - `alias_env: HashMap<String, Type>` をチェック状態に追加
  - ジェネリックエイリアスの型引数展開

- `fav/src/middle/compiler.rs`:
  - `AliasDecl` はコンパイル時に無視（型情報のみ、IR 生成なし）

- `self/checker.fav`:
  - `alias` 宣言の解析と展開ロジックを Favnir 実装に追加

- テスト: `v165000_tests`（5件）:
  - `version_is_16_5_0`
  - `alias_basic`（`alias Email = String` が解析される）
  - `alias_interchangeable`（`Email` と `String` が交換可能であることを型チェック）
  - `alias_generic`（`alias Result2<T> = Result<T, String>` が動作）
  - `alias_in_signature`（エイリアスを使った関数が正常にコンパイル・実行される）

**完了条件（PASS=5）:**
1. `alias Email = String` が解析・型解決される
2. エイリアス型は元の型と交換可能（型エラーなし）
3. ジェネリックエイリアス `alias Result2<T> = Result<T, String>` が動作する
4. エイリアスを引数型・戻り型に使った関数が正常に動作する
5. `type Name(Inner)` との共存に問題がない

---

### v16.6.0 — モジュールシステム強化

**テーマ**: 現在の `use module.fn` は 1 関数ずつのインポートのみ。
複数関数の選択インポート・ワイルドカード・エイリアスを追加し、
大きなプロジェクトでのモジュール管理を楽にする。

**新構文:**

```fav
// 選択インポート
use utils.{ format_date, parse_csv, validate_email }

// ワイルドカードインポート（名前空間が明確な場合）
use helpers.*

// エイリアスインポート
use very.long.module.name as m
use json as J

// re-export（ライブラリ開発向け）
pub use internal.helper_fn
pub use validators.{ check_email, check_phone }
```

**モジュールの可視性:**

```fav
// public: 他のモジュールから use できる
public fn exported_fn() -> String { ... }

// 省略（デフォルト）: モジュール内のみ
fn internal_helper() -> String { ... }
```

**fav.toml でのモジュールパス設定:**

```toml
[project]
name = "my-pipeline"
src = "src/"
modules = ["lib/", "utils/"]   # 追加検索パス
```

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `use module.{ fn1, fn2 }` の解析（`UseDecl::Multi`）
  - `use module.*` の解析（`UseDecl::Wildcard`）
  - `use module as alias` の解析（`UseDecl::Aliased`）
  - `pub use ...` の解析（`UseDecl::ReExport`）

- `fav/src/ast.rs`:
  - `UseDecl` enum に `Multi` / `Wildcard` / `Aliased` / `ReExport` variant 追加

- `fav/src/middle/resolver.rs`:
  - `resolve_multi_use`: 複数関数を個別の `UseDecl::Single` に展開
  - `resolve_wildcard_use`: モジュールファイルを解析して全 public fn をインポート
  - `resolve_aliased_use`: エイリアス名で関数を登録

- `fav/src/middle/checker.rs`:
  - `pub fn` の可視性チェック
  - 非 public 関数への外部からのアクセス → E0324

- テスト: `v166000_tests`（5件）:
  - `version_is_16_6_0`
  - `use_multi_import`（`use utils.{ fn1, fn2 }` が動作する）
  - `use_wildcard_import`（`use helpers.*` が動作する）
  - `use_alias_import`（`use json as J` で `J.stringify` が使える）
  - `module_visibility_check`（非 public 関数へのアクセスで E0324）

**完了条件（PASS=5）:**
1. `use module.{ fn1, fn2 }` で複数関数が同時にインポートされる
2. `use module.*` で全 public 関数がインポートされる
3. `use module as alias` でエイリアス名でアクセスできる
4. `pub use` で re-export が動作する
5. 非 public 関数への外部アクセスが E0324 でブロックされる

---

### v16.7.0 — `fav test` 成熟（assert_eq / test_group / スナップショット）

**テーマ**: v15.3.0 で基礎を作った `fav test` を実用レベルに引き上げる。
`assert_eq` の追加、テストのグループ化、スナップショットテストで
Favnir ネイティブのテストを本格的なテストフレームワークにする。

**新プリミティブ:**

```fav
// 値の等値比較（現在は assert_true(a == b) で代用）
assert_eq(actual, expected)

// 浮動小数点の近似比較
assert_approx_eq(actual, expected, epsilon)

// リストのアサーション
assert_contains(list, element)
assert_length(list, n)

// 文字列のアサーション
assert_str_contains(s, substring)
assert_str_starts_with(s, prefix)

// エラー内容の検証
assert_err_eq(result, "expected error message")

// スナップショットテスト（初回実行で .snap ファイル生成、2 回目から比較）
assert_snapshot(value, "snapshot_name")
```

**テストの構造化:**

```fav
test_group "transform pipeline" {
  test "trims whitespace" {
    let row = { name: "  Alice  " }
    assert_eq(transform(row).name, "Alice")
  }

  test "handles empty string" {
    let row = { name: "" }
    assert_err(transform(row))
  }

  test "preserves email" {
    let row = { name: "Bob", email: "bob@example.com" }
    assert_eq(transform(row).email, "bob@example.com")
  }
}
```

**出力形式（`fav test` 実行時）:**

```
running 3 tests in "transform pipeline"

  test trims whitespace       ... PASS
  test handles empty string   ... PASS
  test preserves email        ... PASS

test result: PASS. 3 passed; 0 failed; 0 skipped.
```

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `test_group "name" { test ... }` 構文追加（`TopLevel::TestGroup`）
  - `assert_eq` / `assert_approx_eq` / `assert_contains` / `assert_length` /
    `assert_str_contains` / `assert_err_eq` / `assert_snapshot` キーワード追加

- `fav/src/ast.rs`:
  - `TopLevel::TestGroup { name: String, tests: Vec<TestDef> }` 追加

- `fav/src/backend/vm.rs`:
  - `AssertEq` / `AssertApproxEq` / `AssertContains` / `AssertLength` /
    `AssertStrContains` / `AssertErrEq` / `AssertSnapshot` opcode 追加
  - `AssertSnapshot`: `.snap/` ディレクトリに JSON として保存 → 次回比較

- `fav/src/driver.rs`:
  - `cmd_test`: `TestGroup` を展開して個別テストとして実行
  - `--update-snapshots` フラグ追加（スナップショット強制更新）
  - グループ別サマリー出力

- テスト: `v167000_tests`（5件）:
  - `version_is_16_7_0`
  - `assert_eq_pass`（等値で PASS）
  - `assert_eq_fail`（不等値で FAIL・適切なメッセージ）
  - `test_group_runs_all`（グループ内の全テストが実行される）
  - `assert_snapshot_creates_file`（初回実行で `.snap/` にファイル生成）

**完了条件（PASS=5）:**
1. `assert_eq(actual, expected)` が等値で PASS・不等値で FAIL する
2. `assert_err_eq(result, "msg")` がエラー内容を検証する
3. `test_group` でテストがグループ化され、グループ別サマリーが出る
4. `assert_snapshot` が初回で `.snap/` ファイルを生成し、2 回目から比較する
5. `--update-snapshots` でスナップショットが更新される

---

### v16.8.0 — パイプライン `tap` オペレータ

**テーマ**: デバッグ・ロギング・モニタリングのための中間観察演算子。
パイプラインを止めずに途中経過を観察できる。本番では `--no-tap` でゼロコスト化。

**構文:**

```fav
// tap: 値を観察して素通し（副作用のみ、値は変更しない）
seq Pipeline =
  Load
  |> tap(|rows| io.println(f"Loaded: {List.length(rows)} rows"))
  |> Transform
  |> tap(|result| io.println(f"Transformed: {result.count} rows"))
  |> Save

// inspect: デバッグ出力（--debug フラグ時のみ実行）
seq Pipeline =
  Load |> inspect |> Transform |> inspect |> Save

// tap でメトリクス記録
seq Pipeline =
  Extract
  |> tap(|rows| metrics.increment("rows_extracted", List.length(rows)))
  |> Transform
  |> tap(|rows| metrics.increment("rows_transformed", List.length(rows)))
  |> Load
```

**`tap` の型:**

```fav
// tap は (T -> Unit) を受け取り、T -> T の関数として振る舞う
fn tap<T>(observer: fn(T) -> Unit) -> fn(T) -> T

// inspect は tap の特殊版（Debug.show_raw でデバッグ出力）
fn inspect<T>(value: T) -> T
```

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `|> tap(fn)` / `|> inspect` をパイプライン演算子として認識
  - `PipelineStep::Tap(Box<Expr>)` / `PipelineStep::Inspect` を AST に追加

- `fav/src/ast.rs`:
  - `PipelineStep::Tap(Box<Expr>)` / `PipelineStep::Inspect` 追加

- `fav/src/middle/compiler.rs`:
  - `Tap`: observer クロージャを呼び出し後、元の値を返す opcode 列を生成
  - `Inspect`: `Debug.show_raw` を呼び出し後、元の値を返す
  - `--no-tap` フラグ時: `Tap` / `Inspect` を `Nop` に置換（コンパイル時除去）

- `fav/src/backend/vm.rs`:
  - `TapCall` opcode: スタック上の値を複製 → observer 呼び出し → 元の値を戻す

- `runes/stdlib/pipeline.fav`:
  - `tap<T>(observer: fn(T) -> Unit) -> fn(T) -> T` の Favnir 実装

- テスト: `v168000_tests`（5件）:
  - `version_is_16_8_0`
  - `tap_passes_value_through`（tap 後も元の値が変わらない）
  - `tap_calls_observer`（observer クロージャが呼ばれる）
  - `inspect_prints_debug`（inspect がデバッグ出力を生成する）
  - `no_tap_flag_skips_observer`（`--no-tap` で observer が呼ばれない）

**完了条件（PASS=5）:**
1. `|> tap(fn)` がパイプラインの値を変えずに observer を呼ぶ
2. `|> inspect` がデバッグ出力を生成する
3. `--no-tap` フラグ指定時に observer が実行されない
4. tap はパイプラインの型シグネチャに影響しない（型チェック透過）
5. 複数の tap をチェーンできる

---

### v17.0.0 — Language Ergonomics マイルストーン宣言

**テーマ**: v16.x シリーズの集大成。「書きたくなる言語」への転換を宣言する。

**実装内容:**

- `Cargo.toml`: バージョンを `17.0.0` に更新

- `CHANGELOG.md`: v16.1.0〜v16.8.0 の全エントリ追加

- `README.md`:
  - 「現在の状態」を v17.0.0 に更新
  - Language Ergonomics 達成を記載（f-string / record spread / stdlib 拡充）
  - バージョン履歴表に v16.1.0〜v17.0.0 エントリ追加

- `site/content/docs/`:
  - `language/string-interpolation.mdx` 最終更新
  - `language/record-update.mdx` 最終更新
  - `language/testing.mdx` 最終更新（v16.7.0 内容追記）
  - `stdlib/list.mdx` / `stdlib/string.mdx` / `stdlib/datetime.mdx` / `stdlib/math.mdx` 新規作成
  - `language/modules.mdx` 最終更新

- テスト: `v170000_tests`（5件）:
  - `version_is_17_0_0`
  - `changelog_has_v16_entries`（CHANGELOG に v16.x エントリが含まれる）
  - `readme_mentions_fstring`（README に f-string が記載されている）
  - `readme_mentions_record_spread`（README に record spread が記載されている）
  - `stdlib_datetime_doc_exists`（`site/content/docs/stdlib/datetime.mdx` が存在する）

**完了条件:**

| 確認項目 | 状態 |
|---|---|
| 全エラーコードに `hint` / `help` が付与されている | [ ] |
| `f"..."` f-string が動作する | [ ] |
| `{ ...row, field: val }` レコードスプレッドが動作する | [ ] |
| `List.group_by` / `DateTime.now` 等の新 stdlib が使える | [ ] |
| `alias Email = String` が動作する | [ ] |
| `use module.{ fn1, fn2 }` が動作する | [ ] |
| `assert_eq` / `test_group` が動作する | [ ] |
| `|> tap(fn)` が動作する | [ ] |
| `cargo test v170000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 依存関係

```
v16.0.0（Production Multi-Cloud）✅
    |
    v16.1.0（エラーメッセージ刷新）← Span 追加が後続の全バージョンに恩恵
    |
    v16.2.0（f-string）           v16.3.0（record spread）   ← 並列実施可能
    |                              |
    v16.4.0（stdlib 拡充）← DateTime は f-string で自然に使われる
    |
    v16.5.0（型エイリアス）        v16.6.0（モジュール強化）  ← 並列実施可能
    |                              |
    v16.7.0（fav test 成熟）← assert_eq 等は stdlib 拡充に依存
    |
    v16.8.0（tap オペレータ）
    |
    v17.0.0（マイルストーン）
```

v16.1.0 は最優先（Span 追加が全後続バージョンに恩恵をもたらす）。
v16.2.0 と v16.3.0 は独立しているため並列実施可能。
v16.5.0 と v16.6.0 も独立しているため並列実施可能。
v16.4.0（stdlib 拡充）は v16.2.0（f-string）が完了してから着手すると相乗効果が高い。

---

## 新規 Cargo 依存（予定）

| Crate | 用途 | 追加バージョン |
|---|---|---|
| `chrono 0.4` | `DateTime` 型の内部実装 | v16.4.0 |
| その他 | なし（既存依存内で対応） | — |

---

## 実装ノート

- **`Span` の導入**: レキサー改修が最も広範な影響を持つ。`Token` に `Span` を持たせ、AST 構築時に伝播させる。既存の `Parser::parse_str` シグネチャは変えず、内部で `Span` を扱う。
- **f-string のネスト制限**: `{` 内に別の `f"..."` を書くことは v16.2.0 では非対応（E0325）。v16.4.0 以降で検討。
- **レコードスプレッドの実行コスト**: `base` の全フィールドを `GetField` で取り出すため、フィールド数 N のレコードなら O(N) のコピーが発生する。v20.x でメモリ最適化予定。
- **`List.group_by` の戻り型**: `Map<String, List<T>>` を返す。キーは文字列のみ（ジェネリックキーは v18.x の bounded generics 以降）。
- **`DateTime` 型の表現**: VM 内部では Unix timestamp（Int）として保持。`DateTime.now()` は `chrono::Utc::now().timestamp()` を返す Rust プリミティブ。Favnir ユーザーには `DateTime` 型として見える。
- **`alias` の展開タイミング**: 型チェック時（`checker.rs`）にのみ展開。コンパイル後の IR には型情報が残らないため、実行時コストはゼロ。
- **`use module.*` のパフォーマンス**: ビルド時にモジュールファイルを一度解析して public fn を列挙。インクリメンタルビルド（v19.3.0）が入るまでは毎回解析する。
- **`tap` の `--no-tap` 最適化**: コンパイル時に `TapCall` opcode を `Nop` に置換するため、実行時オーバーヘッドはゼロ。`--no-tap` は本番デプロイ（`fav deploy`）のデフォルトにすることを推奨。
- **`assert_snapshot` のファイル形式**: JSON Lines（JSONL）で `.snap/snapshot_name.snap` に保存。`--update-snapshots` フラグで上書き更新。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/roadmap-master.md` | v17.0〜v20.0 の全体戦略（v16.x の位置づけ） |
| `versions/roadmap-v15.1-v16.0.md` | 直前ロードマップ（形式参照） |
| `fav/src/middle/checker.rs` | エラー生成箇所（v16.1.0 改修対象） |
| `fav/src/frontend/lexer.rs` | f-string / Span 追加対象 |
| `fav/src/backend/vm.rs` | stdlib / tap / assert opcode 追加対象 |
| `runes/stdlib/` | Favnir stdlib ラッパー実装 |
| `site/content/docs/language/` | ドキュメント追加対象 |
