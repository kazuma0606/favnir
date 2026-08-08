# Spec — v56.9.0 — 安定化・コードフリーズ（Language Power 2.0 前調整）

## 概要

Language Power 2.0 スプリント（v56.1〜v56.8）の全成果を整理・検証し、v57.0 宣言に向けてコードを安定化する。
主な成果物は以下の 2 件:

1. **`site/content/docs/language-power2-overview.mdx`** — Language Power 2.0 全機能の俯瞰ページ（新規作成）
2. **`v56900_tests`** — バージョンチェック + 概要ページ存在確認テスト（2 件追加）

加えて、`cargo clippy -- -D warnings` クリーンを再確認し、v56.1〜v56.8 の全テストが通過していることを検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.9.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.9.0 行
- ベーステスト数: **3246**（v56.8.0 完了時点の実績値）
- 目標テスト数: **3248**（+2）

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.9.0"
```

---

### 2. `site/content/docs/language-power2-overview.mdx` — 新規作成

Language Power 2.0 スプリント（v56.1〜v56.8）の全機能を俯瞰する概要ページ。
各機能の要点・対応バージョン・リンクを 1 ページにまとめる。

**主要セクション**:

- **概要** — Language Power 2.0 の位置づけと設計思想
- **境界付きジェネリクス（v56.1〜v56.2）** — `where T: Interface`・複数 constraint・coherence E0423
- **行多相レコード（v56.3）** — `{ field: Type | r }` 行変数明示・LSP ホバー
- **エフェクト推論 LSP 統合（v56.4）** — inlay hints・`fav check --show-types`
- **OR パターン + ガード強化（v56.5）** — `Ok(x) | Err("retry")`・W037 到達不能パターン
- **as-パターン（v56.6）** — `head @ { id, amount }` バインディング
- **モジュール名前空間（v56.7）** — `import "path" as alias.*`・W038
- **ドキュメント更新（v56.8）** — `bounded-generics.mdx`・`row-polymorphism.mdx`・`effect-inference.mdx`
- **次のステップ** — v57.0 Language Power 2.0 宣言への案内

```mdx
# Language Power 2.0 — Overview

Favnir v56 series (v56.1–v56.9) brings the Language Power 2.0 milestone:
type system features that let developers express intent precisely.

## What's Included

| Version | Feature | Key Addition |
|---|---|---|
| v56.1–56.2 | Bounded Generics | `where T: Interface`, E0422/E0423 |
| v56.3 | Row Polymorphism | `{ field: Type \| r }` row variables |
| v56.4 | Effect Inference LSP | inlay hints, `--show-types` |
| v56.5 | OR Patterns + Guards | `Ok(x) \| Err("retry")`, W037 |
| v56.6 | as-Patterns | `head @ { id, amount }` |
| v56.7 | Module Namespaces | `import "path" as alias.*`, W038 |
| v56.8 | Documentation | bounded-generics, row-polymorphism, effect-inference |
```

---

### 3. `fav/src/driver.rs` — `v56900_tests` 追加

`v56800_tests` の直前に挿入する。

**テスト 1: `cargo_toml_version_is_56_9_0`**

```rust
#[test]
fn cargo_toml_version_is_56_9_0() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("version = \"56.9.0\""),
        "Cargo.toml version should be 56.9.0, got: {}",
        cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
    );
}
```

**テスト 2: `language_power2_overview_exists`**

```rust
#[test]
fn language_power2_overview_exists() {
    let content = include_str!(
        "../../site/content/docs/language-power2-overview.mdx"
    );
    assert!(
        content.contains("Language Power 2.0"),
        "language-power2-overview.mdx should mention Language Power 2.0"
    );
    assert!(
        content.contains("bounded-generics"),
        "language-power2-overview.mdx should reference bounded-generics (v56.1-56.2)"
    );
    assert!(
        content.contains("row-polymorphism"),
        "language-power2-overview.mdx should reference row-polymorphism (v56.3)"
    );
    assert!(
        content.contains("effect-inference"),
        "language-power2-overview.mdx should reference effect-inference (v56.4)"
    );
}
```

---

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.8.0"` → `"56.9.0"` に更新。
（モジュール名・関数名は慣例として変更しない）

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_56_9_0` | `Cargo.toml` version が `"56.9.0"` である |
| `language_power2_overview_exists` | `language-power2-overview.mdx` が `"Language Power 2.0"` / `"bounded-generics"` / `"row-polymorphism"` / `"effect-inference"` を含む |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3248 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_56_9_0` pass
- `language_power2_overview_exists` pass
- `site/content/docs/language-power2-overview.mdx` が新規作成されている
- `language-power2-overview.mdx` に v56.1〜v56.8 の全機能が記載されている
- `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.9.0"` になっている
- `CHANGELOG.md` に v56.9.0 エントリが追加されている
- `versions/current.md` が v56.9.0 / 3248 tests を反映
- 両ロードマップの v56.9.0 実績を COMPLETE に更新

---

## 備考

- **`language-power2-overview.mdx` のパス**: `site/content/docs/language-power2-overview.mdx`
  （`language/` サブディレクトリではなく `docs/` 直下）
- **`include_str!` のパス**: `driver.rs`（`fav/src/driver.rs`）から `../../` で 2 段上がると
  プロジェクトルート（`fav/` の親）に到達し、そこから `site/content/docs/language-power2-overview.mdx`
- **テスト数**: `v56900_tests` に 2 件追加。ベース 3246 + 2 = 3248。
- **clippy 確認**: v56.1〜v56.8 の実装で追加した lint ルール（W037・W038）が自己適用されないことを確認する
- **「コードフリーズ」の意味**: v56.9.0 では新機能追加なし。安定化・ドキュメント整備のみ。
