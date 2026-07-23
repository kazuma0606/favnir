# Tasks: v49.3.0 — `fav check` インクリメンタル型チェック

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3073 passed, 0 failed を確認（ベース確認）
- [x] `sha2 = "0.10"` が `fav/Cargo.toml` に存在することを確認
- [x] `tempfile` が `[dev-dependencies]` に存在することを確認

## T1 — ヘルパー関数追加

- [x] `driver.rs` に `compute_file_fingerprint` 追加
  - [x] `sha2::Sha256::digest` を使用
  - [x] 戻り値: `Option<String>`（hex 文字列）
  - [x] ファイルオープン失敗時は `None` を返す
- [x] `driver.rs` に `file_needs_recheck` 追加
  - [x] キャッシュなし → `true`（要再チェック）
  - [x] フィンガープリント一致 → `false`（スキップ）
  - [x] フィンガープリント不一致 → `true`（要再チェック）
- [x] `driver.rs` に `update_fingerprint_cache` 追加
  - [x] `.fav-cache/` ディレクトリを `create_dir_all` で自動作成
  - [x] `<filename>.fp` にハッシュを書き込む

## T2 — `v493000_tests` 追加

- [x] `v493000_tests` モジュールを `v492000_tests` の直前に追加（2テスト）
  - [x] `incremental_check_skips_unchanged`: 未変更ファイルがスキップされることを確認
  - [x] `incremental_check_detects_change`: 変更ファイルが検知されることを確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.3.0"`
- [x] `cargo test` 3075 passed, 0 failed（3073 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.3.0 エントリ追加（インクリメンタルチェック・SHA-256・`.fav-cache/` を明記）
- [x] `versions/current.md` を v49.3.0（3075 tests）に更新、進行中バージョンを `v49.4.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.3.0 実績を 3075 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: `checker.rs` への実際の hookup（`fav check` コマンドからの呼び出し）はこのバージョンのスコープ外
> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
