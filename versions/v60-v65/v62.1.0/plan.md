# v62.1.0 実装計画 — `fav build` コマンド追加

## フェーズ構成

| フェーズ | 内容 | 対象ファイル |
|---|---|---|
| P1 | `cranelift_aot.rs` に `lower_to_object_pub` を追加（またはインライン実装） | `fav/src/backend/cranelift_aot.rs` |
| P2 | `driver.rs` に `cmd_build_basic` を追加 | `fav/src/driver.rs` |
| P3 | `driver.rs` に `v62100_tests` を追加（2 件） | `fav/src/driver.rs` |
| P4 | ビルド・テスト全通過確認 | — |
| P5 | ドキュメント更新（roadmap / current.md / CHANGELOG / tasks.md）| `versions/` |

---

## P1: `cranelift_aot.rs` — `lower_to_object_pub` 追加

`CraneliftBackend::lower_to_object` は `fn`（非 pub）のため、テストから呼び出せるよう
`pub(crate)` ラッパーを追加する。

```rust
/// v62.1.0: テスト・driver.rs から呼び出すための pub(crate) ラッパー
pub(crate) fn lower_to_object_pub(ir: &IRProgram) -> Result<Vec<u8>, String> {
    Self::lower_to_object(ir)
}
```

`cargo build` でエラーなしを確認してから P2 に進む。

---

## P2: `driver.rs` — `cmd_build_basic` 追加

`cmd_build_native` の直後（L1936 付近）に追加する。

```rust
/// v62.1.0: `fav build` コマンドのテスト用エントリポイント。
/// ファイル I/O を伴わず、ソース文字列から object バイト列長を含む結果文字列を返す。
pub fn cmd_build_basic(src: &str, out: &str) -> String {
    let program = match crate::frontend::parser::Parser::parse_str(src, "<build>") {
        Ok(p) => p,
        Err(e) => return format!("parse error: {e}"),
    };
    let ir = compile_program(&program);
    match crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_pub(&ir) {
        Ok(bytes) => format!("Output: {} ({} bytes)", out, bytes.len()),
        Err(e) => format!("build error: {e}"),
    }
}
```

`cargo build` でエラーなしを確認。

---

## P3: `driver.rs` — `v62100_tests` 追加

`v62000_tests` モジュールの直後（ファイル末尾方向）に挿入。

```rust
// -- v62100_tests (v62.1.0) -- `fav build` コマンド基盤 --
#[cfg(test)]
mod v62100_tests {
    use super::*;

    /// cmd_build_basic が単純な fn main ソースから "Output:" を含む文字列を返すことを確認
    #[test]
    fn cmd_build_outputs_object_file() {
        let src = "fn main() -> Bool { true }";
        let result = cmd_build_basic(src, "pipeline.o");
        assert!(
            result.contains("Output:"),
            "cmd_build_basic should return 'Output:' on success; got: {:?}",
            result
        );
    }

    /// aot 基本パイプライン（fn add + fn main）が cranelift AOT でコンパイルでき、
    /// 非空のオブジェクトバイト列が生成されることを確認。
    /// lower_to_object は fn main 必須のため両方含める。
    #[test]
    fn aot_basic_pipeline_compiles() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { add(1, 2) == 3 }";
        let prog = Parser::parse_str(src, "<aot-test>").expect("parse failed");
        let ir = compile_program(&prog);
        let result = crate::backend::cranelift_aot::CraneliftBackend::lower_to_object_pub(&ir);
        assert!(
            result.is_ok(),
            "cranelift AOT should compile basic fn without error; got: {:?}",
            result.err()
        );
        let bytes = result.unwrap();
        assert!(
            !bytes.is_empty(),
            "compiled object should be non-empty"
        );
    }
}
```

`cargo test v62100` で 2 件 PASS を確認。

---

## P4: ビルド・テスト

```bash
cargo build             # コンパイルエラー 0
cargo test v62100       # 2 件 PASS
cargo test -j 8 -- --test-threads=8  # 3384 passed, 0 failed
```

---

## P5: ドキュメント更新

- `versions/roadmap/roadmap-v62.1-v63.0.md` — v62.1.0 セクションに実績追記
  - **注意**: ロードマップのテスト数（3376）は古い値。実績は 3384 で記録する
- `versions/current.md` — 進行中を v62.1.0 完了に更新、次を v62.2.0 に
- `CHANGELOG.md` — v62.1.0 エントリ追加
- `versions/v60-v65/v62.1.0/tasks.md` — COMPLETE に更新

---

## リスク・注意事項

- `compile_program` は `driver.rs` 内部の `pub(crate)` 関数 — テストモジュールから `use super::*;` で利用可能
- `cranelift_aot.rs` の `lower_to_object` が IRProgram に `main` 関数を要求する（L51〜L55）。
  `aot_basic_pipeline_compiles` では `fn add` のみ（main なし）を渡すため、エラーになる可能性がある。
  → 実装時に動作確認し、`fn main() -> Bool { true }` を追加するか、`lower_to_object` の main 必須条件を回避する
- `cmd_build_basic` のロードマップ定義（`pub fn ... -> String`）と `lower_to_object_pub` の追加の両方が必要
- `cranelift-object` feature フラグ確認（ロードマップ言及）: Cargo.toml L35 に登録済み、feature フラグ不要
