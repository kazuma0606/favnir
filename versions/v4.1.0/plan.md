# Favnir v4.1.0 Implementation Plan

## Theme: Rune マルチファイル対応 — ディレクトリ単位の rune モジュール

---

## Phase 0: バージョン更新

`fav/Cargo.toml` の version を `"4.1.0"` に更新。
`fav/src/main.rs` のヘルプテキスト・バージョン文字列を更新。
追加 Cargo 依存なし。

---

## Phase 1: `UseDecl` — AST / Lexer / Parser

### ast.rs

`UseDecl` ノードと `UseNames` enum を追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseNames {
    Specific(Vec<String>),  // use X.{ a, b }
    Wildcard,               // use X.*
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub module: String,
    pub names:  UseNames,
    pub span:   Span,
}
```

`Decl` enum に `Use(UseDecl)` バリアントを追加。

### lexer.rs

`use` を予約キーワードとして追加（`Token::Use`）。
`*` はすでに `Token::Star` として存在するか確認し、なければ追加。

### parser.rs

`parse_decl` で `Token::Use` を処理する分岐を追加:

```rust
Token::Use => {
    self.advance(); // consume "use"
    let module = self.expect_ident()?;          // "connection"
    self.expect_token(Token::Dot)?;             // "."
    let names = if self.peek_token(Token::Star) {
        self.advance();
        UseNames::Wildcard
    } else {
        self.expect_token(Token::LBrace)?;
        let mut names = vec![];
        loop {
            names.push(self.expect_ident()?);
            if self.eat_token(Token::Comma) { continue; }
            break;
        }
        self.expect_token(Token::RBrace)?;
        UseNames::Specific(names)
    };
    Decl::Use(UseDecl { module, names, span })
}
```

### fmt.rs

`UseDecl` の pretty-print を追加:

```rust
Decl::Use(u) => {
    let names = match &u.names {
        UseNames::Specific(ns) => format!("{{ {} }}", ns.join(", ")),
        UseNames::Wildcard    => "*".to_string(),
    };
    writeln!(f, "use {}.{}", u.module, names)?;
}
```

---

## Phase 2: ディレクトリ rune ロード（driver.rs）

### 解決ロジックの変更

`resolve_rune_path(rune_name, project_root)` 関数（既存または新規）に
ディレクトリ優先ロジックを追加:

```rust
fn resolve_rune_path(name: &str, project_root: &Path) -> Option<RuneSource> {
    let dir_path = project_root.join("runes").join(name);
    let entry    = dir_path.join(format!("{name}.fav"));

    if dir_path.is_dir() && entry.exists() {
        return Some(RuneSource::Directory { dir: dir_path, entrypoint: entry });
    }

    let file_path = project_root.join("runes").join(format!("{name}.fav"));
    if file_path.exists() {
        return Some(RuneSource::SingleFile(file_path));
    }

    None
}

enum RuneSource {
    SingleFile(PathBuf),
    Directory { dir: PathBuf, entrypoint: PathBuf },
}
```

### マルチファイル rune のロード

`load_rune_directory(dir, entrypoint)` を新規実装:

```rust
fn load_rune_directory(
    dir: &Path,
    entrypoint: &Path,
    visited: &mut HashSet<PathBuf>,  // 循環参照検出
) -> Result<Vec<ParsedFavFile>, String> {
    // 1. entrypoint をパース
    let entry_src  = fs::read_to_string(entrypoint)?;
    let entry_prog = parse_source(&entry_src)?;

    // 2. UseDecl を収集
    let use_decls: Vec<&UseDecl> = entry_prog.decls.iter()
        .filter_map(|d| if let Decl::Use(u) = d { Some(u) } else { None })
        .collect();

    // 3. 各 UseDecl に対して対応する .fav をロード（再帰）
    let mut all_files = vec![ParsedFavFile {
        path: entrypoint.to_path_buf(),
        prog: entry_prog,
        is_entrypoint: true,
    }];

    for u in use_decls {
        let mod_path = dir.join(format!("{}.fav", u.module));
        if !mod_path.exists() {
            return Err(format!("E04x1: module '{}' not found in rune directory", u.module));
        }
        if visited.contains(&mod_path) {
            return Err(format!("E04x3: circular use: {}", u.module));
        }
        visited.insert(mod_path.clone());
        let mod_src  = fs::read_to_string(&mod_path)?;
        let mod_prog = parse_source(&mod_src)?;
        all_files.push(ParsedFavFile { path: mod_path, prog: mod_prog, is_entrypoint: false });
    }

    Ok(all_files)
}
```

### 名前解決・マージ戦略

ディレクトリ rune をロードした後:

1. **型定義・関数定義のマージ**: 全ファイルの `type` / `fn` 宣言を 1 つのスコープに集める。
2. **公開制御**: `public` は エントリポイント（`db.fav`）のみ有効。
   内部モジュール（`connection.fav`）の `public` は「rune 内部での参照を許可する」という意味であり、
   外部 API（`import "db"` からアクセス可能な API）にはならない。
3. **`use X.{ a, b }` の検証**: エントリポイントの型チェック時に `a`, `b` が
   `X.fav` のスコープに存在することを確認。

**実装ショートカット（v4.1.0）**:

マージの完全実装は複雑なため、v4.1.0 では以下の単純化アプローチを採用:

- 全ファイル（エントリポイント + 内部モジュール）の AST を **結合**して 1 つのプログラムとして扱う。
- `UseDecl` はロード対象の宣言として機能し、型チェック・コンパイル時はスキップ（no-op）。
- 公開 API はエントリポイントの `public fn` のみ。内部モジュールの `public fn` は
  チェッカーが「rune 内部公開」として区別できるよう `is_internal: bool` フラグをつける。
  v4.1.0 ではこのフラグを単純に無視し、全ての `public fn` を rune API とする
  （v4.2.0 以降で厳格化）。

---

## Phase 3: Checker / Compiler 対応

### checker.rs

`UseDecl` が rune ファイル外（通常プロジェクトファイル）に存在する場合はエラー:

```rust
Decl::Use(u) => {
    if !ctx.is_rune_file {
        return Err(type_error("E04x0", "use is only allowed inside rune files", u.span));
    }
    // Phase 2 のマージ戦略により、ロード時に解決済み → ここでは検証のみ
    self.validate_use_names(u, ctx)?;
    Ok(())
}
```

`validate_use_names`: `u.names` に含まれる名前が内部スコープに存在するか確認。

### compiler.rs

`UseDecl` はコンパイル対象なし（no-op）:

```rust
Decl::Use(_) => { /* no-op — resolved at load time */ }
```

---

## Phase 4: rune 内部からの rune 間インポート

rune ファイル（`connection.fav` 等）に `import "json"` が出現した場合の処理:

`load_rune_directory` の内部ファイルロード時に `import` 文も再帰的に解決する。
解決ロジックは通常ファイルの `import` と同じ（ベア名 → rune 自動検出）。
ただし循環 import チェックをディレクトリ rune のロードパイプラインに組み込む。

Phase 2 の実装ショートカット（AST 結合）のもとでは、内部ファイルの `import "json"` も
外部ファイルの `import "json"` と同様に扱われる（既存ロジックで動く）。

---

## Phase 5: テスト

### parser テスト（frontend/parser.rs の `#[cfg(test)]` ブロック）

- `parse_use_specific` — `use connection.{ connect, close }` が `UseDecl` になる
- `parse_use_wildcard` — `use query.*` が `UseNames::Wildcard` になる
- `parse_use_empty_braces_error` — `use connection.{}` がパースエラーになる

### driver 統合テスト（driver.rs）

- `rune_directory_load_basic` — `runes/db/` + `db.fav` + `connection.fav` を用意して `import "db"` が通る
- `rune_directory_entry_missing_error` — エントリポイントなしはエラー
- `rune_directory_use_missing_module_error` — 存在しない `use X.{...}` はエラー
- `rune_directory_circular_use_error` — 循環 `use` はエラー
- `rune_single_file_backward_compat` — 既存単一ファイル rune が引き続き動作する
- `rune_internal_fn_callable` — 内部モジュールの関数を rune の public fn から呼べる
- `rune_dir_import_other_rune` — rune 内部から `import "json"` が通る

### リグレッションテスト

- 全既存テスト（788 件）がパスすること

---

## Phase 6: examples + docs

- `runes/db/` をマルチファイル化（`db.fav` + `connection.fav` + `query.fav`）
  — v4.2.0 の DB Rune 2.0 の前哨戦として最小構成で分割
- `fav/examples/rune_multifile_demo/src/main.fav` — マルチファイル rune を使うデモ
- `versions/v4.1.0/spec.md` 作成済み
- `versions/v4.1.0/progress.md` 全フェーズ完了時に更新
- `memory/MEMORY.md` を v4.1.0 完了状態に更新

---

## 実装順序と依存関係

```
Phase 0: バージョン更新（独立）
Phase 1: UseDecl（ast/lexer/parser/fmt）— 独立。先に進めると後続が書きやすい
Phase 2: ディレクトリ rune ロード（driver.rs）— Phase 1 完了後
Phase 3: Checker / Compiler 対応 — Phase 1, 2 完了後
Phase 4: rune 間インポート — Phase 2, 3 完了後（ほぼ自動で動く可能性高い）
Phase 5: テスト — Phase 1〜4 完了後
Phase 6: docs — 最後
```

---

## 実装上の注意点

### AST 結合の限界

v4.1.0 の「全ファイル AST 結合」アプローチでは、
内部モジュールの関数がエントリポイントと **同一フラット名前空間** に置かれる。
つまり `connection.connect` ではなく単に `connect` として参照する。

これは v4.2.0 で名前空間スコープを導入する際に改善する。
v4.1.0 ではシンプルな動作を優先し、内部ファイル間で名前衝突が起きないよう
rune 作者が責任を持つ（コーディング規約で対応）。

### is_rune_file コンテキストの伝搬

`UseDecl` を rune ファイル内のみ許可するため、
`checker.rs` に `is_rune_file: bool` フラグをコンテキストとして渡す必要がある。
ロード時にファイルパスが `runes/` 配下かどうかで自動判定するのが最もシンプル。

### 循環参照の検出

`use` の循環はファイルレベルで検出（`visited: HashSet<PathBuf>`）。
DFS でロードし、訪問済みパスに再訪問した場合に E04x3 を返す。
