# Favnir v13.1.0 Tasks

Date: 2026-06-09
Theme: interface 継承仕様確定 + ambient effect 禁止調査

---

## Phase A — AST + parser.rs: interface 継承構文

- [ ] A-1: `fav/src/ast.rs` — `InterfaceDef` に `parent: Option<String>` フィールドを追加
  - `InterfaceDef` を生成している全箇所に `parent: None` を追加（コンパイルエラーを逐次修正）
- [ ] A-2: `fav/src/frontend/parser.rs` — `parse_interface_def` に `: ParentName` optional 解析を追加
  - `interface Name` の後に `Token::Colon` があれば ParentName を読む
  - なければ `parent: None`
- [ ] A-3: `fav/src/fmt.rs` — `format_interface_def` で `parent` が `Some(p)` のとき `: p` を出力

---

## Phase B — checker.rs: 継承フィールド解決 + E0019

- [ ] B-1: `fav/src/middle/checker.rs` — `resolve_interface_fields` 関数を追加
  - 親の interface フィールドを再帰的にマージ（深さ上限 16）
  - フィールドアクセス型解決時に呼び出す
- [ ] B-2: `fav/src/middle/checker.rs` — `check_interface_cycles` 関数を追加
  - DFS で継承グラフの閉路を検出 → E0019 Diagnostic を返す
  - `check_program` の先頭で呼び出し
- [ ] B-3: E0019 のエラーメッセージ確認
  - `fav check` で `interface A: B {}` + `interface B: A {}` → E0019 が出ること

---

## Phase C — compiler.fav / checker.fav: セルフホスト対応

- [ ] C-1: `fav/self/compiler.fav` — `InterfaceDef` レコード型に `parent: String` を追加（`""` = なし）
- [ ] C-2: `fav/self/compiler.fav` — `parse_interface_def` に `: ParentName` の解析を追加
  - 現在のトークンが `:` なら次トークンを parent として読む
  - 既存の `InterfaceDef` 生成箇所に `parent: ""` を追加
- [ ] C-3: `fav/self/checker.fav` — `lookup_interface_field_recursive` 関数を追加
  - `parent` が `""` でなければ再帰的に親を辿ってフィールドを解決
- [ ] C-4: セルフホスト検証
  ```bash
  ./target/debug/fav check self/compiler.fav   # エラーなし
  ./target/debug/fav check self/checker.fav    # エラーなし
  ./target/debug/fav fmt --check self/compiler.fav
  ./target/debug/fav fmt --check self/checker.fav
  ```

---

## Phase D — lint.rs: W008 ambient effect 警告

- [ ] D-1: `fav/src/lint.rs` — `AMBIENT_NAMESPACES` 定数を定義
  - `IO`, `Postgres`, `AWS`, `Snowflake`, `Http`, `Grpc`, `Llm`, `Queue`, `Cache`, `Slack`, `Email`
  - `Gen` の副作用関数: `uuid_raw`, `uuid_v7_raw`, `nano_id`
- [ ] D-2: `fav/src/lint.rs` — `check_ambient_effects(program: &Program) -> Vec<LintWarning>` を実装
  - AST walk で `NS::fn(...)` 呼び出しのうち ambient namespace のものを W008 として収集
  - `ctx.io.println(...)` 形式（フィールドアクセス経由）は除外
  - `LintWarning { code: "W008", message, span }` を返す
- [ ] D-3: `fav/src/driver.rs` — `get_help_text` に `"W008"` エントリを追加
  ```rust
  "W008" => &[
      "pass the capability as a ctx argument: `ctx.io.println(...)`",
      "ambient effects will become E0023 (error) in v14.0",
  ],
  ```

---

## Phase E — driver.rs + main.rs: `--ambient` / `--report` フラグ

- [ ] E-1: `fav/src/driver.rs` — `cmd_check` シグネチャに `ambient: bool, report: bool` を追加
- [ ] E-2: `fav/src/driver.rs` — `cmd_check` 内で `ambient == true` のとき `check_ambient_effects` を呼び出し W008 を出力
- [ ] E-3: `fav/src/driver.rs` — `write_ambient_report` 関数を実装
  - `lab/audit/` ディレクトリを `create_dir_all` で作成
  - W008 警告を Markdown テーブル形式で書き出す
  - `report == true` のとき呼び出す
- [ ] E-4: `fav/src/main.rs` — check ディスパッチに `--ambient` / `--report` フラグを追加
  ```rust
  "--ambient" => { ambient = true; i += 1; }
  "--report"  => { report  = true; i += 1; }
  ```
  - `cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report)` に変更

---

## Phase F — テスト追加

- [ ] F-1: `fav/src/driver.rs` 末尾に `v131000_tests` モジュールを追加
  - `version_is_13_1_0` — `CARGO_PKG_VERSION == "13.1.0"`
  - `interface_inheritance_parsed` — `LoadCtx: CommonCtx` のパース確認（`parent == Some("CommonCtx")`）
  - `interface_inheritance_field_access` — 継承フィールドが解決されること
  - `e0019_circular_interface_detected` — `A: B` + `B: A` → E0019
  - `e0019_single_interface_no_error` — 継承なし → エラーなし
  - `w008_ambient_io_println_detected` — `--ambient` フラグで `IO.println(...)` が W008
  - `w008_no_flag_no_warning` — `--ambient` なしでは W008 なし
  - `w008_pure_list_no_warning` — `List.map(...)` は W008 対象外
- [ ] F-2: `fav/Cargo.toml` — `version = "13.1.0"` に更新
- [ ] F-3: 既存の `version_is_13_0_0` テストを comment out

---

## Phase G — W008 調査レポート生成

- [ ] G-1: ビルド確認
  ```bash
  cd fav && cargo build
  ```
- [ ] G-2: `cargo test` 全通過確認
- [ ] G-3: W008 調査レポート生成（手動）
  ```bash
  ./target/debug/fav check --ambient --report self/compiler.fav
  ./target/debug/fav check --ambient --report self/checker.fav
  cat ../lab/audit/w008-ambient.md
  ```
  - レポートの W008 件数を確認し `versions/v13.1.0/tasks.md` に記録
- [ ] G-4: CI 品質ゲート確認（既存 self-lint は変更なし）
  ```bash
  ./target/debug/fav lint --deny-warnings self/compiler.fav   # W008 は lint 対象外
  ./target/debug/fav lint --deny-warnings self/checker.fav
  ```

---

## Phase H — コミット・CI

- [ ] H-1: `git add -p` で変更を確認
- [ ] H-2: `git commit -m "feat: v13.1.0 — interface 継承 + W008 ambient effect 警告"`
- [ ] H-3: `git push`
- [ ] H-4: `gh run watch` で CI 全 green を確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `interface LoadCtx: CommonCtx { ... }` がパース・コンパイル可能 | |
| 継承フィールドが `checker.rs` / `checker.fav` で解決される | |
| E0019 循環継承が検出される | |
| `fav check --ambient` で W008 が出力される | |
| `fav check`（フラグなし）では W008 は出ない | |
| `fav lint --deny-warnings self/*.fav` → exit 0（W008 は lint 対象外） | |
| `lab/audit/w008-ambient.md` が生成される | |
| `CARGO_PKG_VERSION == "13.1.0"` | |
| `cargo test` 全通過 | |
| CI 全 green | |

---

## W008 調査結果（実装後に記録）

| ファイル | W008 件数 |
|---|---|
| self/compiler.fav | （実装後に記録） |
| self/checker.fav | （実装後に記録） |
| infra/e2e-demo/fav2py/src/pipeline.fav | （実装後に記録） |
| infra/e2e-demo/airgap/src/analyze.fav | （実装後に記録） |
| **合計** | |
