# v33.0.0 — Plan: Language Power マイルストーン宣言

## 実装方針

v32.0.0（Language Polish）と同じ「マイルストーン宣言」パターン。
新機能実装はなく、MILESTONE.md / README.md / CHANGELOG.md の更新と
`v330000_tests`（4 件、`include_str!` のみ）の追加が主な作業。

`cargo clean` は x.0.0 リリース時必須（ロードマップ明記）。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.9.0"` → `"33.0.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_9_0` スタブ化 + `v330000_tests` 追加 |
| `MILESTONE.md` | v33.0.0「Language Power」セクションを先頭に追加 |
| `README.md` | v32.0 説明文（Language Polish 詳細行）の直後に v33.0 マイルストーン宣言を追加 |
| `CHANGELOG.md` | `[v33.0.0]` セクションを先頭に追記 |
| `benchmarks/v33.0.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v33.0.0 に更新 |
| `versions/v30-v35/v33.0.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_9_0` をスタブ化

```rust
// v329000_tests 内（既存の #[test] fn を空スタブに置き換える）
fn cargo_toml_version_is_32_9_0() {
    // Stubbed: version bumped to 33.0.0 in v33.0.0.
}
```

### ② `v330000_tests` を挿入

挿入位置: `v329000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。
（`#[cfg(test)]` も含む v31.7.0 ブロック開始行より前）

```rust
// ── v33.0.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v330000_tests {
    #[test]
    fn cargo_toml_version_is_33_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.0.0"), "Cargo.toml must contain '33.0.0'");
    }

    #[test]
    fn milestone_language_power_declared() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Language Power"), "MILESTONE.md must contain 'Language Power'");
    }

    #[test]
    fn readme_mentions_v33_0() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v33.0"), "README.md must contain 'v33.0'");
    }

    #[test]
    fn benchmark_v33_0_0_exists() {
        let src = include_str!("../../benchmarks/v33.0.0.json");
        assert!(src.contains("33.0.0"), "benchmarks/v33.0.0.json must contain '33.0.0'");
    }
}
```

---

## cargo clean 手順

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
# hello.fav を復元
cat > tmp/hello.fav << 'EOF'
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() -> Bool {
    add(1, 2) == 3
}
EOF
cargo build
cargo test 2>&1 | grep "test result"
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.9.0 完了時点 | — | 2492 |
| `cargo_toml_version_is_32_9_0` スタブ化 | 0 | 2492 |
| `v330000_tests` 追加（4 件） | +4 | **2496** |
| `cargo clean` 後 | 0（件数変わらず） | **2496** |

---

## CHANGELOG 追記内容

```markdown
## [v33.0.0] — 2026-07-03

### Added
- `v330000_tests`: Language Power マイルストーン宣言確認テスト 4 件
  - `cargo_toml_version_is_33_0_0` — バージョン確認
  - `milestone_language_power_declared` — MILESTONE.md に「Language Power」記載確認
  - `readme_mentions_v33_0` — README.md に「v33.0」記載確認
  - `benchmark_v33_0_0_exists` — ベンチマークファイル存在確認
- `MILESTONE.md` — v33.0.0「Language Power」セクションを先頭に追加
- `README.md` — v33.0 マイルストーン宣言を追加

### Notes
- Language Power = 境界付きジェネリクス / 行多相 / where 制約 / スキーマ型 /
  線形型 / 分散アノテーション / 定数ジェネリクス / 型駆動 API 生成 / エフェクト推論
- `cargo clean` 実施（マイルストーン版の必須クリーンアップ）
```
