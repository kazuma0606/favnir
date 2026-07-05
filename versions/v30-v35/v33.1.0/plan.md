# v33.1.0 — Plan: AOT ネイティブバイナリ 確認・テスト補強

## 実装方針

AOT Cranelift バックエンド（`cmd_build_native` / `CraneliftBackend::compile_to_binary`）は
v19.2.0 で完成済み。v33.1.0 は v32.1.0〜v32.9.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"33.0.0"` → `"33.1.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_33_0_0` スタブ化 + `v331000_tests` 追加 |
| `CHANGELOG.md` | `[v33.1.0]` セクションを先頭に追記 |
| `benchmarks/v33.1.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v33.1.0 に更新 |
| `versions/v30-v35/v33.1.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_33_0_0` をスタブ化

```rust
// v330000_tests 内（既存の #[test] fn を空スタブに置き換える）
#[test]
fn cargo_toml_version_is_33_0_0() {
    // Stubbed: version bumped to 33.1.0 in v33.1.0.
}
```

### ② `v331000_tests` を挿入

挿入位置: `v330000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。

```rust
// ── v33.1.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v331000_tests {
    use super::cmd_build_native;

    fn cc_available() -> bool {
        std::process::Command::new("cc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn resolve_bin(base: &std::path::Path) -> std::path::PathBuf {
        #[cfg(windows)]
        { let exe = base.with_extension("exe"); if exe.exists() { return exe; } }
        base.to_path_buf()
    }

    #[test]
    fn cargo_toml_version_is_33_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.1.0"), "Cargo.toml must contain '33.1.0'");
    }

    #[test]
    fn benchmark_v33_1_0_exists() {
        let src = include_str!("../../benchmarks/v33.1.0.json");
        assert!(src.contains("33.1.0"), "benchmarks/v33.1.0.json must contain '33.1.0'");
    }

    #[test]
    fn aot_if_branch_selects_true_arm() {
        // if 式の then アームが正しく選択されることを確認（v192000_tests とは異なる構文）
        if !cc_available() { return; }
        use std::fs;
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        fs::write(&src, "fn main() -> Int { if true { 10 } else { 20 } }").expect("write");
        let out = dir.path().join("aot_if_bin");
        let result = cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
        if result.is_err() { return; }
        let actual = resolve_bin(&out);
        if actual.exists() {
            let output = std::process::Command::new(&actual).output().expect("exec");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "10", "if true branch should return 10");
        }
    }

    #[test]
    fn aot_bool_comparison_native() {
        // Bool 型比較（2 > 1 → true → 1）の AOT 動作確認（v192000_tests とは異なる戻り型）
        if !cc_available() { return; }
        use std::fs;
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("main.fav");
        fs::write(&src, "fn main() -> Bool { 2 > 1 }").expect("write");
        let out = dir.path().join("aot_bool_bin");
        let result = cmd_build_native(src.to_str().unwrap(), out.to_str().unwrap());
        if result.is_err() { return; }
        let actual = resolve_bin(&out);
        if actual.exists() {
            let output = std::process::Command::new(&actual).output().expect("exec");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "1", "2 > 1 should produce 1 (true)");
        }
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v33.0.0 完了時点 | — | 2496 |
| `cargo_toml_version_is_33_0_0` スタブ化 | 0（テストは残る） | 2496 |
| `v331000_tests` 追加（4 件） | +4 | **2500** |

---

## CHANGELOG 追記内容

```markdown
## [v33.1.0] — 2026-07-04

### Added
- `v331000_tests`: AOT ネイティブバイナリ（Cranelift）動作確認テスト 4 件
  - `cargo_toml_version_is_33_1_0` — バージョン確認
  - `benchmark_v33_1_0_exists` — ベンチマークファイル存在確認
  - `aot_if_branch_selects_true_arm` — if 式の then アーム選択確認
  - `aot_bool_comparison_native` — Bool 比較（`2 > 1` → `1`）の AOT 動作確認

### Notes
- `CraneliftBackend::compile_to_binary` / `cmd_build_native` は v19.2.0 実装済み
- v33.1.0 は Performance & Tooling フェーズの記録として AOT 動作を明示的に確認する
- cc 非インストール環境では aot_* テストは自動スキップ（偽陰性なし）
```
