# Favnir v12.10.0 Tasks

Date: 2026-06-09
Theme: 全エラーに `help:` + `fav check --strict` + `fav lint --deny-warnings`

---

## Phase A — `get_help_text` + エラー出力への `help:` 注入（driver.rs）

- [x] A-1: `get_help_text(code: &str) -> &'static [&'static str]` を driver.rs に実装
  - E0001 / E0007 / E0008 / E0009 / E0013 / E0014 / E0015 / E0018
  - W001 / W004 / W006 / W007
  - それ以外は空スライス `&[]` を返す
- [x] A-2: `cmd_check` の型エラー出力ループに help 注入を追加
  - `format_diagnostic` / `format_warning` 内で help テキストを末尾に追記
- [x] A-3: `cmd_lint` の lint 出力ループに help 注入を追加
  - lint コード出力後に `for hint in get_help_text(lint.code)` を追加

---

## Phase B — `fav check --strict`（driver.rs + main.rs）

- [x] B-1: `cmd_check` シグネチャに `strict: bool` を追加
- [x] B-2: `strict=true` 時に `collect_binding_types` を呼び W006 数を確認
  - W006 が 1 件以上あれば `eprintln!` + `process::exit(1)`
- [x] B-3: `main.rs` の `Some("check")` 分岐に `--strict` フラグパースを追加
  - `let mut strict = false;`
  - `"--strict" => { strict = true; i += 1; }`
  - `cmd_check(...)` の呼び出しに `strict` を渡す

---

## Phase C — `fav lint --deny-warnings`（driver.rs + main.rs + ci.yml）

- [x] C-1: `cmd_lint` シグネチャに `deny_warnings: bool` を追加
  - `let should_exit = deny_warnings || !warn_only || has_error_level;`
  - 後方互換: `--warn-only` は引き続き動作
- [x] C-2: `main.rs` の lint 分岐に `--deny-warnings` パースを追加
  - `"--deny-warnings" => { deny_warnings = true; i += 1; }`
- [x] C-3: `ci.yml` の `Self-lint` ステップに `--deny-warnings` を追加
  ```yaml
  ./target/debug/fav lint --deny-warnings self/compiler.fav
  ./target/debug/fav lint --deny-warnings self/checker.fav
  ```

---

## Phase D — `fav.toml [lint]` セクション（toml.rs + driver.rs）

- [x] D-1: `LintTomlConfig { warn_as_error, allow }` を `toml.rs` に追加
- [x] D-2: `FavToml` に `pub lint: Option<LintTomlConfig>` フィールドを追加
- [x] D-3: `cmd_lint` で `fav.toml` を読み `allow` フィルタを適用
  - `allow` リストに含まれるコードを `lints` から除外
- [x] D-4: `cmd_lint` で `warn_as_error` コードがあれば exit 1 に昇格

---

## Phase E — テスト追加（driver.rs + tests/integration.rs）

- [x] E-1: `help_text_e0001_present` — `get_help_text("E0001")` が空でないこと（unit test）
- [x] E-2: `help_text_w006_present` — `get_help_text("W006")` が `"chain"` を含むこと（unit test）
- [x] E-3: `check_strict_w006_exits_1` — W006 あるファイルで `--strict` が exit 1（integration test）
- [x] E-4: `check_strict_no_warning_exits_0` — 警告なしで `--strict` が exit 0（integration test）
- [x] E-5: `lint_deny_warnings_exits_1` — W006 ファイルで `--deny-warnings` が exit 1（integration test）
- [x] E-6: `version_is_12_10_0` — `CARGO_PKG_VERSION == "12.10.0"`（unit test）
- [x] E-7: `cargo test v121000` — unit test 通過確認（4 passed）
- [x] E-8: `cargo test --test integration` — integration test 通過確認（8 passed）

---

## Phase F — バージョン更新・コミット

- [x] F-1: `fav/Cargo.toml` version → `"12.10.0"`
- [x] F-2: `driver.rs` の `version_is_12_9_0` を comment out
- [x] F-3: `cargo test` — 全通過（1415 unit + 8 integration）
- [x] F-4: `git commit -m "feat: v12.10.0 — help: for errors + --strict + --deny-warnings"`
- [x] F-5: `git push` → CI 通過確認（Rust / Integration / Site 全 green）

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| E0001 / E0018 / W006 等のエラー出力に `= help:` が含まれる | ✅ |
| `fav check --strict` が W006 で exit 1 になる | ✅ |
| `fav lint --deny-warnings` が明示的に exit 1 になる | ✅ |
| `fav.toml [lint] allow` で特定警告が抑制される | ✅ |
| CI `Self-lint` に `--deny-warnings` が追加される | ✅ |
| `cargo test` 全通過 | ✅ 1415 件 |
