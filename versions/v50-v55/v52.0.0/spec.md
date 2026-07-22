# Spec: v52.0.0 — Performance & Scale 宣言

Date: 2026-07-20
Status: 設計中

---

## 目的

v51.1〜v51.9 で実装した Performance & Scale 機能群（par 並列実行・バックプレッシャー・ベンチマーク回帰検出・インクリメンタルコンパイル・WASM サイズ最適化）を統合し、
**Favnir v52.0 — Performance & Scale** を正式宣言する。

**宣言文**:

> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

---

## 成果物

### `MILESTONE.md` 更新

先頭に v52.0.0 エントリを追加する。
必須キーワード（テストで検証）: `"Performance & Scale"` を含む。

### `README.md` 更新

`Performance & Scale` マイルストーンへの言及を追加する。
必須キーワード（テストで検証）: `"Performance & Scale"` を含む。

### `CHANGELOG.md` 更新

v52.0.0 エントリを追加する（テストで `v52.0.0` を検証）。

### ★クリーンアップ — `cargo clean`

`cargo clean` でビルドキャッシュを削除する。
`cargo test` が clean state から全通過することを確認する。

---

## テスト仕様

### `cargo_toml_version_is_52_0_0`

```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"52.0.0\""),
    "Cargo.toml version should be 52.0.0");
```

### `changelog_has_v52_0_0`

```rust
let content = include_str!("../../CHANGELOG.md");
assert!(content.contains("v52.0.0"),
    "CHANGELOG.md must contain v52.0.0 entry");
```

### `milestone_has_performance_scale`

```rust
let content = include_str!("../../MILESTONE.md");
assert!(content.contains("Performance & Scale"),
    "MILESTONE.md must contain Performance & Scale");
```

### `readme_mentions_performance_scale`

```rust
let content = include_str!("../../README.md");
assert!(content.contains("Performance & Scale"),
    "README.md must mention Performance & Scale");
```

`include_str!` のパス（`fav/src/driver.rs` 起点）:
- `../Cargo.toml` → `fav/Cargo.toml`
- `../../CHANGELOG.md` → `favnir/CHANGELOG.md`
- `../../MILESTONE.md` → `favnir/MILESTONE.md`
- `../../README.md` → `favnir/README.md`

---

## テスト数

- ベース: 3133（v51.9.0 完了時点）
- `cargo_toml_version_is_51_9_0` 削除: -1
- 新規追加: +4（`v52000_tests` 4 件）
- **完了後合計: 3136 tests passed, 0 failed**（≥ 3135 の要件を満たす）

---

## 完了条件

- `MILESTONE.md` に v52.0.0「Performance & Scale」エントリ追加
- `README.md` に `Performance & Scale` 言及追加
- `CHANGELOG.md` に v52.0.0 エントリ追加
- `fav/Cargo.toml` version → `"52.0.0"`
- `cargo clean` 完了（★クリーンアップ）
- `cargo test` 3136 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `versions/current.md` を v52.0.0（3136 tests）に更新
- `roadmap-v51.1-v52.0.md` の v52.0.0 実績欄を更新
