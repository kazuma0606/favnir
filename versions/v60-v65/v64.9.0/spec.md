# v64.9.0 Spec — 安定化・コードフリーズ（Performance 1.0 前調整）

Version: 64.9.0
Status: 未着手
Base tests: 3447
Target tests: 3449

---

## 概要

v64.1〜v64.8 の全機能が正常動作することを確認する安定化バージョン。
`driver.rs` に `v64900_tests` 2 件を追加し、v64 主要機能の動作確認と
`performance1-overview.mdx` の完成度を検証する。

新規ファイル作成・既存コードの変更はなし。テストモジュールの追加のみ。

ロードマップ `roadmap-v64.1-v65.0.md` の v64.9.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3447 tests passed, 0 failed を確認
- `driver.rs` に `v64900_tests` が存在しないことを確認（新規追加）
- `driver.rs` に `v64800_tests` が存在することを確認（`v64900_tests` の挿入位置）
- `site/content/docs/performance/performance1-overview.mdx` が存在することを確認

---

## 実装スコープ

### 1. `driver.rs` — `v64900_tests` 追加

`v64800_tests` の直前に挿入:

```rust
// -- v64900_tests (v64.9.0) -- 安定化・Performance 1.0 前調整 --
#[cfg(test)]
mod v64900_tests {
    use super::*;

    #[test]
    fn scale_all_v64_features_stable() {
        // v64.1: cmd_build_ci — CI ビルド出力
        let ci_src = "public fn main() -> Int { 42 }";
        let ci_result = cmd_build_ci(ci_src, "out");
        assert!(
            !ci_result.starts_with("ci: error:"),
            "cmd_build_ci should succeed: {ci_result}"
        );

        // v64.4: cmd_profile_flamegraph_aot — flamegraph AOT
        let aot_result = cmd_profile_flamegraph_aot(ci_src);
        assert!(
            !aot_result.starts_with("profile-aot: error:"),
            "cmd_profile_flamegraph_aot should not error: {aot_result}"
        );

        // v64.7: cmd_build_wasm — WASM ビルド
        let wasm_src = concat!(
            "public fn add(a: Int, b: Int) -> Int { a + b }\n",
            "public fn main() -> Unit { IO.println_int(add(1, 2)) }\n"
        );
        let wasm_result = cmd_build_wasm(wasm_src, "out.wasm");
        assert!(
            !wasm_result.starts_with("wasm: error:"),
            "cmd_build_wasm should succeed: {wasm_result}"
        );
    }

    #[test]
    fn performance1_overview_doc_complete() {
        let content = include_str!("../../site/content/docs/performance/performance1-overview.mdx");
        assert!(
            content.contains("Performance 1.0 Overview"),
            "overview should have 'Performance 1.0 Overview' heading: {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("Quick Start"),
            "overview should have 'Quick Start' section: {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("Performance Certification Checklist"),
            "overview should have 'Performance Certification Checklist': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("Benchmark Results"),
            "overview should have 'Benchmark Results' section: {}",
            &content[..content.len().min(200)]
        );
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64900_tests` で 2 件 PASS
  - `scale_all_v64_features_stable` PASS
  - `performance1_overview_doc_complete` PASS
- `cargo test -j 8 -- --test-threads=8` で 3449 tests passed, 0 failed

---

## 非スコープ

- `fav build --target wasm32` の `main.rs` CLI dispatch 統合（v65.0 以降）
- `fav lint --perf` CLI フラグ（`main.rs`）の統合（v65.0 以降）
- clippy / lint クリーン確認（GitHub Actions CI が `cargo clippy` を毎コミット実行しており代替済み。
  ローカルで未コミット変更がある場合は `cargo clippy 2>&1 | grep error` で確認可能だが必須ではない）
- v64.2 `BenchTomlConfig` / v64.5 `benchmarks.mdx` / v64.6 lint perf の個別動作確認
  （`scale_all_v64_features_stable` は v64.1・v64.4・v64.7 の代表機能のみ確認）

---

## 技術ノート

### テスト設計方針

`scale_all_v64_features_stable` は v64 スプリントの **代表的な Rust 関数**（`cmd_build_ci` / `cmd_profile_flamegraph_aot` / `cmd_build_wasm`）を直接呼び出し、
エラープレフィックスで始まらないことを確認する。

`performance1_overview_doc_complete` は `performance1-overview.mdx` の
4 セクション（`"Performance 1.0"` / `"Quick Start"` / `"Performance Certification Checklist"` / `"Benchmark Results"`）の存在を検証する。

### `include_str!` パス

`driver.rs`（`fav/src/driver.rs`）から
`../../site/content/docs/performance/performance1-overview.mdx` を解決すると
`favnir/site/content/docs/performance/performance1-overview.mdx` になる
（`fav/src/` から `../../` = `fav/` の親 = `favnir/`（リポジトリルート））。

### `cmd_profile_flamegraph_aot` の入力

v64.4.0 の実装では `parse_str` が失敗した場合のみ `"profile-aot: error: parse error:..."` を返す。
`"public fn main() -> Int { 42 }"` は IR を生成できるため、`records` が 1 件以上 → `generate_svg` が呼ばれる。
`generate_svg` が成功すれば `"Generated: fav-profile-aot.svg (...)"` を返す。

### `cmd_build_ci` のエラー条件と環境依存

`cmd_build_ci` は Cranelift バックエンドを使う。`fn main() -> Int` は Cranelift で処理可能なシグネチャ。
エラーが返る場合は `"ci: error: build error: ..."` で始まる。

**注意**: Cranelift バックエンドがサポートされていない環境（一部の CI 環境等）では
`"ci: error: build error: ..."` を返す可能性がある。`scale_all_v64_features_stable` の
`cmd_build_ci` アサーションはこのケースでは偽陰性になりうる。
開発環境（Windows 11 / x86-64）では Cranelift は動作確認済みのため、ローカルテストは問題なし。
（詳細は plan.md §注意事項も参照）
