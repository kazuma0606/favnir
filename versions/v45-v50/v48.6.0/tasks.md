# Tasks: v48.6.0 — 循環 import 検出 + E0418

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3056 passed, 0 failed を確認（ベース確認）
- [x] `error_catalog.rs` に E0418 が未登録であること（予約コメントのみ）を確認
- [x] `driver.rs` に `detect_circular_imports` が存在しないことを確認

## T1 — `error_catalog.rs` E0418 追加

- [x] `// ── E0418〜E0419: 予約（将来拡張用）` コメントを `E0418 ErrorEntry` に差し替え
- [x] E0419 予約コメントは残す（`E0418〜E0419` を `E0419` のみに更新）

## T2 — `driver.rs` `detect_circular_imports` 追加

- [x] `cmd_install_runes` の直後に `detect_circular_imports` 関数を追加
  - [x] `pub fn detect_circular_imports(graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>>`
  - [x] DFS カラーリングで循環検出（0=white / 1=gray / 2=black）
  - [x] 循環検出時は循環パス（Vec<String>）を返す
  - [x] 循環なしの場合は `None` を返す
  - [x] `pub` 修飾子をつけること（テストから参照するため）

## T3 — `driver.rs` テスト追加・バージョン更新・完了

- [x] `v486000_tests` モジュールを `v485000_tests` の直前に追加（2テスト）
  - [x] `circular_import_e0418`: `a→b→a` で `Some(cycle)` が返り cycle に `"a"` と `"b"` が含まれる
  - [x] `non_circular_import_ok`: `a→b→c` で `None` が返る
- [x] `fav/Cargo.toml` version → `"48.6.0"`
- [x] `CHANGELOG.md` に v48.6.0 エントリ追加
- [x] `cargo test` 3058 passed, 0 failed（3056 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.6.0（3058 tests）に更新、進行中バージョンを `v48.7.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.6.0 テスト数を実績値 3058 に更新（`roadmap-v45.1-v50.0.md` への反映は v49.0.0 時・変更不要）
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: ファイル解決ロジック（実際の .fav ファイルから依存グラフを構築）は v48.6.0 のスコープ外（MVP スタブのみ）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
