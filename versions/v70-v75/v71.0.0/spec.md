# v71.0.0 Spec — Language Complete 1.0 宣言 ★クリーンアップ

Date: 2026-08-09
Status: 計画中

---

## Background

v70.1〜v70.9 で以下を達成した:
- v70.1: compiler.fav 2 段メソッドチェーン対応
- v70.2: `fav migrate`（IO.* → ctx.io.* 変換）
- v70.3: `fav bench --all`
- v70.4: ErrorReport / 診断 UI 強化
- v70.5: パターンマッチ強化
- v70.6: `bind` 分割束縛
- v70.7: Self-Hosting Coverage Report
- v70.8: `fav doctor` 強化
- v70.9: 安定化・bench.yml strict mode 化

v71.0.0 は **Language Complete 1.0 宣言**バージョン。
compiler.fav が全構文を処理し、CI が積み残しなくグリーンで動作し、
エラーメッセージが修正方法を即座に示し、fav migrate が旧コードを変換する——
その姿を宣言する。

★クリーンアップ: `cargo clean` を実施し、ビルド生成物を削除する。

---

## 宣言文

> 「compiler.fav が Favnir の全構文を処理し、
>  積み残しのない CI が毎回グリーンで終わる。
>  エラーメッセージは修正方法を即座に示し、
>  fav migrate が旧コードを自動で現代に変換する。
>
>  これが Favnir v71.0 — Language Complete 1.0 の姿である。」

---

## Goals

1. **`cargo_toml_version_is_71_0_0`** — Cargo.toml が 71.0.0 であることを確認
2. **`changelog_has_v71_0_0`** — CHANGELOG.md に v71.0.0 エントリが存在することを確認
3. **`milestone_has_language_complete`** — MILESTONE.md に "Language Complete" という文字列が存在することを確認
4. **`readme_mentions_language_complete`** — README.md に "Language Complete" への言及があることを確認
5. テスト 4 件追加（3580 → 3584）

---

## テスト実装

```rust
#[cfg(test)]
mod v71000_tests {
    #[test]
    fn cargo_toml_version_is_71_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"71.0.0\""), "Cargo.toml should declare version 71.0.0");
    }

    #[test]
    fn changelog_has_v71_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v71.0.0]"), "CHANGELOG.md should have v71.0.0 entry");
    }

    #[test]
    fn milestone_has_language_complete() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Language Complete"), "MILESTONE.md should mention Language Complete");
    }

    #[test]
    fn readme_mentions_language_complete() {
        let src = include_str!("../../README.md");
        assert!(src.contains("Language Complete"), "README.md should mention Language Complete");
    }
}
```

---

## MILESTONE.md 追加内容

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
```

---

## README.md 追加内容

v70.0 宣言セクションの直後に v71.0 セクションを追加:

```markdown
## v71.0 — Language Complete 1.0 宣言（2026-08-09）

Favnir v71.0 で「Language Complete 1.0」を宣言しました。
compiler.fav が全構文を処理し、積み残しのない CI が毎回グリーンで終わります。
エラーメッセージは修正方法を即座に示し、fav migrate が旧コードを自動変換します。
```

---

## Success Criteria

- [ ] `cargo_toml_version_is_71_0_0` PASS
- [ ] `changelog_has_v71_0_0` PASS
- [ ] `milestone_has_language_complete` PASS
- [ ] `readme_mentions_language_complete` PASS
- [ ] `cargo test v71000` で 4 件 pass
- [ ] `cargo test` 全体で 3584 tests pass（0 failures）
- [ ] MILESTONE.md に "Language Complete" が含まれること
- [ ] README.md に "Language Complete" が含まれること

---

## Error Codes

新規エラーコードなし

---

## Notes

- ★クリーンアップ: `cargo clean` を実施（ビルド生成物を削除）
- site/ MDX 更新は v71.1.0 以降で実施

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v71000_tests` モジュール追加 + version 文字列更新（`"70.9.0"` → `"71.0.0"`） |
| `fav/Cargo.toml` | `version` を `"70.9.0"` → `"71.0.0"` |
| `CHANGELOG.md` | v71.0.0 エントリ追加 |
| `MILESTONE.md` | v71.0.0 Language Complete 1.0 エントリ追加 |
| `README.md` | v71.0 セクション追加 |
| `versions/current.md` | 進行中バージョンを v71.0.0 に更新 |
