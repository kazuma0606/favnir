# v40.0.0 実装計画 — Enterprise Governance マイルストーン宣言

## 概要

v39.9.0（2810 tests）から v40.0.0（2814 tests）へ。
x.0.0 マイルストーンのため ★クリーンアップ（`cargo clean`）必須。

## ステップ

### Step 1: CHANGELOG.md に [v40.0.0] エントリを追加

`## [v39.9.0]` ヘッダ行の直前に挿入。

```markdown
## [v40.0.0] — 2026-07-11

### Added
- Enterprise Governance マイルストーン宣言（v39.1〜v39.9 達成コンポーネント統合）
- `MILESTONE.md` に v40.0.0 Enterprise Governance セクション追加
- `README.md` に v40.0 マイルストーン宣言行追加
- `v40000_tests` 4 テスト追加（version / changelog / milestone / readme）

---
```

### Step 2: MILESTONE.md に v40.0.0 セクションを追加

挿入位置: `# Favnir Milestones` ヘッダの直後、`## v39.0.0` セクションの直前。
内容は spec.md §MILESTONE.md への追加内容 を参照。

### Step 3: README.md に v40.0 マイルストーン宣言行を追加

挿入位置: `**v39.0（2026-07-10）で、[Intelligence & Assistance]...` 行の直後。

```markdown
**v40.0（2026-07-11）で、[Enterprise Governance](./MILESTONE.md) マイルストーンを宣言しました。**
```

### Step 4: driver.rs — v39900_tests::cargo_toml_version_is_39_9_0 をスタブ化

NOTE コメント + ライブアサーション全体を:
```rust
// Stubbed: version bumped to 40.0.0 — assertion intentionally removed
```
に置き換える。

### Step 5: driver.rs — v40000_tests モジュールを追加（Step 1 完了後）

`v39900_tests` の閉じ `}` の後に追加。spec.md §v40000_tests の設計 のコードブロックをそのまま使用。

### Step 6: Cargo.toml バージョンを 40.0.0 に更新

`version = "39.9.0"` → `version = "40.0.0"`

### Step 7: ★クリーンアップ + cargo test

1. `fav/tmp/hello.fav` 存在確認
2. `cargo clean`
3. `hello.fav` 存在確認（消失していれば復元）
4. `cargo test`（≥ 2814 passed, 0 failed）

### Step 8: ドキュメント更新

- `versions/v36-v40/v40.0.0/tasks.md` を COMPLETE に更新
- `versions/current.md` を v40.0.0（最新安定版）に更新
- `versions/roadmap/roadmap-v39.1-v40.0.md` の v40.0.0 を ✅ に更新

## 依存関係

```
Step 1 (CHANGELOG)
     ↓
Step 5 (v40000_tests) ← Step 4 (スタブ化) ← Step 1 完了確認
     ↓
Step 6 (Cargo.toml)
     ↓
Step 7 (cargo clean + cargo test)
     ↓
Step 8 (ドキュメント更新)
```

Step 2（MILESTONE.md）と Step 3（README.md）は Step 1 と並行して実施可能。

## テスト数予測

| バージョン | tests |
|---|---|
| v39.9.0 | 2810 |
| v39900_tests スタブ化 | ±0 |
| v40000_tests 追加 | +4 |
| **v40.0.0 期待値** | **2814** |
