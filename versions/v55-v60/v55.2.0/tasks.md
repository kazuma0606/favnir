# Tasks — v55.2.0 — セッションウィンドウ + ウォーターマーク本番品質化

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.2.0 セクションを確認
- [x] ベーステスト数 3207（v55.1.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.1.0` であることを確認（更新前）
- [x] `fav/src/toml.rs` の `StreamConfig` 末尾フィールド（`delivery`）を確認（追加位置）
- [x] `fav/src/toml.rs` の `[stream]` パーサーブランチの `_ => {}` 直前への追加位置を確認
- [x] `fav/src/backend/vm.rs` の `VM` 構造体フィールド一覧を確認（`checkpoint_store` の直後に追加）
- [x] `fav/src/backend/vm.rs` の `VM::new_with_db_path` 初期化リストを確認（`checkpoint_store: None,` の直後に追加）
- [x] `fav/src/backend/vm.rs` の `checkpoint_hook` メソッドを確認（直後に追加）
- [x] `fav/src/driver.rs` の `v55100_tests` モジュール位置を確認（直前に挿入）
- [x] `v55100_tests` に `cargo_toml_version_is_55_1_0` テストが存在しないことを確認（削除タスク不要）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.2.0` に更新
- [x] T2: `fav/src/toml.rs` の `StreamConfig` に 2 フィールドを追加
  - [x] `session_gap_sec: Option<u32>`
  - [x] `watermark_max_late_sec: Option<u32>`
- [x] T3: `fav/src/toml.rs` の `[stream]` パーサーブランチに 2 キー解析を追加
  - [x] `"session_gap_sec"` → `current.session_gap_sec = val.trim_matches('"').parse().ok()`
  - [x] `"watermark_max_late_sec"` → `current.watermark_max_late_sec = val.trim_matches('"').parse().ok()`
- [x] T4: `fav/src/backend/vm.rs` の `VM` 構造体に 2 フィールドを追加
  - [x] `pub(crate) late_event_drops: u64`
  - [x] `pub(crate) show_stream_stats: bool`
- [x] T5: `vm.rs` の `VM::new_with_db_path` 初期化部分に追加
  - [x] `late_event_drops: 0`
  - [x] `show_stream_stats: false`
- [x] T6: `vm.rs` の `impl VM` に `observe_late_drop` stub メソッドを追加
- [x] T7: `vm.rs` の `impl VM` に `print_stream_stats` stub メソッドを追加
- [x] T8: `fav/src/driver.rs` に `v55200_tests` モジュールを追加（`v55100_tests` の直前）
  - [x] `window_session_toml_config`（session_gap_sec / watermark_max_late_sec 解析）
  - [x] `watermark_late_event_observe_effect`（late_policy / watermark_max_late_sec 組み合わせ解析）

---

## テスト・検証

- [x] T9: `cargo build` でコンパイルエラーがないことを確認（VM 構造体初期化漏れ等）（T10 実施前に実行）
- [x] T10: `cargo test` 全通過（3209 tests passed, 0 failed）
- [x] T11: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T12: `CHANGELOG.md` に v55.2.0 エントリ追加
- [x] T13: `versions/current.md` を v55.2.0 / 3209 tests に更新
- [x] T14: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.2.0 実績を COMPLETE に更新（テスト数訂正・実装注記含む）
- [x] T15: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.2.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [ ] コードレビュー実施（`/review code`）
- [ ] 指摘事項対応（あれば）

---

## 完了確認

- [x] `window_session_toml_config` pass
- [x] `watermark_late_event_observe_effect` pass
- [x] 3209 tests passed, 0 failed
- [x] `CHANGELOG.md` に v55.2.0 エントリが追加されている
- [x] `versions/current.md` が v55.2.0 を反映
- [x] T14 / T15 のロードマップ更新が完了している
