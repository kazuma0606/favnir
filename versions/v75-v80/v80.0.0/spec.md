# v80.0.0 仕様書 — Favnir 3.0 宣言 ★クリーンアップ

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.9.0 で安定化・コードフリーズが完了した。
v80.0.0 は v75.1〜v79.9 の全スプリントを締めくくる Favnir 3.0 宣言バージョンである。

**宣言文**:
> 「時間が型となり、来歴が型となり、正しさが型となり、実行戦略が型となった。
>
>  FreshnessPolicy がデータの鮮度を保証し、ProvenanceTag が来歴を追い、
>  PipelineInvariant が不変条件を証明し、!Adaptive がコストを最適化する。
>
>  Favnir 3.0 は、データパイプラインが「何を・どこから・どう正しく・どう速く」
>  処理するかを、すべて型で語れる言語である。」

新機能は追加しない。クリーンアップ作業のみ。

---

## Goals

- `cargo clean` でビルドキャッシュを完全クリア
- Cargo.toml バージョンを `80.0.0` に更新
- CHANGELOG.md に v80.0.0 エントリ追加
- MILESTONE.md に「Favnir 3.0 宣言」追記
- README.md に v80.0 達成（Favnir 3.0）追記
- `versions/current.md` を v80.0.0 完了状態に更新
- マスターロードマップを「完了」に更新
- 4 件の宣言テスト追加（3805 → 3809）

---

## テストモジュール仕様

```rust
// --- v80.0.0: Favnir 3.0 宣言 ★クリーンアップ ---
#[cfg(test)]
mod v80000_tests {
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const CHANGELOG:  &str = include_str!("../../CHANGELOG.md");
    const MILESTONE:  &str = include_str!("../../MILESTONE.md");
    const README:     &str = include_str!("../../README.md");

    #[test]
    fn cargo_toml_version_is_80_0_0() {
        assert!(CARGO_TOML.contains("version = \"80.0.0\""), "Cargo.toml must be bumped to 80.0.0");
    }

    #[test]
    fn changelog_has_v80_0_0() {
        assert!(CHANGELOG.contains("[v80.0.0]"), "CHANGELOG.md must have v80.0.0 entry");
    }

    #[test]
    fn milestone_has_favnir_3() {
        assert!(MILESTONE.contains("Favnir 3.0"), "MILESTONE.md must document Favnir 3.0 declaration");
    }

    #[test]
    fn readme_mentions_favnir_3() {
        assert!(README.contains("Favnir 3.0"), "README.md must mention Favnir 3.0");
    }
}
```

注意:
- `use super::*` 不要（`include_str!` + `assert!` のみ）
- `const CARGO_TOML` は `../../Cargo.toml`（`fav/Cargo.toml` から見て 2 段上）
- 宣言バージョン（x.0.0）は 4 テスト追加が慣例

---

## CHANGELOG エントリ形式

```
## [v80.0.0] — 2026-08-16 — Favnir 3.0 宣言 ★クリーンアップ

### Declaration
- Favnir 3.0 宣言: 時間・来歴・正しさ・実行戦略がすべて型で語れる言語へ
- v75.1〜v79.9 の全スプリント（Temporal / Provenance / Verifiable / Execution Effects）完了

### Cleanup
- `cargo clean` 実施（ビルドキャッシュ完全クリア）
- MILESTONE.md に Favnir 3.0 宣言を追記
- README.md に v80.0 達成を追記

### Tests
- `cargo_toml_version_is_80_0_0`: バージョン 80.0.0 を確認
- `changelog_has_v80_0_0`: CHANGELOG エントリを確認
- `milestone_has_favnir_3`: MILESTONE.md の Favnir 3.0 宣言を確認
- `readme_mentions_favnir_3`: README.md の Favnir 3.0 記述を確認
```

---

## Success Criteria

- `cargo test v80000` で 4 件が pass
- `cargo test` で 3809 tests pass（0 failures）
- `cargo_toml_version_is_80_0_0`: Cargo.toml が `version = "80.0.0"` を含む
- `changelog_has_v80_0_0`: CHANGELOG.md が `[v80.0.0]` を含む
- `milestone_has_favnir_3`: MILESTONE.md が `Favnir 3.0` を含む
- `readme_mentions_favnir_3`: README.md が `Favnir 3.0` を含む

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v80000_tests` モジュール追加（末尾） |
| `fav/Cargo.toml` | `version = "80.0.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v80.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | Favnir 3.0 宣言を追記 |
| `README.md` | v80.0 達成（Favnir 3.0）を追記 |
| `versions/current.md` | v80.0.0 完了・次フェーズ計画に更新 |
| `versions/roadmap/roadmap-v79.1-v80.0.md` | v80.0.0 スプリントを「完了」に更新 |

> **Note**: 新機能ファイルの追加はない（宣言・クリーンアップバージョンのため）。
> `cargo clean` は実装開始前（T1）に必ず実施すること。
