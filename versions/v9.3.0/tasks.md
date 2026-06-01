# Favnir v9.3.0 Tasks

Date: 2026-06-01
Theme: fav lint — 静的解析ルールエンジン（W001〜W005）

---

## Phase A: `compiler.fav` — LintWarning 型 + 各ルール実装
（注: checker.fav は IStage/ISeq を持たないため compiler.fav に実装）

- [x] A-1: `type LintWarning = { code: String  message: String  name: String }` を `compiler.fav` に追加
- [x] A-2: W001 — `lint_stage_w001(sd: StageDef) -> List<LintWarning>`
  — `ret_ty == Unit` かつ `effects` 空 → W001 を返す
- [ ] A-3: W004 — 将来版に延期
- [x] A-4: W002 — `find_stage_by_name` + `lint_seq_w002(sd: SeqDef, items: List<Item>)`
  — `seq` 最終 stage に書き込みエフェクトなし → W002
- [x] A-5: W005 — `is_wild_pat` + `is_wildcard_only` + `lint_expr_w005`
  — `EMatch` の腕が `_`/`PWild` 1 本のみ → W005（`_` は `PVar("_")` としてパースされることに注意）
- [x] A-6: W003 — `expr_uses` + `lint_expr_w003`
  — `EBind` の束縛名が後続式に出現しない → W003（`_` はスキップ）
- [x] A-7: `lint_item(item: Item, items: List<Item>) -> List<LintWarning>` を実装
- [x] A-8: `lint_all(prog: Program) -> List<LintWarning>` を実装

---

## Phase B: `compiler.fav` — `lint_source` 公開 API

- [x] B-1: `fn format_warning(w: LintWarning) -> String` を実装
- [x] B-2: `fn format_warnings(ws: List<LintWarning>) -> String` を実装
- [x] B-3: `public fn lint_source(src: String) -> Result<String, String>` を追加
- [x] B-4: 実装完了確認

---

## Phase C: Rust ブリッジ（最小）

- [x] C-1: `lint_source_str(src: &str) -> Result<String, String>` を `compiler_fav_runner.rs` に追加
- [x] C-2: `"Compiler.lint_source_raw"` builtin を `vm.rs` に追加

---

## Phase D: `cli.fav` — `fav lint` コマンド

- [x] D-1: `CmdLint(String, Bool)` を `CliCmd` 型に追加
  — `(path, warn_as_error)`
- [x] D-2: `fn parse_lint_cmd(args: List<String>) -> CliCmd` を実装
  — `--warn-as-error` フラグをパース
- [x] D-3: `parse_named_cmd` に `"lint"` ブランチを追加
- [x] D-4: `fn run_lint(path: String, warn_as_error: Bool) -> Unit !IO` を実装
  — `IO.read_file_raw` → `Compiler.lint_source_raw` → 警告出力 or `IO.exit_raw(1)`
- [x] D-5: `run_help` に lint コマンドの説明を追加
  — `"  lint <file>               Lint a .fav file"`
  — `"  lint --warn-as-error <f>  Lint (exit 1 if warnings)"`
- [x] D-6: `run_version` のバージョン文字列を `"9.3.0"` に更新
- [x] D-7: `fav check fav/self/cli.fav` — コンパイルエラーなし確認

---

## Phase E: 統合テスト（`src/driver.rs` に `lint_tests` モジュール追加）

- [x] E-1: `lint_w001_effectless_sink` — W001 が検出されること
- [x] E-2: `lint_w002_no_write_in_seq` — W002 が検出されること
- [x] E-3: `lint_w003_unused_binding` — W003 が検出されること
- [ ] E-4: `lint_w004_too_many_args` — W004 が検出されること（W004 は将来版に延期）
- [x] E-5: `lint_w005_wildcard_only` — W005 が検出されること
- [x] E-6: `lint_clean_source_no_warnings` — 警告なしソースで空文字列が返ること
- [x] E-7: `cargo test lint_tests` — 全 6 件通過確認

---

## Phase F: self-check + Bootstrap 検証

- [x] F-1: `fav check fav/self/checker.fav` — self-check 通過
- [x] F-2: `cargo test bootstrap` — `bytecode_A == bytecode_B` 維持確認
- [x] F-3: `cargo test` — 全件通過（1173 件以上）確認

---

## Phase G: ドキュメント・バージョン更新

- [x] G-1: `fav/Cargo.toml` の `version` を `"9.3.0"` に更新
- [x] G-2: `versions/v9.3.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] G-3: `memory/MEMORY.md` に v9.3.0 完了を記録
- [x] G-4: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav lint <file>` が W001〜W005 を検出する | |
| `fav lint --warn-as-error <file>` が警告時に終了コード 1 を返す | |
| `fav lint fav/self/compiler.fav` が実行できる（警告ゼロが理想） | |
| `fav check fav/self/checker.fav` が self-check を通る | |
| `bytecode_A == bytecode_B` を維持（Bootstrap） | |
| `cargo test` 全件通過（1173 件以上） | |
