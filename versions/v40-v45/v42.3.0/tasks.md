# v42.3.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2883（前バージョン 2880 + 3）
**実績テスト数**: 2883（v42300_tests 3/3 PASS）

---

## T0 — 事前確認

- [x] `cargo test` が 2880 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.2.0` であることを確認
- [x] `error_catalog.rs` に `E0420` が存在しないことを確認
- [x] `checker.rs` の `CepPatternDef` Pass 2 スタブ行番号を記録（line 2413）
- [x] `v42200_tests::cargo_toml_version_is_42_2_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録（line 44650）
- [x] `v42200_tests` の閉じ `}` の行番号を確認し記録（line 44698）
- [x] `check_cep_pattern_def` 関数が存在しないことを確認
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.4.0 エントリが存在することを確認

---

## T1 — `error_catalog.rs` 更新

- [x] E0406 の直後・E05xx セクションの直前に E0420 エントリを追加
  - code: `"E0420"`, category: `"types"`
  - title: `"cep pattern within_secs must be positive"`

---

## T2 — `checker.rs` 更新

- [x] Pass 2 `Item::CepPatternDef(_) => {}` スタブを `self.check_cep_pattern_def(cd)` に変更
- [x] `check_cep_pattern_def(&mut self, cd: &CepPatternDef)` メソッドを追加
  - `clause.within_secs == Some(0)` → `TypeError::new("E0420", ..., clause.span.clone())` を `push`

---

## T3 — `checker.fav` 設計コメント更新

- [x] `fav/self/checker.fav` の v42.1.0 追加コメント（「v42.3.0 以降に実装予定」）を「E0420 実装済み（within_secs == 0 の検証）」に更新

---

## T4 — `driver.rs` 更新

- [x] `v42200_tests::cargo_toml_version_is_42_2_0` をスタブ化（先に実施）
- [x] `v42300_tests` モジュール（3 テスト）を `v42200_tests` の直前に追加
  - `cargo_toml_version_is_42_3_0`（NOTE コメント付き）
  - `cep_e0420_within_zero`（errors.len()==1、errors[0].code=="E0420"）
  - `e0420_in_error_catalog`（`error_catalog::lookup("E0420").is_some()`）

---

## T5 — Cargo.toml バージョン bump

- [x] `version = "42.2.0"` → `"42.3.0"`

---

## T6 — CHANGELOG.md 更新

- [x] `[v42.3.0]` エントリを `[v42.2.0]` の直前に追加

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2883 を確認（2880 + 3 件）
- [x] `v42300_tests` 3 件 pass を確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.3.0（最新安定版）・v42.4.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.3.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）` を追記）
- [x] `versions/v40-v45/v42.3.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要
- [x] **site/ MDX 追加**: cookbook は v42.8.0 で対応予定のため本バージョンは不要

---

## 最終ステータス

- [x] 全タスク完了

## コードレビュー指摘・対応記録（spec-reviewer）

- [HIGH-1]: spec.md §2 に `Option<i64>` への比較であることの説明を追記
- [HIGH-2]: roadmap v42.3.0 の「型変数を checker.fav で検証」記述をセマンティクス検証（E0420）に修正、イベント型環境検証を v44.x 延期と明記
- [MED-1]: spec.md §2 に `clause.span: Span` の型確認注釈を追記
- [MED-2]: tasks.md T3 に `fav/self/checker.fav` のパスを明記
- [MED-3]: plan.md `cep_e0420_within_zero` に `Int(0)` パース成功の注釈を追加
- [LOW-1]: plan.md T4 にスタブ化先行の注意を追加
- [LOW-2]: tasks.md T0 に v42.4.0 の roadmap 存在確認を追加
