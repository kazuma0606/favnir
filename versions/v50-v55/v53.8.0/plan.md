# Plan: v53.8.0 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3177 passed, 0 failed を確認

# v53800_tests が未存在を確認
rg -n "v53800_tests" fav/src/driver.rs  # → 0 件

# v53700_tests の行番号を確認（挿入位置）
rg -n "v53700_tests" fav/src/driver.rs  # → 行番号を特定

# MILESTONE.md に Integration Sprint が未存在を確認
grep "Integration Sprint" MILESTONE.md  # → 0 件

# Cargo.toml が 53.7.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.7.0"
```

---

## ステップ 2: `MILESTONE.md` に Integration Sprint サマリー追加

ファイル先頭の `# Favnir Milestones` の直後（`## v53.0.0` の直前）に追加:

```md
## v51.0〜v53.0 Integration Sprint サマリー（2026-07-22）

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。」

DX 3.0（v51）・Performance & Scale（v52）・Data Quality & Observability 2.0（v53）の 3 マイルストーンを
**Integration Sprint** として統合。v53.1〜v53.8 で lineage × LSP・par bench・assert_schema 詳細診断・
E2E デモ・cookbook・用語集・CHANGELOG/MILESTONE 整理を完了し、v54.0「Integration Sprint 宣言」への準備を整えた。

---
```

内容確認:
```bash
grep "Integration Sprint" MILESTONE.md  # → 1 件以上
```

---

## ステップ 3: `CHANGELOG.md` — v53.8.0 エントリ追加

v53.7.0 エントリの直上（ファイル先頭側）に追加:

```md
## [v53.8.0] — 2026-07-22 — CHANGELOG / MILESTONE 整理（v51〜v53 まとめ）

### Added
- `MILESTONE.md`: v51.0〜v53.0 Integration Sprint サマリーセクション追加 — DX 3.0 / Performance & Scale / Data Quality 2.0 の統合達成を記録
- `CHANGELOG.md`: v53.8.0 エントリに Integration Sprint サマリー参照を含む（`changelog_has_v51_to_v53_summary` テスト対象）
- `v53800_tests` 追加（`changelog_has_v51_to_v53_summary` / `milestone_integration_sprint_noted`）— 3179 tests

---
```

内容確認:
```bash
grep "Integration Sprint" CHANGELOG.md  # → 1 件以上
```

---

## ステップ 4: `driver.rs` — `v53800_tests` 追加

`v53700_tests` モジュールの直前に `v53800_tests` を追加:

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

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`version = "53.7.0"` → `version = "53.8.0"`

---

## ステップ 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3179 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 7: 後処理

- `CHANGELOG.md` に v53.8.0 エントリ追加済みであることを確認（ステップ 3 で実施）
- `versions/current.md` を v53.8.0（3179 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.8.0 実績欄を COMPLETE に更新（推定値 3173 → 実績 3179 の差異を注記）
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
