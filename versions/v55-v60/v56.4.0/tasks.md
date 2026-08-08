# Tasks — v56.4.0 — エフェクト推論 LSP 統合（inlay hints 表示）

## ステータス: 未実施

---

## 事前確認（T0）

- [ ] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.4.0 セクションを確認
- [ ] ベーステスト数 3233（v56.3.0 完了時点の実績値）を確認
- [ ] `fav/Cargo.toml` が `56.3.0` であることを確認（更新前）
- [ ] `infer_effects_fn` が存在しないことを確認（v35.5.0 で削除済み）
- [ ] `propagate_transitive_effects` が no-op であることを確認（checker.rs L10544）
- [ ] `AMBIENT_NAMESPACES` が `lint.rs` L719 に定義されていることを確認
- [ ] `AMBIENT_GEN_FNS` が `lint.rs` L725 に定義されていることを確認
- [ ] `collect_ambient_in_expr` が非公開（`fn`、非 `pub`）であることを確認（参照用）
- [ ] `handle_inlay_hints` が 4 つのコレクター（bind/stage/fn_return/pipeline）を持つことを確認（追加対象）
- [ ] `collect_inference_annotations` が存在することを確認（driver.rs — 変更しない）
- [ ] `v56300_tests` に `cargo_toml_version_is_56_3_0` が存在することを確認（削除対象）
- [ ] `driver.rs` に `v56400_tests` が存在しないことを確認（新規追加対象）
- [ ] `infer_fn_effects` が `driver.rs` に存在しないことを確認（新規追加対象）
- [ ] `collect_fn_effect_hints` が `lsp/inlay_hints.rs` に存在しないことを確認（新規追加対象）

---

## 実装タスク

- [ ] T1: `fav/Cargo.toml` version を `56.4.0` に更新（56.3.0 から変更）
- [ ] T2: `fav/src/lint.rs` — `collect_used_namespaces` + 非公開ヘルパー追加
  - [ ] `pub fn collect_used_namespaces(block: &crate::ast::Block) -> Vec<String>` を `check_ambient_errors` 直後に追加
  - [ ] `fn collect_ns_in_block(block: &Block, found: &mut BTreeSet<String>)` を追加
  - [ ] `fn collect_ns_in_expr(expr: &Expr, found: &mut BTreeSet<String>)` を追加（spec.md の完全コードに従う）
  - [ ] `collect_ns_in_expr` が `collect_ambient_in_expr` と同じ AST 分岐を持つことを確認
- [ ] T3: `fav/src/lsp/inlay_hints.rs` — `collect_fn_effect_hints` 追加 + `handle_inlay_hints` 更新
  - [ ] `collect_pipeline_type_hints` 直後に `collect_fn_effect_hints(source: &str) -> Vec<InlayHint>` を追加（pub(crate)）
  - [ ] `handle_inlay_hints` に `hints.extend(collect_fn_effect_hints(&doc.source));` を追加（v56.4.0 コメント付き）
  - [ ] hint の label が `" /* !IO !Snowflake */"` 形式であることを確認
  - [ ] hint の position.line が `fd.span.line.saturating_sub(1)` であることを確認
- [ ] T4: `fav/src/driver.rs` — `infer_fn_effects` 追加
  - [ ] `collect_inference_annotations` 直後に `pub fn infer_fn_effects(src: &str) -> Vec<(String, Vec<String>)>` を追加
  - [ ] パース失敗時に空 vec を返す（checker 呼び出しなし）
- [ ] T5: `fav/src/driver.rs` — 既存テスト更新
  - [ ] `v56300_tests::cargo_toml_version_is_56_3_0` を削除
- [ ] T6: `fav/src/driver.rs` — `v56400_tests` モジュールを `v56300_tests` の直前に追加
  - [ ] `cargo_toml_version_is_56_4_0`
  - [ ] `effect_inference_inlay_hint`（`collect_fn_effect_hints` テスト — "IO" を含む hint の確認）
  - [ ] `effect_inference_check_output`（`infer_fn_effects` テスト — "IO" あり/なしの両方を検証）

---

## テスト・検証

- [ ] T7: `cargo build` でコンパイルエラーがないことを確認
- [ ] T8: `cargo test` 全通過（**3235 tests passed, 0 failed**）
  - [ ] `v56400_tests::cargo_toml_version_is_56_4_0` ok
  - [ ] `v56400_tests::effect_inference_inlay_hint` ok
  - [ ] `v56400_tests::effect_inference_check_output` ok
  - [ ] 既存 3233 件全通過
- [ ] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [ ] T10: `CHANGELOG.md` に v56.4.0 エントリを追加（version: `56.3.0 → 56.4.0`）
- [ ] T11: `versions/current.md` を v56.4.0 / 3235 tests に更新
- [ ] T12: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.4.0 実績を COMPLETE に更新
- [ ] T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.4.0 実績欄も COMPLETE に更新

---

## 完了確認

- [ ] `cargo_toml_version_is_56_4_0` pass
- [ ] `effect_inference_inlay_hint` pass（"IO" を含む InlayHint が生成される）
- [ ] `effect_inference_check_output` pass（IO.println fn は "IO"、pure fn は空）
- [ ] **3235 tests passed, 0 failed**
- [ ] `cargo clippy -- -D warnings` クリーン
- [ ] `lint.rs` に `collect_used_namespaces`（pub）が追加されている
- [ ] `lsp/inlay_hints.rs` に `collect_fn_effect_hints`（pub(crate)）が追加されている
- [ ] `handle_inlay_hints` に `collect_fn_effect_hints` 呼び出しが追加されている
- [ ] `driver.rs` に `infer_fn_effects`（pub）が追加されている
- [ ] `v56300_tests::cargo_toml_version_is_56_3_0` が削除されている
- [ ] `CHANGELOG.md` に v56.4.0 エントリが追加されている（version: `56.3.0 → 56.4.0`）
- [ ] `versions/current.md` が v56.4.0 / 3235 tests を反映
- [ ] T12 / T13 のロードマップ更新が完了している
