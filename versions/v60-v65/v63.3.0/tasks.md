# v63.3.0 タスクリスト

Status: COMPLETE
Version: 63.3.0
Base tests: 3410
Target tests: 3412

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3410 tests passed, 0 failed を確認
- [x] `fav/src/cache.rs` が存在し `IncrementalCache` / `stage_hash` が実装されていることを確認
- [x] `driver.rs` に `v63200_tests` が存在することを確認（挿入位置確認）
- [x] `fav/src/error_catalog.rs` の末尾が E0397 で終わっていることを確認（E0428 の挿入位置）

---

## T1: `error_catalog.rs` — E0428 エントリ追加

- [x] `E0397` エントリの後（`];` の直前）に E0428 `incremental_cache_conflict` を追加:
  ```rust
  // ── E0428: キャッシュ型シグネチャ不整合 (v63.3.0) ──────────────────────────
  ErrorEntry {
      code: "E0428",
      title: "incremental_cache_conflict",
      ...
      long_description: Some("Favnir's incremental cache stores..."),
      suggestion: Some("This is a non-fatal warning..."),
  },
  ```
- [x] `cargo build` でエラーなし

---

## T2: `cache.rs` — `check_type_sig` メソッド追加

- [x] `invalidate` メソッドの直後に `check_type_sig` を追加:
  ```rust
  pub fn check_type_sig(&self, stage_name: &str, source_hash: &str, current_sig: &str) -> bool { ... }
  ```
  （ハッシュ一致・シグ不一致時に E0428 を `eprintln!` して `invalidate` を呼び `false` を返す）
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v63300_tests` 追加

- [x] `v63200_tests` の直前（ファイル先頭方向）に以下を挿入:
  （モジュールトップの `use crate::cache::{IncrementalCache, stage_hash};` と
  `use tempfile::TempDir;` は両テストで共用）
  ```rust
  // -- v63300_tests (v63.3.0) -- E0428 キャッシュ型シグネチャ不整合検出 --
  #[cfg(test)]
  mod v63300_tests { ... }
  ```
- [x] `cargo build` でエラーなし（テスト挿入後のインクリメンタル確認）

---

## T4: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test v63300` で 2 件 PASS
  - `incremental_e0428_signature_mismatch` PASS
  - `cache_auto_invalidated` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3412 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.3.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.3.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v63.3.0（3412 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応（code-reviewer）

- [MED-1] `category: "cache"` が全カタログ唯一の孤立カテゴリ → `"build"` に変更（E0427 と同カテゴリに統合）
- [MED-2] `cache_auto_invalidated` テストの `eprintln!` が CI stderr にノイズを出す → テストコメントに「E0428 警告が stderr に出るのは仕様」と明記
- [LOW] `check_type_sig` の2アーム方式で第2アームのガード条件を誤削除するリスク → ネスト構造（外側にハッシュ一致・内側に `if` でシグ分岐）に書き換え

## コードレビュー指摘対応（spec-reviewer）

- [HIGH] `check_type_sig` が `&self` で `invalidate` を呼べる理由が未記述 → 技術ノートに「`invalidate` は `&self`（`std::fs::remove_file` は内部状態不変）のため `check_type_sig` も `&self` で実装可能」を追記
- [MED-1] ロードマップの `load` 記述と新規メソッド `check_type_sig` の乖離 → 技術ノートに「ロードマップの `load` は実装方針の示唆であり公開 API は `check_type_sig`」を明記
- [MED-2] `type_sig` 空文字列の未定義動作 → 技術ノート + 非スコープに追記
- [MED-3] WASM ガード対応が非スコープ未記述 → 非スコープ + 技術ノートに「`mod cache;` 宣言側のガードは v63.1.0 で対応済み」を追記
- [LOW] tasks.md T0 の末尾エントリ記述（E0427 vs E0397） → **false positive**（実際は E0397 が末尾、変更不要）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3412 passed, 0 failed
- 主要実装: `error_catalog.rs`（E0428）+ `cache.rs`（`check_type_sig`）+ `driver.rs`（`v63300_tests`）
- 完了日: 2026-08-02
