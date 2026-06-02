# Favnir v9.8.0 Spec

Date: 2026-06-02
Theme: `fav doc` — `///` ドキュメントコメント + 型シグネチャ → Markdown 自動生成

---

## 概要

Favnir ソースファイルに `///` ドキュメントコメントを記述し、`fav doc` コマンドで Markdown ドキュメントを生成する機能を追加する。

```
fav doc <dir> --out <out_dir>
```

コンパイル不要。ソーステキストをパースして型シグネチャとドキュメントコメントを抽出し、Markdown を出力する。

---

## 構文

```favnir
/// Clamps a float to the range [lo, hi].
fn clamp(v: Float, lo: Float, hi: Float) -> Float { ... }

/// Represents a valid percentage in [0, 100].
type Percent(Float) where |v| v >= 0.0 && v <= 100.0

/// A data transformation stage.
stage normalize(Float) -> Float !IO { ... }
```

- `///` はアイテム直前の行に記述（1 行以上連続可能）
- `//` は通常コメント（ドキュメント生成対象外）
- `public` な `fn`/`type`/`stage`/`seq`/`type Wrapper(Inner)` が生成対象

---

## 生成 Markdown

各 `.fav` ファイルごとに 1 つの `.md` を生成。

```markdown
# mymodule

## Functions

### clamp

```
fn clamp(v: Float, lo: Float, hi: Float) -> Float
```

Clamps a float to the range [lo, hi].

---

## Types

### Percent

```
type Percent(Float) where |v| v >= 0.0 && v <= 100.0
```

Represents a valid percentage in [0, 100].
```

---

## CLI インターフェース

```
fav doc <dir>              # <dir>/**/*.fav をスキャンし、docs/ に Markdown を出力
fav doc <dir> --out <dir>  # 出力先を指定
fav doc <file>             # 単一ファイルをドキュメント化
```

---

## 実装方針

### compiler.fav 内の変更

1. **レキサー**: `scan_collect` で `///` を `TkDocComment(String)` として発行（`//` は従来どおりスキップ）
2. **AST 構造体**: `FnDef`・`TypeDef`・`StageDef`・`SeqDef`・`WrapperDef` に `doc: String` フィールド追加
3. **パーサー**: `parse_items` の各 item パース前に `TkDocComment` を `collect_doc` で収集
4. **ドキュメント生成**: `doc_program(prog: Program) -> String` 関数で Markdown テキストを生成
5. **Public 関数**: `public fn doc_source(src: String) -> Result<String, String>`

### Rust 側の変更

1. **`compiler_fav_runner.rs`**: `doc_source_str(src: &str)` — `fmt_source_str` と同パターン
2. **`vm.rs`**: `"Compiler.doc_source_raw"` primitive 追加
3. **`driver.rs`**: `cmd_doc` 追加（ディレクトリ走査 + ファイルごとに `doc_source_str` 呼び出し）
4. **`main.rs`**: `fav doc <dir> [--out <dir>]` サブコマンド追加

### cli.fav の変更

- `CmdDoc(String, String)` variant 追加（`path`, `out`）
- `parse_doc_cmd` 追加
- `run_doc` 追加（`Compiler.doc_source_raw` 経由）

---

## スコープ外（v9.9.0 以降）

- HTML 出力
- インデックスページ（`index.md`）自動生成
- リンク解決（他モジュールの型/関数への参照）
- `@param` / `@returns` タグ構文
- Rune ドキュメント自動生成（`rune info --doc`）
