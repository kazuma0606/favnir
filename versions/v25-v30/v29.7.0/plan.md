# v29.7.0 Plan — VS Code 拡張 公式リリース

**バージョン**: 29.7.0
**日付**: 2026-06-30
**前バージョン**: v29.6.0 (pagerduty Rune 追加)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.7.0"
```

### T2: extensions/vscode-favnir/package.json 作成

```json
{
  "name": "vscode-favnir",
  "displayName": "Favnir",
  "description": "Favnir language support for VS Code — syntax highlighting, type inference, LSP",
  "version": "1.0.0",
  "publisher": "favnir",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:favnir"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [
      {
        "id": "favnir",
        "aliases": ["Favnir", "fav"],
        "extensions": [".fav"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "favnir",
        "scopeName": "source.favnir",
        "path": "./syntaxes/favnir.tmLanguage.json"
      }
    ],
    "taskDefinitions": [
      {
        "type": "favnir",
        "properties": {
          "command": { "type": "string", "description": "fav command (run/check/test)" }
        }
      }
    ]
  },
  "dependencies": {
    "vscode-languageclient": "^9.0.1"
  },
  "devDependencies": {
    "@types/vscode": "^1.85.0",
    "typescript": "^5.3.3",
    "@vscode/vsce": "^2.22.0"
  },
  "scripts": {
    "compile": "tsc -p ./",
    "package": "vsce package"
  }
}
```

### T3: extensions/vscode-favnir/language-configuration.json 作成

```json
{
  "comments": {
    "lineComment": "//"
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ["<", ">"]
  ],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "<", "close": ">" },
    { "open": "\"", "close": "\"" }
  ],
  "surroundingPairs": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ["\"", "\""]
  ]
}
```

### T4: extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json 作成

TextMate grammar で以下のスコープを定義する:

- `keyword.control.favnir` — `fn` / `stage` / `seq` / `type` / `interface` / `impl` / `match` / `if` / `else` / `let` / `bind` / `par` / `import`
- `keyword.operator.favnir` — `|>` / `++` / `->` / `<-` / `??` / `?`
- `storage.type.favnir` — `String` / `Int` / `Float` / `Bool` / `Unit` / `List` / `Result` / `Option`
- `string.quoted.double.favnir` — ダブルクォート文字列
- `constant.numeric.favnir` — 数値リテラル
- `constant.language.favnir` — `true` / `false` / `unit`
- `comment.line.double-slash.favnir` — `//` コメント
- `entity.name.function.favnir` — 関数名
- `support.type.favnir` — エフェクト（`!Http` / `!Db` / `!Io` / `!Llm` 等）

### T5: extensions/vscode-favnir/src/extension.ts 作成

LSP クライアントを起動し、`fav` バイナリの `--lsp` フラグで LSP サーバーとして接続する。

```typescript
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const serverCommand = vscode.workspace.getConfiguration('favnir').get<string>('serverPath', 'fav');

  const serverOptions: ServerOptions = {
    run: { command: serverCommand, args: ['lsp'], transport: TransportKind.stdio },
    debug: { command: serverCommand, args: ['lsp', '--port', '6009'], transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'favnir' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.fav'),
    },
  };

  client = new LanguageClient('favnir', 'Favnir Language Server', serverOptions, clientOptions);
  client.start();

  // fav run / fav check コマンドを Task Runner から実行できるよう登録
  context.subscriptions.push(
    vscode.commands.registerCommand('favnir.run', () => {
      const terminal = vscode.window.createTerminal('fav run');
      terminal.sendText('fav run');
      terminal.show();
    }),
    vscode.commands.registerCommand('favnir.check', () => {
      const terminal = vscode.window.createTerminal('fav check');
      terminal.sendText('fav check');
      terminal.show();
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  return client.stop();
}
```

### T6: CHANGELOG.md に [v29.7.0] セクション追加

```markdown
## [v29.7.0] — 2026-06-30

### Added
- `extensions/vscode-favnir/` — VS Code 拡張パッケージ（TextMate grammar / LSP クライアント / Task Runner 統合）
- `site/content/docs/tools/vscode-extension.mdx` — VS Code 拡張ドキュメント
- テスト数: 2348 → 2354（+6）
```

### T7: benchmarks/v29.7.0.json 作成

```json
{
  "version": "29.7.0",
  "date": "2026-06-30",
  "milestone": "Ecosystem Maturity (phase 7)",
  "test_count": 2354,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T8: site/content/docs/tools/vscode-extension.mdx 作成

VS Code 拡張のインストール・設定・機能説明を含むドキュメント。

### T9: driver.rs に v297000_tests 6 件追加

```rust
// v297000_tests (v29.7.0) -- VS Code 拡張
#[cfg(test)]
mod v297000_tests {
    #[test]
    fn vscode_package_json_exists() {
        let src = include_str!("../../extensions/vscode-favnir/package.json");
        assert!(
            src.contains("vscode-favnir"),
            "extensions/vscode-favnir/package.json must contain 'vscode-favnir'"
        );
    }
    #[test]
    fn vscode_grammar_exists() {
        let src = include_str!("../../extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json");
        assert!(
            src.contains("source.favnir"),
            "syntaxes/favnir.tmLanguage.json must contain 'source.favnir'"
        );
    }
    #[test]
    fn vscode_language_config_exists() {
        let src = include_str!("../../extensions/vscode-favnir/language-configuration.json");
        assert!(
            src.contains("lineComment"),
            "language-configuration.json must contain 'lineComment'"
        );
    }
    #[test]
    fn vscode_extension_src_exists() {
        let src = include_str!("../../extensions/vscode-favnir/src/extension.ts");
        assert!(
            src.contains("LanguageClient"),
            "extension.ts must contain 'LanguageClient'"
        );
    }
    #[test]
    fn vscode_extension_mdx_exists() {
        let src = include_str!("../../site/content/docs/tools/vscode-extension.mdx");
        assert!(
            src.contains("vscode-favnir"),
            "vscode-extension.mdx must contain 'vscode-favnir'"
        );
    }
    #[test]
    fn changelog_has_v29_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.7.0]") || src.contains("## v29.7.0"),
            "CHANGELOG.md must contain '[v29.7.0]'"
        );
    }
}
```

### T10: cargo test --bin fav v297000 — 6/6 PASS 確認

### T11: cargo test --bin fav — 2354 tests PASS 確認

### T12: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.6.0 | 2348 |
| v29.7.0 | **2354** (+6) |
