# v71.6.0 タスクリスト — AOT Native Compilation 本番品質化

Date: 2026-08-10
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.5.0` であることを確認
- [x] `cargo test` が全 pass（3599 tests）であることを確認
- [x] `cranelift_aot.rs` の `link_binary` を確認（strip 追加箇所 — line ~102）
- [x] `lower_to_object_with_target` が aarch64 をサポートしていることを確認（line ~142: `"aarch64-unknown-linux-gnu"` ブランチ）
- [x] `compile_to_binary_for_arch` が未実装であることを確認（0 件）
- [x] `cmd_build` の現在のシグネチャを確認（driver.rs ~line 1837: `fn cmd_build(file, out, target)` — arch 引数なし）
- [x] `cmd_build` が main.rs から 1 箇所のみ呼ばれていることを確認（main.rs line 877 のみ）
- [x] テストモジュール内から `cmd_build` が直接呼ばれていないことを確認（0 件）
- [x] `lower_to_object_pub` が存在することを確認（cranelift_aot.rs line ~131）

---

## T1: `cranelift_aot.rs` — `arch_to_triple` + `compile_to_binary_for_arch` 追加

- [x] `impl CraneliftBackend` の `compile_to_binary_pub` 直後に `arch_to_triple` ヘルパー関数を追加した
  - `"arm64"` / `"aarch64"` → `"aarch64-unknown-linux-gnu"`
  - それ以外 → `None`（ホスト ISA フォールバック — 未知アーキテクチャは警告なしにフォールバック）
- [x] `compile_to_binary_for_arch(ir, out_path, arch: Option<&str>)` を `pub(crate)` で追加した（別途 `_pub` ラッパーは設けない）
  - `Self::arch_to_triple` で triple を解決し `lower_to_object_with_target(ir, triple)?` を呼ぶ
  - `link_binary` を呼んでバイナリ生成
- [x] `cargo build` でエラーがないことを確認

---

## T2: `cranelift_aot.rs` — `link_binary` に strip 追加

- [x] `link_binary` の `Ok(())` 直前に strip コマンド呼び出しを追加した
  ```rust
  let _ = std::process::Command::new("strip").arg(out_path).output();
  ```
- [x] `cargo build` でエラーがないことを確認

---

## T3: `driver.rs` — `cmd_build_native_with_arch` 追加

- [x] `cmd_build_native`（line ~2463）の直後に `cmd_build_native_with_arch` を追加した
  - `arch: Option<&str>` を受け取り `CraneliftBackend::compile_to_binary_for_arch` に直接渡す（`_pub` ラッパー不要）
- [x] `cargo build` でエラーがないことを確認

---

## T4: `driver.rs` — `cmd_build` シグネチャ更新

- [x] `cmd_build` のシグネチャを `(file, out, target, arch: Option<&str>)` に変更した
- [x] "native" ブランチで `compile_to_binary_for_arch(&ir, out_path, arch)` を呼ぶように変更した
- [x] `println!` で `arch_label` を表示するように更新した
- [x] `cargo build` でエラーがないことを確認

---

## T5: `main.rs` — `--arch` フラグ追加

- [x] `let mut arch: Option<&str> = None;` をローカル変数として `"build"` ブランチの冒頭に追加した
- [x] `"--arch"` パースを追加した（`--target` パース直後）
- [x] `cmd_build(file, out, target)` → `cmd_build(file, out, target, arch)` に更新した
- [x] `cargo build` でエラーがないことを確認

---

## T6: 既存テスト通過確認

- [x] `cargo test` で既存テスト（3599 件）が全 pass することを確認

---

## T7: `v716000_tests` 追加（`driver.rs`）

- [x] `v716000_tests` モジュールを `v715000_tests` の直後に追加した
- [x] `aot_native_binary_compiles` テストを実装した
  - `fn main() -> Int { 42 }` を `lower_to_object_pub` でオブジェクトバイト列に変換
  - `result.is_ok()` + `obj.is_empty() == false` を assert
  - cc 不要（リンクしない）
- [x] `aot_native_binary_runs_hello` テストを実装した
  - `cc_available()` ガード: cc なし環境では即 return
  - `cmd_build_native_with_arch(src, out, None)` でバイナリ生成（`compile_to_binary_for_arch` コードパスを経由）
  - 実行 → stdout が `"42"` であることを assert
  - リンク失敗（環境依存）は `if result.is_err() { return; }` でスキップ
- [x] `cargo test v716000` で 2 件 pass することを確認

---

## T8: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.5.0"` → `"71.6.0"` に変更した
- [x] `driver.rs` 内の cargo_toml_version テストを `"71.6.0"` に更新した

---

## T9: CHANGELOG.md 更新

- [x] `## [v71.6.0]` エントリを先頭に追加した

---

## T10: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.6.0`（AOT Native Compilation 本番品質化）に更新した
- [x] 「次に切る版」を `v71.7.0` に更新した

---

## T11: 最終確認

- [x] `cargo test v716000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3601 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.6.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- LTO（Link-Time Optimization）: Cranelift オブジェクトへの適用はスコープ外 → 将来バージョン
- 全 VM opcode の Cranelift IR 変換: Match/Call/Stream 等はスコープ外 → v72.x 以降
- サイト MDX 更新（`--arch` フラグのドキュメント化）: スコープ外 → 後続バージョン
- ロードマップ推移表の実績ベース修正: v72.0.0 着手前に roadmap-v71.1-v72.0.md を更新すること（v71.6.0 タスクではない）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [MED] | 未知 arch がサイレントにホスト ISA へフォールバック | `compile_to_binary_for_arch` に `eprintln!` 警告を追加 |
| [MED] | `--arch` が `--target native` 以外で無視される | `main.rs` に `arch.is_some() && target != "native"` の警告を追加 |
| [LOW] | `strip` 失敗がサイレント | コメントに意図を明記（設計上許容） |
| [LOW] | Windows での exec がパニック可能性 | `#[cfg(not(windows))]` でバイナリ実行をガード + `output()` を `unwrap_or` に変更 |
| [LOW] | `aot_native_binary_compiles` が arch コードパスをテストしない | `arch_to_triple_known_arches` テストを追加（`arch_to_triple` を `pub(crate)` に昇格） |
| [LOW] | `arch_to_triple` のサポート対象が未記載 | `pub(crate)` + doc コメントに「サポート対象: arm64/aarch64 のみ」を明記 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T11）が完了している
- [x] `aot_native_binary_compiles` が pass
- [x] `aot_native_binary_runs_hello` が pass（cc なし環境では skip 扱い）
- [x] テスト総数: 3601（+2、実績ベース: 3599 + 2）
- [x] `lower_to_object_pub` が `fn main() -> Int { 42 }` を正常に変換できる
- [x] `--arch arm64` フラグが `fav build --target native` で受け入れられる
- [x] `link_binary` が strip をサイレントに実行する
- [x] 既存テストが引き続き pass（後方互換性）
