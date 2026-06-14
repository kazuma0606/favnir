# v16.1.0 Spec — エラーメッセージ品質向上

Date: 2026-06-14
Branch: master

---

## テーマ

言語の第一印象を決めるエラーメッセージを `rustc` スタイルに刷新する。
現状は「何が」しか伝わらないが、「どこで」「なぜ」「どう直すか」まで伝えるようにする。

**現状:**
```
[E0001] undefined variable: user_id
```

**目標:**
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

---

## スコープ

### A: Cargo バージョン更新

```toml
version = "16.1.0"
```

### B: `Span` 構造体（lexer.rs / parser.rs）

トークンと主要 AST ノードに位置情報を付与する。

```rust
// fav/src/span.rs（新規）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub file:  u32,   // ソースファイル ID（ソーステーブルへのインデックス）
    pub line:  u32,   // 1-based 行番号
    pub col:   u32,   // 1-based 列番号（バイト位置）
    pub len:   u32,   // トークンの長さ（バイト数）
}
```

**付与対象 AST ノード（優先順位順）:**

| ノード | 理由 |
|---|---|
| `Expr::Var(name, span)` | E0001 未定義変数 — 最頻出エラー |
| `Expr::Call { fn_name, span, .. }` | E0007 未定義関数呼び出し |
| `Stmt::Bind { name, span, .. }` | E0018 再束縛禁止 |
| `FnDef { name, span, .. }` | 関数定義位置 |
| `TypeAnnotation { span, .. }` | E0009 型不一致 |

全 AST ノードへの完全な Span 伝播は v16.2.0 以降で段階的に進める。
v16.1.0 では上記 5 種類のノードに集中して確実に動作させる。

**Lexer 変更:**

```rust
// Token に Span を付与
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

// Lexer 内で行・列を追跡
struct Lexer {
    source: Vec<char>,
    pos:    usize,
    line:   u32,
    col:    u32,
    file_id: u32,
}
```

### C: `error.rs`（新規）— Diagnostic 型と表示エンジン

```rust
// fav/src/error.rs（新規）

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code:  String,            // "E0001"
    pub msg:   String,            // "undefined variable: user_id"
    pub span:  Span,              // ソース位置
    pub hints: Vec<String>,       // "= ヒント: ..." 行
    pub notes: Vec<String>,       // "= note: ..." 行
}

/// rustc スタイルの診断メッセージを生成する
pub fn format_diagnostic(
    diag:   &Diagnostic,
    source: &str,        // ソースファイル全文
    file:   &str,        // ファイル名（表示用）
    color:  bool,        // カラー出力フラグ
) -> String
```

**出力形式の仕様:**

```
[E0001] undefined variable: user_id          ← 1 行目: コード + メッセージ
 --> src/pipeline.fav:12:5                   ← 2 行目: ファイル:行:列
  |                                          ← 3 行目: 余白
12 |   transform(user_id, name)              ← 4 行目: ソース行（行番号付き）
  |             ^^^^^^^ この変数は未定義です  ← 5 行目: アンダーライン + ラベル
  |                                          ← 6 行目: 余白
  = ヒント: `userId` (line 8) の typo ではないですか？   ← ヒント行
  = 参照: https://favnir.dev/errors/E0001              ← URL 行
```

- `^` アンダーラインの長さは `span.len` で決定
- カラー時: コード・アンダーラインは赤、ヒントは青（ANSI エスケープ）
- `--no-color` 時: プレーンテキスト

### D: Levenshtein typo 候補（checker.rs）

```rust
/// name に近い候補を candidates から最大 max_results 件返す
/// 距離 ≤ threshold のもののみ返す
pub fn levenshtein_candidates(
    name:        &str,
    candidates:  &[&str],
    threshold:   usize,      // 2 固定
    max_results: usize,      // 3 固定
) -> Vec<String>
```

Levenshtein 距離の実装は `strsim` crate（新規依存）を使用。

**ヒント生成の対象:**

| エラーコード | 候補元 |
|---|---|
| E0001 未定義変数 | 現在スコープ内の変数名 |
| E0007 未定義関数 | 定義済み関数名 |
| E0009 型不一致 | 定義済み型名 |

### E: 各エラーコードへの hint / note 追加（checker.rs）

全エラーコード（E0001〜E0320）に最低 1 件の `hint` または `help` テキストを付与する。
優先実装対象（最頻出 10 件）：

| コード | メッセージ | hint |
|---|---|---|
| E0001 | undefined variable: X | typo 候補 or「`bind X <-` を使いましたか？」 |
| E0007 | undefined function: X | typo 候補 |
| E0008 | wrong argument count | 「期待: N 個、実際: M 個」を明示 |
| E0009 | type mismatch | 「期待: T、実際: U」+ typo 候補 |
| E0013 | where constraint failed | 制約式を表示 |
| E0018 | rebind not allowed | 「`bind` で同名変数への再代入は禁止。別名を使ってください」 |
| E0252 | unknown effect | 「`!Db` / `!IO` / `!AWS` 等が使用可能です」 |
| E0314 | missing !AWS effect | 「この関数に `!AWS` エフェクトを追加してください」 |
| E0319 | missing !Stream effect | 「この関数に `!Stream` エフェクトを追加してください」 |
| E0322 | Display not implemented | 「f-string には String/Int/Float/Bool のみ使用可能です」 |

### F: `driver.rs` 更新

- `fav check` / `fav run` のエラー出力を `format_diagnostic` 経由に統一
- `--no-color` フラグ追加:
  ```
  fav check --no-color src/pipeline.fav
  ```
  CI パイプラインで ANSI コードが邪魔になる場合に使用

### G: テスト（v161000_tests — 5 件）

1. `version_is_16_1_0`: Cargo.toml version == "16.1.0"
2. `error_output_has_line_number`: E0001 出力に ` --> ` が含まれる
3. `error_output_has_caret`: E0001 出力に `^` アンダーラインが含まれる
4. `error_output_has_hint`: E0001 出力に ヒントテキストが含まれる
5. `error_output_has_url`: E0001 出力に `favnir.dev/errors/` が含まれる

### H: サイトドキュメント（errors/ ディレクトリ）

`site/content/docs/errors/` に最頻出 20 エラーの詳細ページを新規作成:

```
errors/
├── index.mdx          # エラーコード一覧
├── E0001.mdx          # undefined variable
├── E0007.mdx          # undefined function
├── E0008.mdx          # wrong argument count
├── E0009.mdx          # type mismatch
├── E0013.mdx          # where constraint failed
├── E0018.mdx          # rebind not allowed
├── E0252.mdx          # unknown effect
├── E0314.mdx          # missing !AWS effect
├── E0319.mdx          # missing !Stream effect
└── ...（E0020 まで）
```

各ページの構成:
- エラーの原因
- よくある間違いパターン（コード例）
- 正しい修正方法（コード例）

---

## 完了条件

1. `cargo test v161000` → 5/5 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "16.1.0"`
4. `fav check` エラー出力に `-->` 行・`^` アンダーライン・hint・URL が含まれる
5. `fav check --no-color` でプレーンテキスト出力になる
6. Levenshtein ≤ 2 の typo 候補が E0001 / E0007 で提示される

---

## 新規 Cargo 依存

| Crate | バージョン | 用途 |
|---|---|---|
| `strsim` | `0.11` | Levenshtein 距離計算（typo 候補生成） |

`strsim` は軽量（`unsafe` なし、依存ゼロ）。WASM ターゲットでも動作する。

---

## 既知の制約・スコープ外

- Span の付与は優先 5 ノードのみ（全 AST ノードへの完全付与は v16.2.0 以降）
- 複数エラーの同時報告（現状は最初のエラーで停止）は v16.2.0 以降
- `self/checker.fav`（セルフホスト checker）への Diagnostic 統合は v16.2.0 以降
- IDE / LSP への Diagnostic 連携（既存の `fav/src/lsp.rs`）は v16.3.0 以降
- エラーコード URL（`https://favnir.dev/errors/Exxxx`）のリンク先は本 v16.1.0 で作成する

---

## 参照

- `versions/roadmap-v16.1-v17.0.md` — v16.1.0 セクション
- `fav/src/middle/checker.rs` — 全エラー生成箇所（Diagnostic に移行対象）
- `fav/src/driver.rs` — エラー表示箇所
- `fav/src/frontend/lexer.rs` — Token 定義（Span 追加対象）
- `fav/src/frontend/parser.rs` — AST 構築（Span 伝播対象）
- `fav/src/ast.rs` — AST ノード定義（Span フィールド追加対象）
