# Favnir v12.5.0 Tasks

Date: 2026-06-08
Theme: `fav run --verbose` + `fav check --json` / `--show-types`

---

## Phase A — `fav run --verbose`

- [x] A-1: `RunConfig` に `verbose: bool` / `trace: bool` フィールド追加
- [x] A-2: `cmd_run` のフラグパースで `--verbose` / `--trace` を読む
- [x] A-3: VM の stage 開始・終了時に `[TRACE] stage X: enter/exit` を stderr 出力
- [x] A-4: bind opcode 実行後に `[TRACE]   bind x <- ... → Ok(...)` を stderr 出力
  - `--verbose`: 値を最大 200 文字でトランケート、末尾に `[N chars]` を付与
  - `--trace`: 制限なし
- [x] A-5: `SeqStageCheck` escape 時に `[TRACE] seq X: stopped at stage N/M (StageName)` 出力
- [x] A-6: `fav.toml` の `[run] verbose = true` を `RunConfig` に反映

---

## Phase B — `fav check --json`

- [x] B-1: `CheckError` / `CheckOutput` 構造体を定義（`#[derive(serde::Serialize)]`）
- [x] B-2: `cmd_check` に `--json` フラグ追加
- [x] B-3: checker 出力から `code` / `message` / `file` / `line` / `col` / `suggestion` を収集
- [x] B-4: `--json` 時に `serde_json::to_string_pretty` で stdout 出力
- [x] B-5: エラーなしの場合 `{ "errors": [], "warnings": [], "ok": true }` を出力
- [x] B-6: exit code は既存と同じ（エラーあり → 1）

---

## Phase C — `fav check --show-types`

- [x] C-1: `cmd_check` に `--show-types` フラグ追加
- [x] C-2: checker の bind/chain ごとの推論型を収集
- [x] C-3: テキスト出力フォーマット実装（`file:line  bind name : Type  ← W006`）
- [x] C-4: `--json --show-types` 組み合わせ時に `"bindings"` フィールドを JSON に追加

---

## Phase D — テスト追加

- [x] D-1: `verbose_logs_stage_enter`
- [x] D-2: `verbose_logs_bind_result`
- [x] D-3: `verbose_logs_seq_stopped`
- [x] D-4: `verbose_truncates_long_values`
- [x] D-5: `check_json_output_format`
- [x] D-6: `check_json_ok_true_on_success`
- [x] D-7: `check_json_includes_suggestion`
- [x] D-8: `check_show_types_bind`
- [x] D-9: `check_show_types_w006_marked`
- [x] D-10: `version_is_12_5_0`
- [x] D-11: `cargo test v12500` — 10 件通過確認

---

## Phase E — バージョン更新・コミット

- [x] E-1: `fav/Cargo.toml` version → `"12.5.0"`
- [x] E-2: `cargo test` — 全通過（1386 件）
- [x] E-3: `git commit -m "feat: v12.5.0 — fav run --verbose + fav check --json / --show-types"`
- [ ] E-4: `git push` → CI 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav run --verbose` で stage / bind のトレースが stderr に出る | ✅ |
| `fav run --trace` で値の長さ制限なし出力 | ✅ |
| `fav check --json` で JSON 形式のエラー出力 | ✅ |
| `fav check --show-types` で bind ごとの推論型表示 | ✅ |
| `--json --show-types` 組み合わせで `bindings` フィールド付き JSON | ✅ |
| `cargo test v12500` 10 件通過 | ✅ |
| `cargo test` 全通過 | ✅ 1386 件 |
