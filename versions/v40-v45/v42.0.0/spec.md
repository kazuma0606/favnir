# v42.0.0 仕様書 — Type Precision 宣言 ★クリーンアップ

**フェーズ**: Type Precision（v41.x スプリント）
**前バージョン**: v41.9.0（v42.0 前調整・安定化、2870 tests）
**目標テスト数**: 2874（+4）

---

## 概要

v41.1〜v41.9 の Type Precision スプリントを「**Type Precision 宣言**」として正式に宣言するマイルストーンバージョン。

**宣言文:**

> 「`type Age = Int where (>= 0)` で値の意味を型に刻める。
>  タプルパターンとガード付き match でより精緻な分岐が書ける。
>  Newtype は内側の型の演算を自動継承する。
>
>  これが Favnir v42.0 — Type Precision の姿である。」

Rust コードの機能追加はなし。ドキュメント・メタデータ整備と ★クリーンアップのみ。

---

## 現状確認

| ファイル | 状態 |
|---|---|
| `MILESTONE.md` | 先頭エントリは `v41.0.0 — Streaming Foundations`。`Type Precision` は未掲載 |
| `README.md` | v41.0 の記述あり。`Type Precision` は未掲載 |
| `fav/Cargo.toml` | version: `41.9.0` |
| `CHANGELOG.md` | `[v41.9.0]` が先頭エントリ |
| `fav/src/driver.rs` | `v41900_tests::cargo_toml_version_is_41_9_0` が NOTE コメント付きライブアサーション |
| `site/content/docs/type-precision.mdx` | 存在（v41.9.0 で作成済み） |

---

## スコープ

### v42.0.0 に含む

1. **`MILESTONE.md` 更新** — `v42.0.0 — Type Precision` エントリを先頭に追加
2. **`README.md` 更新** — `Type Precision`（v42.0）の記述を追加
3. **`fav/Cargo.toml`** — version: `41.9.0` → `42.0.0`
4. **`CHANGELOG.md`** — `[v42.0.0]` エントリを `[v41.9.0]` の直前に追加
5. **`fav/src/driver.rs`** — `v41900_tests` スタブ化 + `v42000_tests` 4 件追加
6. **`cargo test`** — 全通過（2874 tests, 0 failed）
7. **★`cargo clean`** — クリーンアップ実施 + `cargo test` 再通過確認

### スコープ外

- 新規言語機能
- `site/content/docs/type-precision.mdx` 変更（v41.9.0 で作成済み）
- 新規 MDX ファイル作成

---

## 実装方針

### 1. `MILESTONE.md` 更新

v41.0.0 エントリの直前（先頭）に v42.0.0 エントリを追加:

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
```

### 2. `README.md` 更新

v41.0 記述の直後に v42.0 の一行を追加:

```markdown
**v42.0（2026-07-12）で、[Type Precision](./MILESTONE.md) マイルストーンを宣言しました。**
```

### 3. `driver.rs` テスト更新

#### 3a. `v41900_tests::cargo_toml_version_is_41_9_0` スタブ化

```rust
fn cargo_toml_version_is_41_9_0() {
    // Stubbed: version bumped to 42.0.0 -- assertion intentionally removed
}
```

#### 3b. `v42000_tests` モジュール追加（`v41900_tests` の直前）

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

`v42000_tests` は `include_str!` のみ使用のため `use super::*` 不要。

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `MILESTONE.md` | v42.0.0 エントリ追加（先頭） | 中（約 25 行） |
| `README.md` | `Type Precision` 記述 1 行追加 | 小 |
| `fav/Cargo.toml` | version: `41.9.0` → `42.0.0` | 1 行 |
| `CHANGELOG.md` | `[v42.0.0]` エントリ追加 | 数行 |
| `fav/src/driver.rs` | `v41900_tests` スタブ化 + `v42000_tests` 追加（4 件）※スタブ化はアサーション除去のみ、総数増減に影響しない | 小 |

Rust ソースコード変更なし（宣言・クリーンアップのみ）。

---

## テスト計画

### Rust テスト（driver.rs）— 4 件

```rust
mod v42000_tests {
    #[test]
    fn cargo_toml_version_is_42_0_0() { /* include_str! Cargo.toml */ }
    #[test]
    fn changelog_has_v42_0_0() { /* include_str! CHANGELOG.md */ }
    #[test]
    fn milestone_has_type_precision() { /* include_str! MILESTONE.md */ }
    #[test]
    fn readme_mentions_type_precision() { /* include_str! README.md */ }
}
```

テスト数: 2870 + 4 = **2874**

---

## ★cargo clean 手順

1. `cargo test` 全通過（2874 tests, 0 failed）を確認
2. `cargo clean` を実行
3. `fav/tmp/hello.fav` の存在を確認（v41.0.0 実績では `cargo clean` 後も保持された）
   - 万一消えた場合: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }` で復元
4. `cargo test` を再実行し 2874 passed / 0 failed を確認

---

## 完了条件

### 自動検証（cargo test）

- `cargo test` 全通過（2874 tests passed, 0 failed）
- `v42000_tests::cargo_toml_version_is_42_0_0` pass
- `v42000_tests::changelog_has_v42_0_0` pass
- `v42000_tests::milestone_has_type_precision` pass
- `v42000_tests::readme_mentions_type_precision` pass

### 実装者による手動確認

- `MILESTONE.md` の先頭に `v42.0.0 — Type Precision` エントリが存在する
- `README.md` に `Type Precision` の記述が含まれる
- `cargo clean` 完了・`cargo test` 再通過確認
