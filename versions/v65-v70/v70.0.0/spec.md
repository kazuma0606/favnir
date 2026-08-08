# v70.0.0 仕様書

## 概要

**バージョン**: v70.0.0
**テーマ**: Intelligent ETL 1.0 宣言 ★クリーンアップ
**ベーステスト数**: 3555（v69.9.0 完了後）
**目標テスト数**: 3559（+4）

---

## Background

v65.1.0〜v69.9.0 のスプリントを経て、Favnir は「AI データエンジニアリングのための型安全言語」として成熟した。
v70.0.0 はその集大成として「Intelligent ETL 1.0」を宣言するマイルストーンバージョンである。

**宣言文**:

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

---

## Goals

1. `fav/Cargo.toml` version を `"70.0.0"` に更新
2. `MILESTONE.md` 先頭に v70.0.0「Intelligent ETL 1.0」エントリを追加
3. `README.md` に v70.0.0 宣言を追記
4. `CHANGELOG.md` 先頭に v70.0.0 エントリを追加
5. `driver.rs` に `v70000_tests`（4 件）を追加（`v69900_tests` の直前）
6. `cargo clean` ★クリーンアップ実施後、`fav/tmp/hello.fav` を復元
7. `cargo test` で 3559 tests passed を確認

---

## 追加テスト（driver.rs）

モジュール名: `v70000_tests`（`v69900_tests` の直前に挿入、降順ルール）

```rust
fn cargo_toml_version_is_70_0_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("version = \"70.0.0\""), ...);
}

fn changelog_has_v70_0_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("v70.0.0"), ...);
}

fn milestone_has_intelligent_etl() {
    let src = include_str!("../../MILESTONE.md");
    assert!(src.contains("Intelligent ETL"), ...);
}

fn readme_mentions_intelligent_etl() {
    let src = include_str!("../../README.md");
    assert!(
        src.contains("Intelligent ETL") || src.contains("v70.0"),
        ...
    );
}
```

---

## MILESTONE.md エントリ（先頭に追加）

```markdown
## v70.0.0（2026-08-08）— Intelligent ETL 1.0

> 「型チェックが、LLM の出力を安全にする。...（宣言文全文）」

**Intelligent ETL 1.0** の宣言バージョン。v65.1〜v69.9 で実装した
Math Rune 群・AI Rune 群・Playground 拡張・E2E AI ETL デモ・
パフォーマンスベースラインの統合を宣言した。
```

---

## CHANGELOG.md エントリ（先頭に追加）

```markdown
## [v70.0.0] — 2026-08-08 — Intelligent ETL 1.0 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v70.0.0「Intelligent ETL 1.0」宣言文エントリを追加
- `v70000_tests`: 4 件追加（3555 → 3559 tests）
  - `cargo_toml_version_is_70_0_0`
  - `changelog_has_v70_0_0`
  - `milestone_has_intelligent_etl`
  - `readme_mentions_intelligent_etl`
- Intelligent ETL 機能群（v65.1〜v69.9）の成果を統合

### Changed
- `fav/Cargo.toml` version `"69.0.0"` → `"70.0.0"`
- `README.md` に Intelligent ETL 1.0 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `cargo clean` 後は `fav/tmp/hello.fav` を復元すること（bootstrap テスト要件）
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"69.0.0"` → `"70.0.0"` |
| `MILESTONE.md` | 先頭に v70.0.0 エントリ追加 |
| `README.md` | v70.0.0 宣言文追記 |
| `CHANGELOG.md` | 先頭に v70.0.0 エントリ追加 |
| `fav/src/driver.rs` | `v70000_tests` モジュール追加（v69900_tests の直前） |
| `versions/current.md` | 最新安定版を v70.0.0 に更新 |
| `versions/roadmap/roadmap-v69.1-v70.0.md` | v70.0.0 行を「完了 ✓」に更新 |

---

## Success Criteria

1. `cargo test --bin fav -- --test-threads=8` で **3559 tests passed, 0 failed**
2. `fav/Cargo.toml` の version が `"70.0.0"`
3. `MILESTONE.md` 先頭に「Intelligent ETL」を含むエントリ
4. `CHANGELOG.md` 先頭に `[v70.0.0]` エントリ
5. `README.md` に `"Intelligent ETL"` または `"v70.0"` を含む記述
6. `cargo clean` 後 `fav/tmp/hello.fav` が正しく復元されている

---

## ★クリーンアップ注意事項

`cargo clean` 後は `fav/tmp/hello.fav` が削除される。
bootstrap テスト（`bootstrap_c2_artifact_roundtrip`）が依存するため、必ず以下の内容で復元する:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```
