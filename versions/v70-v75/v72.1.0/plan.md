# v72.1.0 実装計画 — VS Code 拡張（本格実装）

Date: 2026-08-12

---

## 依存関係

```
Step 1（editors/vscode/ ディレクトリ作成）
  └→ Step 2（package.json 作成）
       └→ Step 3（extension.ts 作成）
            └→ Step 4（syntaxes/favnir.tmGrammar.json 作成）
                 └→ Step 5（v721000_tests 追加 + cargo_toml_version 更新）
                      └→ Step 6（Cargo.toml バージョン更新）
                           └→ Step 7（cargo test v721000 確認）
                                └→ Step 8（cargo test 全体確認）
                                     └→ Step 9（CHANGELOG.md 更新）
                                          └→ Step 10（versions/current.md 更新）
```

---

## 実装ステップ

### Step 1: `editors/vscode/` ディレクトリ作成

```
mkdir -p editors/vscode/syntaxes/
```

### Step 2: `editors/vscode/package.json` 作成

VS Code 拡張マニフェスト。以下のフィールドが必須:
- `"name": "favnir"` — テストが `contains("\"favnir\"")` を検査
- `"publisher": "favnir"` — Marketplace 公開者
- `"engines": { "vscode": "^1.85.0" }` — 対応 VS Code バージョン
- `"contributes.languages"` に `.fav` 拡張子を登録（テストが `contains("\".fav\"")` を検査）
- `"contributes.grammars"` に TextMate 文法を登録
- `"activationEvents": ["onLanguage:favnir"]`
- `"main": "./out/extension.js"`

### Step 3: `editors/vscode/extension.ts` 作成

LSP クライアント実装。テスト要件:
- `contains("LanguageClient")` — vscode-languageclient を使用
- `contains("fav")` — サーバーコマンドに `fav` を指定
- `contains("lsp")` — サーバー引数に `lsp` を指定

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
  client = new LanguageClient(
    'favnir',
    'Favnir Language Server',
    serverOptions,
    clientOptions,
  );
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
```

### Step 4: `editors/vscode/syntaxes/favnir.tmGrammar.json` 作成

TextMate 文法定義。最小実装:
- `scopeName`: `"source.fav"`
- キーワードパターン: `fn|bind|pub|type|interface|const|stage|pipeline|import|where|phantom|true|false`
- コメント: `//.*$`
- 文字列: `"[^"]*"`
- 数値: `\b[0-9]+(\.[0-9]+)?\b`

### Step 5: `v721000_tests` 追加（`driver.rs`）

`v72000_tests` モジュールの直後に追加:

```rust
// ── v72.1.0: VS Code 拡張 ────────────────────────────────────────────────────
#[cfg(test)]
mod v721000_tests {
    #[test]
    fn vscode_extension_package_json_valid() {
        let src = include_str!("../../editors/vscode/package.json");
        assert!(src.contains("\"favnir\""), "package.json should contain extension name 'favnir'");
        assert!(src.contains("\"publisher\""), "package.json should contain publisher field");
        assert!(src.contains("\".fav\""), "package.json should register .fav extension");
    }

    #[test]
    fn vscode_extension_lsp_integration() {
        let src = include_str!("../../editors/vscode/extension.ts");
        assert!(src.contains("LanguageClient"), "extension.ts should use LanguageClient");
        assert!(src.contains("fav"), "extension.ts should reference 'fav' LSP command");
        assert!(src.contains("lsp"), "extension.ts should reference 'lsp' argument");
    }
}
```

> **注意**: `include_str!` のパスは `fav/src/` 基準で `../../editors/vscode/...` となる。
> `use` は不要（`include_str!` のみ使用）。

### Step 6: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- `Cargo.toml`: `72.0.0` → `72.1.0`
- `driver.rs` 内の `"72.0.0"` バージョンアサーション文字列を `"72.1.0"` に replace_all
  - replace_all は「`version = \"72.0.0\"`」という文字列全体を対象にすること（数値 `72.0.0` 単体ではなく、`version = "..."` 形式の assert 文字列）
  - 同時に、古いモジュールに残存するエラーメッセージ（例: `"Cargo.toml version should be 72.0.0"`）も `"72.1.0"` に更新されることを確認する

### Step 7: `cargo test v721000` — 2 件 pass 確認

### Step 8: `cargo test` 全体 — 3614 tests pass 確認

### Step 9: `CHANGELOG.md` に v72.1.0 エントリ追加

先頭に `## [v72.1.0]` エントリを追加。

### Step 10: `versions/current.md` 更新

- 進行中: v72.1.0（VS Code 拡張）
- 次: v72.2.0

---

## 注意事項

- `editors/vscode/` は Rust crate ではないため `Cargo.toml` への追加不要
- `include_str!("../../editors/vscode/package.json")` のパスは `fav/src/driver.rs` から見た相対パス
  - `fav/src/` → `../../` → `favnir/` → `editors/vscode/package.json`
- `extension.ts` は TypeScript ファイルだが、テストは Rust の `include_str!` で静的チェックするだけ（TS コンパイルは不要）
- replace_all で `"72.0.0"` を `"72.1.0"` に変換する際、既存 assert エラーメッセージも同時に更新されることに注意（前バージョンでの教訓：エラーメッセージも正しく更新すること）
