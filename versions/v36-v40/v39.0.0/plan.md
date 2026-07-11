# v39.0.0 実装計画 — Intelligence & Assistance マイルストーン宣言

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `CHANGELOG.md` | 追記 | `[v39.0.0]` エントリ追加 |
| `fav/tmp/hello.fav` | 確認/復元 | `cargo clean` 後の消失チェック・必要なら復元 |
| `MILESTONE.md` | 追記 | v39.0.0 Intelligence & Assistance 宣言セクションを先頭に挿入 |
| `README.md` | 追記 | v39.0 マイルストーン宣言行を v38.0 行の直後に追加 |
| `fav/src/driver.rs` | 変更 | `v38900_tests::cargo_toml_version_is_38_9_0` スタブ化 |
| `fav/src/driver.rs` | 変更 | `v39000_tests` モジュール（4 テスト）追加 |
| `fav/Cargo.toml` | 更新 | `version = "38.9.0"` → `"39.0.0"` |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v39.0.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v39.0.0、次バージョン v39.1.0 |
| `versions/v36-v40/v39.0.0/tasks.md` | 更新 | COMPLETE ステータスに更新 |

## 実装順序

### Step 1: CHANGELOG.md に [v39.0.0] エントリ追加

`## [v38.9.0]` の直前に挿入:

```markdown
## [v39.0.0] — 2026-07-10

### Added
- Intelligence & Assistance マイルストーン宣言
- `MILESTONE.md` に v39.0.0 宣言セクション追加
- `README.md` に v39.0 マイルストーン宣言追加
- `v39000_tests` 4 テスト追加

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

### Step 3: MILESTONE.md — v39.0.0 セクション追加

`# Favnir Milestones` ヘッダの直後（`## v38.0.0` の直前）に v39.0.0 セクションを挿入する。
spec.md の「MILESTONE.md への追加内容」に従い、宣言文・達成コンポーネント表・宣言日・`---` セパレータを含む。

挿入後の先頭順序: `v39.0.0` → `v38.0.0` → `v37.0.0` → ...

### Step 4: README.md — v39.0 マイルストーン宣言行追加

`**v38.0（2026-07-10）で、[Multi-Source ETL Power](./MILESTONE.md) マイルストーンを宣言しました。**` の直後に追加:

```markdown
**v39.0（2026-07-10）で、[Intelligence & Assistance](./MILESTONE.md) マイルストーンを宣言しました。**
```

### Step 5: driver.rs — `v38900_tests::cargo_toml_version_is_38_9_0` をスタブ化

```rust
// Stubbed: version bumped to 39.0.0 — assertion intentionally removed
```

**注意**: `changelog_has_v38_9_0` / `ai_overview_doc_exists` / `suggest_rs_has_llm_suggest` はスタブ化しない。

### Step 6: driver.rs — `v39000_tests` モジュール追加（T3・T4 完了後に実施）

`v38900_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。
spec.md §v39000_tests の設計 に従い 4 テストを追加:

```rust
// ── v39000_tests (v39.0.0) — Intelligence & Assistance マイルストーン宣言 ──────
#[cfg(test)]
mod v39000_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.0.0"), "Cargo.toml must contain version 39.0.0");
    }

    #[test]
    fn changelog_has_v39_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.0.0]"), "CHANGELOG.md must contain [v39.0.0]");
    }

    #[test]
    fn milestone_has_intelligence_and_assistance() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Intelligence & Assistance"),
            "MILESTONE.md must contain Intelligence & Assistance"
        );
    }

    #[test]
    fn readme_mentions_intelligence_assistance() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Intelligence & Assistance"),
            "README.md must contain Intelligence & Assistance"
        );
    }
}
```

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.9.0` → `39.0.0` に更新。

### Step 8: cargo test

`cargo test` を実行し 2785 passed / 0 failed を確認。

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v39.0.0 を ✅ にマーク
- `versions/current.md` を v39.0.0（最新安定版）・v39.1.0（次バージョン）に更新
- `versions/v36-v40/v39.0.0/tasks.md` を COMPLETE に更新

## 依存関係

- `cargo clean` は Step 1 完了後すぐ実施可能
- Step 2 手順 5 の `cargo build`（クリーン後ビルド確認）は `v39000_tests` 追加前の暫定確認
- Step 6（v39000_tests 追加）は Step 3（MILESTONE.md）・Step 4（README.md）完了後に着手（`include_str!` の assert がコンパイル時に評価されるため）
- Step 7（Cargo.toml）は Step 1〜6 すべて完了後

## リスク

| リスク | 対処 |
|---|---|
| `cargo clean` で `fav/tmp/hello.fav` が消失 | Step 2 の手順 1・3 で前後チェック。消失時は正確な内容で復元 |
| MILESTONE.md 挿入位置のミス | `# Favnir Milestones` の直後・`## v38.0.0` の直前であることを Edit 前後で確認 |
| README.md 挿入位置のミス | `v38.0（2026-07-10）` 行を `old_string` に含めて一意に特定 |
| v39000_tests の `include_str!` パス | `../../MILESTONE.md` / `../../README.md` は `fav/src/` から 2 階層上 = `favnir/` ルート。既存パターンと同一 |
