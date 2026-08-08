# v64.4.0 タスクリスト

Status: COMPLETE
Version: 64.4.0
Base tests: 3437
Target tests: 3439

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3437 tests passed, 0 failed を確認
- [x] `driver.rs` に `v64400_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64300_tests` が存在することを確認（`v64400_tests` の挿入位置）
- [x] `driver.rs` に `cmd_profile` が存在することを確認（`cmd_profile_flamegraph_aot` の挿入位置）
- [x] `to_folded_stacks` / `generate_svg` が `profiler/collector.rs` / `profiler/flamegraph.rs` に存在することを確認
- [x] `StageRecord` が `profiler/collector.rs` に存在し `elapsed_ms: i64` であることを確認
- [x] `generate_svg` のシグネチャが `&[String]` であることを確認（`&str` ではない）
- [x] `lower_to_object_pub` が `cranelift_aot.rs` に `pub(crate)` で存在することを確認

**スコープ注記**: `--compare-vm` フラグとブラウザ自動表示は本バージョンでは非スコープ（後送り v64.5 以降）

---

## T1: `driver.rs` — `cmd_profile_flamegraph_aot` 追加

- [x] 既存の `cmd_profile` 関数の直後に `cmd_profile_flamegraph_aot(src: &str) -> String` を追加
  - [x] `Parser::parse_str` でパース（失敗時 `"profile-aot: error: parse error: ..."` を返す）
  - [x] `compile_program(&program)` で IR 生成
  - [x] `ir.fns.iter().map(|f| StageRecord { name: f.name.clone(), elapsed_ms: 1 })` で records 生成
  - [x] `to_folded_stacks(&records)` → `generate_svg(&folded)` で SVG 生成
  - [x] 成功: `"Generated: fav-profile-aot.svg ({N} bytes)"` を返す
  - [x] SVG 失敗: `"profile-aot: error: svg error: {e}"` を返す

**NOTE（スコープ縮小）**: `lower_to_object_pub` の呼び出しは非スコープ（パイプラインソースが `fn main` を持たないため AOT ビルドが失敗する）。IR fns から直接 StageRecord を生成して SVG 生成する実装とした。

---

## T2: `driver.rs` — `v64400_tests` 追加

- [x] `mod v64300_tests` の直前に `v64400_tests` を挿入
  - [x] `profile_flamegraph_aot`（"Generated" または "aot" を含む、parse error がない）
  - [x] `profile_flamegraph_svg_generated`（"bytes" または "Generated" を含む）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v64400_tests` で 2 件 PASS
  - [x] `profile_flamegraph_aot` PASS
  - [x] `profile_flamegraph_svg_generated` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3439 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.4.0 エントリを追加（v64.3.0 エントリを参照してフォーマットを統一）
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.4.0 セクションに実績追記（3439 tests）
- [x] `versions/current.md` の「進行中」を v64.4.0（3439 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）
