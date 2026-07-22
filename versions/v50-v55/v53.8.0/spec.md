# Spec: v53.8.0 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

Status: COMPLETE
Date: 2026-07-22

---

## 概要

v51〜v53 の 3 スプリント（DX 3.0・Performance & Scale・Data Quality 2.0）の達成を
`MILESTONE.md` に Integration Sprint サマリーとして記録し、
`CHANGELOG.md` の v53.8.0 エントリに v51〜v53 Integration Sprint への参照を含める。

Rust テストで MILESTONE.md・CHANGELOG.md の内容を検証し、記録漏れを防ぐ。

---

## 実装スコープ

### 1. `MILESTONE.md` — Integration Sprint サマリー追加

ファイル先頭（既存の v53.0.0 エントリの直前）に追加:

```md
## v51.0〜v53.0 Integration Sprint サマリー（2026-07-22）

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。」

DX 3.0（v51）・Performance & Scale（v52）・Data Quality & Observability 2.0（v53）の 3 マイルストーンを
**Integration Sprint** として統合。v53.1〜v53.8 で lineage × LSP・par bench・assert_schema 詳細診断・
E2E デモ・cookbook・用語集・CHANGELOG/MILESTONE 整理を完了し、v54.0「Integration Sprint 宣言」への準備を整えた。
```

必須要件:
- `Integration Sprint` という文字列を含む
- v51・v52・v53 各マイルストーンへの言及を含む

---

### 2. `CHANGELOG.md` — v53.8.0 エントリ追加

```md
## [v53.8.0] — 2026-07-22 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

### Added
- `MILESTONE.md`: v51.0〜v53.0 Integration Sprint サマリーセクション追加
- `CHANGELOG.md`: v53.8.0 エントリに Integration Sprint サマリー参照を含む
- `v53800_tests` 追加 — 3179 tests
```

必須要件:
- `v51` への参照を含む
- `v53` への参照を含む
- `Integration Sprint` または `統合スプリント` を含む

---

### 3. テスト仕様

`v53800_tests` モジュールを `driver.rs` に追加（`v53700_tests` の直前）:

```rust
// -- v53800_tests (v53.8.0) -- CHANGELOG / MILESTONE 整理 --
#[cfg(test)]
mod v53800_tests {
    #[test]
    fn changelog_has_v51_to_v53_summary() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v51"), "CHANGELOG must reference v51 releases");
        assert!(content.contains("v53"), "CHANGELOG must reference v53 releases");
        assert!(
            content.contains("Integration Sprint") || content.contains("統合スプリント"),
            "CHANGELOG must contain Integration Sprint summary"
        );
    }

    #[test]
    fn milestone_integration_sprint_noted() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Integration Sprint"),
            "MILESTONE.md must note the Integration Sprint (v51~v53)"
        );
    }
}
```

パス:
- `include_str!("../../CHANGELOG.md")`: `fav/src/driver.rs` → `../../` = `favnir/` ✓
- `include_str!("../../MILESTONE.md")`: 同上 ✓

---

## バージョン更新

- `fav/Cargo.toml`: `"53.7.0"` → `"53.8.0"`

---

## 完了条件

- `cargo test` 3179 passed, 0 failed（ベース 3177 + 2 件追加）
- `v53800_tests` 2 件 pass:
  - `changelog_has_v51_to_v53_summary`
  - `milestone_integration_sprint_noted`
- `cargo clippy -- -D warnings` クリーン
- `MILESTONE.md` に `Integration Sprint` が含まれる
- `CHANGELOG.md` に `v51` / `v53` / `Integration Sprint` が含まれる

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | Integration Sprint サマリー追加 |
| `CHANGELOG.md` | v53.8.0 エントリ追加 |
| `fav/src/driver.rs` | `v53800_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `versions/current.md` | v53.8.0 / 3179 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.8.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `changelog_has_v51_to_v53_summary` は `"v51"` / `"v53"` を個別チェックするが、
  これらは既存の CHANGELOG エントリにも含まれるため弱いアサーションである。
  第3アサート `content.contains("Integration Sprint")` が実質的な検証となる。
  将来的には `"Integration Sprint"` 単独チェックで十分だが、後方互換のため現状維持。
- `v53800_tests` は `use super::*` 不要（`include_str!` マクロのみ使用）。
- MILESTONE.md のサマリー範囲は "v53.1〜v53.8"（v53.8.0 自身の整理作業を含む）。
  "v53.1〜v53.7" と書くと v53.8.0 が漏れるため注意。
