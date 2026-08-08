# Tasks — v55.4.0 — ストリーム結合（inner join / left outer join）

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.4.0 セクションを確認
- [x] ベーステスト数 3211（v55.3.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.3.0` であることを確認（更新前）
- [x] `fav/src/backend/vm.rs` の `VMStream::Join` バリアント（L1615〜L1621 付近）の直後に `JoinLeft` を追加できることを確認
- [x] `fav/src/backend/vm.rs` の `Stream.join` アーム末尾（`// ── end v26.4.0 Stream.* ──` 直前）を確認（`Stream.join_inner` / `Stream.join_left` の挿入位置）
- [x] `fav/src/backend/vm.rs` の `materialize_stream` 内 `VMStream::Join` アーム末尾（`Ok(out)` の後）を確認（`VMStream::JoinLeft` アームの挿入位置）
- [x] `fav/src/driver.rs` の `v55300_tests` モジュール位置を確認（直前に `v55400_tests` を挿入）
- [x] `v55300_tests` に `cargo_toml_version_is_55_3_0` テストが存在しないことを確認（削除タスク不要）
- [x] `List.range(1, 3)` が `[1, 2]`、`List.range(2, 4)` が `[2, 3]` であることを確認（テスト期待値の根拠）
- [x] `VMValue::Unit` → `Value::Unit` 変換が `impl From<VMValue> for Value` で実装済みであることを確認

---

## 事前作業

- [x] T0a: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.4.0 ベース値を 3211 + 2 = 3213 に訂正

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.4.0` に更新
- [x] T2: `fav/src/backend/vm.rs` の `VMStream` enum に `JoinLeft` バリアントを追加（`Join` の直後）
  - [x] `left: Box<VMStream>`
  - [x] `right: Box<VMStream>`
  - [x] `join_fn: VMValue`
  - [x] `window_secs: i64`
- [x] T3: `vm.rs` に `Stream.join_inner` primitive を追加（`Stream.join` アームの直後）
  - [x] 4 引数チェック（stream1, stream2, join_fn, window_secs）
  - [x] `window_secs <= 0` バリデーション
  - [x] `VMStream::Join { left, right, join_fn, window_secs }` を返す（既存バリアント再利用）
- [x] T4: `vm.rs` に `Stream.join_left` primitive を追加（`Stream.join_inner` アームの直後）
  - [x] 4 引数チェック（stream1, stream2, join_fn, window_secs）
  - [x] `window_secs <= 0` バリデーション
  - [x] `VMStream::JoinLeft { left, right, join_fn, window_secs }` を返す
- [x] T5: `vm.rs` の `materialize_stream` に `VMStream::JoinLeft` アームを追加（`VMStream::Join` アームの直後）
  - [x] left / right を materialize
  - [x] nested-loop で join_fn を呼び出し
  - [x] マッチした場合: `[l, r]` ペアを push
  - [x] マッチしなかった場合: `[l, Unit]` を push（`!matched` 判定）
- [x] T6: `fav/src/driver.rs` に `v55400_tests` モジュールを追加（`v55300_tests` の直前）
  - [x] `stream_join_inner_matches`（`[[2,2]]` 1 件の検証）
  - [x] `stream_join_left_preserves_unmatched`（`[[1,Unit],[2,2]]` 2 件の検証）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認（T2 と T5 を同時適用後に実行）
- [x] T8: `cargo test` 全通過（3213 tests passed, 0 failed）
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v55.4.0 エントリ追加
- [x] T11: `versions/current.md` を v55.4.0 / 3213 tests に更新
- [x] T12: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.4.0 実績を COMPLETE に更新
- [x] T13: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.4.0 実績欄も COMPLETE に更新
- [x] ドキュメント MDX: v55.8 でまとめて追加するため本バージョンはスキップ

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [MED] `Stream.join_inner` と `Stream.join` が同一バリアントを共有する意図を vm.rs にコメントで明示
  - [LOW] `checker.rs` の型テーブルに `join_inner` / `join_left` を登録（`Type::Stream(Unknown)`）
  - その他 [LOW]: `expect()` は既存パターン踏襲・`Unit` プレースホルダーは設計判断・clone は既存パターン → 対応不要

---

## 完了確認

- [x] `stream_join_inner_matches` pass
- [x] `stream_join_left_preserves_unmatched` pass
- [x] 3213 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `vm.rs` に `VMStream::JoinLeft` バリアントが追加されている
- [x] `vm.rs` に `Stream.join_inner` / `Stream.join_left` primitive が追加されている
- [x] `CHANGELOG.md` に v55.4.0 エントリが追加されている
- [x] `versions/current.md` が v55.4.0 を反映
- [x] T12 / T13 のロードマップ更新が完了している
