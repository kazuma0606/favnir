# Tasks — v55.6.0 — CEP（複合イベント処理）Stream 統合

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.6.0 セクションを確認
- [x] ベーステスト数 3215（v55.5.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.5.0` であることを確認（更新前）
- [x] `fav/src/middle/compiler.rs` の namespace 登録リストに `"State"` が存在し、`"CEP"` が未登録であることを確認
- [x] `fav/src/middle/checker.rs` に `("Stream", "join_left")` エントリが存在することを確認（`CEP` 型登録の挿入位置）
- [x] `fav/src/middle/checker.rs` に `("CEP", "sequence")` が存在しないことを確認（新規追加）
- [x] `fav/src/backend/vm.rs` の `is_known_builtin_namespace` に `"State"` が存在し、`"CEP"` が未登録であることを確認
- [x] `fav/src/backend/vm.rs` の `call_builtin` に `Stream.join_left` アームが存在することを確認（CEP 挿入位置）
- [x] `fav/src/backend/vm.rs` に `CEP.sequence` が存在しないことを確認（新規追加）
- [x] `fav/src/driver.rs` の `v55500_tests` モジュール位置を確認（直前に `v55600_tests` を挿入）
- [x] CI self-lint 対象（`self/compiler.fav` / `self/checker.fav`）に今回の変更が影響しないか確認（vm.rs / compiler.rs / checker.rs の変更は Favnir ソースに非依存のため影響なし）

---

## 事前作業

- [x] T0a: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.6.0 完了条件テスト数を 3218 → 3217 に訂正（v55.5.0 実績 3215 + 2）— spec-reviewer 対応として実施済み

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.6.0` に更新
- [x] T2: `fav/src/middle/compiler.rs` に `"CEP"` を追加（`"State"` エントリの直後）
- [x] T3: `fav/src/middle/checker.rs` に CEP 型登録を追加（`("Stream", "join_left")` の直後）
  - [x] `("CEP", "sequence") => Some(Type::List(Box::new(Type::Unknown)))` // v55.6.0
  - [x] `("CEP", "skip_until") => Some(Type::List(Box::new(Type::Unknown)))` // v55.6.0
  - [x] `("CEP", _) => Some(Type::Unknown)`
- [x] T4: `fav/src/backend/vm.rs` の `is_known_builtin_namespace` に `"CEP"` を追加（`"State"` の直後）
- [x] T5: `fav/src/backend/vm.rs` の `call_builtin` に `CEP.sequence` を追加（`Stream.join_left` アームの直後）
  - [x] 引数数チェック（2 引数）
  - [x] events: `VMValue::List` 取得
  - [x] preds: `VMValue::List` 取得
  - [x] `preds.is_empty()` ガード（空リスト → 空リスト返却）
  - [x] 各開始位置から greedy 前向き探索（`self.call_value` でクロージャ呼び出し）
  - [x] マッチ部分列を `VMValue::List(FavList::new(current))` として収集
  - [x] `Ok(VMValue::List(FavList::new(results)))` を返却
- [x] T6: `fav/src/backend/vm.rs` の `call_builtin` に `CEP.skip_until` を追加（`CEP.sequence` の直後）
  - [x] 引数数チェック（2 引数）
  - [x] events: `VMValue::List` 取得
  - [x] pred: `VMValue` 取得
  - [x] `found` フラグで先頭スキップ → `self.call_value` で述語評価
  - [x] `Ok(VMValue::List(FavList::new(result)))` を返却
- [x] T7: `fav/src/driver.rs` に `v55600_tests` モジュールを追加（`v55500_tests` の直前）
  - [x] `cep_stream_integration`（CEP.sequence 2 マッチ検証）
  - [x] `cep_stateful_persistence`（CEP.skip_until + State.set/get_or_default 検証）

---

## テスト・検証

- [x] T8: `cargo build` でコンパイルエラーがないことを確認
- [x] T9: `cargo test` 全通過（3217 tests passed, 0 failed）
- [x] T10: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T11: `CHANGELOG.md` に v55.6.0 エントリ追加
- [x] T12: `versions/current.md` を v55.6.0 / 3217 tests に更新
- [x] T13: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.6.0 実績を COMPLETE に更新
- [x] T14: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.6.0 実績欄も COMPLETE に更新
- [x] ドキュメント MDX: v55.8 でまとめて追加するため本バージョンはスキップ

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [HIGH] `cep_stream_integration` コメントの走査省略 → pos=1〜4 全判定を明記
  - [MED] `cep_stateful_persistence` の assert_eq! メッセージが `[start,a,b]` → `[3,4,5]` に修正
  - [MED] 空 events / 空 preds の境界ケース → `vm.rs` の CEP.sequence コメントに境界条件を明記
  - [LOW] `CEP.skip_until` の inclusive セマンティクス → `vm.rs` にコメント追加

---

## 完了確認

- [x] `cep_stream_integration` pass
- [x] `cep_stateful_persistence` pass
- [x] 3217 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `compiler.rs` に `"CEP"` が namespace 登録されている
- [x] `checker.rs` に `("CEP", "sequence")` / `("CEP", "skip_until")` が登録されている
- [x] `vm.rs` の `is_known_builtin_namespace` に `"CEP"` が追加されている
- [x] `vm.rs` の `call_builtin` に `CEP.sequence` / `CEP.skip_until` が追加されている
- [x] `CHANGELOG.md` に v55.6.0 エントリが追加されている
- [x] `versions/current.md` が v55.6.0 / 3217 tests を反映
- [x] T13 / T14 のロードマップ更新が完了している

---

## 実装メモ

**Favnir にリストリテラル構文は存在しない:**
- `[...]` は Favnir ではリスト内包記法（`[expr | x <- src]`）専用
- 複数要素のリスト生成は `collect { yield x; yield y; }` または `List.range` で代替
- テストでは `collect { yield fn_ref; }` + `List.range(start, end)` で events/preds リストを構築

**CEP.sequence/CEP.skip_until は call_builtin に実装（vm_call_builtin 不可）:**
- `self.call_value(artifact, pred, args)` が必要なため `&mut self` が必須
- `vm_call_builtin` は free function でエラー型が `String`、`self.call_value` が使えない

**CEP namespace の二重登録要件:**
- `compiler.rs` 登録リスト（未登録だと `IRExpr::Global(u16::MAX)` → runtime error）
- `is_known_builtin_namespace`（未登録だと `LoadGlobal` での名前解決に失敗）
- v55.5.0 の `"State"` は `is_known_builtin_namespace` に既存登録済みだったが `"CEP"` は両方追加が必要
