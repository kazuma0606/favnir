# v72.1.0 タスクリスト — VS Code 拡張（本格実装）

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x]`fav/Cargo.toml` のバージョンが `72.0.0` であることを確認
- [x]`cargo test` が 3612 tests pass（0 failures）であることを確認
- [x]`driver.rs` に `v72000_tests` モジュールが存在することを確認
- [x]`driver.rs` に `v721000_tests` が未存在であることを確認
- [x]`editors/vscode/` ディレクトリが未存在であることを確認

---

## T1: `editors/vscode/` ディレクトリ + `package.json` 作成

- [x]`editors/vscode/syntaxes/` ディレクトリを作成した
- [x]`editors/vscode/package.json` を作成した
- [x]`"name": "favnir"` フィールドが含まれていることを確認
- [x]`"publisher"` フィールドが含まれていることを確認
- [x]`.fav` 拡張子が `contributes.languages` に登録されていることを確認

---

## T2: `editors/vscode/extension.ts` 作成

- [x]`editors/vscode/extension.ts` を作成した
- [x]`LanguageClient` が含まれていることを確認
- [x]`fav` コマンドが含まれていることを確認
- [x]`lsp` 引数が含まれていることを確認
- [x]`activate` / `deactivate` 関数が実装されていることを確認

---

## T3: `editors/vscode/syntaxes/favnir.tmGrammar.json` 作成

- [x]`editors/vscode/syntaxes/favnir.tmGrammar.json` を作成した
- [x]`scopeName: "source.fav"` が含まれていることを確認
- [x]キーワード（`fn`, `bind`, `type` 等）のパターンが定義されていることを確認

---

## T4: `v721000_tests` 追加（`driver.rs`）

> **前提**: T1・T2・T3 完了済みであること（`include_str!` がファイル未存在だと `cargo build` でコンパイルエラーになる）。

- [x]`v72000_tests` モジュールの直後に `v721000_tests` モジュールを追加した
- [x]`#[cfg(test)]` のみ（`use` 不要 — `include_str!` のみ使用）
- [x]`vscode_extension_package_json_valid` テストを実装した
  - `include_str!("../../editors/vscode/package.json")` を使用
  - `"\"favnir\""` / `"\"publisher\""` / `"\".fav\""` の 3 条件を assert
- [x]`vscode_extension_lsp_integration` テストを実装した
  - `include_str!("../../editors/vscode/extension.ts")` を使用
  - `"LanguageClient"` / `"fav"` / `"lsp"` の 3 条件を assert
- [x]`cargo build` でエラーがないことを確認

---

## T5: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x]`fav/Cargo.toml` の `version = "72.0.0"` → `version = "72.1.0"` に変更した
- [x]`driver.rs` 内の `"72.0.0"` バージョンアサーション文字列を `"72.1.0"` に replace_all した

---

## T6: 部分テスト確認

- [x]`cargo test v721000` で 2 件 pass することを確認

---

## T7: 全体テスト確認

- [x]`cargo test` 全体で 3614 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [x]`## [v72.1.0]` エントリを先頭に追加した

---

## T9: `versions/current.md` 更新

- [x]「進行中バージョン」を `v72.1.0`（VS Code 拡張）に更新した
- [x]「次に切る版」を `v72.2.0` に更新した

---

## T10: 最終確認

- [x]`cargo test v721000` で 2 件 pass することを確認
- [x]`cargo test` 全体で 3614 tests pass（0 failures）であることを確認
- [x]`fav/Cargo.toml` のバージョンが `72.1.0` であることを確認
- [x]`editors/vscode/package.json` が存在することを確認
- [x]`editors/vscode/extension.ts` が存在することを確認
- [x]`editors/vscode/syntaxes/favnir.tmGrammar.json` が存在することを確認
- [x]`versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- `editors/vscode/tsconfig.json` / webpack 設定: 別タスク
- VS Code Marketplace への実際の公開: 別タスク
- `language-configuration.json`: v72.2.0 以降
- `site/` MDX 更新: 別タスク（TypeScript ビルド設定完了・動作確認後に記述予定 — v72.2.0 以降）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | extension.ts — fileName のシェルインジェクション（スペース・メタ文字でコマンド分割される） | `JSON.stringify(fileName)` でクォートして修正 |
| [MED] | extension.ts — `client: LanguageClient` が未初期化のまま deactivate に渡りうる | `client: LanguageClient \| undefined` に型変更 |
| [MED] | tmGrammar.json — `///` ルールが `//` ルールに先に吸収される（到達不能） | `///` パターンを先に記述するよう順序を入れ替え |
| [MED] | tmGrammar.json — `let` が Favnir キーワードとして登録（Favnir は `bind` を使い `let` は無効構文） | `let` をキーワードリストから削除 |
| [LOW] | package.json — `activationEvents: ["onLanguage:favnir"]` は VS Code 1.74+ で冗長 | `activationEvents: []` に変更 |
| [LOW] | 過去テストのエラーメッセージに古いバージョン番号が残存（replace_all の構造的問題） | 既知問題として記録、次バージョン carry-over |

---

## 完了チェックリスト

- [x]全タスク（T0〜T10）が完了している
- [x]`vscode_extension_package_json_valid` が pass
- [x]`vscode_extension_lsp_integration` が pass
- [x]テスト総数: 3614（+2）
