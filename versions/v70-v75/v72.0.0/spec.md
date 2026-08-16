# v72.0.0 spec — Type System 2.0 宣言 ★クリーンアップ

Date: 2026-08-11

---

## Background

v71.1〜v71.9 で Type System 2.0 の全機能を実装・安定化した:

| バージョン | 機能 |
|---|---|
| v71.1 | 依存型 `Vec<T>[N]` + E0421 次元不一致 |
| v71.2 | Refined Types（`where self > expr`）+ E0425 |
| v71.3 | Phantom Types（`type X = phantom T`）|
| v71.4 | Const / Compile-Time Evaluation（`const N: Int = expr`）|
| v71.5 | Generic Constraints（`<T: A & B>`、`<T: impl A>`）|
| v71.6 | AOT Native Compilation 本番品質化（`--arch arm64`、strip）|
| v71.7 | WebAssembly テストカバレッジ確立 |
| v71.8 | 型推論強化（型注釈省略）|
| v71.9 | 安定化・コードフリーズ（E2E テスト）|

v72.0.0 でこれらをまとめて **Type System 2.0** として宣言し、クリーンアップを実施する。

---

## 宣言文

> 「依存型がベクトルの次元を守り、refined type がゼロ除算を型で止める。
>  Phantom type が ID の混用を防ぎ、定数がコンパイル時に評価される。
>  AOT バイナリが Docker 不要で動き、WASM がパイプラインをブラウザへ運ぶ。
>
>  これが Favnir v72.0 — Type System 2.0 の姿である。」

---

## Goals

1. `cargo clean` でビルドアーティファクトをクリーンアップ
2. `Cargo.toml` バージョンを `72.0.0` に更新
3. `CHANGELOG.md` に `v72.0.0` エントリを追加
4. `MILESTONE.md` に「Type System 2.0」マイルストーンを追記
5. `README.md` に v72.0 達成を追記
6. `versions/current.md` を更新（進行中: v72.0.0、次: v72.1.0）
7. `v72000_tests` 4 件を追加

---

## テスト詳細

```rust
// v72000_tests — 宣言チェック（v71000_tests と同パターン）

fn cargo_toml_version_is_72_0_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("version = \"72.0.0\""), "Cargo.toml should declare version 72.0.0");
}

fn changelog_has_v72_0_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("[v72.0.0]"), "CHANGELOG.md should have v72.0.0 entry");
}

fn milestone_has_type_system_2() {
    let src = include_str!("../../MILESTONE.md");
    assert!(src.contains("Type System 2.0"), "MILESTONE.md should mention Type System 2.0");
}

fn readme_mentions_type_system_2() {
    let src = include_str!("../../README.md");
    assert!(
        src.contains("Type System 2.0") || src.contains("v72.0"),
        "README.md should mention Type System 2.0 or v72.0"
    );
}
```

---

## Success Criteria

- `cargo test v72000` で 4 件 pass（0 failures）
  - `cargo_toml_version_is_72_0_0` pass
  - `changelog_has_v72_0_0` pass
  - `milestone_has_type_system_2` pass
  - `readme_mentions_type_system_2` pass
- `cargo test` 全体で 3612 tests pass（3608 + 4）
- `fav/Cargo.toml` のバージョンが `72.0.0`
- `CHANGELOG.md` に `[v72.0.0]` エントリが存在
- `MILESTONE.md` に「Type System 2.0」エントリが存在
- `README.md` に「v72.0」または「Type System 2.0」の記述が存在

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v72000_tests` モジュール追加（4 テスト）+ cargo_toml_version 更新 |
| `fav/Cargo.toml` | バージョン `71.9.0` → `72.0.0` |
| `CHANGELOG.md` | `## [v72.0.0]` エントリ追加 |
| `MILESTONE.md` | Type System 2.0 マイルストーン追記 |
| `README.md` | v72.0 達成追記 |
| `versions/current.md` | 進行中: v72.0.0 / 次: v72.1.0 |

---

## スコープ外

- `site/` MDX 更新: 別タスク
- v72.1.0 以降の機能実装: 次スプリント
