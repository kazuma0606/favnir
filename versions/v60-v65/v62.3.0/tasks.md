# v62.3.0 タスクリスト

Status: COMPLETE
Version: 62.3.0
Base tests: 3386
Target tests: 3388

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3386 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の `cranelift-codegen` features が `["x86"]` のみであることを確認（`"arm64"` 未追加）
- [x] `cranelift-native = { version = "0.117" }` が `Cargo.toml` に登録済みであることを確認
- [x] `target_lexicon` が `Cargo.toml` に登録されていないことを確認（Cargo.lock で推移的依存として存在を確認）
- [x] `cranelift_aot.rs` が `impl CraneliftBackend` ブロック内にメソッドを持つことを確認（`Self::` 呼び出し可否の確認）
- [x] `cranelift_aot.rs` に `lower_to_object_with_target` が **存在しない** ことを確認
- [x] `driver.rs` に `cmd_build_link_target` が **存在しない** ことを grep で確認
- [x] `driver.rs` に `v62200_tests` が存在することを確認（挿入位置確認）
- [x] `main.rs` の `--link` ブランチが `cmd_build_link` を呼んでいることを確認
- [x] cranelift 0.117 の aarch64 ISA lookup API を確認
  - `cranelift_codegen::isa::lookup_by_name("aarch64")` の存在確認 → 存在する（内部で `lookup(triple!(name))` を呼ぶ）
  - `target_lexicon` の直接追加は不要（Cargo.lock に推移的依存として存在）

---

## T1: Cargo.toml — `"arm64"` feature 追加

- [x] `cranelift-codegen = { version = "0.117", features = ["x86", "arm64"] }` に変更
- [x] `cargo build` でエラーなし

---

## T2: `cranelift_aot.rs` — `lower_to_object_with_target` 追加

- [x] `use cranelift_codegen::{isa, settings, Context};` に `isa` を追加
- [x] `impl CraneliftBackend` ブロック内の `lower_to_object_pub` の直後に `fn lower_to_object_with_target` を追加
  - `None` / `"x86_64-unknown-linux-gnu"` → `cranelift_native::builder()`（既存ロジック）
  - `"aarch64-unknown-linux-gnu"` → `isa::lookup_by_name("aarch64")`
  - その他 → `Err(format!("unsupported target triple: {t}"))`
- [x] `pub(crate) fn lower_to_object_with_target_pub` ラッパーを追加（`impl CraneliftBackend` 内）
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `cmd_build_link_target` 追加・`cmd_build_link` 変更

- [x] `cmd_build_link` の直後に `pub fn cmd_build_link_target(src: &str, out: &str, target: Option<&str>) -> String` を追加
- [x] `cmd_build_link` を `cmd_build_link_target(src, out, None)` の薄いラッパーに変更
- [x] `cargo build` でエラーなし
- [x] `cargo test v62200` で既存 2 件が引き続き PASS することを確認

---

## T4: `main.rs` — `--link` ブランチに `aot_target` 接続

- [x] `--link` ブランチ内で `target.contains('-')` チェックで `aot_target` を取り出す
- [x] `cmd_build_link` → `cmd_build_link_target` に切り替え
- [x] `cargo build` でエラーなし

---

## T5: `driver.rs` — `v62300_tests` 追加

- [x] `v62200_tests` の直前（ファイル先頭方向）に `v62300_tests` モジュールを挿入
- [x] `use super::*;` を先頭に追加
- [x] `aot_cross_compile_aarch64` テスト追加
- [x] `aot_target_triple_parsed` テスト追加（`None` 成功・未サポート Err・CLI スモーク確認）
- [x] `cargo test v62300` で 2 件 PASS

---

## T6: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62300` で 2 件 PASS
- [x] `cargo test v62200` で既存 2 件が引き続き PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3388 tests passed, 0 failed を確認

---

## T7: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.3.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v62.3.0（3388 tests）に更新、「次」を v62.4.0 に
- [x] `CHANGELOG.md` に v62.3.0 エントリを追加
- [x] `site/content/docs/runtime/aot.mdx` — v62.9.0 で対応予定のため本バージョンでは作成しない（スコープ外）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- **[HIGH] `lower_to_object_with_target` の `unwrap()` がパニックリスク** — `flag_builder.set(...).unwrap()` を `map_err(|e| format!("flag set error: {e}"))` + `?` に変換。修正済み。
- **[MED] `lower_to_object` と `lower_to_object_with_target` のコード重複（DRY 違反）** — `lower_to_object` を `Self::lower_to_object_with_target(ir, None)` を呼ぶ 1 行ラッパーに変更して重複を排除。修正済み。
- **[MED] `"x86_64-unknown-linux-gnu"` 明示アームのテストが欠如** — `aot_target_triple_parsed` に `Some("x86_64-unknown-linux-gnu")` が `Ok` を返すアサーションを追加。修正済み。
- **[LOW] `"aarch64"` 文字列が非定数（ドキュメント非保証）** — `CRANELIFT_AARCH64_NAME` 定数を `impl CraneliftBackend` に追加し、`isa::lookup_by_name` の引数を定数参照に変更。修正済み。
- **[LOW] `target.contains('-')` による AOT triple 識別の将来拡張リスク** — `--link` ブランチ限定かつ現状 `-` を含む非 triple 値は存在しないため対応不要。今後の CLI 設計改善時に考慮する。受け入れ。

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3388 passed, 0 failed（ベース 3386 + 2）
- 追記: `cranelift_codegen::isa::lookup_by_name("aarch64")` が利用可能であり `target_lexicon` の直接追加は不要
- 完了日: 2026-08-01
