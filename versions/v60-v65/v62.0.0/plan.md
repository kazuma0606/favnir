# v62.0.0 実装計画 — Language Polish 宣言 ★クリーンアップ

## フェーズ構成

| フェーズ | 内容 | 対象ファイル |
|---|---|---|
| P1 | `Cargo.toml` バージョン更新 | `fav/Cargo.toml` |
| P2 | `MILESTONE.md` 宣言エントリ追加 | `MILESTONE.md` |
| P3 | `README.md` 言及追加 | `README.md` |
| P4 | `CHANGELOG.md` エントリ追加 | `CHANGELOG.md` |
| P5 | `v62000_tests` 追加（4 件）| `fav/src/driver.rs` |
| P6 | ビルド・テスト全通過確認 | — |
| P7 | ★クリーンアップ（`cargo clean`）+ 再ビルド確認 | — |
| P8 | ドキュメント更新（roadmap / current.md）| `versions/` |

---

## P1: Cargo.toml バージョン更新

```toml
# before
version = "61.0.0"
# after
version = "62.0.0"
```

`fav/Cargo.toml` の L3 を編集。

---

## P2: MILESTONE.md 宣言エントリ追加

既存の v32.0.0 Language Polish エントリとは別に、**v62.0.0 Language Polish** として v61 スプリント成果をまとめた新エントリを追記する。

追記内容:
- 宣言文（spec.md から転記）
- v61.1〜v61.9 の機能一覧
- テスト数: 3382 tests

---

## P3: README.md 言及追加

バージョン履歴テーブル（`v60〜v65` 行付近）に v62.0 Language Polish を追記する。
既存の v32.0 Language Polish 言及（L88）とは独立したエントリ。

---

## P4: CHANGELOG.md エントリ追加

```markdown
## [v62.0.0] — 2026-08-01 — Language Polish 宣言 ★クリーンアップ
```

v61.1〜v61.9 の全機能を箇条書きで集約する。

---

## P5: v62000_tests 追加

`v61900_tests` モジュールの**直後（ファイル末尾）**に `v62000_tests` モジュールを追加。

```rust
// -- v62000_tests (v62.0.0) -- Language Polish 宣言 --
#[cfg(test)]
mod v62000_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_62_0_0() {
        let cargo = include_str!("../../Cargo.toml");
        assert!(cargo.contains("version = \"62.0.0\""), ...);
    }

    #[test]
    fn changelog_has_v62_0_0() {
        let cl = include_str!("../../../CHANGELOG.md");
        assert!(cl.contains("v62.0.0"), ...);
    }

    #[test]
    fn milestone_has_language_polish() {
        let ms = include_str!("../../../MILESTONE.md");
        // v32.0.0 の既存記述と区別するため v62.0.0 との組み合わせを確認
        assert!(ms.contains("v62.0.0") && ms.contains("Language Polish"), ...);
    }

    #[test]
    fn readme_mentions_language_polish() {
        let readme = include_str!("../../../README.md");
        // v32.0 の既存記述と区別するため v62.0 との組み合わせを確認
        assert!(readme.contains("v62.0") && readme.contains("Language Polish"), ...);
    }
}
```

各テストは `include_str!` で対象ファイルを読み込み、期待文字列が含まれることを `assert!` で確認する。

---

## P6: ビルド・テスト

```bash
cargo test v62000   # 4 件 PASS 確認
cargo test -j 8 -- --test-threads=8  # 3382 passed, 0 failed 確認
```

---

## P7: ★クリーンアップ

```bash
cargo clean
cargo build  # クリーン後ビルド成功確認
cargo test -j 8 -- --test-threads=8  # クリーン後フルテスト（3382 passed 確認）
```

クリーン後に `fav/tmp/hello.fav` が消える可能性があるため、消えた場合は復元する（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）。

---

## P8: ドキュメント更新

- `versions/roadmap/roadmap-v61.1-v62.0.md` — v62.0 セクションに実績を追記
- `versions/current.md` — 進行中を v62.0.0 完了に、次を v62.1.0 or 次ロードマップへ
- `versions/v60-v65/v62.0.0/tasks.md` — COMPLETE に更新

---

## リスク・注意事項

- `cargo clean` 後の `fav/tmp/hello.fav` 消失 → 復元が必要
- `include_str!` パスは `fav/src/driver.rs` からの相対パス（`../../` で `favnir/` ルートへ）
- `MILESTONE.md` L742 と `README.md` L88 に v32.0.0 時点の既存 "Language Polish" 記述があり、単独チェックでは T2/T3 の追記なしにテストが通過してしまう。このため assert 条件を `"v62.0.0"` / `"v62.0"` との AND 条件に強化（P5 参照）。T2/T3 の追記は必須。
