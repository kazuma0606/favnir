# v79.0.0 実装計画 — Execution Effects 1.0 宣言 ★クリーンアップ

Date: 2026-08-16

---

## 実装順序

### Step 1: cargo clean

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

ビルドキャッシュをクリアする。`fav/tmp/hello.fav` は target/ 外なので影響なし。

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に以下エントリを追加:

```
## [v79.0.0] — 2026-08-16 — Execution Effects 1.0 宣言 ★クリーンアップ

### Added
- Execution Effects 1.0 宣言（v78.1〜v78.9 の全 Execution Effects 基盤の完成を宣言）

### Tests
- `cargo_toml_version_is_79_0_0`: Cargo.toml のバージョンが 79.0.0 であることを検証
- `changelog_has_v79_0_0`: CHANGELOG.md に v79.0.0 エントリが存在することを検証
- `milestone_has_execution_effects`: MILESTONE.md に「Execution Effects 1.0」が存在することを検証
- `readme_mentions_execution_effects`: README.md に「Execution Effects」が存在することを検証
```

---

### Step 3: MILESTONE.md 更新

`## v78.0.0` 節の直前に v79.0.0 節を追加（spec.md 参照）。

---

### Step 4: README.md 更新

`## v78.0 — Verifiable Pipelines 宣言` の直前に v79.0 節を追加（spec.md 参照）。

---

### Step 5: driver.rs — v79000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.0.0: Execution Effects 1.0 宣言 ★クリーンアップ ---
#[cfg(test)]
mod v79000_tests {
    #[test]
    fn cargo_toml_version_is_79_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"79.0.0\""), "Cargo.toml version must be 79.0.0");
    }

    #[test]
    fn changelog_has_v79_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v79.0.0]"), "CHANGELOG.md must contain [v79.0.0]");
    }

    #[test]
    fn milestone_has_execution_effects() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(ms.contains("Execution Effects 1.0"), "MILESTONE.md must contain 'Execution Effects 1.0'");
    }

    #[test]
    fn readme_mentions_execution_effects() {
        let rm = include_str!("../../README.md");
        assert!(rm.contains("Execution Effects"), "README.md must mention 'Execution Effects'");
    }
}
```

注意: 宣言バージョンにつき `use super::*` は不要（外部シンボル未使用）。

---

### Step 6: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "78.9.0"` を `version = "79.0.0"` に更新。

driver.rs 内の `78.9.0` バージョン文字列アサーションを `79.0.0` に一括更新（`replace_all: true`）。

更新後に `grep -c "78.9.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v78.9.0: 安定化・コードフリーズ ---` の 1 件のみ）

---

### Step 7: versions/current.md 更新

- `## 進行中バージョン` → `**v79.0.0**（Execution Effects 1.0 宣言）`
- `## 次に切る版` → `**v79.1.0**（次スプリント開始予定）`

---

### Step 8: 最終確認

```bash
cargo test v79000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3787 tests pass、v79000 4 件 pass を確認。

---

## 依存順序サマリ

```
cargo clean
  → CHANGELOG 更新（Step 2）
  → MILESTONE 更新（Step 3）
  → README 更新（Step 4）
  → driver.rs テスト追加（Step 5）
  → Cargo.toml バージョン更新（Step 6）  ← Step 5 より後（include_str! の version 文字列チェック）
  → current.md 更新（Step 7）
  → 最終確認（Step 8）
```
