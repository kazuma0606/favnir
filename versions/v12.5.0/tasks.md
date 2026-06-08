# Favnir v12.5.0 Tasks

Date: 2026-06-08
Theme: `fav run --verbose` + `fav check --json` / `--show-types`

---

## Phase A — `fav run --verbose`

- [ ] A-1: `RunConfig` に `verbose: bool` / `trace: bool` フィールド追加
- [ ] A-2: `cmd_run` のフラグパースで `--verbose` / `--trace` を読む
- [ ] A-3: VM の stage 開始・終了時に `[TRACE] stage X: enter/exit` を stderr 出力
- [ ] A-4: bind opcode 実行後に `[TRACE]   bind x <- ... → Ok(...)` を stderr 出力
  - `--verbose`: 値を最大 200 文字でトランケート、末尾に `[N chars]` を付与
  - `--trace`: 制限なし
- [ ] A-5: `SeqStageCheck` escape 時に `[TRACE] seq X: stopped at stage N/M (StageName)` 出力
- [ ] A-6: `fav.toml` の `[run] verbose = true` を `RunConfig` に反映

---

## Phase B — `fav check --json`

- [ ] B-1: `CheckError` / `CheckOutput` 構造体を定義（`#[derive(serde::Serialize)]`）
- [ ] B-2: `cmd_check` に `--json` フラグ追加
- [ ] B-3: checker 出力から `code` / `message` / `file` / `line` / `col` / `suggestion` を収集
- [ ] B-4: `--json` 時に `serde_json::to_string_pretty` で stdout 出力
- [ ] B-5: エラーなしの場合 `{ "errors": [], "warnings": [], "ok": true }` を出力
- [ ] B-6: exit code は既存と同じ（エラーあり → 1）

---

## Phase C — `fav check --show-types`

- [ ] C-1: `cmd_check` に `--show-types` フラグ追加
- [ ] C-2: checker の bind/chain ごとの推論型を収集
- [ ] C-3: テキスト出力フォーマット実装（`file:line  bind name : Type  ← W006`）
- [ ] C-4: `--json --show-types` 組み合わせ時に `"bindings"` フィールドを JSON に追加

---

## Phase D — テスト追加

- [ ] D-1: `verbose_logs_stage_enter` — `[TRACE] stage X: enter` が stderr に出る
- [ ] D-2: `verbose_logs_bind_result` — bind 結果が `→ Ok(...)` 形式で出る
- [ ] D-3: `verbose_logs_seq_stopped` — seq fail-fast 時に `stopped at stage N/M` が出る
- [ ] D-4: `verbose_truncates_long_values` — 200 文字超の値がトランケートされる
- [ ] D-5: `check_json_output_format` — `--json` で正しい JSON 構造が出る
- [ ] D-6: `check_json_ok_true_on_success` — エラーなしで `"ok": true`
- [ ] D-7: `check_json_includes_suggestion` — `suggestion` フィールドが存在する
- [ ] D-8: `check_show_types_bind` — bind ごとの型が表示される
- [ ] D-9: `check_show_types_w006_marked` — W006 対象の bind に `← W006` が付く
- [ ] D-10: `version_is_12_5_0` — `CARGO_PKG_VERSION == "12.5.0"`
- [ ] D-11: `cargo test v12500 -- --nocapture` — 10 件通過確認

---

## Phase E — バージョン更新・コミット

- [ ] E-1: `fav/Cargo.toml` version → `"12.5.0"`
- [ ] E-2: `cargo test` — 全通過確認（1376 件 + 新規 10 件 = 1386 件程度）
- [ ] E-3: `git commit -m "feat: v12.5.0 — fav run --verbose + fav check --json / --show-types"`
- [ ] E-4: `git push` → CI 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav run --verbose` で stage / bind のトレースが stderr に出る | |
| `fav run --trace` で値の長さ制限なし出力 | |
| `fav check --json` で JSON 形式のエラー出力 | |
| `fav check --show-types` で bind ごとの推論型表示 | |
| `--json --show-types` 組み合わせで `bindings` フィールド付き JSON | |
| `cargo test v12500` 10 件通過 | |
| `cargo test` 全通過 | |
