# v60.7.0 Spec — `fav fmt` ルール拡張（コメント保持・行長制限・`.favfmt` 設定）

Date: 2026-07-31
Status: 未着手

---

## 概要

既存の `fav fmt`（v9.2、`fmt.rs`）に以下の 3 機能を追加する。

1. **コメント保持**: `//` 行コメントを `fmt` 後も正しく残す
2. **行長制限**: デフォルト 100 文字。`FmtConfig` に保持（実際の折り返しはスコープ外）
3. **`.favfmt` 設定ファイル**: プロジェクトルートの `.favfmt` TOML を読み込んでフォーマット動作を制御

---

## 動機

現行の `format_program(prog: &Program) -> String` は AST ベース再生成のため、
パーサーが読み飛ばした `//` コメントが消えてしまう。
また、チームのコーディング規約（行長・インデント幅）を `.favfmt` で統一できるようにする。

---

## `.favfmt` ファイル仕様

```toml
# .favfmt — フォーマット設定ファイル（TOML 形式）
max_line_length = 100
indent_width = 2
preserve_comments = true
trailing_comma = "always"
```

フィールド:

| キー | 型 | デフォルト | 説明 |
|---|---|---|---|
| `max_line_length` | usize | 100 | 行長制限（超過行は将来バージョンで折り返し） |
| `indent_width` | usize | 4 | インデント幅（スペース数） |
| `preserve_comments` | bool | true | `//` コメント行を保持するか |
| `trailing_comma` | String | `"always"` | 末尾カンマポリシー（将来利用） |

---

## スコープ整理（ロードマップとの差分）

ロードマップには以下 3 項目が記載されているが、本バージョンでは部分的に延期とする。

| ロードマップ項目 | 本バージョンの扱い |
|---|---|
| `//` 行コメントの保持 | **実装する** |
| インラインコメント（行末 `// ...`）の保持 | スコープ外（v60.8 以降に延期）。パーサーが行末コメントを AST に含まないため、正確な位置復元には追加 lexer 変更が必要 |
| 行長制限を超える式の自動折り返し | スコープ外（v60.8 以降に延期）。式の途中での折り返しは AST ノードごとの幅計算が必要で本スプリントのスコープを超える。`max_line_length` は設定として保持するのみ |
| `.favfmt` を `toml.rs` でパース | `toml.rs` の汎用パーサーは使わず `FmtConfig::from_toml_str` としてシンプルなキー=値パーサーを `fmt.rs` 内に実装する。理由: `.favfmt` は 4 フィールドのみで汎用 TOML パーサー依存は過剰 |

---

## 実装方針

### `fmt.rs` への追加

#### `FmtConfig` 構造体

```rust
/// v60.7.0: フォーマット設定（.favfmt ファイルから読み込む）
pub struct FmtConfig {
    pub max_line_length: usize,
    pub indent_width: usize,
    pub preserve_comments: bool,
    pub trailing_comma: String,
}

impl Default for FmtConfig { ... }   // デフォルト値は上表のとおり
impl FmtConfig {
    pub fn from_toml_str(s: &str) -> Self { ... }  // key = value パース
}
```

**注意**: `indent_width` フィールドは設定値として `FmtConfig` に保持するが、
`format_program` 内部の `Formatter` は現時点で 4 スペース固定（`"    ".repeat(self.indent)`）のため、
`indent_width` の値はフォーマット出力に反映されない。反映は v60.8 以降に延期。

#### `format_with_config` 関数（公開 API）

```rust
pub fn format_with_config(prog: &Program, source: &str, config: &FmtConfig) -> String
```

- `format_program(prog)` を呼び出して基本フォーマットを得る
- `config.preserve_comments` が `true` の場合、`reinsert_comments(source, &formatted)` でコメントを復元
- それ以外は `formatted` をそのまま返す

#### `reinsert_comments` 関数（非公開ヘルパー）

```rust
fn reinsert_comments(original: &str, formatted: &str) -> String
```

アルゴリズム:
1. オリジナルソースを行ごとにスキャン
2. `//` で始まるコメント行を `pending_comments` に蓄積
3. 非コメント行に到達したら `{trimmed_line → comments_block}` として `comment_map` に記録
4. フォーマット済みソースを行ごとに走査し、`comment_map` に anchor として登録されている行の直前にコメントブロックを挿入
5. 同一 anchor が複数回出現しても最初の 1 回のみ挿入（`inserted` HashSet で管理）

**既知の制限**（スコープ内で許容）:
- 同一内容の行が複数ある場合、最初の行にしかコメントが挿入されない
- インラインコメント（行末 `// ...`）は対象外（スコープ外）
- 文字列リテラル中の `//` は誤検出の可能性がある（スコープ外）

### `driver.rs` の `cmd_fmt` 更新

`format_program(&program)` を呼んでいる箇所を
`format_with_config(&program, &source, &config)` に置き換える。

`config` は以下の優先順位で決定:
1. CWD またはプロジェクトルートの `.favfmt` ファイルが存在すれば `FmtConfig::from_toml_str` でパース
2. 存在しなければ `FmtConfig::default()`

---

## テスト仕様

### `fmt_preserves_comments`

```
ソース:
  // pipeline comment
  stage Foo: Int -> Int = |x| { x + 1 }

config: FmtConfig { preserve_comments: true, ..Default::default() }

期待: format_with_config の出力に "// pipeline comment" が含まれる
      かつ "stage Foo" も含まれる
```

### `fmt_respects_favfmt_config`

このテストは「設定値のパース」を検証する（出力への反映はスコープ外）。
テスト名は「config がパースされること」の意味で使用する。

```
.favfmt 文字列:
  max_line_length = 80
  indent_width = 2
  preserve_comments = true

期待: FmtConfig::from_toml_str 後
  config.max_line_length == 80
  config.indent_width == 2
  config.preserve_comments == true
```

---

## 完了条件

- `fmt_preserves_comments` pass
- `fmt_respects_favfmt_config` pass
- 総テスト数: **3344** tests passed, 0 failed（ベース 3342 + 2）
- `cargo build` でコンパイルエラーなし
