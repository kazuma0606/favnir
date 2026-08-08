# v60.9.0 Spec — 安定化・DX チェックリスト

Date: 2026-07-31
Status: COMPLETE

---

## 概要

v60.1〜v60.8 で追加した全 DX 機能が統合されていることを確認する安定化スプリント。
新規機能追加は行わず、**統合テスト 2 件の追加**のみを実施する。

---

## 検証対象機能（ロードマップ確認項目）

| 確認項目 | 実装バージョン | 検証方法 |
|---|---|---|
| `fav check --fix` 出力と LSP span 情報の一致 | v60.2 / v60.4 | `dx_e2e_check_fix_lsp_consistent` テスト |
| `.favfmt` 設定が `fav fmt` で読み込まれる | v60.7 | `FmtConfig::from_toml_str` の動作確認（v60.7 で既存テスト済み） |
| REPL で `:load` → `:debug` → マルチライン入力 E2E | v60.5 | `dx_repl_pipeline_e2e` テスト |
| `fav explain-error` が `long_description` を表示する | v60.6 | `dx_e2e_check_fix_lsp_consistent` 内で `cmd_explain_error_collect` 確認 |
| `fav doc --format html` が動作する | v60.8 | `cmd_doc_html_str` の動作確認（v60.8 で既存テスト済み） |

---

## テスト仕様

### `dx_e2e_check_fix_lsp_consistent`

v60.2（`fav check --fix`）と v60.6（`fav explain-error` + `long_description`）の統合を確認する。

```
1. cmd_check_fix_src で typo のないクリーンなソースを処理
   → fix 出力が空または "0 fixes" 相当であることを確認

2. cmd_explain_error_collect("E0102") を呼び出す
   → Some を返すことを確認
   → 出力に "Long Description" セクションが含まれることを確認（v60.6.0 の成果）

使用関数: cmd_check_fix_src / cmd_explain_error_collect
```

### `dx_repl_pipeline_e2e`

v60.5（REPL `:load` / `:debug` / マルチライン）の統合を確認する。

```
1. tempdir に pipeline.fav を作成（stage AddOne）
2. handle_load_cmd → session.def_names に "AddOne" が含まれることを確認
3. handle_debug_cmd("AddOne", &session) → 出力に "AddOne" が含まれることを確認
4. needs_continuation のマルチライン判定を確認:
   - "bind x <-\\" → true（バックスラッシュ継続）
   - "stage S: Int -> Int = |x| {" → true（未閉じブレース）
   - "bind x <- 42" → false（完結した行）

使用関数: handle_load_cmd / handle_debug_cmd / needs_continuation / ReplSession
```

---

## ベーステスト数の注意点

ロードマップ記載の「ベース 3346 + 2 = 3348」は v60.8.0 実装前の想定値。
v60.8.0 の実装では XSS テスト追加（`doc_rune_description_xss_escaped`）により 3347 になった。

実際のテスト数目標: **3347 + 2 = 3349** tests passed, 0 failed

---

## 完了条件

- `dx_e2e_check_fix_lsp_consistent` pass
- `dx_repl_pipeline_e2e` pass
- 総テスト数: **3349** tests passed, 0 failed
- `cargo build` でコンパイルエラーなし
