# Spec: v48.9.0 — Module ドキュメント + migration guide + v49.0 前調整

## 概要

v48.1〜v48.8 で実装した Module & Package 2.0 の機能（新 import 構文・`fav.toml [runes]` 解決・循環検出・rune コマンド群）を
ドキュメントとして整理し、ユーザーが旧構文から新構文へ移行できるよう guide を提供する。

- `site/content/docs/module-system.mdx` — Favnir モジュールシステム解説ページ
- `site/content/docs/migration-guide-import.mdx` — 旧 `import rune "X"` → 新 `import X` 移行ガイド

コード変更は MDX 作成のみ。`driver.rs` に存在確認テスト 2 件を追加。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/module-system.mdx` | 新規作成（モジュールシステム解説）|
| `site/content/docs/migration-guide-import.mdx` | 新規作成（import 移行ガイド）|
| `fav/src/driver.rs` | `v489000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.9.0"` |
| `CHANGELOG.md` | v48.9.0 エントリ追加 |

---

## MDX 内容

### `module-system.mdx`

以下のセクションで構成する:

1. **概要** — パッケージ import とローカル import の区別
2. **パッケージ import** — `import kafka` / `import kafka as k` 構文
3. **ローカル import** — `import "./src/helpers" as helpers` 構文
4. **`fav.toml [runes]` 宣言** — 依存関係の唯一の真実
5. **循環 import の検出** — E0418 エラー説明
6. **旧構文（非推奨）** — W035 警告・移行案内

フロントマター:
```yaml
---
title: "Module System"
order: 1
category: "Language"
description: "Favnir のモジュールシステム — パッケージ import とローカル import の使い方"
---
```

### `migration-guide-import.mdx`

以下のセクションで構成する:

1. **背景** — なぜ構文を変えたか（明確な区別・`fav.toml` 一元管理）
2. **移行手順** — 3 ステップ（`fav.toml [runes]` 宣言 → import 書き換え → `fav install`）
3. **変換対応表** — 旧構文 vs 新構文
4. **よくある質問** — W035 警告が出たら・`fav.toml` がない場合

フロントマター:
```yaml
---
title: "Import Migration Guide"
order: 2
category: "Guides"
description: "旧 import rune \"X\" 構文から新しいパッケージ import 構文への移行ガイド"
---
```

---

## テスト（+2）

```rust
#[test]
fn module_system_doc_exists() {
    let content = include_str!("../../site/content/docs/module-system.mdx");
    assert!(content.contains("Module System"), "module-system.mdx should contain 'Module System'");
    assert!(content.contains("E0418") || content.contains("import"),
        "module-system.mdx should mention E0418 or import");
}

#[test]
fn import_migration_guide_exists() {
    let content = include_str!("../../site/content/docs/migration-guide-import.mdx");
    assert!(content.contains("import rune") || content.contains("migration") || content.contains("W035"),
        "migration-guide-import.mdx should mention import rune, migration, or W035");
}
```

テスト数: 3063 → **3065**（+2）

---

## 注意事項

- MDX ファイルのパス: `site/content/docs/` 直下（サブディレクトリなし）
- `include_str!` のパスは `fav/src/driver.rs` からの相対パス: `../../site/content/docs/<file>.mdx`
- コードフリーズ: v49.0 に向けて本バージョン以降はコード追加なし（宣言・クリーンアップのみ）
- `cargo clean` は v49.0.0 で実施（本バージョンはスコープ外）

---

## 完了条件

- `cargo test` 3065 passed, 0 failed（3063 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.9.0"`
- `CHANGELOG.md` に v48.9.0 エントリ追加
- `versions/current.md` を v48.9.0（3065 tests）に更新、進行中バージョンを `v49.0.0` に更新
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.9.0 実績を記入
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
