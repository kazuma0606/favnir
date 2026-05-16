# Favnir v1.6.0 仕様書 — 言語表現力 + 開発ループ改善

作成日: 2026-05-08

> **テーマ**: 文字列補間・レコード分解パターン・テストツール改善・ファイル監視により、
> 日常的なコーディングの表現力と開発体験を向上させる。
>
> **前提**: v1.5.0 完了（462 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.6.0` がビルドされ HELP テキストに反映される |
| 1 | 文字列補間 | `$"Hello {name}!"` 構文がパース・型検査・実行できる |
| 2 | レコード分解パターン | `match u { { name, age } -> ... }` がパース・型検査・実行できる |
| 3 | `fav test` 強化 | `--filter`・テスト統計・`assert_matches` が動く |
| 4 | `fav watch` コマンド | ファイル変更を検出して自動的に `check`/`test`/`run` が再実行される |
| 5 | テスト・ドキュメント | 全テスト通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.6.0"`
- `main.rs`: HELP テキスト `v1.6.0`
- HELP に `watch` コマンドを追加

---

## 3. Phase 1 — 文字列補間

### 3-1. 構文

```fav
// f-string: $" で始まる文字列リテラル
$"Hello {name}!"
$"User: {user.name}, age: {user.age}"
$"Result: {if ok { "yes" } else { "no" }}"
$"Value: {Int.show.show(count)}"
```

- `$"..."` で囲んだ文字列内の `{...}` が式として評価される
- `{` を文字として使いたい場合は `\{` でエスケープ
- 式の型は `String` でなければならない（他の型は `Debug.show(x)` で自動変換）
- 入れ子の補間（補間式の中に `$"..."` を書く）はサポートしない（E053 エラー）

### 3-2. 型検査規則

補間式の型 `T` が `String` でない場合、`Debug.show(x)` を自動適用する。
`Debug.show` が定義されていない型（ユーザー定義型で Show キャップを実装していない場合）は
コンパイルエラー E054 とする。

| 補間式の型 | 処理 |
|---|---|
| `String` | そのまま使用 |
| `Int` / `Float` / `Bool` | `Debug.show(x)` を自動適用 |
| ユーザー定義型（Show 実装あり） | `Debug.show(x)` を自動適用 |
| ユーザー定義型（Show 実装なし） | E054 エラー |

### 3-3. エラーコード

| コード | 条件 |
|---|---|
| E053 | 文字列補間の `{...}` 内に `$"..."` を書いた（入れ子補間） |
| E054 | 補間式の型が `Show` を実装していない |

### 3-4. 脱糖

```fav
$"Hello {name}, age: {age}!"
```

は以下と等価:

```fav
"Hello " ++ name ++ ", age: " ++ Debug.show(age) ++ "!"
```

`++` は `String.concat` 相当の連結演算子（既存の `String` 連結を利用）。

### 3-5. AST の変更

```rust
// ast.rs に追加
pub enum FStringPart {
    Lit(String),
    Expr(Box<Expr>),
}

// Expr に追加
Expr::FString(Vec<FStringPart>, Span),
```

### 3-6. 字句解析の変更

```rust
// lexer.rs
// $" を検出したら FString トークンに入る
Token::FStringRaw { raw: String, span: Span }
// raw は $" ... " の内容文字列（{ } の外 = Lit、{ } の中 = Expr ソース）
```

具体的な手順:
1. `$` を読んだ後に `"` が続く場合、FString モードに入る
2. `{` (ネスト深さ 0) → Expr 収集開始
3. `{` ネスト深さ > 0 → Expr 内の `{` として扱う
4. `}` でネスト深さが 0 に戻る → Expr 収集終了
5. `\"` または末尾の `"` で FString 終了
6. `\{` → リテラル `{` として取り込む

### 3-7. パーサーの変更

```rust
// parser.rs に追加
fn parse_fstring(raw: &str, base_span: Span) -> Result<Expr, ParseError> {
    // raw を Lit / Expr ソーステキスト のリストに分割
    // Expr ソースを再帰的に Parser::parse_expr で解析
    // FStringPart::Lit / FStringPart::Expr を構築して Expr::FString を返す
}
```

`Token::FStringRaw` を `parse_expr` の中で `parse_fstring` に委譲する。

### 3-8. チェッカーの変更

```rust
// checker.rs
// check_expr の FString ケース
Expr::FString(parts, span) => {
    for part in parts {
        if let FStringPart::Expr(inner) = part {
            let ty = self.check_expr(inner);
            match ty {
                Type::String => {}  // そのまま
                Type::Int | Type::Float | Type::Bool => {}  // Debug.show 適用
                Type::Named(_, _) => {
                    // Show cap の impl を確認、なければ E054
                    if !self.has_show_impl(&ty) {
                        self.errors.push(E054 ...);
                    }
                }
                _ => {} // Unknown は通す
            }
        }
    }
    Type::String
}
```

### 3-9. コンパイラ / VM の変更

`Expr::FString(parts)` を IRExpr に変換:
- `String` 型の parts を `++` 連結に脱糖
- 非 `String` 部分を `IRExpr::Call(Debug.show, [inner])` で包む

VM は既存の文字列連結ロジックを使用するため VM 本体の変更は不要。

---

## 4. Phase 2 — レコード分解パターン

### 4-1. 構文

```fav
match user {
    // pun: フィールド名をそのまま変数名として束縛
    { name, age } -> $"name={name} age={Debug.show(age)}"

    // 別名: フィールドに異なる変数名を付ける
    { name: n, age: a } -> $"n={n} a={Debug.show(a)}"

    // 部分一致: 必要なフィールドだけ
    { name } -> name

    // ネスト: フィールドの値にさらにパターンを適用
    { address: { city } } -> city

    // ガード付き
    { age } if age >= 18 -> "adult"
    { age } -> "minor"

    _ -> "unknown"
}
```

### 4-2. 型検査規則

- スクルーティニーの型が `Record` でない場合 → E055 エラー
- パターン内のフィールド名が Record 型に存在しない場合 → E056 エラー
- パターン変数の型はフィールドの型に一致

### 4-3. エラーコード

| コード | 条件 |
|---|---|
| E055 | レコード分解パターンをレコード型以外の値に適用した |
| E056 | レコード分解パターンに存在しないフィールド名を指定した |

### 4-4. AST の変更

```rust
// ast.rs の Pattern に追加
Pattern::Record(Vec<RecordPatternField>, Span),

pub struct RecordPatternField {
    pub field: String,
    pub pattern: Option<Box<Pattern>>,  // None = pun (同名変数)
    pub span: Span,
}
```

### 4-5. パーサーの変更

`parse_pattern` で `{` トークンを検出した場合に `parse_record_pattern` を呼ぶ。

```
record_pattern ::= "{" record_field_pattern ("," record_field_pattern)* "}"
record_field_pattern ::= IDENT (":" pattern)?
```

フィールドの後に `:` + パターンがなければ pun（同名変数束縛）。

### 4-6. チェッカーの変更

```rust
// check_pattern の Record ケース
Pattern::Record(fields, span) => {
    let record_ty = scrutinee_ty;
    match record_ty {
        Type::Named(name, _) => {
            // record_ty が Record 型かチェック → E055
            let type_def = self.type_registry.get(&name);
            for field_pat in fields {
                // フィールドの存在確認 → E056
                // pun の場合: env.define(field.field.clone(), field_ty)
                // alias の場合: check_pattern(field.pattern, field_ty)
            }
        }
        _ => self.errors.push(E055 ...)
    }
    record_ty
}
```

### 4-7. IR / コンパイラの変更

```rust
// ir.rs の IRPattern に追加
IRPattern::Record(Vec<(String, IRPattern)>),
// (field_name, sub_pattern) のリスト
```

コンパイラ: `Pattern::Record` → `IRPattern::Record` に変換。
フィールド pun の場合は `IRPattern::Bind(field_name)` と同等。

### 4-8. VM の変更

```rust
// vm.rs のパターンマッチング
IRPattern::Record(fields) => {
    // scrutinee が VMValue::Record(map) であることを確認
    for (field, sub_pattern) in fields {
        let field_val = map.get(field)?;
        if !match_pattern(sub_pattern, field_val, env) { return None; }
    }
    Some(())
}
```

---

## 5. Phase 3 — `fav test` 強化

### 5-1. `--filter` フラグ

```
fav test                          // すべてのテストを実行
fav test --filter "user"          // 説明に "user" を含むテストのみ実行
fav test --filter "parse,validate" // カンマ区切りで複数フィルター（OR 条件）
fav test examples/math.test.fav   // ファイル指定
```

実装: `test "desc" { ... }` の `desc` 文字列に対して `filter_pattern` を部分一致チェック。

### 5-2. テスト統計の改善

現在の出力:
```
PASS  test name
FAIL  test name
```

v1.6.0 の出力:
```
running 5 tests in examples/math.test.fav
  PASS  addition works           (0.2ms)
  PASS  subtraction works        (0.1ms)
  FAIL  multiplication works
        assertion failed: expected 6, got 7
  PASS  division works           (0.1ms)
  PASS  edge cases               (0.3ms)

test result: 4 passed; 1 failed; 0 filtered; finished in 0.8ms
```

- 各テストの実行時間（`std::time::Instant` で計測）
- テスト結果サマリー（passed / failed / filtered）
- フィルター適用時: `filtered` カウント

### 5-3. `assert_matches` ビルトイン

```fav
// パターンに一致するかを検証
assert_matches(value, some(_))          // Option::Some であれば通過
assert_matches(value, ok(_))            // Result::Ok であれば通過
assert_matches(result, err("not found")) // 特定のエラーと一致
```

構文:
```
assert_matches(<expr>, <pattern>)
```

- `<pattern>` はコンストラクタパターン（variant 名 + オプションのペイロードパターン）
- 一致しない場合はテスト失敗（`assert_eq` と同様）
- AST レベルでは `Expr::AssertMatches(expr, pattern, span)` として表現

### 5-4. `--no-capture` フラグ

テスト本体内の `IO.println` などの出力を stdout に流す（デフォルトは抑制）。

```
fav test --no-capture
```

---

## 6. Phase 4 — `fav watch` コマンド

### 6-1. CLI

```
fav watch                                  // check を繰り返す（デフォルト）
fav watch --cmd test                       // test を繰り返す
fav watch --cmd run                        // run を繰り返す
fav watch --cmd "check,test"              // 複数コマンドを順番に実行
fav watch src/main.fav                     // 特定ファイルのみ監視
```

### 6-2. 動作

1. 起動時: 対象コマンドを一度実行
2. `.fav` ファイルの変更を検出する（`notify` クレートを使用）
3. 変更検出後: 端末をクリア + コマンド再実行
4. Ctrl+C で終了

```
[watch] starting...
[watch] running: fav check
✓ no errors

[watch] watching 3 files for changes...
[watch] changed: src/main.fav
[watch] running: fav check
error[E001]: ...
  --> src/main.fav:5:3
   ...

[watch] watching 3 files for changes...
```

### 6-3. 依存関係

```toml
# Cargo.toml に追加
notify = { version = "6", features = ["macos_kqueue"] }
```

`notify` クレートのクロスプラットフォーム対応:
- Windows: `ReadDirectoryChangesW`
- macOS: `kqueue`
- Linux: `inotify`

### 6-4. 実装方針

```rust
// driver.rs に追加
pub fn cmd_watch(file: Option<&str>, cmd: &str) {
    // 初回実行
    run_watch_command(file, cmd);

    // ファイル一覧を収集（fav.toml の src ディレクトリ or 指定ファイル）
    let paths = collect_watch_paths(file);

    // notify でウォッチャーを設定
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    for path in &paths {
        watcher.watch(path, notify::RecursiveMode::NonRecursive)?;
    }

    eprintln!("[watch] watching {} files for changes...", paths.len());

    loop {
        match rx.recv() {
            Ok(Ok(event)) if is_modify_event(&event) => {
                // デバウンス（連続変更の防止）: 50ms 待機
                std::thread::sleep(Duration::from_millis(50));
                // キューを消化
                while rx.try_recv().is_ok() {}
                // ターミナルクリア
                print!("\x1b[2J\x1b[H");
                eprintln!("[watch] changed: {:?}", event.paths[0]);
                run_watch_command(file, cmd);
                eprintln!("[watch] watching {} files for changes...", paths.len());
            }
            Err(_) => break,
            _ => {}
        }
    }
}

fn run_watch_command(file: Option<&str>, cmd: &str) {
    for c in cmd.split(',') {
        match c.trim() {
            "check" => cmd_check(file),
            "test"  => cmd_test(file),
            "run"   => cmd_run(file, false, None),
            other   => eprintln!("[watch] unknown command: {}", other),
        }
    }
}
```

---

## 7. Phase 5 — テスト・ドキュメント

### 7-1. テスト要件

#### 文字列補間テスト

| テスト名 | 検証内容 |
|---|---|
| `fstring_simple_parse` | `$"Hello {name}!"` がパースできる |
| `fstring_multiple_parts` | 複数の補間を含む文字列がパースできる |
| `fstring_non_string_auto_show` | `Int` 補間が `Debug.show` で自動変換される |
| `fstring_exec_correct_output` | 実行時に正しい文字列が生成される |
| `fstring_escape_brace` | `\{` がリテラル `{` として扱われる |
| `fstring_e054_no_show` | Show 未実装型で E054 が発生する |

#### レコード分解パターンテスト

| テスト名 | 検証内容 |
|---|---|
| `record_pat_pun_parse` | `{ name, age }` がパースできる |
| `record_pat_alias_parse` | `{ name: n, age: a }` がパースできる |
| `record_pat_check_ok` | 正しいフィールド参照で型検査が通る |
| `record_pat_e055_non_record` | 非レコード型への適用で E055 が発生する |
| `record_pat_e056_unknown_field` | 存在しないフィールドで E056 が発生する |
| `record_pat_exec_pun` | pun パターンが実行時に正しく束縛される |
| `record_pat_exec_alias` | alias パターンが実行時に正しく束縛される |
| `record_pat_partial` | 部分一致（フィールドの一部のみ指定）が動く |
| `record_pat_nested` | ネストしたレコード分解が動く |

#### `fav test` 強化テスト

| テスト名 | 検証内容 |
|---|---|
| `test_filter_matches_description` | `--filter` で説明が一致するテストだけ実行される |
| `test_filter_excludes_non_matching` | `--filter` で一致しないテストが除外される |
| `test_stats_summary` | 統計サマリーに passed/failed/filtered が表示される |
| `assert_matches_some_ok` | `assert_matches(some_val, some(_))` が通る |
| `assert_matches_fail` | 不一致で assert_matches がテスト失敗を引き起こす |

#### `fav watch` テスト

| テスト名 | 検証内容 |
|---|---|
| `watch_collect_paths_fav_files` | `collect_watch_paths` が `.fav` ファイルを返す |
| `watch_run_command_check` | `run_watch_command("check")` が型エラーを検出できる |

（`fav watch` の統合テストはインタラクティブなため、ユニットテストのみ）

### 7-2. example ファイル

- `examples/fstring_demo.fav` — 文字列補間の各パターンを示す
- `examples/record_match.fav` — レコード分解パターンの実例
- `examples/watch_demo.fav` — `fav watch --cmd test` での開発サイクルを示す（`math.test.fav` 相当）

### 7-3. ドキュメント更新

- `versions/v1.6.0/langspec.md` を新規作成
  - `$"..."` 文字列補間構文と型変換ルール
  - E053 / E054 エラーコード
  - レコード分解パターン構文 (pun / alias / partial / nested)
  - E055 / E056 エラーコード
  - `assert_matches` ビルトイン
  - `fav test --filter` / テスト統計
  - `fav watch` コマンド
- `README.md` に v1.6.0 セクション追加

---

## 8. 完了条件（Done Definition）

- [x] `$"Hello {name}!"` が文字列 `"Hello Alice!"` を生成する
- [x] 非 String 補間式に `Debug.show` が自動適用される
- [x] Show 未実装型の補間で E054 が発生する
- [x] `match user { { name, age } -> ... }` がパース・実行できる
- [x] 存在しないフィールドのレコード分解で E056 が発生する
- [x] `fav test --filter "keyword"` で説明が一致するテストだけ実行される
- [x] テスト結果に passed/failed/filtered/時間 が表示される
- [x] `assert_matches(value, some(_))` が型検査・実行できる
- [x] `fav watch --cmd check` が `.fav` ファイル変更を検出して再実行する
- [x] v1.5.0 の全テスト（462）が引き続き通る
- [x] `cargo build` で警告ゼロ
- [x] `Cargo.toml` バージョンが `"1.6.0"`

---

## 9. 先送り一覧（v1.6.0 では対応しない）

| 制約 | バージョン |
|---|---|
| 文字列補間内の入れ子補間（`$"outer {$"inner"}"`) | v2.0.0 以降 |
| レコード分解でのスプレッド（`{ name, ..rest }`） | v2.0.0 以降 |
| artifact の explain metadata 圧縮（gzip） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` / JSON キー renaming | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking） | v2.0.0 以降 |
| エフェクトの `use` による再エクスポート | v2.0.0 |
| エフェクト階層（`effect Foo extends Bar`） | v2.0.0 以降 |
| `fav lint` カスタムルールプラグイン | v2.0.0 以降 |
| `fav test --coverage` | v1.7.0 以降 |
| `fav watch` の複数ディレクトリ監視 | v1.7.0 以降 |
