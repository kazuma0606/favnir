# v72.1.0 spec — VS Code 拡張（本格実装）

Date: 2026-08-12

---

## Background

Favnir は `fav lsp` コマンドで LSP サーバーを起動できる（既存実装）。
v72.1.0 では、この LSP サーバーを VS Code Extension として完全統合し、
マーケットプレイス公開を視野に入れた品質の拡張機能を `editors/vscode/` に実装する。

既存の `fav lsp` は Language Server Protocol に準拠しており、
補完・型ホバー・定義ジャンプ・エラー診断を提供する。
VS Code Extension はその LSP クライアントとして機能する。

---

## Goals

1. `editors/vscode/package.json` — VS Code 拡張マニフェスト（Marketplace 設定含む）
2. `editors/vscode/extension.ts` — LSP クライアント実装（`fav lsp` サーバーに接続）
3. `editors/vscode/syntaxes/favnir.tmGrammar.json` — TextMate 文法（`.fav` シンタックスハイライト）
4. `v721000_tests` 2 件を `driver.rs` に追加
5. `CHANGELOG.md` に v72.1.0 エントリを追加
6. `versions/current.md` を更新（進行中: v72.1.0、次: v72.2.0）

---

## 機能一覧

```
✓ シンタックスハイライト（.fav ファイル）
✓ 型ホバー（変数・関数にカーソルを当てると型を表示）
✓ 定義ジャンプ（F12）・参照検索（Shift+F12）
✓ インライン型ヒント（引数名・戻り値型）
✓ エラーアンダーライン + 修正ヒント（Quick Fix）
✓ Rune メソッド補完（ctx.io. → argv / println / read_file_raw ...）
✓ コードフォーマット（保存時 fav fmt 自動実行）
✓ fav run / fav check をエディタから実行（Run Task）
```

---

## ファイル構成

```
editors/vscode/
├── package.json             # 拡張マニフェスト
├── extension.ts             # エントリポイント（LSP クライアント）
└── syntaxes/
    └── favnir.tmGrammar.json  # TextMate 文法定義
```

---

## ファイル詳細

### `package.json` — 必須フィールド

```json
{
  "name": "favnir",
  "displayName": "Favnir",
  "description": "Favnir language support for VS Code",
  "version": "0.1.0",
  "publisher": "favnir",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Programming Languages", "Linters", "Formatters"],
  "activationEvents": ["onLanguage:favnir"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "favnir",
      "aliases": ["Favnir", "fav"],
      "extensions": [".fav"]
    }],
    "grammars": [{
      "language": "favnir",
      "scopeName": "source.fav",
      "path": "./syntaxes/favnir.tmGrammar.json"
    }]
  }
}
```

### `extension.ts` — LSP クライアント（主要部分）

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverOptions: ServerOptions = {
    command: 'fav',
    args: ['lsp'],
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'favnir' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.fav'),
    },
  };
  client = new LanguageClient('favnir', 'Favnir Language Server', serverOptions, clientOptions);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
```

### `syntaxes/favnir.tmGrammar.json` — 最小 TextMate 文法

キーワード（`fn`, `bind`, `pub`, `type`, `interface`, `const`, `stage`, `pipeline`, `import`, `where`, `phantom`）、
コメント（`//`）、文字列リテラル、数値リテラルをハイライト。

---

## テスト詳細

```rust
// v721000_tests — VS Code 拡張ファイル存在・内容確認

fn vscode_extension_package_json_valid() {
    let src = include_str!("../../editors/vscode/package.json");
    assert!(src.contains("\"favnir\""), "package.json should contain extension name 'favnir'");
    assert!(src.contains("\"publisher\""), "package.json should contain publisher field");
    assert!(src.contains("\".fav\""), "package.json should register .fav extension");
}

fn vscode_extension_lsp_integration() {
    let src = include_str!("../../editors/vscode/extension.ts");
    assert!(src.contains("LanguageClient"), "extension.ts should use LanguageClient");
    assert!(src.contains("fav"), "extension.ts should reference 'fav' LSP command");
    assert!(src.contains("lsp"), "extension.ts should reference 'lsp' argument");
}
```

---

## Success Criteria

- `cargo test v721000` で 2 件 pass（0 failures）
  - `vscode_extension_package_json_valid` pass
  - `vscode_extension_lsp_integration` pass
- `cargo test` 全体で 3614 tests pass（3612 + 2）
- `editors/vscode/package.json` が存在し `"favnir"` / `"publisher"` / `".fav"` を含む
- `editors/vscode/extension.ts` が存在し `LanguageClient` / `fav` / `lsp` を含む
- `editors/vscode/syntaxes/favnir.tmGrammar.json` が存在する

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `editors/vscode/package.json` | 新規作成（拡張マニフェスト） |
| `editors/vscode/extension.ts` | 新規作成（LSP クライアント） |
| `editors/vscode/syntaxes/favnir.tmGrammar.json` | 新規作成（TextMate 文法） |
| `fav/src/driver.rs` | `v721000_tests` モジュール追加（2 テスト）+ version アサーション更新 |
| `fav/Cargo.toml` | version `72.0.0` → `72.1.0` |
| `CHANGELOG.md` | `## [v72.1.0]` エントリ追加 |
| `versions/current.md` | 進行中: v72.1.0、次: v72.2.0 |

---

## スコープ外

- `editors/vscode/` のビルド設定（`tsconfig.json` / webpack 等）: 別タスク
- VS Code Marketplace への実際の公開: 別タスク
- `language-configuration.json`（括弧ペア等）: v72.2.0 以降
