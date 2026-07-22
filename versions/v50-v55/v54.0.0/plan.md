# Plan: v54.0.0 — Integration Sprint 宣言

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3181 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54000_tests が未存在を確認
rg -n "v54000_tests" fav/src/driver.rs  # → 0 件

# v53900_tests の行番号を確認（挿入位置）
rg -n "v53900_tests" fav/src/driver.rs  # → 行番号を特定

# hello.fav が正しい内容であることを確認
cat fav/tmp/hello.fav
# → fn add(a: Int, b: Int) -> Int { a + b }
# → fn main() -> Bool { add(1, 2) == 3 }

# Cargo.toml が 53.9.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.9.0"
```

---

## ステップ 2: `MILESTONE.md` — v54.0.0 宣言セクション追加

ファイル先頭（`# Favnir Milestones` の直後、「v51.0〜v53.0 Integration Sprint サマリー」の直前）に追加:

```md
## v54.0.0（2026-07-22）— Integration Sprint

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。
>
>  これが Favnir v54.0 — Integration の姿である。」

**Integration Sprint** の宣言バージョン。v53.1〜v53.8 の統合作業（lineage × LSP・par bench・assert_schema 詳細診断・
E2E デモ・cookbook・用語集・CHANGELOG/MILESTONE 整理・integration-overview ドキュメント）および
v53.9 のコードフリーズを経て、v51.0〜v53.0 の 3 マイルストーンを一体として機能させた。

---
```

内容確認:
```bash
grep "Integration Sprint" MILESTONE.md  # → 複数件
```

---

## ステップ 3: `README.md` — v54.0 宣言追記

`**v53.0（2026-07-22）で...` の行の直上（ファイル先頭側）に追加:

```md
**v54.0（2026-07-22）で、[Integration Sprint](./MILESTONE.md) マイルストーンを宣言しました。**
エディタはデータの来歴を示し、並列パイプラインの性能は計測可能で、スキーマ違反は即座に修正できる。Favnir の 3 つの柱（DX 3.0 / Performance & Scale / Data Quality 2.0）が一体となった — これが **Integration Sprint** の宣言です。

```

内容確認:
```bash
grep "Integration Sprint" README.md  # → 1 件以上
```

---

## ステップ 4: `CHANGELOG.md` — v54.0.0 エントリ追加

v53.9.0 エントリの直上（ファイル先頭側）に追加:

```md
## [v54.0.0] — 2026-07-22 — Integration Sprint 宣言

### Added
- `MILESTONE.md`: v54.0.0 Integration Sprint 宣言セクション追加（宣言文・v53.1〜v53.9 完了サマリー）
- `README.md`: v54.0 Integration Sprint マイルストーン宣言を追加
- `v54000_tests` 追加（`cargo_toml_version_is_54_0_0` / `changelog_has_v54_0_0` / `milestone_has_integration_sprint` / `readme_mentions_integration_sprint`）— 3185 tests

---
```

---

## ステップ 5: `driver.rs` — `v54000_tests` 追加 + `v53900_tests` 空化

### 5a: `v53900_tests` の直前に `v54000_tests` を追加

```rust
// -- v54000_tests (v54.0.0) -- Integration Sprint 宣言 --
#[cfg(test)]
mod v54000_tests {
    #[test]
    fn cargo_toml_version_is_54_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"54.0.0\""), "Cargo.toml must have version 54.0.0");
    }

    #[test]
    fn changelog_has_v54_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v54.0.0"), "CHANGELOG.md must contain v54.0.0 entry");
    }

    #[test]
    fn milestone_has_integration_sprint() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Integration Sprint"),
            "MILESTONE.md must contain Integration Sprint declaration"
        );
    }

    #[test]
    fn readme_mentions_integration_sprint() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("Integration Sprint"),
            "README.md must mention Integration Sprint"
        );
    }
}
```

### 5b: `cargo_toml_version_is_53_9_0` を空化

```rust
fn cargo_toml_version_is_53_9_0() {
    // v54.0.0 にバンプしたためアサートを空化。
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 6: `fav/Cargo.toml` バージョン更新

`version = "53.9.0"` → `version = "54.0.0"`

---

## ステップ 7: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3185 passed, 0 failed（テスト数 ≥ 3179 ✓）

```bash
cargo clippy -- -D warnings
```

---

## ステップ 8: ★クリーンアップ（`cargo clean`）

```bash
cargo clean
```

clean 後に `fav/tmp/hello.fav` が残っていることを確認してから、テストを再実行:

```bash
cat fav/tmp/hello.fav  # 存在確認

cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3185 passed, 0 failed を再確認
```

---

## ステップ 9: 後処理

- `CHANGELOG.md`: v54.0.0 エントリ追加済みであることを確認（ステップ 4 で実施）
- `versions/current.md` を v54.0.0（3185 tests）に更新
- `roadmap-v53.1-v54.0.md` の v54.0.0 実績欄を COMPLETE に更新（`cargo clean` 後テスト通過を明記）
- コードレビュー [LOW] 2 件対応:
  - MILESTONE.md v54.0 説明文を "v53.1〜v53.8 統合作業 + v53.9 コードフリーズ" に分離修正（"v53.1〜v53.9 の統合作業" の不正確な表現を解消）
  - Integration Sprint サマリーの範囲表記を "v53.1〜v53.8 統合作業・v53.9 コードフリーズ完了・v54.0 宣言達成" に更新
- `tasks.md` を COMPLETE に更新（T0〜T6 全 `[x]`）
