# v42.5.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2888（前バージョン 2886 + 2）
**実績テスト数**: 2889（v42500_tests 3/3 PASS — code-reviewer 指摘で `max_inflight_zero_is_parse_error` を追加）

---

## T0 — 事前確認

- [x] `cargo test` が 2886 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.4.0` であることを確認
- [x] `ast.rs` の `CircuitBreakerAnnotation` 末尾行番号を記録（line 606）
- [x] `ast.rs` の `TrfDef.circuit_breaker` フィールド行番号を記録（line 648）
- [x] `ast.rs` の `parse_trf_def` 内 `circuit_breaker: None,` 行番号を記録（line 2098）
- [x] `parser.rs` の `parse_circuit_breaker_annotation` 末尾行番号を記録（line 590）
- [x] `parser.rs` の `let circuit_breaker_ann = ...` 行番号を記録（line 611）
- [x] `parser.rs` の `td.circuit_breaker = circuit_breaker_ann;` 出現 2 か所の行番号を記録（line 643, 670）
- [x] `driver.rs` の `v42400_tests` 閉じ `}` 行番号を記録（line 44646）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.5.0 エントリが存在することを確認

---

## T1 — `ast.rs` — `MaxInflightAnnotation` 構造体追加

- [x] `CircuitBreakerAnnotation` の直後に `MaxInflightAnnotation { n: u64, span: Span }` を追加

---

## T2 — `ast.rs` — `TrfDef.max_inflight` フィールド追加

- [x] `circuit_breaker: Option<CircuitBreakerAnnotation>` の直後に `max_inflight: Option<MaxInflightAnnotation>` を追加

---

## T3 — `parser.rs` — `parse_max_inflight_annotation()` 追加

- [x] `parse_circuit_breaker_annotation()` の直後に `parse_max_inflight_annotation()` を追加
- [x] n <= 0 の場合は ParseError を返す

---

## T4 — `parser.rs` — `parse_item` に呼び出し追加

- [x] `let circuit_breaker_ann = ...` の直後に `let max_inflight_ann = self.parse_max_inflight_annotation()?;` を追加

---

## T5 — `parser.rs` — `td.max_inflight` 代入追加（2 か所）

- [x] `TokenKind::Stage` アームの `td.circuit_breaker = ...` 直後に `td.max_inflight = max_inflight_ann;`
- [x] `TokenKind::Async + Stage` アームの `td.circuit_breaker = ...` 直後に `td.max_inflight = max_inflight_ann;`

---

## T6 — `parser.rs` — `parse_trf_def` に `max_inflight: None` 追加

- [x] `circuit_breaker: None,` の直後に `max_inflight: None,` を追加

---

## T7 — `driver.rs` — `v42500_tests` モジュール追加

- [x] `v42400_tests` の閉じ `}` の直前（降順配置）に `v42500_tests` を挿入
- [x] `cargo_toml_version_is_42_4_0` をスタブ化（先に実施）
- [x] `cargo_toml_version_is_42_5_0`（NOTE コメント付き）
- [x] `max_inflight_annotation_parses`（n == 100 を assert）

---

## T8 — Cargo.toml バージョン bump

- [x] `version = "42.4.0"` → `"42.5.0"`

---

## T9 — CHANGELOG.md 更新

- [x] `[v42.5.0]` エントリを `[v42.4.0]` の直前に追加

---

## T10 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2888 を確認（2886 + 2 件）
- [x] `v42500_tests` 2 件 pass を確認

---

## T11 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.5.0（最新安定版）・v42.6.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.5.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] `versions/v40-v45/v42.5.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: ロードマップ v42.5.0 が「VM 追加・一時停止」と明記したまま → roadmap に延期注釈追加（`#[max_inflight(...)]` 構文修正 + ※ v44.x 延期注釈）
- [MED-1]: アノテーション構文が位置引数形式で既存の名前付き引数形式と異なる理由が未説明 → spec §構文に「引数が 1 つのため名前省略」の説明を追記
- [MED-2]: ロードマップの v42.4.0 実績値未記録（テスト数起点の検証） → tasks.md T0 item 1 で実測確認する手順を維持（問題なし）
- [MED-3]: テスト用ソースの型シグネチャ `List -> Unit` が不明瞭 → `List -> List` に修正し注釈追加
- [LOW-1]: `fav fmt` がアノテーションを出力しない（ラウンドトリップ破損）が非スコープに未記載 → spec §非スコープに「fav fmt 実行でアノテーションが消える、v44.x で対応」を追記
- [LOW-2]: tasks T7 と plan T7 の挿入位置記述が不一致 → tasks T7 を「v42400_tests の閉じ `}` の直前」に統一

## code-reviewer 指摘・対応記録

- [MED-1]: ネガティブテスト未整備（`#[max_inflight(0)]` がエラーになることを担保するテストがない） → `max_inflight_zero_is_parse_error` テストを追加（2889 テスト）
- [MED-2]: `raw <= 0` ガードの実効範囲が不明瞭（負数は `Minus` + `Int(1)` にトークン化されるため実質 `raw == 0` のみカバー） → parser.rs に説明コメント追加
- [LOW]: `fav fmt` がアノテーションを出力しないラウンドトリップ破損が CHANGELOG に未記載 → CHANGELOG `[v42.5.0]` に Known Limitations セクションを追加
