# v38.0.0 実装計画 — Multi-Source ETL Power マイルストーン宣言

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `CHANGELOG.md` | 追記 | `[v38.0.0]` エントリ追加 |
| `fav/tmp/hello.fav` | 確認/復元 | `cargo clean` 後の消失チェック・必要なら復元 |
| `MILESTONE.md` | 追記 | v38.0.0 Multi-Source ETL Power 宣言セクションを先頭に挿入 |
| `README.md` | 追記 | v38.0 マイルストーン宣言行を v37.0 行の直後に追加 |
| `fav/src/driver.rs` | 変更 | `v37900_tests::cargo_toml_version_is_37_9_0` スタブ化 |
| `fav/src/driver.rs` | 変更 | `v38000_tests` モジュール（4 テスト）追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.9.0"` → `"38.0.0"` |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v38.0.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.0.0、次バージョン v38.1.0 |
| `versions/v36-v40/v38.0.0/tasks.md` | 更新 | COMPLETE ステータスに更新 |

## 実装順序

### Step 1: CHANGELOG.md に [v38.0.0] エントリ追加

`## [v37.9.0]` の `---` セパレータ直後に挿入:

```markdown
## [v38.0.0] — 2026-07-09

### Added
- Multi-Source ETL Power マイルストーン宣言
- `MILESTONE.md` に v38.0.0 宣言セクション追加
- `README.md` に v38.0 マイルストーン宣言追加
- `v38000_tests` 4 テスト追加

---
```

### Step 2: ★クリーンアップ — cargo clean

1. `fav/tmp/hello.fav` の存在と内容を確認（Read で確認）
2. `cargo clean` を実行
3. `fav/tmp/hello.fav` が消失していないか確認（存在チェック）
4. 消失していた場合は以下の内容で復元:
   ```
   fn add(a: Int, b: Int) -> Int { a + b }
   fn main() -> Bool { add(1, 2) == 3 }
   ```
5. `cargo build` でコンパイルエラーがないことを確認

**注意**: Step 2 は Step 1 完了後すぐ実施可能（他のステップに依存しない）。

### Step 3: MILESTONE.md — v38.0.0 セクション追加

`# Favnir Milestones` ヘッダの直後（`## v37.0.0` の直前）に v38.0.0 セクションを挿入する。
spec.md の「MILESTONE.md への追加内容」に従い、宣言文・達成コンポーネント表・宣言日・`---` セパレータを含む。

挿入後の先頭順序: `v38.0.0` → `v37.0.0` → `v36.0.0` → ...

### Step 4: README.md — v38.0 マイルストーン宣言行追加

`**v37.0（2026-07-09）で、[Data Quality First](./MILESTONE.md) マイルストーンを宣言しました。**` の直後に追加:

```markdown
**v38.0（2026-07-09）で、[Multi-Source ETL Power](./MILESTONE.md) マイルストーンを宣言しました。**
```

### Step 5: driver.rs — `v37900_tests::cargo_toml_version_is_37_9_0` スタブ化

```rust
// Stubbed: version bumped to 38.0.0 — assertion intentionally removed
```

**注意**: `changelog_has_v37_9_0` / `lineage_text_has_summary_line` / `multi_source_etl_doc_exists` はスタブ化しない。

### Step 6: driver.rs — `v38000_tests` モジュール追加

`v37900_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。
spec.md §v38000_tests の設計 に従い 4 テストを追加:

```rust
// ── v38000_tests (v38.0.0) — Multi-Source ETL Power マイルストーン宣言 ─────────
#[cfg(test)]
mod v38000_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_38_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.0.0"), "Cargo.toml must contain version 38.0.0");
    }

    #[test]
    fn changelog_has_v38_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.0.0]"), "CHANGELOG.md must contain [v38.0.0]");
    }

    #[test]
    fn milestone_has_multi_source_etl_power() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Multi-Source ETL Power"),
            "MILESTONE.md must contain Multi-Source ETL Power"
        );
    }

    #[test]
    fn readme_mentions_multi_source_etl() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Multi-Source ETL"),
            "README.md must contain Multi-Source ETL"
        );
    }
}
```

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `37.9.0` → `38.0.0` に更新。

### Step 8: cargo test

`cargo test` を実行し 2741 passed / 0 failed を確認。

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v37.1-v38.0.md` の v38.0.0 を ✅ にマーク
- `versions/current.md` を v38.0.0（最新安定版）・v38.1.0（次バージョン）に更新
- `versions/v36-v40/v38.0.0/tasks.md` を COMPLETE に更新

## 依存関係

- `cargo clean` は Step 1 完了後すぐ実施可能
- Step 2 手順 5 の `cargo build`（クリーン後ビルド確認）は `v38000_tests` 追加前の暫定確認であり、コンパイルが通ることを確認するだけ
- Step 6（v38000_tests 追加）完了後は再度 `cargo build` を実行してコンパイルエラーがないことを確認してから Step 7（Cargo.toml 更新）に進む
- Step 6（v38000_tests）は Step 3（MILESTONE.md）・Step 4（README.md）完了後に着手（`include_str!` の assert がコンパイル時に評価されるため）
- Step 7（Cargo.toml）は Step 1〜6 すべて完了後

## リスク

| リスク | 対処 |
|---|---|
| `cargo clean` で `fav/tmp/hello.fav` が消失 | Step 2 の手順 1・3 で前後チェック。消失時は正確な内容で復元 |
| MILESTONE.md 挿入位置のミス | `# Favnir Milestones` の直後・`## v37.0.0` の直前であることを Edit 前後で確認 |
| README.md 挿入位置のミス | `v37.0（2026-07-09）` 行を `old_string` に含めて一意に特定 |
| v38000_tests の `include_str!` パス | `../../MILESTONE.md` / `../../README.md` は `fav/src/` から 2 階層上 = `favnir/` ルート。既存パターンと同一 |
