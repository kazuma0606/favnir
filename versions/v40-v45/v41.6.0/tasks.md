# v41.6.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2865（前バージョン 2862 + 3）
**実績テスト数**: 2865

---

## T0 — 事前確認

- [x] `cargo test` が 2862 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.5.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.6.0 を確認
- [x] `v41500_tests::cargo_toml_version_is_41_5_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 44626
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `checker.fav` の `collect_variant_constructors` 関数の `IWrapper(wd)` ケース行番号を確認: 2138
- [x] `checker.fav` の `infer_op` 関数の行番号を確認: 317
- [x] `checker.fav` の `EBinOp` ケース in `infer_expr` の行番号を確認: 1858
- [x] `checker.fav` に `op_to_str` 関数が存在するか確認: 存在しない → 簡易版（Float/Int のみ）を使用
- [x] **確認済み前提（変更不要）**: `let` 束縛は checker.fav で使用不可。ネスト `env_insert` を使うこと
- [x] **確認済み前提（変更不要）**: `env_insert` 戻り値は `List<KVPair>`（Result ではない）
- [x] **確認済み前提（変更不要）**: `collect_variant_constructors` 戻り値は `List<KVPair>`（Result ではない）

---

## T1 — checker.fav: collect_variant_constructors に newtype inner 登録追加

- [x] `IWrapper(wd)` ケースをネスト `env_insert` 形式に変更:
  ```favnir
  collect_variant_constructors(List.drop(items, 1),
      env_insert(
          env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, ret_ty)),
          String.concat("__newtype__", wd.name), wd.inner))
  ```

---

## T2 — checker.fav: `infer_op_with_newtypes` 追加

- [x] `infer_op` の直後（`fn io_fn` の前）に追加:
  ```favnir
  fn infer_op_with_newtypes(op: Op, lty: String, rty: String, env: List<KVPair>) -> Result<String, String> {
      if is_arith_op(op) && (lty == rty) {
          match env_lookup(env, String.concat("__newtype__", lty)) {
              Some(inner) =>
                  if (inner == "Float") || (inner == "Int") {
                      Result.ok(lty)
                  } else {
                      infer_op(op, lty, rty)
                  }
              None => infer_op(op, lty, rty)
          }
      } else {
          infer_op(op, lty, rty)
      }
  }
  ```

---

## T3 — checker.fav: EBinOp ハンドラー更新

- [x] `infer_op(op, lty, rty)` → `infer_op_with_newtypes(op, lty, rty, env)` に変更

---

## T4 — driver.rs テストモジュール更新

- [x] `v41500_tests::cargo_toml_version_is_41_5_0` をスタブ化
- [x] `v41600_tests` モジュール（3 テスト）を末尾に追加:
  - `cargo_toml_version_is_41_6_0`
  - `changelog_has_v41_6_0`
  - `checker_fav_has_newtype_arith`

---

## T5 — Cargo.toml バージョン bump

- [x] `version = "41.5.0"` → `"41.6.0"`

---

## T6 — CHANGELOG.md 更新

- [x] `[v41.6.0]` エントリを `[v41.5.0]` の直前に追加

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2865 を確認（実績: 2865）
- [x] `v41600_tests` 3 件すべて pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.6.0（最新安定版）・v41.7.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.6.0 を完了済みにマーク
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.6.0 の「推定 2858」を「2865」に修正（誤記 → 実装前に修正済み）
- [x] v41.7〜v42.0 のテスト数推定値も v41.5.0 実績（2862）を起点に再計算・修正済み
- [x] `versions/v40-v45/v41.6.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要

---

## コードレビュー指摘と対応

### [MED] `is_arith_op` に `OpMod` が含まれるため Newtype `%` 演算が通過する
- **指摘**: `is_arith_op` は OpAdd/OpSub/OpMul/OpDiv/OpMod の 5 つを true にするため、Newtype で `%` 演算が型チェックを通過する
- **対応**: `is_basic_arith_op`（OpMod を除く四則のみ）を追加し、`infer_op_with_newtypes` のガードを `is_basic_arith_op` に変更 ✅

### [LOW] `"Unknown"` 型両辺一致で不要な env ルックアップが発生
- **指摘**: `lty == rty == "Unknown"` の場合に `env_lookup(env, "__newtype__Unknown")` が呼ばれる（無害だがノイズ）
- **対応**: `infer_op_with_newtypes` のガードに `lty != "Unknown"` を追加 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み
- [x] code-reviewer 指摘対応済み
