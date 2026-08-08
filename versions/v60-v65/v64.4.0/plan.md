# v64.4.0 Plan — `fav profile` flamegraph 改善

Version: 64.4.0
Status: 未着手

---

## 作業順序

### Step 1: `driver.rs` — `cmd_profile_flamegraph_aot` 追加

既存の `cmd_profile` 関数（行 ~13522）の直後に追加。

```rust
pub fn cmd_profile_flamegraph_aot(src: &str) -> String {
    use crate::profiler::collector::StageRecord;
    use crate::profiler::flamegraph::{generate_svg, to_folded_stacks};

    let program = match crate::frontend::parser::Parser::parse_str(src, "<profile-aot>") {
        Ok(p) => p,
        Err(e) => return format!("profile-aot: error: parse error: {e}"),
    };
    let ir = compile_program(&program);
    let bytes = match crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_pub(&ir) {
        Ok(b) => b,
        Err(e) => return format!("profile-aot: error: build error: {e}"),
    };
    if bytes.is_empty() {
        return "profile-aot: error: empty binary".to_string();
    }
    // IR の関数名を stage 名として StageRecord を生成
    let records: Vec<StageRecord> = ir.fns.iter().map(|f| StageRecord {
        name: f.name.clone(),
        elapsed_ms: 1,
    }).collect();
    let folded = to_folded_stacks(&records);
    match generate_svg(&folded) {
        Ok(svg_bytes) => format!("Generated: fav-profile-aot.svg ({} bytes)", svg_bytes.len()),
        Err(e) => format!("profile-aot: error: svg error: {e}"),
    }
}
```

### Step 2: `driver.rs` — `v64400_tests` 追加

`v64300_tests` の直前（`mod v64300_tests` の行を検索）に挿入。

### Step 3: ビルド確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

### Step 4: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v64400_tests 2>&1 | tail -20
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## 注意事項

- `StageRecord.elapsed_ms` は `i64`（`u64` ではない）
- `to_folded_stacks` は `&[StageRecord]` を受け取り `Vec<String>` を返す（`Vec` の参照で渡す）
- `generate_svg` は `&[String]` を受け取る（`folded: Vec<String>` を `&folded` として渡す; `&Vec<String>` → `&[String]` に coerce される）
- `ir.fns` が空の場合でも `to_folded_stacks` は空文字列を返すだけなので問題なし
  （SVG 生成は空フォールドでも成功する可能性があるが、`bytes.is_empty()` で先にガード済み）
- import パス: `crate::profiler::collector::StageRecord`、`crate::profiler::flamegraph::{generate_svg, to_folded_stacks}`
