# Favnir v7.7.0 Tasks

Date: 2026-05-28
Theme: checker.fav 基本機能パリティ（エフェクト追跡 / builtin 全登録 / エラーコード / match 網羅性）

---

## Phase A: エフェクト追跡（checker.fav）

- [x] A-1: `FnDef` に `effects: List<String>` フィールドを追加
- [x] A-2: `ns_to_effect(ns: String) -> String` — IO/Compiler→"IO", Cache/Queue/Email→各名前
- [x] A-3: `infer_expr_effects(expr: Expr) -> List<String>` — 式から使用エフェクトを収集
- [x] A-4: `infer_arms_effects(arms: Expr) -> List<String>` — match arms のエフェクト収集
- [x] A-5: `has_effect(effects: List<String>, eff: String) -> Bool`
- [x] A-6: `check_effects_all(declared: List<String>, inferred: List<String>) -> Option<String>` — E0003
- [x] A-7: `check_fn_def` にエフェクトチェックを統合

---

## Phase B: builtin 名前空間全登録（checker.fav）

- [x] B-1: `io_fn(fname)` — argv / read_file_raw / write_stdout_raw / write_stderr_raw / exit_raw / list_dir_raw / file_stat_raw / path_join_raw / home_dir_raw / cwd_raw / is_dir_raw / println
- [x] B-2: `compiler_fn(fname)` — check_raw / lineage_text_raw
- [x] B-3: `cache_fn(fname)` — get_raw / set_raw / del_raw / exists_raw / del_prefix_raw
- [x] B-4: `queue_fn(fname)` — send_raw / recv_raw / ack_raw / delete_raw
- [x] B-5: `email_fn(fname)` — send_raw
- [x] B-6: `str_fn` 拡張 — starts_with / ends_with / trim / upper / lower / to_upper / to_lower / join / repeat / replace / split / compare / to_bytes
- [x] B-7: `list_fn` 拡張 — fold / fold_left / find / partition / empty / sort_by / zip_with / group_by / intersperse / zip / chunk
- [x] B-8: `map_fn` 拡張 — empty / contains_key / delete / to_list / from_entries / merge_with / count_by
- [x] B-9: `int_fn` 拡張（compare / band / shr / bor）/ `float_fn` 追加（to_string / from_int）
- [x] B-10: `builtin_ret_ty` に `Compiler` / `Cache` / `Queue` / `Email` / `Float` 分岐を追加

---

## Phase C: エラーコード（checker.fav）

- [x] C-1: `fmt_err(code: String, msg: String) -> String` を追加
- [x] C-2: `infer_op` の arithmetic エラーを E0001 に変更
- [x] C-3: `infer_op` の logical エラーを E0002 に変更
- [x] C-4: `check_effects_all` のエラーを E0003 に変更（A-6 と同時実装）
- [x] C-5: `check_match_exhaustive` のエラーを E0004 に変更（D-5 と同時実装）

---

## Phase D: match 網羅性チェック（checker.fav）

- [x] D-1: `pat_ctor_name(pat: Pat) -> String` — パターンのコンストラクタ名を返す
- [x] D-2: `collect_arm_ctors(arms: Expr) -> List<String>` — match arms のコンストラクタ収集
- [x] D-3: `eq_str_ctor` / `list_contains` / `has_wildcard_ctor` ヘルパー追加
- [x] D-4: `check_option_exhaustive(ctors: List<String>) -> Bool`
- [x] D-5: `check_result_exhaustive(ctors: List<String>) -> Bool`
- [x] D-6: `check_match_exhaustive(scrut_ty: String, arms: Expr) -> Option<String>` — E0004
- [x] D-7: `infer_expr` の `EMatch` ハンドラを `check_match_exhaustive` 呼び出しに更新

---

## Phase E: テスト

### checker.fav 内テスト（9 件追加）

- [x] E-1: `fmt_err format` — E0001: bad 形式確認
- [x] E-2: `effect io detected` — ECall("IO", ...) → ["IO"]
- [x] E-3: `effect cache detected` — ECall("Cache", ...) → ["Cache"]
- [x] E-4: `effect none for pure` — ELit(LInt) → []
- [x] E-5: `builtin io list_dir_raw` — "Result"
- [x] E-6: `builtin cache get_raw` — "Option"
- [x] E-7: `builtin compiler check_raw` — "Result"
- [x] E-8: `match option exhaustive ok` — None + Some → None エラー
- [x] E-9: `match option missing some` — None のみ → E0004

### driver.rs 統合テスト（3 件追加）

- [x] E-10: `checker_fav_effect_tracking_test` — IO エフェクト違反 → E0003
- [x] E-11: `checker_fav_builtin_coverage_test` — Cache.get_raw → "Option"
- [x] E-12: `checker_fav_exhaustiveness_test` — Option 非網羅 → E0004

---

## Phase F: 最終確認・ドキュメント

- [x] F-1: `fav check fav/self/checker.fav` — no errors
- [x] F-2: `cargo test` — 1102+ tests passing（+12 新規）
- [x] F-3: `site/content/docs/language/self-host-checker.mdx` 作成
- [x] F-4: このファイルを完了状態に更新
- [x] F-5: commit

---

## 完了条件

- `fav/self/checker.fav` が `fav check` を通る
- 統合テスト 12 件追加済み
- 既存テスト 1091 件が全件通る（1102+ passing）
- ドキュメント 1 ページ追加

---

## 実装ノート（既知の制約）

- `bind inside closure 不可` → `list_contains` 等のヘルパーを外部関数として定義し `|x| eq_str_ctor(x, s)` のように使う
- `else if` 非対応 → `else { if ... }` + 閉じ括弧数に注意
- `FnDef.effects` 追加後、既存テスト内の全 FnDef リテラルに `effects: List.empty()` 追加が必要
- `infer_arms` のシグネチャは変更せず、`check_match_exhaustive` は `infer_expr` の EMatch ハンドラから直接呼ぶ
- `infer_expr_effects` は `-> List<String>` を返す純粋関数（Result でラップしない）
- `collect_arm_ctors` の `bind ctor <- pat_ctor_name(pat)` は単なる let バインド（pat_ctor_name が String を返すため）
- `List.empty()` は v7.2.0 追加済み
