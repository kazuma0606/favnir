# v41.8.0 仕様書 — Type Precision cookbook

**フェーズ**: Type Precision（v41.x スプリント）
**前バージョン**: v41.7.0（W030 lint、2867 tests）
**目標テスト数**: 2868（+1）

---

## 概要

v41.1.0〜v41.7.0 で実装した Type Precision 機能（refinement type alias / W030 lint / Newtype 自動 impl）を
ユーザー向けドキュメントとして整備する。

1. **`site/content/cookbook/refinement-types.mdx`** — 新規作成（実用的なレシピ集）
2. **`site/content/docs/language/refinement-types.mdx`** — 更新（type alias refinement + W030 セクション追加）

---

## 現状確認

| ファイル | 状態 | 内容 |
|---|---|---|
| `site/content/cookbook/refinement-types.mdx` | **未作成** | — |
| `site/content/docs/language/refinement-types.mdx` | **存在する** | パラメータ refinement（`fn f(x: Int where { x > 0 })`）のみ記載。type alias refinement と W030 の記述が不足 |

---

## スコープ

### v41.8.0 に含む

- **cookbook 新規作成**: `site/content/cookbook/refinement-types.mdx`
  - frontmatter（`title`, `description`）
  - type alias refinement の実用例（ドメイン型 + W030 lint 例）
  - Newtype 自動 impl との組み合わせ例

- **docs 更新**: `site/content/docs/language/refinement-types.mdx`
  - 末尾に「Type Alias Refinement（v41.1.0+）」セクションを追加
  - 末尾に「W030: 冗長ガード lint（v41.7.0+）」セクションを追加

- **driver.rs テスト**: `v41800_tests`（1 件）

### スコープ外

- Newtype 専用 cookbook（v42.0+ で `type-precision.mdx` に統合）
- サイトのナビゲーション更新（静的サイト生成の設定変更は v41.9.0）

---

## 実装方針

### 1. `site/content/cookbook/refinement-types.mdx` 新規作成

frontmatter:
```markdown
---
title: "Refinement Type でドメイン制約を型に刻む"
description: "type alias の where 節で値の不変条件を型レベルで表現する実用パターン"
---
```

コンテンツ構成:
1. 基本: `type PositiveInt = Int where |v| v >= 0` の定義と使い方
2. 実用例: USD / Celsius / UserId などのドメイン型
3. W030 lint: 冗長ガードの検出例
4. Newtype 算術との組み合わせ（現時点では refinement と Newtype の直接組み合わせは未対応。将来バージョンで対応予定）

### 2. `site/content/docs/language/refinement-types.mdx` 更新

既存ファイルの末尾に以下を追加:

**注意**: 既存ファイルはパラメータ refinement（`where { b != 0 }` 形式）を扱う。
type alias refinement は `where |v| pred`（クロージャ形式）を使うが、両者は別機能。
追加セクションで「別機能である」ことを明示し、記法の混乱を防ぐ。

```markdown
## Type Alias Refinement（v41.1.0+）

パラメータ refinement（上記）とは別に、`type` alias に `where |v| pred` 節で
型レベルの invariant を定義できる。クロージャ変数 `v` が型の値を表す:

​```fav
type PositiveInt = Int    where |v| v >= 0
type Name        = String where |v| String.length(v) > 0
​```

## W030: 冗長ガード lint（v41.7.0+）

refinement 型の invariant が既に保証している条件を if で再確認すると W030 警告が出る:

​```fav
type PositiveInt = Int where |v| v >= 0

fn double(x: PositiveInt) -> Int {
  if x >= 0 {  // W030: redundant guard — PositiveInt invariant が既に保証している
    x * 2
  } else { 0 }
}
​```
```

### 3. driver.rs テスト（1 件）

```rust
mod v41800_tests {
    #[test]
    fn cargo_toml_version_is_41_8_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.8.0"), "Cargo.toml must contain version 41.8.0");
    }
}
```

**注意**: ロードマップは「1 件」だが、`cargo_toml_version_is_41_8_0` の 1 件のみとする。
cookbook ファイルの存在確認テストも追加できるが、テスト数が増えてロードマップの「+1」から外れる。
ロードマップ推定の 2868 を守るため 1 件のみとする。

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `site/content/cookbook/refinement-types.mdx` | 新規作成 | 中（約 80 行） |
| `site/content/docs/language/refinement-types.mdx` | 末尾にセクション追加 | 小（約 25 行） |
| `fav/src/driver.rs` | `v41700_tests::cargo_toml_version_is_41_7_0` スタブ化 + `v41800_tests` 追加（1 件） | 小 |
| `fav/Cargo.toml` | version: `41.7.0` → `41.8.0` | 1 行 |
| `CHANGELOG.md` | `[v41.8.0]` エントリ追加 | 数行 |

---

## テスト計画

### Rust テスト（driver.rs）— 1 件

```rust
mod v41800_tests {
    #[test]
    fn cargo_toml_version_is_41_8_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.8.0"), "Cargo.toml must contain version 41.8.0");
    }
}
```

---

## 完了条件

### 自動検証（cargo test）

- `cargo test` 全通過（2868 tests passed, 0 failed）
- `v41800_tests::cargo_toml_version_is_41_8_0` pass

### 実装者による手動確認

- `site/content/cookbook/refinement-types.mdx` が存在し、frontmatter・W030 記述・まとめ表を含む
- `site/content/docs/language/refinement-types.mdx` の末尾に「Type Alias Refinement」と「W030」セクションが追加されている
- 追加セクションでパラメータ refinement（`where { }`）と type alias refinement（`where |v|`）が別機能であることが明記されている
