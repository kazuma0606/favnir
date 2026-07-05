# v31.1.0 実装計画 — エラーメッセージ v2（rustc スタイル）

## 前提

- `fav/Cargo.toml` version = `31.0.0`
- `cargo test` — 2422 passed（0 failures）
- v31.0.0 が COMPLETE であること

---

## 事前調査（実施済み結果）

| 項目 | 確認結果 |
|---|---|
| `error_catalog.rs::error_hint()` | 存在しない。`lookup()` / `list_all()` のみ |
| `driver.rs::format_diagnostic()` (line 47) | 実装済み — rustc スタイル完全実装 |
| `driver.rs::get_help_text()` (line 150) | 実装済み — E0001/E0007/E0008/E0009/E0013/E0014/E0015/E0018 のみ |
| `TypeError` 構造体 | `span`, `hints: Vec<String>` フィールドあり |
| `fmt.rs` の `format_diagnostic()` | 存在しない（`fmt.rs` は AST フォーマッター専用） |

**本バージョンの実装作業**: `driver.rs::get_help_text()` に E0002〜E0006、E0010 を追加するのみ。

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.0.0"` → `version = "31.1.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`**
- `v310000_tests::cargo_toml_version_is_31_0_0` をスタブ化（コメント付き）

### Step 3: get_help_text() 拡充

**`fav/src/driver.rs`** の `get_help_text()` 関数（line 150 付近）に以下を追加:

```rust
"E0002" => &[
    "the condition must be a Bool expression",
],
"E0003" => &[
    "pattern match requires an enum type or literal",
],
"E0004" => &[
    "the right-hand side of bind must return Result<T>",
],
"E0005" => &[
    "check that the type annotation matches the inferred type",
],
"E0006" => &[
    "all match arms must return the same type",
],
"E0010" => &[
    "implement all required methods declared in the interface",
],
```

> 既存アーム（E0001/E0007/E0008/E0009 等）の前後に挿入する。

### Step 4: v311000_tests 追加

**`fav/src/driver.rs`** の v310000_tests モジュールの前に追加:

```rust
// ── v31.1.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v311000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.1.0\""), "Cargo.toml must contain version = \"31.1.0\"");
    }
    #[test]
    fn benchmark_v31_1_0_exists() {
        let src = include_str!("../../benchmarks/v31.1.0.json");
        assert!(src.contains("31.1.0"), "benchmarks/v31.1.0.json must contain '31.1.0'");
    }
    #[test]
    fn get_help_text_e0002_is_set() {
        let hints = get_help_text("E0002");
        assert!(!hints.is_empty(), "get_help_text(\"E0002\") must return non-empty hints");
    }
    #[test]
    fn get_help_text_e0005_is_set() {
        let hints = get_help_text("E0005");
        assert!(!hints.is_empty(), "get_help_text(\"E0005\") must return non-empty hints");
    }
}
```

> `use super::*` あり（`get_help_text` は `driver.rs` 内の非 pub 関数のため）。

### Step 5: CHANGELOG.md 追記

先頭に追加:

```markdown
## [v31.1.0] — 2026-07-02

### Changed
- `driver.rs::get_help_text()` — E0002/E0003/E0004/E0005/E0006/E0010 に hint を追加
- `Cargo.toml` version: `31.0.0` → `31.1.0`

### Added
- `benchmarks/v31.1.0.json` 追加
```

### Step 6: benchmarks/v31.1.0.json 作成

```json
{
  "version": "31.1.0",
  "date": "2026-07-02",
  "milestone": "Language Polish",
  "tests_passed": 2426,
  "tests_failed": 0,
  "notes": "get_help_text() extended for E0002-E0006, E0010"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+4 件 = 2426 想定）。

### Step 7: versions/current.md 更新

- 「最新安定版」欄を v31.1.0 に更新
- 「進行中バージョン」を「なし（v31.1.0 完了直後）」に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.0.0` → `31.1.0` |
| `fav/src/driver.rs` | 更新 | v310000 スタブ化 + `get_help_text()` 拡充 + v311000_tests 追加 |
| `CHANGELOG.md` | 更新 | [v31.1.0] セクション追加 |
| `benchmarks/v31.1.0.json` | 新規 | ベンチマーク結果 |
| `versions/current.md` | 更新 | v31.1.0 に更新 |

---

## 完了判定

- `cargo test v311000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
