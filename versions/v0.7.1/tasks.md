# Favnir v0.7.1 タスク一覧

更新日: 2026-04-30

> [ ] 未完了 / [x] 完了
>
> **ゴール**: `~/.vscode/extensions/favnir-language/` に配置してローカルでハイライトが動くこと
> **方針**: Node.js・vsce・Marketplace 不要。3ファイルを置くだけで完結させる

---

## S-1-A: ファイル雛形

- [x] `editors/favnir-vscode/` ディレクトリ作成
- [x] `editors/favnir-vscode/package.json` 作成
  - [x] 言語 ID `favnir`、拡張子 `.fav` の登録
  - [x] grammar パス・scopeName の設定
- [x] `editors/favnir-vscode/language-configuration.json` 作成
  - [x] 括弧ペア `{}` `[]` `()`
  - [x] 行コメント `//`
  - [x] auto-closing pairs（`"`, `{`, `[`, `(`）

## S-1-B: TextMate grammar 本体

- [x] `editors/favnir-vscode/syntaxes/favnir.tmLanguage.json` 作成
- [x] コメント（`//`）
- [x] 文字列リテラル（エスケープ `\n` `\t` `\"` `\\` 含む）
- [x] 数値リテラル（整数 `42` / アンダースコア区切り `1_000` / 浮動小数点 `3.14`）
- [x] 真偽値（`true` / `false`）
- [x] エフェクト注釈（`!Io` `!Db` `!File` `!Network` `!Trace` `!Emit`）
- [x] 定義キーワード（`fn` / `trf` / `flw` / `type` / `cap` / `impl`）
- [x] 制御・その他キーワード（`bind` / `chain` / `yield` / `collect` / `match` / `if` / `else` / `where` / `namespace` / `use` / `emit` / `for`）
- [x] 可視性修飾子（`pub` / `public`）
- [x] 組み込み namespace（`IO` / `List` / `String` / `Map` / `Option` / `Result` / `File` / `Json` / `Csv` / `Db` / `Http` / `Debug` / `Trace` / `Emit`）
- [x] 型名（`Int` / `Float` / `Bool` / `Unit`）
- [x] 組み込みコンストラクタ（`some` / `none` / `ok` / `err`）
- [x] 演算子（`|>` / `<-` / `->` / `|` / 算術 / 比較）
- [x] 定義名の強調（`fn NAME` / `trf NAME:` / `flw NAME:` / `type NAME` / `cap NAME`）

## S-1-C: ローカル配置・動作確認

- [ ] `editors/favnir-vscode/` を `~/.vscode/extensions/favnir-language/` にコピー
- [ ] VS Code を再起動または `Developer: Reload Window`
- [ ] `examples/hello.fav` を開いて目視確認
  - [ ] キーワードが着色されている
  - [ ] `//` コメントが着色されている
  - [ ] 型注釈（`: Int`、`-> String`）が着色されている
  - [ ] エフェクト（`!Io`、`!File`）が着色されている
  - [ ] 組み込み namespace（`IO.println`）が着色されている

---

## オプション（将来版）

- [x] **S-2**: Tree-sitter grammar（`editors/tree-sitter-favnir/grammar.js` + highlights.scm）
- [x] **S-3**: `.vsix` パッケージング（`vsce`）
- [x] **S-4**: VS Code Marketplace 公開
- [x] **S-5**: GitHub Linguist 登録（`.fav` をリポジトリで自動認識）
