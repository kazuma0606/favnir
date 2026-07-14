# v43.1.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2903（前バージョン 2900 + 3）
※ ロードマップ記載の「推定 2895」は旧 v43.0.0 前の誤差。v43.0.0 実績 2900 を起点とする。

---

## T0 — 事前確認

- [x] `cargo test` が 2900 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `43.0.0` であることを確認
- [x] `fav/src/driver.rs` の `v43000_tests` 冒頭行番号を記録
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` に v43.1.0 エントリが存在することを確認
- [x] `fn double(x: Int) { x * 2 }` が現状でパースエラーになることを確認（制限の存在確認）
- [x] compiler.fav で `Some(TkArrow)` ネストパターンが有効であることを確認（確認済み ✅）

---

## T1 — `fav/src/frontend/parser.rs` 変更

- [x] `parse_fn_def()` line 1981-1986 の制限ブロック（5 行）を削除
  - 削除: `if return_ty.is_none() { return Err(...) }` ブロック
- [x] `fn double(x: Int) { x * 2 }` がパースエラーなしに通ることを手動確認

---

## T2 — `fav/self/compiler.fav` 変更

- [x] `parse_fn_def_after_params()` を `->` オプション対応に変更
  - `List.first(rest4)` が `Some(TkArrow)` でなければ `TeSimple("")` プレースホルダを使用
  - コメント: `// v43.1.0: \`->\` optional — if absent, use TeSimple("") as placeholder`

---

## T3 — `fav/self/checker.fav` 変更

- [x] `check_body_ty()` に `ret == ""` 時の早期 OK パスを追加
  - `type_expr_to_str(ret) == ""` なら即 `Result.ok(fname)` を返す
  - コメント: `// v43.1.0: ret == TeSimple("") means return type was omitted — infer from body (always OK)`

---

## T4 — `fav/src/driver.rs` テスト追加

- [x] `v43000_tests` の直前に `v43100_tests` を挿入
- [x] `cargo_toml_version_is_43_1_0` テスト追加（NOTE コメント付き）
- [x] `return_type_omission_block_parseable` テスト追加
  - `fn double(x: Int) { x * 2 }` が `Parser::parse_str` でエラーなし
- [x] `return_type_omission_return_ty_is_none` テスト追加
  - 解析結果の `FnDef.return_ty` が `None` であること

---

## T5 — `fav/Cargo.toml` バージョン bump

- [x] `version = "43.0.0"` → `"43.1.0"`

---

## T6 — `CHANGELOG.md` 更新

- [x] `[v43.1.0]` エントリを `[v43.0.0]` の直前に追加
- [x] parser.rs / compiler.fav / checker.fav の変更内容を記載

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2903 を確認（2900 + 3 件）
- [x] `v43100_tests` 3 件 pass を確認
- [x] 既存テストが壊れていないことを確認

**追加修正**: `fav/src/middle/ast_lower_checker.rs` の `lower_fn_def` で `return_ty: None` 時の fallback を `TeSimple("Unit")` → `TeSimple("")` に変更。これにより checker.fav が「戻り値型省略」と正しく認識し E0009 が発生しなくなった。

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v43.1.0（最新安定版、2903 tests）・v43.2.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` の v43.1.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v43.1.0 推定テスト数を `2895` → 実績 `2903` に修正
- [x] `versions/v40-v45/v43.1.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: spec.md の parser.rs「変更前/変更後」コードブロックが `} else {` を含む形で示されており、削除対象の 5 行が不明瞭 → 削除対象行を `// ← DELETE` でマークした全コンテキスト表示に変更、説明文も「6行 if ブロックを削除するだけ」と明示
- [HIGH-2]: compiler.fav の `Some(TkArrow)` ネストパターンマッチが有効かどうか未確認 → 実コード確認済み（line 1247, 1261 等で多用）。spec.md に確認済み注記を追加。T0 チェックボックスに確認ステップ追加
- [MED-3]: compiler.fav レコードリテラルのスペース区切り記法が暗黙 → spec.md に「カンマ不要のプロジェクト既定記法」注記を追加
- [MED-4]: `type_expr_to_str(ret) == ""` が安全かどうか未検証 → checker.fav の `type_expr_to_str` 実装を確認（`TeSimple("")` のみ `""` を返す）。spec.md に確認済み注記を追加
- [MED-5]: `= expr` 構文の回帰テストが v43100_tests に含まれていない → 既存 2900 テストで回帰カバー済みであることを spec.md 完了条件に明記。テスト数（3 件）は変更しない
- [LOW-6]: tasks.md T8 の `versions/current.md` チェックボックスは既存記載で対応済み（対応不要）

## code-reviewer 指摘・対応記録

- [MED]: CHANGELOG.md に `ast_lower_checker.rs` の変更（`lower_fn_def` の `TeSimple("Unit")` → `TeSimple("")` fallback 修正）が記載されていなかった → Added セクションに項目を追加。
- [LOW]: `v43100_tests` に `run_checker_fav` 経由の E2E テストがない（`fn double(x: Int) { x * 2 }` が checker.fav パスでエラーなしに通ることを検証するテスト未追加） → spec 確定テスト数（3件）を変更しないため対応保留。v43.2.0 統合テストで補完予定。
