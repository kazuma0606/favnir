# v42.4.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2886（前バージョン 2883 + 3）
**実績テスト数**: 2886（v42400_tests 3/3 PASS）

---

## T0 — 事前確認

- [x] `cargo test` が 2883 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.3.0` であることを確認
- [x] `vm.rs` に `VMStream::Split` バリアントが存在することを確認し行番号を記録（line 1530）
- [x] `vm.rs` の `"Stream.split"` ブロック行番号を記録（line 4794）
- [x] `vm.rs` の `materialize_stream` 内 `VMStream::Split` アーム行番号を記録（line 5580）
- [x] `checker.rs` の `("Stream", "to_list")` 行番号を記録（line 6745）
- [x] `checker.rs` の `("Stream", _)` catch-all 行番号を記録（line 6746）
- [x] `driver.rs` の `v42300_tests` 閉じ `}` 行番号を記録（line 44644）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.4.0 エントリが存在することを確認

---

## T1 — `vm.rs` — `VMStream::Join` バリアント追加

- [x] `VMStream::Split` の直後に `Join { left, right, join_fn, window_secs }` バリアントを追加

---

## T2 — `vm.rs` — `"Stream.join"` プリミティブ追加

- [x] `"Stream.split"` ブロックの直後に `"Stream.join"` アームを追加（4 引数検証 + エラーメッセージ）

---

## T3 — `vm.rs` — `materialize_stream` に `VMStream::Join` アーム追加

- [x] `VMStream::Split` アームの直後に `VMStream::Join` アームを追加（nested-loop join）

---

## T4 — `checker.rs` — `("Stream", "join")` 型推論エントリ追加

- [x] `("Stream", "to_list")` の直後に `("Stream", "join") => Some(Type::Stream(Box::new(Type::Unknown)))` を追加

---

## T5 — `driver.rs` — `v42400_tests` モジュール追加

- [x] `v42300_tests` モジュールの直前（降順配置）に `v42400_tests` モジュールを挿入
- [x] `cargo_toml_version_is_42_4_0`（NOTE コメント付き）
- [x] `stream_join_type_check_ok`
- [x] `stream_join_vm_basic`

---

## T6 — Cargo.toml バージョン bump

- [x] `version = "42.3.0"` → `"42.4.0"`

---

## T7 — CHANGELOG.md 更新

- [x] `[v42.4.0]` エントリを `[v42.3.0]` の直前に追加

---

## T8 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2886 を確認（2883 + 3 件）
- [x] `v42400_tests` 3 件 pass を確認

---

## T9 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.4.0（最新安定版）・v42.5.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.4.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] `versions/v40-v45/v42.4.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: ロードマップ「checker.fav 型安全チェック」延期根拠の不整合 → roadmap に延期注釈追加、spec §非スコープに延期理由・破壊的変更予告を追記
- [HIGH-2]: `stream_join_vm_basic` アサーションが `[[2, 2]]` を実質検証しない → `Int(1)`/`Int(3)` 非存在チェックを追加して厳格化（実装では `Value::List` 直接比較に変更）
- [MED-1]: `window_secs` 無視の説明が §2/§3 に分散 → spec §3 冒頭に説明追加
- [MED-2]: 期待値説明の左右曖昧 → spec §5 を「左ストリーム値 2 と右ストリーム値 2 がマッチ」に修正
- [MED-3]: `Type::Unknown` の将来影響未記述 → spec §4 にペア型推論延期の説明追加
- [LOW-1]: `versions/current.md` が影響範囲テーブルに未掲載 → spec §影響範囲に行追加
- [LOW-2]: plan.md と tasks.md の挿入位置表現が不統一 → tasks.md T5 を plan.md と統一

## code-reviewer 指摘・対応記録

- [MED]: `window_secs <= 0` のバリデーションがコメント「primitive で validate する」と矛盾して欠落 → `if window_secs <= 0 { return Err(...) }` を `VMValue::Int(window_secs)` アームに追加
- [LOW]: `stream_join_type_check_ok` の `bind _ <-` が型検証として弱い → `bind joined <- ... Stream.to_list(joined)` に変更し join 結果を Stream として使えることも検証

## 実装上の追記事項

- `Stream.from([1, 2])` リスト直接指定構文はパーサー未対応 → テストで `Stream.from(List.range(1, 3))` に変更
- `build_artifact` / `exec_artifact_main` は `use super::{build_artifact, exec_artifact_main}` 経由でテストモジュールからアクセス
