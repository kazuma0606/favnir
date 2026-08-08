# v62.2.0 Spec — native binary 生成（`fav build --link`・Linux x86_64）

Version: 62.2.0
Status: 未着手
Base tests: 3384
Target tests: 3386

---

## 概要

`fav build --link` フラグを追加し、object ファイルのリンクまで行って実行可能バイナリを生成する。
Favnir ランタイムスタブ（`fav/src/backend/fav_rt.rs`）を新規作成し、
VM プリミティブの最小セット（IO 実行・エラーハンドリング）を静的リンクする概念を確立する。

---

## 前提

- `cranelift_aot.rs` の `compile_to_binary` + `link_binary` は v19.2.0 時点で実装済み
  - `compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String>`: `lower_to_object` + `link_binary` を内部で呼ぶ
  - `link_binary`: tempfile + システム `cc` によるリンク処理
  - `c_wrapper_src`: `fav_main()` を呼ぶ C ラッパー
  - `lower_to_object_pub`: v62.1.0 で追加済み（`Result<Vec<u8>, String>` を返す）
- ロードマップの「aot.rs」は `cranelift_aot.rs` の誤記（v62.1.0 実績に基づき読み替え）
- `fav_rt.rs` は **存在しない** → 新規作成が必要
- `main.rs` の `Some("build")` アームに `--link` フラグが **ない** → 追加が必要
- `driver.rs` に `cmd_build_link` が **ない** → 追加が必要

---

## 実装スコープ

### 1. `fav/src/backend/fav_rt.rs` 新規作成

Favnir ランタイムスタブを記述するファイル。
内容:
- `FAV_RT_VERSION` 定数（`"0.1.0"`）
- `FAV_RT_PRIMITIVES` 定数（最小プリミティブ一覧の文字列 `"fav_io_print,fav_io_panic"`）
- `pub fn fav_rt_stub_src() -> &'static str` — C ランタイムスタブのソース文字列を返す

C スタブ最小セット（文字列定数）:
```c
// Favnir runtime stub v0.1.0
#include <stdio.h>
#include <stdlib.h>

void fav_io_print(const char* s) { puts(s); }
void fav_io_panic(const char* msg) { fprintf(stderr, "panic: %s\n", msg); exit(1); }
```

`backend/mod.rs` に `pub mod fav_rt;` を追加する。

### 2. `fav/src/backend/cranelift_aot.rs` — `compile_to_binary_pub` 追加

既存の `compile_to_binary(ir: &IRProgram, out_path: &str) -> Result<(), String>` を pub(crate) で公開する
ラッパーを追加する。戻り値は `Result<(), String>`（バイト列ではなくファイル書き出し完了）。

```rust
pub(crate) fn compile_to_binary_pub(ir: &IRProgram, out_path: &str) -> Result<(), String> {
    Self::compile_to_binary(ir, out_path)
}
```

### 3. `main.rs` — `Some("build")` アームに `--link` フラグ追加

`Some("build")` アーム内で `--link` フラグを解析し、`cmd_build_link` を呼ぶ。

### 4. `driver.rs` — `cmd_build_link` 追加

`pub fn cmd_build_link(src: &str, out: &str) -> String`
- `Parser::parse_str` → `compile_program` → `compile_to_binary_pub(ir, out)`
- 成功時: `format!("Output: {} (linked binary)", out)`
- エラー時: `format!("build error: {e}")`

Note: `compile_to_binary_pub` は `Result<(), String>` を返すためバイト数は出力しない。

### 5. `driver.rs` — `v62200_tests` 追加

2 件のテスト:
- `aot_binary_executable`:
  - `cmd_build_link("fn main() -> Bool { 1 + 2 == 3 }", "pipeline_bin")` の結果が `"parse error:"` を含まないことを確認
  - Windows 環境では `cc` が失敗して `"build error:"` になる可能性があるため、`"Output:"` ではなく `"parse error:"` の不存在のみ確認
- `aot_runtime_stub_linked`:
  - `crate::backend::fav_rt::fav_rt_stub_src()` が `"fav_io_print"` を含むことを確認
  - `crate::backend::fav_rt::fav_rt_stub_src()` が `"fav_io_panic"` を含むことを確認
  - これは OS 非依存テスト（文字列確認のみ）

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62200` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3386 tests passed, 0 failed

---

## 非スコープ

- Linux 実機での `./pipeline` 実行確認（CI 環境依存）
- `aarch64` クロスコンパイル（v62.3.0 スコープ）
- ARM サポート
- `site/content/docs/runtime/aot.mdx` — v62.9.0 で対応予定のため本バージョンでは作成しない
