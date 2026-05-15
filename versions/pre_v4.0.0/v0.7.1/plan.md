# Favnir v0.7.1 実装計画

更新日: 2026-04-30

---

## フェーズ構成

```
Phase S-1: VS Code 拡張（TextMate grammar）← v0.7.1 の対象
Phase S-2: Tree-sitter grammar             ← 将来版
Phase S-3: Marketplace 公開               ← v1.0.0 以降
```

---

## Phase S-1: VS Code 拡張

### 目標

`.fav` ファイルを VS Code で開いたときに適切な色分けが行われること。
Node.js・vsce 不要。ファイルをコピーするだけでインストール完結。

### 成果物

```
editors/favnir-vscode/
├── package.json
├── syntaxes/
│   └── favnir.tmLanguage.json
└── language-configuration.json
```

### 実装ステップ

1. `editors/favnir-vscode/` ディレクトリ作成
2. `package.json` 作成（言語 ID `favnir`、拡張子 `.fav` の登録）
3. `language-configuration.json` 作成（括弧ペア・コメント設定）
4. `syntaxes/favnir.tmLanguage.json` 作成（以下の順番で追加）
   1. コメント（`//`）
   2. 文字列リテラル（エスケープシーケンス含む）
   3. 数値リテラル（整数・浮動小数点・アンダースコア区切り）
   4. 真偽値（`true` / `false`）
   5. エフェクト注釈（`!Io` `!Db` `!File` 等）
   6. 定義キーワード（`fn` / `trf` / `flw` / `type` / `cap` / `impl`）
   7. 制御・その他キーワード（`bind` / `chain` / `match` / `if` 等）
   8. 可視性修飾子（`pub` / `public`）
   9. 組み込み namespace（`IO` / `List` / `String` 等）
   10. 型名（`Int` / `Float` / `Bool` / `Unit`）
   11. 組み込みコンストラクタ（`some` / `none` / `ok` / `err`）
   12. 演算子（`|>` / `<-` / `->` / `|` 等）
   13. 定義名の強調（`fn NAME` / `trf NAME` / `type NAME`）
5. ローカルインストール・動作確認

### インストール手順（Windows）

```cmd
xcopy /E /I /Y editors\favnir-vscode "%USERPROFILE%\.vscode\extensions\favnir-language"
```

その後 VS Code を再起動または `Developer: Reload Window`。

---

## Phase S-2: Tree-sitter grammar（将来）

### 目標

より正確な構文認識（ネスト・エラー耐性）。Neovim / Helix / Zed / GitHub での利用。

### 実装ステップ

1. `editors/tree-sitter-favnir/` ディレクトリ作成
2. `grammar.js` に Favnir の文法を定義
3. `tree-sitter generate` でパーサを生成
4. VS Code 拡張の grammar を Tree-sitter に切り替え
5. Neovim / Helix 用の highlight クエリ（`.scm`）を追加

---

## Phase S-3: VS Code Marketplace 公開（v1.0.0 以降）

### 前提条件

- Favnir 言語仕様が v1.0.0 に到達
- Microsoft publisher アカウント取得済み

### 手順

```bash
cd editors/favnir-vscode
vsce publish
```
