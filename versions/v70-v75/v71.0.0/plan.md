# v71.0.0 Plan — Language Complete 1.0 宣言

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 0: cargo clean ★クリーンアップ

```bash
cd fav && cargo clean
```

ビルド生成物（target/）を削除する。

---

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "70.9.0"` → `"71.0.0"`

driver.rs 内の全バージョン文字列も一括更新:
```bash
# driver.rs 内の "70.9.0" を "71.0.0" に置換（replace_all）
```

---

### Step 2: MILESTONE.md 更新

`MILESTONE.md` の先頭（v70.0.0 エントリの直前）に v71.0.0 エントリを追加:

```markdown
## v71.0.0（2026-08-09）— Language Complete 1.0

> 「compiler.fav が Favnir の全構文を処理し、
>  積み残しのない CI が毎回グリーンで終わる。
>  エラーメッセージは修正方法を即座に示し、
>  fav migrate が旧コードを自動で現代に変換する。
>
>  これが Favnir v71.0 — Language Complete 1.0 の姿である。」

**Language Complete 1.0** の宣言バージョン。v70.1〜v70.9 で実装した
compiler.fav 完全化・診断 UI 強化・fav migrate・bench.yml strict mode の統合を宣言した。

**v70.1〜v70.9 達成内容:**
- compiler.fav: 2 段メソッドチェーン / bind 分割束縛 / if-guard パターン対応
- `fav migrate`: !Effect → ctx.io.* 自動変換
- `fav bench --all`: JSON 形式ベンチマーク出力
- ErrorReport / `suggest_similar_name` 診断 UI
- `fav self-coverage`: self-hosting 網羅率レポート
- `fav doctor`: Paper Rune 検出 / CHANGELOG 整合性チェック
- bench.yml strict mode 化（Compare ステップ無条件実行）

---
```

---

### Step 3: README.md 更新

`README.md` の v70.0 セクションの直後に v71.0 セクションを追加:

```markdown
## v71.0 — Language Complete 1.0 宣言（2026-08-09）

Favnir v71.0 で「Language Complete 1.0」を宣言しました。
compiler.fav が全構文を処理し、積み残しのない CI が毎回グリーンで終わります。
エラーメッセージは修正方法を即座に示し、fav migrate が旧コードを自動変換します。
```

---

### Step 4: driver.rs に `v71000_tests` を追加

`v709000_tests` の直後（driver.rs 末尾）に追加:

```rust
// ── v71.0.0: Language Complete 1.0 宣言 ──────────────────────────────────────

#[cfg(test)]
mod v71000_tests {
    #[test]
    fn cargo_toml_version_is_71_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("version = \"71.0.0\""),
            "Cargo.toml should declare version 71.0.0"
        );
    }

    #[test]
    fn changelog_has_v71_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v71.0.0]"),
            "CHANGELOG.md should have v71.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_language_complete() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Language Complete"),
            "MILESTONE.md should mention Language Complete"
        );
    }

    #[test]
    fn readme_mentions_language_complete() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Language Complete"),
            "README.md should mention Language Complete"
        );
    }
}
```

Note: この時点では `changelog_has_v71_0_0` はまだ FAIL（CHANGELOG 未更新のため）。
Step 5（CHANGELOG 更新）完了後に `cargo test v71000` で 4 件 pass を確認すること。

---

### Step 5: CHANGELOG.md 更新

v71.0.0 エントリを先頭（v70.9.0 エントリの直前）に追加。
ヘッダー形式: `## [v71.0.0] — 2026-08-09 — Language Complete 1.0 宣言`
（テスト `changelog_has_v71_0_0` が `[v71.0.0]` という文字列を検索するため、角括弧を必ず含めること）

---

### Step 6: versions/current.md 更新

進行中バージョンを v71.0.0 に更新。

---

### Step 7: 最終確認

- `cargo test v71000` で 4 件 pass
- `cargo test` 全体で 3584 tests pass（0 failures）
- MILESTONE.md / README.md に "Language Complete" が含まれること
