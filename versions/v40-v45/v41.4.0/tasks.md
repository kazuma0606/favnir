# v41.4.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2859（前バージョン 2856 + 3）
**実績テスト数**: 2859

---

## T0 — 事前確認

- [x] `cargo test` が 2856 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.3.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.4.0 を確認
- [x] `v41300_tests::cargo_toml_version_is_41_3_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録（44584行）
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `ast_lower_checker.rs` の `lower_arms` 関数行番号を確認（247行）
- [x] `ast_lower_checker.rs` の `v3` ヘルパー行番号を確認（35行）
- [x] `checker.fav` の `EArm(Pat, Expr, Expr)` 行番号を確認（48行）
- [x] `checker.fav` の `EArmNil` 行番号を確認（49行）
- [x] `checker.fav` の `infer_arms_effects` 行番号を確認（1206行）
- [x] `checker.fav` の `check_rebind` の EArm ケース行番号を確認（1482行）
- [x] `checker.fav` の `check_w006_arms` 行番号を確認（1536行）
- [x] `checker.fav` の `infer_arms` 行番号を確認（1603行）
- [x] `checker.fav` の `collect_arm_ctors` 行番号を確認（1642行）
- [x] `checker.fav` の `str_eq` 関数が存在することを確認（`collect_arm_ctors` で使用）
- [x] `v3` ヘルパーの定義（HashMap レコードパターン）を確認し、`v4` の実装形式を決定

---

## T1 — ast_lower_checker.rs: v4 ヘルパー追加

- [x] `v3` ヘルパーの直後に `v4` を追加（HashMap レコードパターン、`_0`〜`_3` フィールド）

---

## T2 — ast_lower_checker.rs: lower_arms 変更

- [x] `lower_arms` を変更: `arm.guard.is_some()` の場合 `v4("EArmG", ...)` を emit

---

## T3 — checker.fav: EArmG バリアント追加

- [x] `Expr` 型の `EArmNil` 直後に `| EArmG(Pat, Expr, Expr, Expr)  // v41.4.0: (pat, guard_expr, body, rest)` を追加

---

## T4 — checker.fav: infer_arms_effects に EArmG ケース追加

- [x] EArm ケース直後に EArmG ケースを追加（guard + body + rest のエフェクトを収集）

---

## T5 — checker.fav: check_rebind に EArmG ケース追加

- [x] EArm ケース直後に EArmG ケースを追加（guard → body → rest の順でチェック）

---

## T6 — checker.fav: check_w006_arms に EArmG ケース追加

- [x] EArm ケース直後に EArmG ケースを追加（guard + body + rest の W006 チェック）

---

## T7 — checker.fav: infer_arms に EArmG ケース追加

- [x] EArm ケース直後に EArmG ケースを追加（body の型推論、EArm と同じロジック）

---

## T8 — checker.fav: collect_arm_ctors に EArmG ケース追加（網羅性チェックのコア）

- [x] EArm ケース直後に EArmG ケースを追加
- [x] ガード付き `_`（PVar/PWild）は `collect_arm_ctors(rest)` のみ（catch-all カウントしない）
- [x] ガード付きコンストラクタは `List.push` する（ctor カバレッジには寄与）
- [x] `str_eq` が `collect_arm_ctors` のスコープで参照可能であることを確認

---

## T9 — driver.rs テストモジュール更新

- [x] `v41300_tests::cargo_toml_version_is_41_3_0` をスタブ化（assert 本体を削除）
- [x] `v41400_tests` モジュール（3 テスト）を末尾に追加

---

## T10 — Cargo.toml バージョン bump

- [x] `version = "41.3.0"` → `"41.4.0"`

---

## T11 — CHANGELOG.md 更新

- [x] `[v41.4.0]` エントリを `[v41.3.0]` の直前に追加

---

## T12 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2859 を確認（実績: 2859）
- [x] `v41400_tests` 3 件すべて pass を確認
- [x] 既存の `EArm` を使うパス（Option/Result exhaustiveness）が壊れていないことを確認

---

## T13 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.4.0（最新安定版）・v41.5.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.4.0 を COMPLETE にマーク
- [x] `versions/v40-v45/v41.4.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

| 優先度 | 内容 | 対応 |
|---|---|---|
| [MED] | `infer_arms` EArmG ケースで guard 式を `infer_expr` に通していない（未定義変数等が検出されない） | `infer_expr(guard, pat_env)` を追加し、エラーを伝播。Bool 強制は v41.5.0+ 延期のコメントも追記 |
| [LOW] | `collect_arm_ctors` EArmG ケースの `pat_ctor_name` 依存が暗黙的 | NOTE コメントで `pat_ctor_name` が PVar/PWild 両方に `"_"` を返す仕様依存を明示 |

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] 3件・[MED] 3件 → 全対応）
- [x] code-reviewer 指摘対応済み（[MED] 1件・[LOW] 1件 → 全対応）
