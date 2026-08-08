# v62.1.0 仕様書 — `fav build` コマンド追加（cranelift object ファイル出力）

## 概要

`fav build` コマンドのテスト用エントリポイント `cmd_build_basic` を `driver.rs` に追加し、
Cranelift ベースの AOT コンパイル基盤（object ファイル出力）を Rust テスト 2 件で保証する。

ロードマップ: `versions/roadmap/roadmap-v62.1-v63.0.md` § v62.1.0

---

## 事前調査（実装前確認済み）

| 項目 | 状態 | 備考 |
|---|---|---|
| `fav/src/backend/cranelift_aot.rs` | 既存（v19.2.0 から） | ロードマップ記載の `aot.rs` に相当 |
| `cmd_build_native(src, out)` | 既存（`driver.rs` L1929）| ファイル I/O を伴う |
| `Some("build")` アーム | 既存（`main.rs` L652） | 新規追加不要 |
| `-o` / `--output` フラグ解析 | 既存（`main.rs` L671〜L676）| `Some("build")` アーム内で実装済み |
| `cranelift-object` 依存 | 既存（`Cargo.toml` L35） | 追加不要 |
| `cmd_build_basic` | **未実装** | 本バージョンで追加 |

**ロードマップのテスト数が古い**: ロードマップ記載のベース 3374・ターゲット 3376 は v62.0.0 実装前の値。
実際のベースは **3382**（v62.0.0 完了時）、ターゲットは **3384**（+2）。

**`aot.rs` 新規作成スキップの根拠**: ロードマップは `fav/src/backend/aot.rs` の新規作成を記載しているが、
v19.2.0 時点で同等の `cranelift_aot.rs` が実装済みであるため新規ファイル作成は省略する。
v62.1.0 スコープ（`lower_to_object_pub` ラッパー追加）は既存ファイルへの追加で十分。
独立した `aot.rs` モジュールへの分離は将来の機能拡張時（v62.2.0 以降）に必要に応じて検討する。

---

## 変更内容

### 1. `backend/cranelift_aot.rs` — `lower_to_object_pub` 追加（オプション A を採用）

`CraneliftBackend::lower_to_object` は非 pub のため `pub(crate)` ラッパーを追加する。

```rust
/// v62.1.0: テスト・driver.rs から呼び出すための pub(crate) ラッパー
pub(crate) fn lower_to_object_pub(ir: &IRProgram) -> Result<Vec<u8>, String> {
    Self::lower_to_object(ir)
}
```

### 2. `driver.rs` — `cmd_build_basic` 追加

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

### 3. `driver.rs` — `v62100_tests` 追加（2 件）

**注意**: `lower_to_object` は `ir.fns` から `fn main` を検索し、見つからない場合 `Err` を返す（`cranelift_aot.rs` L51〜L55）。
両テストのソースに `fn main` を含める必要がある。

| テスト名 | 検証内容 | テストソース |
|---|---|---|
| `cmd_build_outputs_object_file` | `cmd_build_basic` の成功メッセージに `"Output:"` が含まれることを確認 | `"fn main() -> Bool { true }"` |
| `aot_basic_pipeline_compiles` | `lower_to_object_pub` が非空バイト列を返すことを確認 | `"fn main() -> Bool { 1 + 2 == 3 }"` （関数呼び出しは AOT v19.2.0 未サポートのため純算術式） |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **3384 tests passed, 0 failed**
  - ベース 3382 + 2 = 3384
- `pub fn cmd_build_basic` として公開されている（`grep 'pub fn cmd_build_basic' driver.rs` で確認）
- `v62100_tests` の 2 件が PASS

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v62.1-v63.0.md`
- 前バージョン: v62.0.0（3382 tests、Language Polish 宣言完了）
- 既存実装: `fav/src/backend/cranelift_aot.rs`（v19.2.0）、`cmd_build_native`（`driver.rs` L1929）
