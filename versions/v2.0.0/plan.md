# Favnir v2.0.0 実装プラン

作成日: 2026-05-09

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "2.0.0"
```

```rust
// main.rs
const HELP: &str = "fav v2.0.0 ...";

// 例: bench/migrate サブコマンドを HELP に追加
//   migrate  [--in-place] [--dry-run] [--check] <file|dir>
//            migrate v1.x code to v2.0.0 syntax
```

```rust
// src/backend/artifact.rs — FVC バージョンバイト
const FVC_VERSION: u8 = 0x20;  // was 0x06 (v0.6.0), now v2.0.0
```

---

## Phase 1 — 旧キーワード削除（`trf`/`flw`/`cap`）

### 1-1. レキサー（`src/frontend/lexer.rs`）

変更なし。`TokenKind::Trf`, `Flw`, `Cap` は引き続き認識する。
（これにより "unexpected token" の代わりに親切なパーサーエラーが出せる）

### 1-2. パーサー（`src/frontend/parser.rs`）

#### `parse_item` の変更

```rust
// Before:
TokenKind::Trf | TokenKind::Stage => Ok(Item::TrfDef(self.parse_trf_def(vis, false)?)),

// After:
TokenKind::Trf => {
    let span = self.peek_span().clone();
    self.advance(); // consume 'trf'
    Err(ParseError {
        message: "keyword `trf` has been removed in v2.0.0; use `stage` instead (run `fav migrate`)".to_string(),
        span,
    })
},
TokenKind::Stage => Ok(Item::TrfDef(self.parse_trf_def(vis, false)?)),
```

同様に `TokenKind::Flw` と `TokenKind::Cap` も:

```rust
TokenKind::Flw => {
    let span = self.peek_span().clone();
    self.advance();
    Err(ParseError {
        message: "keyword `flw` has been removed in v2.0.0; use `seq` instead (run `fav migrate`)".to_string(),
        span,
    })
},
TokenKind::Cap => {
    let span = self.peek_span().clone();
    self.advance();
    Err(ParseError {
        message: "keyword `cap` has been removed in v2.0.0; use `interface` instead".to_string(),
        span,
    })
},
```

#### `parse_abstract_item` の変更

```rust
// abstract trf → E2001
TokenKind::Trf => {
    let span = self.peek_span().clone();
    self.advance();
    Err(ParseError {
        message: "keyword `abstract trf` has been removed; use `abstract stage` instead".to_string(),
        span,
    })
},
TokenKind::Stage => Ok(Item::AbstractTrfDef(self.parse_abstract_trf_def(visibility)?)),

// abstract flw → E2002
TokenKind::Flw => {
    let span = self.peek_span().clone();
    self.advance();
    Err(ParseError {
        message: "keyword `abstract flw` has been removed; use `abstract seq` instead".to_string(),
        span,
    })
},
TokenKind::Seq => Ok(Item::AbstractFlwDef(self.parse_abstract_flw_def(visibility)?)),
```

#### `parse_trf_def` / `parse_abstract_trf_def` の変更

```rust
// Before:
self.expect_any(&[TokenKind::Trf, TokenKind::Stage])?;

// After:
self.expect(&TokenKind::Stage)?;
```

同様に `parse_flw_def*` も `expect_any` → `expect(TokenKind::Seq)` に変更。

### 1-3. テスト・example の更新

以下のファイルに含まれる `trf`/`flw`/`cap` を `stage`/`seq`/`interface` に書き換える:

**example ファイル**（`examples/*.fav`）:
- `abstract_flw_basic.fav` → `abstract seq`
- `abstract_flw_inject.fav` → `abstract seq`
- `dynamic_inject.fav` → `abstract seq`
- `pipeline.fav` → `stage`/`seq`
- その他 `trf`/`flw` を含む全 example

**テストコード**（`src/middle/checker.rs`, `src/frontend/parser.rs`, etc.）:
- インラインテストの `trf`/`flw` ソース文字列を一括更新
- `cap` 使用テスト → `interface` に更新

**更新スクリプト方針**:
実装時に `grep -r "\"trf\|flw\|cap"` で一覧を取り、手動確認しながら更新する。

---

## Phase 2 — `abstract stage` / `abstract seq` 確認

Phase 1 でパーサーが `abstract stage` / `abstract seq` のみ受け付けるようになることで
自動的に完了する。

`parse_abstract_trf_def` / `parse_abstract_flw_def` 内部では `TokenKind::Stage` / `TokenKind::Seq`
を `expect` するよう変更済みのため、追加変更は不要。

---

## Phase 3 — `fav migrate` コマンド

### 3-1. ドライバー (`src/driver.rs`)

```rust
pub fn cmd_migrate(
    path: &str,
    in_place: bool,
    dry_run: bool,
    check_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let migrated = migrate_source(&source);

    if dry_run || !in_place {
        show_migration_diff(path, &source, &migrated);
        if check_mode && source != migrated {
            std::process::exit(1);
        }
    }
    if in_place && source != migrated {
        std::fs::write(path, &migrated)?;
        println!("migrated: {}", path);
    }
    Ok(())
}

fn migrate_source(source: &str) -> String {
    // 行ごとに処理
    source.lines().map(|line| migrate_line(line)).collect::<Vec<_>>().join("\n")
}

fn migrate_line(line: &str) -> String {
    // コメント行はスキップ
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return line.to_string();
    }

    let line = replace_keyword_token(line, "abstract trf ", "abstract stage ");
    let line = replace_keyword_token(&line, "abstract flw ", "abstract seq ");
    let line = replace_keyword_token(&line, "public trf ",   "public stage ");
    let line = replace_keyword_token(&line, "public flw ",   "public seq ");
    let line = replace_keyword_token(&line, "private trf ",  "private stage ");
    let line = replace_keyword_token(&line, "private flw ",  "private seq ");
    let line = replace_keyword_token_start(&line, "trf ",    "stage ");
    let line = replace_keyword_token_start(&line, "flw ",    "seq ");
    // cap は自動変換不可: TODOコメントを挿入
    if is_cap_definition(&line) {
        format!("// TODO(fav-migrate): convert `cap` to `interface` manually\n{}", line)
    } else {
        line
    }
}

// キーワードの単純な境界チェック付き置換
// "trf_count" のような識別子を誤爆しない
fn replace_keyword_token(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)  // 単純 replace (from に trailing space があるため識別子衝突しない)
}

fn replace_keyword_token_start(s: &str, from: &str, to: &str) -> String {
    let trimmed = s.trim_start();
    if trimmed.starts_with(from) {
        let indent_len = s.len() - trimmed.len();
        let indent = &s[..indent_len];
        format!("{}{}", indent, trimmed.replacen(from, to, 1))
    } else {
        s.to_string()
    }
}

fn is_cap_definition(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("cap ") || t.starts_with("public cap ") || t.starts_with("private cap ")
}

fn show_migration_diff(path: &str, original: &str, migrated: &str) {
    if original == migrated {
        println!("{}: no changes needed", path);
        return;
    }
    println!("Would migrate {}:", path);
    for (i, (orig, new)) in original.lines().zip(migrated.lines()).enumerate() {
        if orig != new {
            println!("  {}: - {}", i + 1, orig);
            println!("  {}: + {}", i + 1, new);
        }
    }
}
```

### 3-2. ディレクトリ対応

```rust
pub fn cmd_migrate_dir(
    dir: &str,
    in_place: bool,
    dry_run: bool,
    check_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let fav_files = collect_fav_files_recursive(dir);  // v1.7.0 で実装済み
    let mut any_changed = false;
    for path in fav_files {
        let path_str = path.to_string_lossy().to_string();
        cmd_migrate(&path_str, in_place, dry_run, check_mode)?;
        // check_mode の場合は変更があれば記録
        let source = std::fs::read_to_string(&path)?;
        if migrate_source(&source) != source { any_changed = true; }
    }
    if check_mode && any_changed { std::process::exit(1); }
    Ok(())
}
```

### 3-3. `main.rs` CLI 追加

```rust
// コマンド解析部分に追加
"migrate" => {
    let mut in_place = false;
    let mut dry_run = false;
    let mut check_mode = false;
    let mut dir: Option<String> = None;
    let mut target: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--in-place" => in_place = true,
            "--dry-run"  => dry_run = true,
            "--check"    => check_mode = true,
            "--dir"      => { i += 1; dir = Some(args[i].clone()); }
            arg          => target = Some(arg.to_string()),
        }
        i += 1;
    }

    if let Some(d) = dir {
        driver::cmd_migrate_dir(&d, in_place, dry_run, check_mode)?;
    } else if let Some(t) = target {
        driver::cmd_migrate(&t, in_place, dry_run, check_mode)?;
    } else {
        eprintln!("Usage: fav migrate [--in-place] [--dry-run] <file|--dir dir>");
    }
}
```

---

## Phase 4 — セルフホスト・マイルストーン

### 4-1. `String.length` の追加（必要な場合）

```rust
// src/backend/vm.rs — vm_call_builtin
"String.length" => {
    if args.len() != 1 { return Err(...); }
    match &args[0] {
        VMValue::Str(s) => Ok(VMValue::Int(s.len() as i64)),
        _ => Err(...),
    }
}
```

```rust
// src/middle/compiler.rs — 標準ビルトイン登録
"String" に "length" を追加
```

### 4-2. Favnir 製レキサー (`examples/selfhost/lexer.fav`)

```favnir
// examples/selfhost/lexer.fav
// Favnir サブセット・レキサー（v2.0.0 セルフホスト・マイルストーン）

type Token =
    | IntLit(Int)
    | Ident(String)
    | KwFn
    | KwStage
    | KwSeq
    | KwBind
    | KwIf
    | KwElse
    | Plus
    | Minus
    | Star
    | Slash
    | EqEq
    | BangEq
    | Eof

type LexResult = { token: Token  rest: String }

fn is_digit(c: String) -> Bool {
    bind code <- String.to_int(c)
    Option.is_some(code)
}

fn is_alpha(c: String) -> Bool {
    bind lower <- collect {
        yield "a"; yield "b"; yield "c"; yield "d"; yield "e";
        yield "f"; yield "g"; yield "h"; yield "i"; yield "j";
        yield "k"; yield "l"; yield "m"; yield "n"; yield "o";
        yield "p"; yield "q"; yield "r"; yield "s"; yield "t";
        yield "u"; yield "v"; yield "w"; yield "x"; yield "y"; yield "z";
    }
    List.any(lower, |ch| ch == c)
}

stage skip_spaces: String -> String = |s| {
    if String.length(s) == 0 {
        s
    } else {
        bind first <- String.char_at(s, 0)
        if first == " " {
            skip_spaces(String.slice(s, 1, String.length(s)))
        } else {
            s
        }
    }
}

stage next_token: String -> LexResult = |s| {
    bind s <- skip_spaces(s)
    if String.length(s) == 0 {
        LexResult { token: Eof  rest: "" }
    } else {
        bind first <- String.char_at(s, 0)
        bind rest  <- String.slice(s, 1, String.length(s))
        if first == "+" { LexResult { token: Plus   rest: rest } }
        else if first == "-" { LexResult { token: Minus  rest: rest } }
        else if first == "*" { LexResult { token: Star   rest: rest } }
        else if first == "/" { LexResult { token: Slash  rest: rest } }
        else { LexResult { token: Eof  rest: "" } }
    }
}

public fn main() -> Unit !Io {
    bind r <- next_token("+ rest")
    match r.token {
        Plus  => IO.println("OK: got Plus")
        other => IO.println("unexpected token")
    }
}
```

### 4-3. テストファイル (`examples/selfhost/lexer.test.fav`)

```favnir
// examples/selfhost/lexer.test.fav

test "next_token: Plus" {
    bind r <- next_token("+")
    assert_matches(r.token, Plus)
}

test "next_token: Minus" {
    bind r <- next_token("-")
    assert_matches(r.token, Minus)
}

test "next_token: Eof on empty" {
    bind r <- next_token("")
    assert_matches(r.token, Eof)
}

test "skip_spaces: strips leading spaces" {
    bind s <- skip_spaces("  abc")
    assert_eq(s, "abc")
}
```

---

## Phase 5 — テスト・ドキュメント

### 5-1. ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `Cargo.toml` | version = "2.0.0" |
| `src/main.rs` | HELP v2.0.0、migrate コマンド追加 |
| `src/frontend/parser.rs` | trf/flw/cap → エラー、expect_any 縮小 |
| `src/driver.rs` | cmd_migrate, cmd_migrate_dir, migrate_source 追加 |
| `src/backend/artifact.rs` | FVC_VERSION = 0x20 |
| `src/backend/vm.rs` | String.length 追加（必要な場合） |
| `src/middle/checker.rs` | 全 trf/flw/cap 使用テスト → stage/seq/interface |
| `src/frontend/parser.rs` (tests) | 同上 |
| `src/backend/vm_stdlib_tests.rs` | 同上 |
| `examples/abstract_flw_*.fav` | abstract seq に更新 |
| `examples/pipeline.fav` | stage/seq に更新 |
| `examples/selfhost/lexer.fav` | NEW: Favnir 製レキサー |
| `examples/selfhost/lexer.test.fav` | NEW: レキサーテスト |
| `versions/v2.0.0/langspec.md` | NEW: v2.0.0 言語仕様書 |
| `versions/v2.0.0/migration-guide.md` | NEW: v1.x → v2.0.0 移行ガイド |

### 5-2. テスト追加箇所

```rust
// src/frontend/parser.rs — parser tests 末尾
#[test]
fn trf_keyword_removed_e2001() {
    let result = Parser::parse_str("trf F: Int -> Int = |x| x", "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("trf"));
}

#[test]
fn flw_keyword_removed_e2002() {
    let result = Parser::parse_str("stage F: Int -> Int = |x| x\nflw P = F", "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("flw"));
}
```

```rust
// src/driver.rs — driver tests 末尾
#[test]
fn migrate_trf_to_stage() {
    let src = "trf double: Int -> Int = |x| x * 2\n";
    let result = migrate_source(src);
    assert!(result.contains("stage double"));
    assert!(!result.contains("trf double"));
}

#[test]
fn migrate_flw_to_seq() {
    let src = "stage F: Int -> Int = |x| x\nflw P = F\n";
    let result = migrate_source(src);
    assert!(result.contains("seq P"));
    assert!(!result.contains("flw P"));
}

#[test]
fn migrate_no_false_positive_in_identifiers() {
    let src = "bind trf_count <- 42\n";
    let result = migrate_source(src);
    // "trf_count" should NOT be changed to "stage_count"
    assert!(result.contains("trf_count"));
}

#[test]
fn migrate_abstract_trf() {
    let src = "abstract trf Processor: Int -> Int\n";
    let result = migrate_source(src);
    assert!(result.contains("abstract stage Processor"));
}

#[test]
fn migrate_public_trf() {
    let src = "public trf Processor: Int -> Int = |x| x\n";
    let result = migrate_source(src);
    assert!(result.contains("public stage Processor"));
}
```

---

## 実装上の注意事項

### `trf`/`flw` 識別子衝突の回避

`migrate_line` の置換対象は trailing space 付きの `"trf "` / `"flw "` であるため、
`trf_count` や `flw_result` のような識別子は変換されない。

ただし行末の `trf`（スペースなし）は変換されない — これは意図的。
`trf` が識別子の末尾に来るケース（`bind x <- get_trf` など）は変換不要なため。

### `cap` の特殊扱い

`cap` → `interface` の変換は構文的に1対1でないため自動変換は危険。
`cap Eq<T> = { equals: T -> T -> Bool }` は `interface Eq<T> { equals: Self -> Self -> Bool }` に
変換される必要があるが、構文が異なる。

移行ガイド (`migration-guide.md`) に手動変換の手順を詳述する。

### テストの一括更新方針

`grep -rn '"trf\|flw\|cap' src/` で全ての inline テストを特定し、
一つずつ確認しながら `stage`/`seq`/`interface` に書き換える。

自動化スクリプト（Bash）を使ってドライランし、確認後に適用する。
