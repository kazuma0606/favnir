# v24.0.0 — VM in Favnir マイルストーン宣言 タスク

## ステータス: COMPLETE（2026-06-23）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"23.8.0"` であること
- [x] `grep -n "mod v238000_tests\|mod v240000_tests" fav/src/driver.rs | head -5` — v240000_tests 未存在
- [x] `grep -n "run_with_vm\|--vm" fav/src/driver.rs fav/src/main.rs | head -5` — 全 0 件
- [x] `grep -n "VM in Favnir" README.md | head -3` — 0 件であること

---

### T1: `fav/src/driver.rs` — `run_with_vm` 追加

- [x] `cmd_run_precompiled` の直後に `pub fn run_with_vm(vm_src: &str, bytecode_hex: &str, globals_entries: &[(usize, &str)]) -> Result<Value, String>` を追加
  - `globals_entries` を `Mut.set(globals, idx, VMStr("val"))` の Favnir コードに変換
  - `val` の `\` と `"` をエスケープ（`replace('\\', "\\\\").replace('"', "\\\"")`）
  - vm_src + main 関数（globals セットアップ + `vm_run_named` + `vmval_display`）を format! で結合
  - `Lexer::new` → `Parser::new` → `build_artifact` → `exec_artifact_main`
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: `fav/src/main.rs` — `--vm / --hex` フラグ追加

- [x] `--precompiled` ブロックの直後、`// Parse --db / ...` の前に `--vm` フラグブロックを追加
  - `args.iter().position(|a| a == "--vm")` で vm_pos を取得
  - `args.get(vm_pos + 1)` で vm_path
  - `args.iter().position(|a| a == "--hex")` で hex_pos
  - `args.get(hex_pos + 1)` で bytecode_hex
  - `std::fs::read_to_string(vm_path)` で vm_src
  - `driver::run_with_vm(&vm_src, bytecode_hex, &[])` → `v.display()` / eprintln + exit(1)
- [x] **事後確認**: `cargo check --bin fav` — エラー 0
- [x] **後方互換確認**: `cargo test v238000 --bin fav` — 6/6 PASS

---

### T3: `fav/src/driver.rs` — `v240000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_8_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T4-1 より前に必須）**: `v238000_tests::version_is_23_8_0` テスト関数を**削除**
- [x] **T3-2**: `v240000_tests` モジュールを `v238000_tests` の直後に追加（5 件）
  - `version_is_24_0_0`
  - `run_with_vm_hello`（hex `"12000016"` + globals=[(0,"hello")] → `"hello"`）
  - `run_with_vm_string_trim`（hex `"12000040010012020015010016"` + 3 globals → `"hi"`）
  - `changelog_has_v24_0_0`
  - `readme_has_vm_in_favnir`
- [x] `cargo test v240000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1930 件合格）を確認

---

### T4: Cargo.toml + CHANGELOG + README + benchmarks

> **注意**: T3-1 の `version_is_23_8_0` 削除完了後に Cargo.toml を更新すること。

- [x] **事前確認**: `grep "\[v24\." CHANGELOG.md | head -3` — 0 件
- [x] `fav/Cargo.toml` の `version = "23.8.0"` → `"24.0.0"` に変更
- [x] `CHANGELOG.md` 先頭に v24.0.0 エントリを追加
- [x] `README.md` の `v23.0.0` 現在の状態セクションの直上に "VM in Favnir" セクションを追加
  - `**v24.0.0（2026-06-23）— VM in Favnir マイルストーン宣言**` を含むこと
  - 達成実績表（v23.1〜v24.0 の 9 行）を含むこと
- [x] `benchmarks/v24.0.0.json` を新規作成（test_count: 1930）
- [x] `cargo test v240000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1930 件合格）

---

## テスト一覧（v240000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_0_0` | Cargo.toml に `version = "24.0.0"` | — |
| `run_with_vm_hello` | `run_with_vm` + hex `"12000016"` + globals[(0,"hello")] | `"hello"` |
| `run_with_vm_string_trim` | `run_with_vm` + GetField+Call(1) bytecode | `"hi"` |
| `changelog_has_v24_0_0` | CHANGELOG.md に `[v24.0.0]` | — |
| `readme_has_vm_in_favnir` | README.md に `VM in Favnir` | — |

---

## 完了条件チェックリスト

- [x] `driver::run_with_vm` が追加される（pub fn）
- [x] `main.rs` に `--vm / --hex` フラグが追加される
- [x] `v238000_tests::version_is_23_8_0` が削除済み（T4-1 より前）
- [x] `cargo test v240000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1930 件合格）
- [x] `CHANGELOG.md` に v24.0.0 エントリ
- [x] `benchmarks/v24.0.0.json` 作成済み（test_count: 1930）
- [x] `README.md` に "VM in Favnir マイルストーン宣言" セクション追加済み

---

## 実装時の注意事項（実績）

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `run_with_vm` の `format!` 内で vm.fav ソースの `{` `}` が問題になるか | vm.fav ソースは format! の**引数値**として渡されるため、その中の `{` `}` はテンプレートとして再解釈されない。`execute_hello_via_vm` テストで動作実証済み |
| 2 | globals_entries の val に `"` が含まれる場合 | `replace('"', "\\\"")` で事前エスケープ。テストデータには `"` は含まれないため基本問題なし |
| 3 | `--vm` フラグと `--debug` / `--precompiled` の評価順序 | `--vm` を `--precompiled` の直後に配置することで既存フラグより後に評価される |
| 4 | README の挿入位置 | `v23.0.0` の現在の状態セクションの**直上**（`---\n\n## 現在の状態` の前）に挿入 |
| 5 | `version_is_23_8_0` 削除順序 | Cargo.toml 更新（T4-1）前に必ず T3-1 を完了させること |
| 6 | `Value` の `println!` | `Value` は `Display` 未実装 → `v.display()` メソッドを使用 |

---

## コードレビュー対応（2026-06-23）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `globals_entries` の制御文字（`\n`/`\r`/`\t`）エスケープ漏れ | `replace` チェーンに `\n`/`\r`/`\t` を追加 |
| [MED] | `--hex` 単独使用時にサイレントで `cmd_run` に流れる | `--hex` あり `--vm` なし の場合に明示的エラー + `process::exit(1)` を追加 |
| [LOW] | エラーケーステスト不在（不正 hex / vm 実行エラー） | スキップ（次バージョン検討） |
| [LOW] | エスケープ動作テスト不在 | スキップ（次バージョン検討） |
