# v43.2.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2907（前バージョン 2903 + 4）
※ code-reviewer [HIGH] 対応で E0410 E2E テストを 1 件追加（当初予定 3 件 → 4 件）
※ ロードマップ記載の「推定 2898」は旧推定（v43.0 前の誤差）。v43.1.0 実績 2903 を起点とする。

---

## T0 — 事前確認

- [x] `cargo test` が 2903 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `43.1.0` であることを確認
- [x] `error_catalog.rs` に E0410/E0411 が存在しないことを確認
- [x] `checker.fav` の `check_body_ty` に E0410 が存在しないことを確認

---

## T1 — `fav/src/error_catalog.rs` 変更

- [x] E0406 エントリの直後、`// ── E042x: CEP パターン` コメントの直前に E041x セクションを挿入
- [x] `E0410` エントリ追加（ambiguous return type）
- [x] `E0411` エントリ追加（inferred return type mismatch）

---

## T2 — `fav/self/checker.fav` 変更

- [x] `check_body_ty` の `type_expr_to_str(ret) == ""` 分岐内に E0410 パスを追加
  - `apply_subst(r.subst, r.ty) == "Unknown"` → `Result.err(fmt_err("E0410", ...))`
  - コメント: `// v43.2.0: if body infers Unknown, E0410 (ambiguous return type)`

---

## T3 — `fav/src/driver.rs` — 構造体・関数・show_types 拡張

- [x] `FnReturnInfo` struct を `BindingInfo` の直後に追加
- [x] `collect_fn_inferred_return_types` 関数を `collect_binding_types` の直後に追加
  - `FnDef.return_ty.is_none()` の関数を収集
- [x] `cmd_check` の `show_types` ブロックを拡張（fn inferred return type 行を追加）

---

## T4 — `fav/src/driver.rs` — v43200_tests 追加

- [x] `v43100_tests` の直前に `v43200_tests` モジュールを挿入
- [x] `cargo_toml_version_is_43_2_0` テスト追加（NOTE コメント付き）
- [x] `e0410_e0411_in_error_catalog` テスト追加
- [x] `checker_fav_check_body_ty_has_e0410` テスト追加

---

## T5 — `fav/Cargo.toml` バージョン bump

- [x] `version = "43.1.0"` → `"43.2.0"`
- [x] `v43100_tests::cargo_toml_version_is_43_1_0` をスタブ化（空ボディ）

---

## T6 — `CHANGELOG.md` 更新

- [x] `[v43.2.0]` エントリを `[v43.1.0]` の直前に追加
- [x] E0410/E0411、checker.fav 変更、driver.rs 変更内容を記載

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2906 を確認（2903 + 3 件）
- [x] `v43200_tests` 3 件 pass を確認
- [x] 既存テストが壊れていないことを確認
- [x] `fav check --show-types <戻り値型省略の .fav ファイル>` を実行し fn 行（`fn double : (return type inferred from body)`）が出力されることを確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v43.2.0（最新安定版、2906 tests）・v43.3.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` の v43.2.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v43.2.0 推定テスト数を `2898` → 実績 `2906` に修正
- [x] `versions/v40-v45/v43.2.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [MED-1]: E0411 description がロードマップの「省略型と明示型の不一致」と意味がずれていた（呼び出し側との不一致になっていた） → spec.md の E0411 description をロードマップ定義に合わせて修正。コメントで「省略型と明示型の不一致」の意味を明示。
- [MED-2]: `FnReturnInfo` に `serde::Serialize` がなく将来の JSON 統合時に混乱する可能性 → spec.md 影響範囲セクションに「FnReturnInfo は `#[derive(Debug)]` のみ、テキスト専用。JSON 統合は v43.9.0 以降」と追記。
- [LOW-1]: `--json` フラグとの併用時挙動が未定義 → spec.md 影響範囲に「json=true の場合は既存 JSON パスを経由するため collect_fn_inferred_return_types は呼ばれない」と追記。
- [LOW-2]: T7 に `--show-types` 実動作確認チェックボックスがなかった → T7 にチェックボックス追加。

## code-reviewer 指摘・対応記録

- [HIGH]: `checker_fav_check_body_ty_has_e0410` が文字列存在確認のみで E0410 の実際の発火を検証していない → `return_type_omission_e0410_triggered` テストを追加（`collect { () }` で Unknown 型を誘導し E0410 発火を確認）。テスト数 2906 → 2907。
- [MED-1]: E0407〜E0409 欠番の予約コメントがなく番号衝突リスク → E041x セクション前に `E0407〜E0409: 予約` コメントを追加。
- [MED-2]: E0411 description に `(v43.3.0+ で検出開始)` という実装状況メモが混入 → description からメモを削除し、コード上のコメントに移動。
- [MED-3]: `--show-types` での二重パース → TODO コメントを追加（v43.9.0 --show-inference で統合予定）。
- [LOW]: `include_str!("../src/error_catalog.rs")` を他テストと統一して `include_str!("error_catalog.rs")` に変更。
- [LOW (却下)]: `v43200_tests` の出現順がバージョン昇順に反するという指摘 → プロジェクト慣例は降順（新バージョンが先）のため対応不要。
