# v25.0.0 実装計画 — Practical Self-Hosting マイルストーン宣言

## 前提確認

- `fav/Cargo.toml` の version = `"24.8.0"` であること
- `cargo test --bin fav` が 1969 件 PASS であること
- `grep -n "mod v248000_tests" fav/src/driver.rs` が存在すること
- `grep -n "mod v250000_tests" fav/src/driver.rs` が未存在であること
- `STABILITY.md` が既存（v24.4.0 で作成済み）であること

> **注**: ロードマップ §v25.0「最終テスト」2〜5 番（`fav run --vm=self/vm.fav ...` 系・4-stage bootstrap Stage 4）は、
> vm.fav Phase 6（`CallFn` オペコード / ユーザー定義関数ディスパッチ）が未実装のため **v25.x に延期**（spec.md §スコープ外参照）。
> v25.0.0 では項目 1（`cargo test` 全件 PASS）のみ達成する。

---

## Step 0: 事前確認コマンド

```bash
grep -n "version = " fav/Cargo.toml
cargo test --bin fav 2>&1 | grep "test result: ok"
grep -n "mod v248000_tests" fav/src/driver.rs | head -3
grep -n "mod v250000_tests" fav/src/driver.rs | head -3
grep -n "v1.x" STABILITY.md | head -3
```

---

## Step 1: `MILESTONE.md` 作成（リポジトリルート）

**ファイル**: `C:\Users\yoshi\favnir\MILESTONE.md`

必須キーワード:
- `"Practical Self-Hosting"` — テスト `milestone_md_has_selfhost_declaration` で検証
- `"compiler.fav"` — テスト `milestone_md_has_selfhost_declaration` で検証

内容構成:
- タイトル: `# Practical Self-Hosting Milestone`
- v25.0.0 = v1.0 リリース候補宣言
- 達成済みコンポーネント表（compiler.fav / checker.fav / cli.fav / vm.fav）
- 各コンポーネントの達成バージョン（v8.5.0〜v24.0.0）
- VM エンジン（実行基盤）は Rust で永続維持する旨の説明
- 最終テスト手順（ロードマップ §v25.0「最終テスト」5 項目のうち項目 1 のみ達成済み。項目 2〜5 は vm.fav Phase 6 未実装のため v25.x に延期）

---

## Step 2: `README.md` 更新

**ファイル**: `C:\Users\yoshi\favnir\README.md`

必須キーワード（いずれか）:
- `"v25.0"` または `"v1.0"` — テスト `readme_mentions_v1_release` で検証

変更内容:
- バッジセクションまたはバージョン表記に `v25.0` / `v1.0` を追加
- マイルストーン達成セクションを追記
- インストール手順が v25.0.0 を参照するよう更新

---

## Step 3: `site/content/docs/v1-release.mdx` 作成

**ファイル**: `C:\Users\yoshi\favnir\site\content\docs\v1-release.mdx`

必須キーワード:
- `"v25.0"` — テスト `site_v1_release_page_exists` で検証

内容構成:
- タイトル: v1.0 リリースノート
- v25.0 マイルストーン達成の宣言
- v24.1〜v24.8 の各バージョンで達成した機能一覧
- v1.x 後方互換性保証（STABILITY.md への参照リンク）

---

## Step 4: `versions/roadmap-v20.1-v25.0.md` 更新

**ファイル**: `C:\Users\yoshi\favnir\versions\roadmap-v20.1-v25.0.md`

注: `versions/roadmap-master.md` は v17〜v20 用のため対象外。v24.x / v25.x の記述は `roadmap-v20.1-v25.0.md` にある。

変更内容:
- v24.1〜v24.8 を「完了」に更新
- v25.0.0 を「宣言済み」に更新

---

## Step 5: `fav/src/driver.rs` — v250000_tests 追加

**モジュール名**: `v250000_tests`（`v248000_tests` の直後に追加）

**テスト件数**: 5 件（削除なし）

### include_str! パスの注意事項

`driver.rs` は `fav/src/driver.rs` にあるため、リポジトリルートへのパスは `../../`:

| ファイル | include_str! パス |
|---|---|
| `MILESTONE.md` | `"../../MILESTONE.md"` |
| `README.md` | `"../../README.md"` |
| `STABILITY.md` | `"../../STABILITY.md"` |
| `site/content/docs/v1-release.mdx` | `"../../site/content/docs/v1-release.mdx"` |
| `CHANGELOG.md` | `"../../CHANGELOG.md"` |

### テスト実装

```rust
#[cfg(test)]
mod v250000_tests {
    #[test]
    fn milestone_md_has_selfhost_declaration() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Practical Self-Hosting"), ...);
        assert!(content.contains("compiler.fav"), ...);
    }

    #[test]
    fn readme_mentions_v1_release() {
        let content = include_str!("../../README.md");
        assert!(content.contains("v25.0"), ...);
    }

    #[test]
    fn stability_md_exists() {
        let content = include_str!("../../STABILITY.md");
        assert!(content.contains("v1.x"), ...);
    }

    #[test]
    fn site_v1_release_page_exists() {
        let content = include_str!("../../site/content/docs/v1-release.mdx");
        assert!(content.contains("v25.0"), ...);
    }

    #[test]
    fn changelog_has_v25_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v25.0.0]"), ...);
    }
}
```

---

## Step 6: Cargo.toml バージョン更新

```
version = "24.8.0" → "25.0.0"
```

---

## Step 7: CHANGELOG.md + benchmarks

- `CHANGELOG.md` 先頭に `[v25.0.0]` エントリを追加
- `benchmarks/v25.0.0.json` を新規作成（test_count: 1974、duration_ms: 17600）

---

## テスト件数計画

| 操作 | 件数 |
|---|---|
| 前バージョン合計 | 1969 |
| v248000_tests から削除（version_is なし） | 0 |
| v250000_tests 追加 | +5 |
| **合計** | **1974** |
