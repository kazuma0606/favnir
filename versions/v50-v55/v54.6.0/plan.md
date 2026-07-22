# Plan: v54.6.0 — README / CONTRIBUTING 最終整備

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3195 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54600_tests が未存在を確認
rg -n "v54600_tests" fav/src/driver.rs  # → 0 件

# v54500_tests の行番号を確認（挿入位置）
rg -n "v54500_tests" fav/src/driver.rs

# README.md に "Production 3.0" が未存在を確認
grep "Production 3.0" README.md  # → 0 件

# CONTRIBUTING.md に "fav doctor" が未存在を確認
grep "fav doctor" CONTRIBUTING.md  # → 0 件

# Cargo.toml が 54.5.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.5.0"
```

---

## ステップ 2: `README.md` — Production 3.0 言及追加

v54.0 Integration Sprint マイルストーン宣言文の直後（v53.0 より前）に追記:

```markdown
v54.1〜v54.5（2026-07-22〜23）で Production 3.0 に向けた最終整備を完了しました。
全エラーコードへの `fav explain --error` 対応（v54.1）・`fav run --watch-diff/--watch-summary`（v54.2）・
パフォーマンスリグレッション CI 統合（v54.3）・`fav dq-report`（v54.4）・`fav doctor`（v54.5）が揃い、
開発者が自信を持って本番へ踏み出せるツールチェーンが完成しました。
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `CONTRIBUTING.md` — 手順追記

テスト手順セクションの**直前**に「環境診断」セクションを追加（詳細は spec.md §2 参照）。
テスト手順セクションの**直後**に「ベンチマーク・パフォーマンス確認」セクションを追加。

注意: `fav bench` コマンド例には `--all` を含めない（実装上 no-op であるため）。
注意: `cargo build` 確認はドキュメントのみ変更のため不要。`driver.rs` に `include_str!` を追加する次ステップ後に実施する。

---

## ステップ 4: `driver.rs` — `v54600_tests` 追加

`v54500_tests` の直前に追加:

```rust
// -- v54600_tests (v54.6.0) -- README / CONTRIBUTING 最終整備 --
#[cfg(test)]
mod v54600_tests {
    use super::*;

    #[test]
    fn readme_has_production3_mention() {
        let readme = include_str!("../../README.md");
        assert!(readme.contains("Production 3.0"), "...");
        assert!(readme.contains("v54.1"), "...");
    }

    #[test]
    fn contributing_has_doctor_step() {
        let contributing = include_str!("../../CONTRIBUTING.md");
        assert!(contributing.contains("fav doctor"), "...");
    }
}
```

`cargo build` → コンパイルエラーなし確認（`include_str!` パス検証）。

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`version = "54.5.0"` → `version = "54.6.0"`

---

## ステップ 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3197 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 7: 後処理

- `CHANGELOG.md`: v54.6.0 エントリ追加（v54.5.0 の直上）
- `versions/current.md` を v54.6.0（3197 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.6.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T7 全 `[x]`）

コードレビュー対応（実施済み）:
- [MED] `v54600_tests` に `use super::*` 欠落 → 追加（慣習統一）
- [MED] `fav bench --all` が no-op でドキュメントが誤解を招く → `--all` を削除
- [MED] README.md の v54.1〜v54.5 ブロックが v54.0 宣言より上に挿入 → v54.0 直後に移動
- [LOW] `readme_has_production3_mention` が v54.6.0 追加行を特定しない → `"v54.1"` アサーション追加
