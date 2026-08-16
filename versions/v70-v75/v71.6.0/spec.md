# v71.6.0 仕様書 — AOT Native Compilation 本番品質化

Date: 2026-08-10
Status: 計画中

---

## Background

Favnir は v19.2.0 から Cranelift バックエンドによる AOT ネイティブコンパイルをサポートしている。
`fav build --target native pipeline.fav -o pipeline_bin` で ELF バイナリを生成できる。

しかし以下の制限が残っている：

1. **クロスコンパイル非対応（CLI レベル）**: `cranelift_aot.rs` の `lower_to_object_with_target` は
   aarch64 をサポートするが、`fav build --target native` に `--arch` フラグがなく CLI から利用できない。
2. **バイナリ最適化なし**: `link_binary` は `cc` でリンクするだけで `strip` をかけず、
   デバッグシンボルが残った大きいバイナリを生成する。
3. **テスト不足**: `aot_native_binary_compiles` / `aot_native_binary_runs_hello` が未実装。

v71.6.0 では上記 3 点を解消し、本番品質のネイティブバイナリ生成フローを完成させる。

---

## Goals

1. `fav build --target native --arch arm64 file.fav -o out` でクロスコンパイルができる
2. `link_binary` でリンク後に `strip` を自動実行してバイナリサイズを削減する
3. `compile_to_binary_for_arch(ir, out_path, arch)` 関数を追加してアーキテクチャを指定可能にする
4. `cmd_build_native_with_arch` ドライバ関数を追加する
5. `main.rs` に `--arch` フラグを追加する
6. テスト 2 件追加: `aot_native_binary_compiles` + `aot_native_binary_runs_hello`
7. テスト総数: 3599 + 2 = 3601 件

---

## Syntax / API

```bash
# ELF バイナリ生成（Linux x86_64 — 既存）
$ fav build --target native pipeline.fav -o pipeline_bin
Compiling pipeline.fav → native (linux/amd64)
Binary: ./pipeline_bin

# ARM64 クロスコンパイル（v71.6.0 新規）
$ fav build --target native --arch arm64 pipeline.fav -o pipeline_arm
Compiling pipeline.fav → native (linux/arm64)
Binary: ./pipeline_arm

# バイナリサイズ（strip 適用により削減）
$ ls -lh pipeline_bin
-rwxr-xr-x 1 user group 4.2M pipeline_bin  # v71.6.0: strip 前より小さい
```

---

## 実装スコープ

### 1. `fav/src/backend/cranelift_aot.rs`

#### `compile_to_binary_for_arch` 追加

```rust
/// v71.6.0: arch 指定付き compile_to_binary。
/// - `arch = None` → ホスト ISA（既存の compile_to_binary と同等）
/// - `arch = Some("arm64")` | `Some("aarch64")` → `"aarch64-unknown-linux-gnu"` triple
pub fn compile_to_binary_for_arch(
    ir: &IRProgram,
    out_path: &str,
    arch: Option<&str>,
) -> Result<(), String> {
    let triple = arch.and_then(Self::arch_to_triple);
    let obj_bytes = Self::lower_to_object_with_target(ir, triple)?;
    let wrapper_src = Self::c_wrapper_src();
    Self::link_binary(&obj_bytes, &wrapper_src, out_path)
}

fn arch_to_triple(arch: &str) -> Option<&'static str> {
    match arch {
        "arm64" | "aarch64" => Some("aarch64-unknown-linux-gnu"),
        _ => None,
    }
}
```

#### `link_binary` に strip 追加

リンク成功後、`strip` コマンドが利用可能であれば自動的に実行する。
`strip` が存在しない環境（Windows 等）では無視して続行する。

```rust
// リンク後の strip（バイナリサイズ最適化）
let _ = std::process::Command::new("strip")
    .arg(out_path)
    .output(); // strip 非対応環境では無視
```

**`compile_to_binary_for_arch` の可視性**: 既存の `compile_to_binary_pub` / `lower_to_object_pub` パターンと揃えるため
`compile_to_binary_for_arch` 自体を `pub(crate)` にして `driver.rs` から直接呼ぶ。
別途 `_pub` ラッパーは設けない（冗長なため）。

### 2. `fav/src/driver.rs`

#### `cmd_build_native_with_arch` 追加

```rust
/// v71.6.0: arch 指定付き native コンパイル（テスト用エントリポイント）
pub(crate) fn cmd_build_native_with_arch(
    src_path: &str,
    out_path: &str,
    arch: Option<&str>,
) -> Result<(), String> {
    let source = std::fs::read_to_string(src_path)
        .map_err(|e| format!("read error: {e}"))?;
    let program = crate::frontend::parser::Parser::parse_str(&source, src_path)
        .map_err(|e| format!("parse error: {e}"))?;
    let ir = compile_program(&program);
    crate::backend::cranelift_aot::CraneliftBackend::compile_to_binary_for_arch_pub(&ir, out_path, arch)
}
```

#### `cmd_build` "native" ブランチに `arch` パラメータ対応

`cmd_build` のシグネチャに `arch: Option<&str>` を追加し、
"native" ブランチで `compile_to_binary_for_arch` を呼ぶ。

```rust
pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>, arch: Option<&str>) {
    // ...
    "native" => {
        // ...
        CraneliftBackend::compile_to_binary_for_arch(&ir, out_path, arch)
        // ...
    }
}
```

#### `v716000_tests` 追加

```rust
#[cfg(test)]
mod v716000_tests {
    use super::cmd_build_native;

    fn cc_available() -> bool { ... }

    /// Cranelift オブジェクト生成が成功することを確認（cc 不要）
    #[test]
    fn aot_native_binary_compiles() {
        let src = "fn main() -> Int { 42 }";
        // lower_to_object_pub のみ使用（リンクなし）
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let ir = compile_program(&program);
        let result = CraneliftBackend::lower_to_object_pub(&ir);
        assert!(result.is_ok(), "lowering to object should succeed: {:?}", result);
        let obj = result.unwrap();
        assert!(!obj.is_empty(), "object bytes should be non-empty");
    }

    /// `main() -> Int { 42 }` をコンパイルして実行し、"42" が出力されることを確認
    #[test]
    fn aot_native_binary_runs_hello() {
        if !cc_available() { return; }
        // cmd_build_native → compile_to_binary → リンク実行
        // cc が存在する環境でのみ実行
    }
}
```

### 3. `fav/src/main.rs`

`fav build` の引数パース部分に `--arch` を追加:

```rust
"--arch" => {
    arch = Some(args.get(i + 1).unwrap_or_else(|| {
        eprintln!("error: --arch requires a value");
        process::exit(1);
    }));
    i += 2;
}
```

`cmd_build(file, out, target)` → `cmd_build(file, out, target, arch)` に引数追加。

---

## Error Codes

新規エラーコードなし。

---

## Success Criteria

- [x] `lower_to_object_pub` で `fn main() -> Int { 42 }` のオブジェクトバイトが非空で生成される
- [x] cc 利用可能環境で `cmd_build_native_with_arch(path, out, None)` が成功し `"42"` が出力される
- [x] `--arch arm64` フラグが `fav build --target native` で受け入れられる
- [x] `link_binary` が `strip` をサイレントに試行する
- [x] テスト総数: 3599 + 2 = 3601 件

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/backend/cranelift_aot.rs` | `compile_to_binary_for_arch` + `arch_to_triple` + strip in `link_binary` |
| `fav/src/driver.rs` | `cmd_build_native_with_arch` + `cmd_build` シグネチャ更新 + `v716000_tests` |
| `fav/src/main.rs` | `--arch` フラグ追加・`cmd_build` 呼び出し更新 |
| `fav/Cargo.toml` | `version = "71.6.0"` |
| `CHANGELOG.md` | v71.6.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン更新 |

**変更しないファイル**: `ast.rs`、`middle/checker.rs`、`fmt.rs`、`error_catalog.rs`

---

## 注意事項

- **ロードマップ項目「LTO」**: ロードマップは "LTO / strip" を列挙しているが、Cranelift オブジェクトに LTO を適用するには
  `cranelift-object` 以外のリンカ連携が必要であり v71.6.0 スコープ外とする。
  strip のみを実装し、LTO は将来バージョンに先送り。

- **ロードマップ項目「全 VM opcode を Cranelift IR に変換」**: 現行 `cranelift_aot.rs` は Int/Bool/BinOp/If/Block/Local をサポート。
  Match, Call, Stream 等の全 opcode カバレッジは本バージョンのスコープ外とする。
  v71.6.0 は --arch フラグと strip 追加に集中し、opcode 拡張は v72.x 以降とする。

- **未知 arch のフォールバック**: `arch_to_triple` で `_ => None` となるため、`--arch x86_64` 等の未知アーキテクチャは
  警告なしにホスト ISA にフォールバックする。意図的な設計であり、注意事項として文書化する。

- **cmd_build のシグネチャ変更影響**: `cmd_build` は `main.rs` から 1 箇所のみ呼び出されている（line 877）。
  テストモジュールからの直接呼び出しがないことを T0 で確認してから変更する。

- **Windows での strip**: `strip` コマンドは Windows では存在しないことが多い。
  `Command::output()` の Result は無視する（`let _ = ...`）。

- **cc_available ガード**: `aot_native_binary_runs_hello` は cc なし環境でスキップ。
  CI（Windows）でも fail しない設計にする。テストは `cmd_build_native_with_arch` を経由する。

- **テスト数ベース**: ロードマップの 3591+2=3593 は旧予測値。実績は 3599+2=3601。
  v72.0.0 宣言テスト（ロードマップ roadmap-v71.1-v72.0.md 308行の推移表）も実績と乖離しているため、
  v72.0.0 着手前にロードマップの推移表を実績ベースに更新すること。

- **サイト MDX**: `--arch` フラグは CLI 公開 API だが、サイト MDX 更新は v71.6.0 スコープ外とする。
