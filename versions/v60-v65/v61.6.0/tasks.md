# v61.6.0 タスクリスト

Status: COMPLETE
Version: 61.6.0
Base tests: 3369
Target tests: 3371

---

## T0: 事前確認

- [x] `cargo test 2>&1 | tail -5` でベース 3369 tests passed, 0 failed を確認
- [x] `checker.rs` の `Checker` 構造体フィールド名（`type_defs`, `record_fields` 等）を確認
- [x] `TypeBody` の実際の定義を確認（`TypeBody::Record(Vec<Field>)` where `Field.ty: TypeExpr`）
- [x] `ty_to_str` / `type_to_string` → 実際は `Type::display()` メソッドを使用
- [x] E0009 を発行している call site を grep で全量列挙（checker.rs には E0009 なし、E0103 が target）
- [x] `type_error_h` のシグネチャを確認（`hints: Vec<String>` — L1529）
- [x] `v61500_tests` モジュールが driver.rs に存在することを grep で確認（L48581）

**重要発見**:
- E0009 は Rust checker.rs では発行されない（self-hosted checker.fav が発行）
- 実装対象は E0103 (`Expr::Pipeline` の L5122-5137 call site)
- `record_fields: HashMap<String, Vec<(String, Type)>>` が最適（`type_defs` ではなく）
- `diff_types` は `Type::Named` vs scalar / List element diff のみカバー（`unify` 直前に追加）

---

## T1: error_catalog.rs — E0103 `long_description` 更新

- [x] `error_catalog.rs` E0103 エントリを確認
- [x] `long_description` を差分表示対応テキストに更新（構造的差分 hint の説明を追記）

---

## T2: checker.rs — `diff_types` 関数追加

- [x] `diff_types(expected: &Type, found: &Type, record_fields: &HashMap<String, Vec<(String, Type)>>) -> Option<String>` を追加
  - Named vs スカラー: `record_fields` を参照してフィールド一覧を表示
  - Named vs Named: missing fields diff
  - `List<A>` vs `List<B>`: 要素型を再帰的に差分表示
- [x] 関数の配置: `unify` 関数の直前（L368 付近）

---

## T3: checker.rs — E0103 call site を `type_error_h` に更新

- [x] `Expr::Pipeline` の E0103 call site（`Some((input, output))` アーム）を特定
- [x] `diff_types(&current, input, &self.record_fields)` を呼び出し
- [x] `type_error_h("E0103", msg, span, hints)` に変更

---

## T4: driver.rs — E0009 explain テキスト更新 + `v61600_tests` 追加

- [x] E0009 `fav explain` テキスト（L11474）に Record 型差分ヒントの説明を追加
- [x] `v61500_tests` モジュールの直前に `v61600_tests` を挿入
- [x] `type_error_diff_display_record` テスト追加
  - `r: Row` を `use_str(s: String)` に渡すパイプラインで E0103 + diff hint を確認
- [x] `type_error_suggestion_e0009` テスト追加
  - `get_explain_text("E0009")` が "Record" を含むことを確認

---

## T5: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v61600` で 2 件 PASS を確認
- [x] `cargo test -j 8 -- --test-threads=8` で 3371 tests passed, 0 failed を確認

---

## T6: ドキュメント更新

- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.6.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v61.6.0（3371 tests）に更新、「次」を v61.7.0 に
- [x] `CHANGELOG.md` に v61.6.0 エントリを追加（※別途実施）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

（実装後に記録）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3371 passed, 0 failed
- 完了日: 2026-08-01
