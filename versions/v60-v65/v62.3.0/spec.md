# v62.3.0 Spec — `fav build --target` クロスコンパイルサポート

Version: 62.3.0
Status: 未着手
Base tests: 3386
Target tests: 3388

---

## 概要

`fav build --target <triple>` でクロスコンパイルをサポートする。
`cranelift-codegen` features に `"arm64"` を追加し、
`lower_to_object` のホスト固定 ISA 選択を target triple 指定に対応させる。

サポート triple:
- `x86_64-unknown-linux-gnu`（ホストネイティブにフォールバック）
- `aarch64-unknown-linux-gnu`（クロスコンパイル）

注: ロードマップ記述の `aot.rs` は `cranelift_aot.rs` を指す（v62.1.0 実績確立済み）。

---

## 前提確認（T0 で実施）

- `cranelift-codegen` features は現在 `["x86"]` のみ → `"arm64"` 追加が必要
- `cranelift-native = { version = "0.117" }` は Cargo.toml に登録済み
- `target_lexicon` は Cargo.toml に **登録されていない** → aarch64 lookup に必要な場合は追加要
- `lower_to_object` は `impl CraneliftBackend` の private メソッド（`cranelift_native::builder()` ホスト固定）
- `lower_to_object_with_target` は `cranelift_aot.rs` に **存在しない**
- `cmd_build_link_target` は `driver.rs` に **存在しない**
- `main.rs` `--link` ブランチは `cmd_build_link` を呼ぶ → `cmd_build_link_target` に切り替える

---

## 実装スコープ

### 1. `Cargo.toml` — `cranelift-codegen` features に `"arm64"` 追加

```toml
cranelift-codegen = { version = "0.117", features = ["x86", "arm64"] }
```

`target_lexicon` クレートの追加が必要な場合は同時に追加する（T0 で判断）。

### 2. `cranelift_aot.rs` — `lower_to_object_with_target` 追加（`impl CraneliftBackend` 内）

`lower_to_object` はホスト固定のため変更せず、`impl CraneliftBackend` ブロック内に新メソッドを追加する。

```rust
fn lower_to_object_with_target(
    ir: &IRProgram,
    target: Option<&str>,
) -> Result<Vec<u8>, String>
```

ISA 選択ロジック:
- `None` または `"x86_64-unknown-linux-gnu"` → `cranelift_native::builder()`（既存ロジック流用）
- `"aarch64-unknown-linux-gnu"` → cranelift の aarch64 ISA builder を使用
- その他 → `Err(format!("unsupported target triple: {t}"))`

pub(crate) ラッパーも `impl CraneliftBackend` 内に追加:
```rust
pub(crate) fn lower_to_object_with_target_pub(
    ir: &IRProgram,
    target: Option<&str>,
) -> Result<Vec<u8>, String> {
    Self::lower_to_object_with_target(ir, target)
}
```

**cranelift API 確認ポイント（T0 で判断）**:
- `cranelift_codegen::isa::lookup_by_name("aarch64")` が利用可能かを確認する
- 利用不可の場合: `target_lexicon::Triple` をパースして `cranelift_codegen::isa::lookup(triple)` を使う
  → その場合 `target_lexicon` クレートを Cargo.toml に追加する

### 3. `main.rs` — `--link` ブランチに target triple を接続

`--link` ブランチで `target` が triple 形式（`"-"` を含む）なら AOT target として `cmd_build_link_target` に渡す。

```rust
if link {
    let aot_target = match target {
        Some(t) if t.contains('-') => Some(t),
        _ => None,
    };
    println!("{}", driver::cmd_build_link_target(&src, out_path, aot_target));
}
```

Note: `target` 変数は graphql/proto/schema と共用だが、これらは `"-"` を含まないため衝突しない。

### 4. `driver.rs` — `cmd_build_link_target` 追加・`cmd_build_link` 変更

新関数:
```rust
pub fn cmd_build_link_target(src: &str, out: &str, target: Option<&str>) -> String
```
- parse → compile → `lower_to_object_with_target_pub(ir, target)`（object bytes のみ生成）
- 成功時: `format!("Output: {} ({} bytes)", out, bytes.len())`
- エラー時: `format!("build error: {e}")`

既存関数の変更:
```rust
pub fn cmd_build_link(src: &str, out: &str) -> String {
    cmd_build_link_target(src, out, None)
}
```
（`v62200_tests` の `cmd_build_link` 呼び出しはそのまま動作する）

### 5. `driver.rs` — `v62300_tests` 追加（3 assertions）

- `aot_cross_compile_aarch64`:
  - `lower_to_object_with_target_pub(ir, Some("aarch64-unknown-linux-gnu"))` が `Ok(bytes)` かつ `!bytes.is_empty()`
- `aot_target_triple_parsed`:
  - `lower_to_object_with_target_pub(ir, None)` が `Ok(_)`
  - `lower_to_object_with_target_pub(ir, Some("unsupported-triple"))` が `Err(_)`
  - `cmd_build_link_target(src, "out", Some("aarch64-unknown-linux-gnu"))` の結果が `"parse error:"` を含まない（CLI 経路のスモーク確認）

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62300` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3388 tests passed, 0 failed

---

## 非スコープ

- `aarch64` バイナリの実機実行確認
- `--target` の `--link` 以外（object のみ出力）への対応
- `site/content/docs/runtime/aot.mdx` — v62.9.0 スコープのため作成しない
