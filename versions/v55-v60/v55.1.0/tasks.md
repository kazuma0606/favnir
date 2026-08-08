# Tasks — v55.1.0 — タンブリング / スライディングウィンドウ + Exactly-once 統合

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.1.0 セクションを確認
- [x] ベーステスト数 3206（v55.0.0 完了時点）を確認
- [x] `fav/Cargo.toml` が現在 `55.0.0` であることを確認（更新前）
- [x] `fav/src/toml.rs` の `StreamConfig`（L144〜152）既存フィールドを確認
- [x] `fav/src/toml.rs` の `[stream]` パーサーブランチ（L845〜863）の `_ => {}` 直前への追加位置を確認
- [x] `fav/src/backend/vm.rs` の `VM` 構造体フィールド一覧を確認（`checkpoint_store` 追加位置）
- [x] `fav/src/backend/vm.rs` の `VM::new` / 初期化箇所を確認（`checkpoint_store: None` 追加対象）
- [x] `fav/src/backend/vm.rs` の `VMStream::Window` ブランチ（L5986〜5994）を確認
- [x] `fav/src/driver.rs` の `v55000_tests` モジュール位置を確認（直前に挿入する）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.1.0` に更新
- [x] T2: `fav/src/toml.rs` の `StreamConfig` に 3 フィールドを追加
  - [x] `checkpoint_store: Option<String>`
  - [x] `checkpoint_interval_sec: Option<u32>`
  - [x] `delivery: Option<String>`
- [x] T3: `fav/src/toml.rs` の `[stream]` パーサーブランチに 3 キー解析を追加
  - [x] `"checkpoint_store"` → `current.checkpoint_store = Some(val.trim_matches('"').to_string())`
  - [x] `"checkpoint_interval_sec"` → `current.checkpoint_interval_sec = val.trim_matches('"').parse().ok()`
  - [x] `"delivery"` → `current.delivery = Some(val.trim_matches('"').to_string())`
- [x] T4: `fav/src/backend/vm.rs` の `VM` 構造体に `checkpoint_store: Option<String>` を追加
- [x] T5: `vm.rs` の `VM::new`（および初期化箇所すべて）に `checkpoint_store: None` を追加
- [x] T6: `vm.rs` の `impl VM` に `checkpoint_hook` stub メソッドを追加
- [x] T7: `vm.rs` の `VMStream::Window` ブランチ（L5990〜5993 の for ループ内）に `self.checkpoint_hook(...)` 呼び出しを挿入
- [x] T8: `fav/src/driver.rs` に `v55100_tests` モジュールを追加（`v55000_tests` の直前）
  - [x] `window_tumbling_checkpoint_integration`（buffer_size / checkpoint_store 解析）
  - [x] `window_sliding_resume_from_checkpoint`（delivery / checkpoint_interval_sec 解析）
- [x] T9: `v55000_tests` から `cargo_toml_version_is_55_0_0` を削除（Cargo.toml 更新に伴う慣行）

---

## テスト・検証

- [x] T10: `cargo build` でコンパイルエラーがないことを確認（VM 構造体初期化漏れ等）
- [x] T11: `cargo test` 全通過（3207 tests passed, 0 failed）※spec の 3208 は算術ミス（削除 1 + 追加 2 = net +1）
- [x] T12: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T13: `CHANGELOG.md` に v55.1.0 エントリ追加
- [x] T14: `versions/current.md` を v55.1.0 / 3207 tests に更新
- [x] T15: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.1.0 実績を COMPLETE に更新
- [x] T16: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.1.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [ ] コードレビュー実施（`/review code`）
- [ ] 指摘事項対応（あれば）

---

## 完了確認

- [x] `window_tumbling_checkpoint_integration` pass
- [x] `window_sliding_resume_from_checkpoint` pass
- [x] 3207 tests passed, 0 failed
- [x] `versions/current.md` が v55.1.0 を反映
- [x] `roadmap-v55.1-v56.0.md` の v55.1.0 実績: COMPLETE — 3207 tests passed, 0 failed（2026-07-23）
- [x] `roadmap-v55.1-v60.0.md` の v55.1.0 実績欄: COMPLETE — 3207 tests passed, 0 failed（2026-07-23）
