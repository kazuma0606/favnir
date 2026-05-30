# Favnir v9.0.0 Tasks

Date: 2026-05-30
Theme: セルフホスト完成宣言（--legacy 非推奨化・バージョン更新）

---

## Phase A: `--legacy` フラグ非推奨化

- [x] A-1: `driver.rs` — `cmd_run` の legacy ブランチに deprecation 警告を追加
  — `eprintln!("warning: --legacy is deprecated since v9.0.0 ...")`
  — 変更ファイル: `fav/src/driver.rs`
- [x] A-2: `main.rs` — `--legacy` コメントに `[deprecated since v9.0.0]` を追加
  — 変更ファイル: `fav/src/main.rs`

---

## Phase B: バージョン定数更新

- [x] B-1: `fav/Cargo.toml` の `version` を `"9.0.0"` に変更
  — `fav --version` が `fav 9.0.0` を出力するようになる

---

## Phase C: 宣言テスト追加（driver.rs）

- [x] C-1: `self_hosting_complete_tests` モジュールを `driver.rs` に追加
  — `v900_self_hosting_apis_exist` テスト: 公開 API の存在を型レベルで確認
  — `compile_src_str_to_bytes` / `compile_project_to_bytes` の fn pointer を取る

---

## Phase D: 確認・ドキュメント

- [x] D-1: `cargo test v900` — 新規テスト通ること ✓
- [x] D-2: `cargo test dispatch` — 全 dispatch テスト通ること ✓
- [x] D-3: `cargo test checker_fav` — self-check 通ること（17 件）✓
- [x] D-4: `cargo test` — 全件通ること（1136 tests）✓
- [x] D-5: tasks.md 完了状態に更新・MEMORY.md 更新・commit

---

## 完了条件

- `--legacy` フラグ使用時に deprecation 警告が出る
- `fav --version` が `9.0.0` を返す
- セルフホスト API の存在を確認するテストが通る
- 既存テスト全件通る

---

## セルフホスト完成状態（v9.0.0 到達時点）

| コンポーネント | 状態 |
|---|---|
| `fav check` | Favnir pipeline（checker.fav）✅ |
| `fav run`（単一ファイル） | Favnir pipeline（compiler.fav）✅ |
| `fav run`（rune import） | Favnir pipeline（ソース結合）✅ |
| `fav run`（fav.toml プロジェクト） | Favnir pipeline（プロジェクト収集）✅ |
| VM | Rust（恒久・設計上）|
| ファイルパス解決 | Rust（OS インターフェース）|
| `--legacy` | deprecated ⚠️ |
