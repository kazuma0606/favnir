# v29.7.0 Spec — VS Code 拡張 公式リリース

**バージョン**: 29.7.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 7)
**前バージョン**: v29.6.0 (pagerduty Rune 追加)

---

## 概要

Favnir の LSP サーバー（v9.11.0 から実装済み）を VS Code Marketplace に正式公開する。
`extensions/vscode-favnir/` に VS Code 拡張パッケージを追加し、`vsce package` で `.vsix` が生成できる状態にする。

LSP サーバーは `fav/src/lsp/` に completion / definition / hover / diagnostics / signature help 等が実装済みであり、
本バージョンでは **VS Code クライアント側**の構築に集中する。

> **ポジショニング**: pagerduty（v29.6）でインシデント通知が揃った。
> VS Code 拡張を Marketplace に公開することで、開発者が IDE を離れることなく Favnir を書ける環境が完成する。
> `fav add stripe` の体験を「書いて → 補完されて → エラーが赤く出る」で締める。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `extensions/vscode-favnir/package.json` | Marketplace メタデータ・貢献点定義 |
| `extensions/vscode-favnir/language-configuration.json` | コメント・括弧・自動閉じ設定 |
| `extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json` | TextMate grammar（シンタックスハイライト）|
| `extensions/vscode-favnir/src/extension.ts` | LSP クライアント + Task Runner 統合 |
| `fav/Cargo.toml` | version 29.6.0 → 29.7.0 |
| `CHANGELOG.md` | `[v29.7.0]` セクション追加 |
| `benchmarks/v29.7.0.json` | ベンチマーク記録 |
| `site/content/docs/tools/vscode-extension.mdx` | VS Code 拡張ドキュメント |
| `fav/src/driver.rs` | `v297000_tests` 6 件追加 |

---

## VS Code 拡張 API

### 提供機能

| 機能 | 実装方式 | 状態 |
|---|---|---|
| シンタックスハイライト | TextMate grammar（`syntaxes/favnir.tmLanguage.json`）| 本バージョンで追加 |
| エラー / 警告のリアルタイム表示 | LSP `textDocument/publishDiagnostics` | LSP 実装済み |
| 補完 | LSP `textDocument/completion` | LSP 実装済み（completion.rs）|
| 定義ジャンプ（F12） | LSP `textDocument/definition` | LSP 実装済み（definition.rs）|
| ホバー表示 | LSP `textDocument/hover` | LSP 実装済み（hover.rs）|
| シグネチャヘルプ | LSP `textDocument/signatureHelp` | LSP 実装済み（signature.rs）|
| `fav run` / `fav check` 統合 | VS Code Task Runner（`tasks.json` テンプレート）| 本バージョンで追加 |

### `package.json` メタデータ

```json
{
  "name": "vscode-favnir",
  "displayName": "Favnir",
  "description": "Favnir language support for VS Code — syntax highlighting, type inference, LSP",
  "version": "1.0.0",
  "publisher": "favnir",
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:favnir"],
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

---

## テスト戦略

### v297000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `vscode_package_json_exists` | `extensions/vscode-favnir/package.json` が存在し `vscode-favnir` を含む |
| `vscode_grammar_exists` | `extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json` が存在し `source.favnir` を含む |
| `vscode_language_config_exists` | `extensions/vscode-favnir/language-configuration.json` が存在し `lineComment` を含む |
| `vscode_extension_src_exists` | `extensions/vscode-favnir/src/extension.ts` が存在し `LanguageClient` を含む |
| `vscode_extension_mdx_exists` | `site/content/docs/tools/vscode-extension.mdx` が存在し `vscode-favnir` を含む |
| `changelog_has_v29_7_0` | `CHANGELOG.md` に `[v29.7.0]` が存在する |

テスト数: 2348 → **2354**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "29.7.0"
- [ ] `extensions/vscode-favnir/package.json` が存在し `vscode-favnir` を含む
- [ ] `extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json` が存在する（TextMate grammar）
- [ ] `extensions/vscode-favnir/language-configuration.json` が存在する
- [ ] `extensions/vscode-favnir/src/extension.ts` が存在し LSP クライアント起動コードを含む
- [ ] `CHANGELOG.md` に `[v29.7.0]` セクションあり
- [ ] `benchmarks/v29.7.0.json` 存在（test_count: 2354）
- [ ] `site/content/docs/tools/vscode-extension.mdx` 存在
- [ ] `cargo test --bin fav v297000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2354 tests PASS

---

## スコープ外

- `vsce package` の実際の実行（Node.js / TypeScript ビルド環境が別途必要）
  → ロードマップ v29.7 完了条件の「`vsce package` で `.vsix` が生成できる」は **v29.7 完了後に手動実施**
- VS Code Marketplace への実際のアップロード（手動作業）
  → ロードマップ v30.0 完了条件の「Marketplace 公開・インストール確認」は **v29.7 成果物を使い手動実施**
- `tsconfig.json`（`vsce` 実行環境構築時に別途追加）
- TypeScript のコンパイルチェック（`tsc`）— CI 分離
- Inlay Hints（型推論結果のインライン表示）— LSP 拡張として v30.x+ で対応
- Windows / Linux / macOS 向けのバイナリパッケージ同梱
