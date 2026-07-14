# v42.0.0 実装プラン — Type Precision 宣言 ★クリーンアップ

**フェーズ**: Type Precision（v41.x スプリント）
**目標テスト数**: 2874（+4）

---

## ステップ概要

1. `MILESTONE.md` — v42.0.0 エントリを先頭に追加
2. `README.md` — `Type Precision`（v42.0）記述追加
3. `fav/Cargo.toml` — version bump（41.9.0 → 42.0.0）
4. `CHANGELOG.md` — `[v42.0.0]` エントリ追加
5. `fav/src/driver.rs` — `v41900_tests` スタブ化 + `v42000_tests` 4 件追加
6. `cargo test` — 2874 passed / 0 failed を確認
7. ★`cargo clean` + `cargo test` 再確認

---

## Step 1: `MILESTONE.md` 更新

`## v41.0.0 — Streaming Foundations` の直前（ファイル先頭）に以下を挿入:

```markdown
## v42.0.0 — Type Precision（2026-07-12）

> 「`type Age = Int where (>= 0)` で値の意味を型に刻める。
>  タプルパターンとガード付き match でより精緻な分岐が書ける。
>  Newtype は内側の型の演算を自動継承する。
>
>  これが Favnir v42.0 — Type Precision の姿である。」

v42.0.0 をもって、Favnir の **Type Precision** を正式に宣言する。

### 達成コンポーネント（v41.1〜v41.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type alias | v41.1 | `type Age = Int where \|v\| v >= 0` |
| Refinement invariant + E0404〜E0406 | v41.2 | fav check 統合 |
| タプルパターン match | v41.3 | `match (status, count) { ... }` |
| ガード付き match | v41.4 | `n if n >= 90 => "A"` |
| Row polymorphism | v41.5 | record spread `{ ..u, active: true }` |
| Newtype 自動 impl | v41.6 | `type Kg(Float)` — 算術演算子自動委譲 |
| W030 lint | v41.7 | 冗長 refinement ガード検出 |
| Type Precision cookbook + docs | v41.8 | refinement-types.mdx 整備 |
| v42.0 前調整・安定化 | v41.9 | type-precision.mdx 新規作成 |

**宣言日**: 2026-07-12

---
```

---

## Step 2: `README.md` 更新

現在の v41.0 記述（行 107 付近）の直後に追加:

```markdown
**v42.0（2026-07-12）で、[Type Precision](./MILESTONE.md) マイルストーンを宣言しました。**
```

確認: `readme_mentions_type_precision` テストが `src.contains("Type Precision")` を検証する。

---

## Step 3: Cargo.toml バージョン bump

```toml
version = "42.0.0"
```

---

## Step 4: CHANGELOG.md 更新

`[v41.9.0]` の直前に追加:

```markdown
## [v42.0.0] — 2026-07-12

### Added
- `MILESTONE.md`: `v42.0.0 — Type Precision` マイルストーン宣言エントリを追加
- README.md に `Type Precision`（v42.0）の記述を追加
- driver.rs `v42000_tests` 4 件追加（`cargo_toml_version_is_42_0_0` / `changelog_has_v42_0_0` / `milestone_has_type_precision` / `readme_mentions_type_precision`）

### Changed
- `fav/Cargo.toml`: version `41.9.0` → `42.0.0`
```

---

## Step 5: driver.rs テスト更新

### 5a. `v41900_tests::cargo_toml_version_is_41_9_0` スタブ化

対象（NOTE コメント付きライブアサーション）を探し、以下に置き換え:

```rust
fn cargo_toml_version_is_41_9_0() {
    // Stubbed: version bumped to 42.0.0 -- assertion intentionally removed
}
```

### 5b. `v42000_tests` モジュール追加

> **挿入位置**: `v41900_tests` の直前（上）。driver.rs は「新しいバージョンが上」の降順配置。

```rust
// -- v42000_tests (v42.0.0) -- Type Precision 宣言 --
#[cfg(test)]
mod v42000_tests {
    #[test]
    fn cargo_toml_version_is_42_0_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.0.0"), "Cargo.toml must contain version 42.0.0");
    }

    #[test]
    fn changelog_has_v42_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v42.0.0]"), "CHANGELOG.md must contain [v42.0.0]");
    }

    #[test]
    fn milestone_has_type_precision() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Type Precision"), "MILESTONE.md must contain Type Precision");
    }

    #[test]
    fn readme_mentions_type_precision() {
        let src = include_str!("../../README.md");
        assert!(src.contains("Type Precision"), "README.md must mention Type Precision");
    }
}
```

---

## Step 6: cargo test（クリーンアップ前）

```bash
cargo test -j 8 -- --test-threads=8
```

確認事項:
- `2874 passed; 0 failed`
- `v42000_tests` 4 件すべて ok

---

## Step 7: ★cargo clean + 再確認

```bash
# fav/ ディレクトリで実行
cargo clean

# fav/tmp/hello.fav の存在確認（消えた場合は復元）
# 正しい内容:
#   fn add(a: Int, b: Int) -> Int { a + b }
#   fn main() -> Bool { add(1, 2) == 3 }

cargo test -j 8 -- --test-threads=8
# 2874 passed; 0 failed を確認
```

---

## 注意事項

- `include_str!("../../MILESTONE.md")` — `fav/src/driver.rs` から `../../` = `favnir/` root
- `include_str!("../../README.md")` — 同様に root の README.md
- `cargo clean` 後も `fav/tmp/hello.fav` は通常保持されるが、消えた場合は復元すること
- MILESTONE.md の先頭行は `# Favnir Milestones` → その直後に v42.0.0 エントリを挿入
