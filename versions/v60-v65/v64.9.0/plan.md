# v64.9.0 Plan — 安定化・コードフリーズ（Performance 1.0 前調整）

Version: 64.9.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

- ベーステスト数 3447 の確認
- `driver.rs` に `v64800_tests` が存在し `v64900_tests` がないことを確認
- `performance1-overview.mdx` の 4 セクション存在確認（Quick Start / Performance Certification Checklist / Benchmark Results）

### Step 2: `driver.rs` — `v64900_tests` 追加

`// -- v64800_tests` コメント行の直前に以下を挿入:

```rust
// -- v64900_tests (v64.9.0) -- 安定化・Performance 1.0 前調整 --
#[cfg(test)]
mod v64900_tests {
    use super::*;

    #[test]
    fn scale_all_v64_features_stable() {
        // v64.1: cmd_build_ci
        let ci_src = "public fn main() -> Int { 42 }";
        let ci_result = cmd_build_ci(ci_src, "out");
        assert!(
            !ci_result.starts_with("ci: error:"),
            "cmd_build_ci should succeed: {ci_result}"
        );

        // v64.4: cmd_profile_flamegraph_aot
        let aot_result = cmd_profile_flamegraph_aot(ci_src);
        assert!(
            !aot_result.starts_with("profile-aot: error:"),
            "cmd_profile_flamegraph_aot should not error: {aot_result}"
        );

        // v64.7: cmd_build_wasm
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

### Step 3: ビルド・テスト

```bash
cargo build 2>&1 | tail -5
cargo test --bin fav v64900_tests 2>&1 | tail -10
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

### Step 4: ドキュメント更新（T4）

- `CHANGELOG.md` 先頭に v64.9.0 エントリ追加
- `roadmap-v64.1-v65.0.md` v64.9.0 セクションに実績追記
- `versions/current.md` を v64.9.0（3449 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `scale_all_v64_features_stable` は `use super::*;` が必要（`cmd_build_ci` / `cmd_profile_flamegraph_aot` / `cmd_build_wasm` を呼ぶため）
- `performance1_overview_doc_complete` も同じモジュール内のため `use super::*;` でカバー
- `include_str!` パス: `"../../site/content/docs/performance/performance1-overview.mdx"`
- `cmd_profile_flamegraph_aot` の成功条件: `"profile-aot: error:"` で**始まらない**（warnings は OK）
- `cmd_build_ci` の動作: Cranelift バックエンドで `fn main() -> Int { 42 }` を処理
  → 環境によって `"ci: ok — Output: ..."` または Cranelift サポート外エラーを返す可能性あり
  → assert は `!ci_result.starts_with("ci: error:")` の否定形のみ（Cranelift 環境非依存ではない点に注意）
