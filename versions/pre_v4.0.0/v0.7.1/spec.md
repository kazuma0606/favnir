# Favnir v0.7.1 シンタックスハイライト仕様

更新日: 2026-04-30

> 対象: VS Code 拡張（TextMate grammar）
> ファイル拡張子: `.fav`
> 方式: JSON TextMate grammar（`favnir.tmLanguage.json`）

---

## 1. トークン定義

### 1-1. キーワード

| トークン | 種別 | TextMate スコープ |
|---|---|---|
| `fn` | 関数定義 | `keyword.declaration.fn.favnir` |
| `trf` | transform 定義 | `keyword.declaration.trf.favnir` |
| `flw` | flow 定義 | `keyword.declaration.flw.favnir` |
| `type` | 型定義 | `keyword.declaration.type.favnir` |
| `cap` | capability 定義 | `keyword.declaration.cap.favnir` |
| `impl` | capability 実装 | `keyword.declaration.impl.favnir` |
| `pub` `public` | 可視性修飾子 | `keyword.other.visibility.favnir` |
| `bind` | バインディング | `keyword.other.bind.favnir` |
| `chain` | chain バインディング | `keyword.other.chain.favnir` |
| `yield` | collect 内 yield | `keyword.other.yield.favnir` |
| `collect` | collect 式 | `keyword.other.collect.favnir` |
| `match` | パターンマッチ | `keyword.control.match.favnir` |
| `if` `else` | 条件分岐 | `keyword.control.conditional.favnir` |
| `where` | パターンガード | `keyword.control.where.favnir` |
| `namespace` | 名前空間宣言 | `keyword.other.namespace.favnir` |
| `use` | インポート | `keyword.other.use.favnir` |
| `for` | (予約) | `keyword.control.loop.favnir` |
| `emit` | イベント発行 | `keyword.other.emit.favnir` |

### 1-2. 可視性修飾子

`pub` / `public` を強調するための独立ルール。

スコープ: `keyword.other.visibility.favnir`

### 1-3. エフェクト注釈

`!Io` `!Db` `!File` `!Network` `!Trace` `!Emit` を着色する。
パターン: `![A-Z][A-Za-z]*(<[^>]*>)?`

スコープ: `storage.modifier.effect.favnir`

### 1-4. 組み込み namespace

関数呼び出しの形式 `Namespace.method(...)` で使われる名前空間。

| トークン | TextMate スコープ |
|---|---|
| `IO` `List` `String` `Map` `Option` `Result` | `support.class.builtin.favnir` |
| `File` `Json` `Csv` `Db` `Http` `Debug` `Trace` `Emit` | `support.class.builtin.favnir` |

パターン: `\b(IO|List|String|Map|Option|Result|File|Json|Csv|Db|Http|Debug|Trace|Emit)\b`

### 1-5. 型名

| トークン | TextMate スコープ |
|---|---|
| `Int` `Float` `Bool` `Unit` | `support.type.primitive.favnir` |
| `String` `List` `Map` | `support.type.primitive.favnir` |

型注釈コンテキスト（`: T`、`-> T`）の他、単独でも着色する。

### 1-6. 組み込みコンストラクタ

| トークン | 種別 | TextMate スコープ |
|---|---|---|
| `some` `none` | Option コンストラクタ | `support.function.constructor.favnir` |
| `ok` `err` | Result コンストラクタ | `support.function.constructor.favnir` |
| `true` `false` | 真偽値リテラル | `constant.language.boolean.favnir` |

### 1-7. リテラル

| 種別 | パターン例 | TextMate スコープ |
|---|---|---|
| 整数 | `42` `1_000` | `constant.numeric.integer.favnir` |
| 浮動小数点 | `3.14` `1.0e-3` | `constant.numeric.float.favnir` |
| 文字列 | `"hello"` | `string.quoted.double.favnir` |

文字列内のエスケープ（`\n` `\t` `\"` `\\`）:
スコープ: `constant.character.escape.favnir`

### 1-8. 演算子・記号

| 種別 | トークン | TextMate スコープ |
|---|---|---|
| パイプライン | `\|>` | `keyword.operator.pipeline.favnir` |
| バインド矢印 | `<-` | `keyword.operator.bind.favnir` |
| 戻り値型 | `->` | `keyword.operator.arrow.favnir` |
| ADT 区切り | `\|` (単独) | `keyword.operator.pipe.favnir` |
| 算術 | `+` `-` `*` `/` `%` | `keyword.operator.arithmetic.favnir` |
| 比較 | `==` `!=` `<` `>` `<=` `>=` | `keyword.operator.comparison.favnir` |
| 型修飾 `?` | `T?` の `?` | `keyword.operator.optional.favnir` |
| 型修飾 `!` | `T!` の `!` | `keyword.operator.fallible.favnir` |

### 1-9. コメント

```
// これはコメント
```

スコープ: `comment.line.double-slash.favnir`

### 1-10. 定義名の強調

| 構文 | 強調対象 | TextMate スコープ |
|---|---|---|
| `fn name(...)` | `name` | `entity.name.function.favnir` |
| `trf name:` | `name` | `entity.name.function.trf.favnir` |
| `flw name:` | `name` | `entity.name.function.flw.favnir` |
| `type name` | `name` | `entity.name.type.favnir` |
| `cap name<T>` | `name` | `entity.name.type.cap.favnir` |

---

## 2. ファイル構成

```
editors/favnir-vscode/
├── package.json                      ← VS Code 拡張マニフェスト
├── syntaxes/
│   └── favnir.tmLanguage.json        ← TextMate grammar 本体
└── language-configuration.json       ← 括弧ペア・コメント設定
```

### package.json（抜粋）

```json
{
  "name": "favnir-language",
  "contributes": {
    "languages": [{
      "id": "favnir",
      "aliases": ["Favnir", "fav"],
      "extensions": [".fav"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "favnir",
      "scopeName": "source.favnir",
      "path": "./syntaxes/favnir.tmLanguage.json"
    }]
  }
}
```

### language-configuration.json

```json
{
  "comments": { "lineComment": "//" },
  "brackets": [["{","}"],["[","]"],["(",")"]]
}
```

---

## 3. インストール方法（開発中）

Node.js・vsce 不要。2ファイルを置くだけで完結。

```bash
# Windows
xcopy /E /I editors\favnir-vscode "%USERPROFILE%\.vscode\extensions\favnir-language"

# macOS / Linux
cp -r editors/favnir-vscode ~/.vscode/extensions/favnir-language
```

VS Code を再起動（または `Ctrl+Shift+P` → `Developer: Reload Window`）して動作確認。

---

## 4. 将来拡張

| 優先度 | 内容 |
|---|---|
| ★★☆ | Tree-sitter grammar（より高精度なハイライト） |
| ★★☆ | Zed エディタ対応 |
| ★☆☆ | GitHub Linguist 登録（`.fav` をリポジトリで自動認識） |
| ★☆☆ | Neovim / Helix 対応（Tree-sitter 経由） |

---

## 5. 公開方針

| フェーズ | 配布方法 |
|---|---|
| 開発中（現在） | `editors/favnir-vscode/` を直接コピーしてローカルインストール |
| 言語仕様安定後（v1.0.0） | VS Code Marketplace に公開（`vsce publish`） |
