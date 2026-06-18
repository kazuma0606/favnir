# v18.1.0 — エフェクト推論（Effect Inference）タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/middle/checker.rs` — `EffectSet` / `ns_to_effect` / `collect_effects_from_expr` 追加

- [x] `pub type EffectSet = std::collections::HashSet<Effect>` を追加
- [x] `fn ns_to_effect(ns: &str) -> Option<Effect>` を追加（Postgres/IO/S3/Kafka/Snowflake/BigQuery/Http/Llm）
- [x] `fn collect_effects_from_expr(expr: &Expr, out: &mut EffectSet)` を追加（再帰的 expr 走査）
- [x] `pub fn infer_effects_fn(fn_def: &FnDef) -> EffectSet` を追加

### T2: `fav/src/middle/checker.rs` — `fn_effects_registry` と推移的推論

- [x] `Checker` struct に `fn_effects_registry: HashMap<String, EffectSet>` フィールドを追加
- [x] `register_item_signatures` の FnDef ハンドラで `infer_effects_fn` を呼び登録
- [x] `call_graph: HashMap<String, Vec<String>>` を収集（fn ボディ中の関数呼び出しを記録）
- [x] `fn propagate_transitive_effects(registry, call_graph)` を実装（fixpoint 最大 10 ラウンド）
- [x] `check_fn_def` でエフェクト宣言がない場合に推論結果を適用
- [x] 明示宣言との整合性検査: 推論 ⊄ 明示 → E0336、推論 ⊊ 明示 → W010

### T3: `fav/src/driver.rs` — `--show-effects` と W010 出力

- [x] `cmd_check` のシグネチャに `show_effects: bool` を追加
- [x] `show_effects` が true の場合に `fn_effects_registry` の内容を表示
- [x] W010 警告（余分なエフェクト宣言）を `format_warnings` で出力
- [x] `fav/src/main.rs` で `--show-effects` フラグを解析して `cmd_check` に渡す

### T4: `self/checker.fav` — Favnir 実装追加

- [x] `ns_to_effect_str(ns: String) -> String` を追加
- [x] `infer_effects_from_stmts(stmts: List<Stmt>) -> List<String>` を追加

### T5: `fav/src/driver.rs` — `v181000_tests` 追加

- [x] `v180000_tests` の `version_is_18_0_0` テストを削除
- [x] `infer_effects_from_src(src, fn_name) -> EffectSet` ヘルパー関数を追加
- [x] `v181000_tests` モジュールを追加（5件）:
  - [x] `version_is_18_1_0`
  - [x] `effect_inference_db`（Postgres.query_raw → !Db）
  - [x] `effect_inference_multi`（Postgres + IO → !Db !IO）
  - [x] `effect_inference_pure`（副作用なし → 空集合）
  - [x] `effect_inference_transitive`（!Db 持つ fn を呼ぶ fn にも !Db）

### T6: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.0.0` → `18.1.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T7: `site/content/docs/language/effect-inference.mdx` 作成

- [x] エフェクト推論の概要（現状との比較コード例）を記載
- [x] ネームスペース → エフェクト マッピング表を記載
- [x] 明示宣言との共存方法を記載
- [x] `fav check --show-effects` の使い方を記載
- [x] W010 警告の説明を記載

---

## テスト（v181000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_1_0` | Cargo.toml に "18.1.0" が含まれる |
| `effect_inference_db` | `Postgres.query_raw` を含む fn に `!Db` が推論される |
| `effect_inference_multi` | `Postgres.*` と `IO.*` → `!Db !IO` が推論される |
| `effect_inference_pure` | 副作用なし fn のエフェクトが空集合 |
| `effect_inference_transitive` | `!Db` を持つ fn を呼ぶ fn にも `!Db` が推論される |

---

## 完了条件チェックリスト

- [x] `fav/Cargo.toml` のバージョンが `18.1.0`
- [x] `infer_effects_fn` が `fn_def` からエフェクトを正しく収集する
- [x] `fn_effects_registry` に推移的エフェクトが登録される
- [x] エフェクト宣言なしの fn が `Postgres.*` を使っても `fav check` が通る
- [x] `fav check --show-effects` が推論エフェクトを表示する
- [x] W010 警告が余分なエフェクト宣言に対して出力される
- [x] `site/content/docs/language/effect-inference.mdx` が存在する
- [x] `cargo test v181000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（checker.rs 基盤）
T2（推移的推論）          ← T1 完了後
T3（driver.rs）           ← T2 完了後
T4（checker.fav）         ← T1 と並列可
→ T5（v181000_tests）    ← T1〜T4 すべて完了後
T6（バージョン更新）      ← T5 完了後
T7（ドキュメント）        ← T6 と並列可

T1〜T4 のうち T4 は T1 と並列実施可能。T5 のみ最後に実施。
