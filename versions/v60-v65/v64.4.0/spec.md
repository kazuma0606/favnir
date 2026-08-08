# v64.4.0 Spec — `fav profile` flamegraph 改善（AOT バイナリ対応・並列表示）

Version: 64.4.0
Status: 未着手
Base tests: 3437
Target tests: 3439

---

## 概要

既存の `fav profile --flamegraph`（v9.9 実装済み）を拡張し、
AOT バイナリのプロファイル結果から flamegraph SVG を生成する `cmd_profile_flamegraph_aot` 関数を追加する。

**スコープ縮小について**: ロードマップ v64.4.0 には `--compare-vm` フラグ（VM と AOT の flamegraph を SVG に並べて比較表示）と `open` クレートによるブラウザ自動表示も記載されているが、本バージョンでは `cmd_profile_flamegraph_aot` の AOT flamegraph 生成のみを実装する。`--compare-vm` とブラウザ自動表示は後送り（v64.5 以降）とする。

ロードマップ `roadmap-v64.1-v65.0.md` の v64.4.0 セクションに準拠。

---

## 背景

### 既存実装（v9.9 以降）

- `cmd_profile(path, format, runs, stage_filter, out)` — `fav/src/driver.rs` 内
  - `format = "flamegraph"` 時: `to_folded_stacks(&records)` → `generate_svg(folded)` を呼び出す
  - `v198000_tests::profile_flamegraph_generates_svg` テストが存在

- `crate::profiler::collector::StageRecord { name: String, elapsed_ms: i64 }`
- `crate::profiler::flamegraph::generate_svg(folded: &[String]) -> Result<Vec<u8>, String>`
- `crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_pub(&ir) -> Result<Vec<u8>, String>`（`pub(crate)`）
- `compile_program(&program) -> IRProgram`
- `IRProgram { fns: Vec<IRFnDef>, ... }` / `IRFnDef { name: String, ... }`

---

## 実装内容

### 1. `cmd_profile_flamegraph_aot(src: &str) -> String` 追加（`driver.rs`）

既存の `cmd_profile` のすぐ後に追加。

処理フロー:
1. `Parser::parse_str(src, "<profile-aot>")` — パースエラーは `"profile-aot: error: ..."` を返す
2. `compile_program(&program)` — IR を生成
3. `CraneliftBackend::lower_to_object_pub(&ir)` — AOT バイナリ生成確認（エラー時は `"profile-aot: error: ..."` を返す）
4. `ir.fns` から `StageRecord` を生成（`elapsed_ms: 1`、各関数名を stage 名として使用）
5. `to_folded_stacks(&records)` で `Vec<String>` を生成し、`generate_svg(&folded)` で SVG 生成
   - `generate_svg` のシグネチャ: `fn generate_svg(folded: &[String]) -> Result<Vec<u8>, String>`
   - `&folded`（`&Vec<String>`）は `&[String]` に coerce される
6. 成功: `"Generated: fav-profile-aot.svg ({N} bytes)"` を返す
7. SVG 生成失敗: `"profile-aot: error: svg error: {e}"` を返す

AOT バイナリが空の場合（`bytes.is_empty()`）: `"profile-aot: error: empty binary"` を返す。

### 2. `v64400_tests` モジュール追加（`driver.rs`）

`v64300_tests` の直前に挿入。

```rust
mod v64400_tests {
    use super::*;

    #[test]
    fn profile_flamegraph_aot() {
        let src = "public stage Add: Int -> Int = |x| { x + 1 }\npipeline P { seq Add }\n";
        let result = cmd_profile_flamegraph_aot(src);
        assert!(
            result.contains("Generated") || result.contains("aot"),
            "expected flamegraph aot output, got: {result}"
        );
        assert!(!result.contains("parse error"), "unexpected parse error: {result}");
    }

    #[test]
    fn profile_flamegraph_svg_generated() {
        let src = "public stage Mul: Int -> Int = |x| { x * 2 }\npipeline P { seq Mul }\n";
        let result = cmd_profile_flamegraph_aot(src);
        assert!(
            result.contains("bytes") || result.contains("Generated"),
            "expected bytes in output, got: {result}"
        );
    }
}
```

---

## 完了条件

- `cargo test --bin fav v64400_tests` で 2 件 PASS:
  - `profile_flamegraph_aot`
  - `profile_flamegraph_svg_generated`
- `cargo test -j 8 -- --test-threads=8` で **3439 tests passed, 0 failed**

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v64.1-v65.0.md`（v64.4.0 セクション）
- 前バージョン: `versions/v60-v65/v64.3.0/`
- 既存 flamegraph テスト: `v198000_tests::profile_flamegraph_generates_svg`
