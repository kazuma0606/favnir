# v74.3.0 実装計画 — Documentation Site 2.0

Date: 2026-08-13

---

## 実装ステップ

### Step 1: MDX ファイル 3 件を作成

`site/content/docs/v2/` ディレクトリ以下に以下を作成する。

**`getting-started.mdx`**（最小限の内容でテストが通る構成）:
```mdx
---
title: Getting Started
---

# Getting Started

Favnir を 5 分で試す。

## インストール

```bash
cargo install fav
```

## Hello World パイプライン

```favnir
stage Hello {
  bind msg <- "Hello, Favnir!"
  ctx.io.println(msg)
}
```

`fav run hello.fav` で実行する。
```

**`migration-v35-v75.mdx`**（v35 / v75 を含む移行ガイド）:
```mdx
---
title: Migration Guide v35 to v75
---

# Migration Guide — v35 → v75

v35 から v75 への移行手順。

## 主な変更点

- `!Effect` 構文を廃止（v35.4.0 でハードエラー化）
- `ctx.field.method()` 構文に移行（v13.6.0〜）
- `bind` 再束縛禁止（E0018、v12.1.0〜）
```

**`language-reference.mdx`**（構文一覧）:
```mdx
---
title: Language Reference
---

# Language Reference

Favnir の全構文一覧。

## bind

```favnir
bind x <- expr
```

## stage

```favnir
stage Name { ... }
```
```

### Step 2: `v743000_tests` モジュールを `driver.rs` に追加

`v742000_tests` の直後に追加する。

```rust
// --- v74.3.0: Documentation Site 2.0 ---

#[cfg(test)]
mod v743000_tests {
    #[test]
    fn docs_site2_getting_started_exists() {
        let src = include_str!("../../site/content/docs/v2/getting-started.mdx");
        assert!(src.contains("Getting Started"), "getting-started title missing");
        assert!(src.contains("Favnir"), "Favnir mention missing");
    }

    #[test]
    fn docs_site2_migration_guide_v35_to_v75() {
        let src = include_str!("../../site/content/docs/v2/migration-v35-v75.mdx");
        assert!(src.contains("Migration"), "migration guide title missing");
        assert!(src.contains("v35"), "v35 reference missing");
        assert!(src.contains("v75"), "v75 reference missing");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.2.0"` → `version = "74.3.0"`
- `driver.rs` 内の `version = "74.2.0"` 参照を `version = "74.3.0"` に replace_all（コメント・セクションヘッダー内の `74.2.0` は置換不要）
- `version should be 74.2.0` を `version should be 74.3.0` に replace_all（アサートメッセージのみ対象）
- `cargo build` を実行すると `Cargo.lock` が自動的に `version = "74.3.0"` に更新される

### Step 4: テスト確認

- `cargo test v743000` で 2 件 pass を確認
- `cargo test` 全体で 3675 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.3.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-13 (v74.3.0)`
- 進行中: `v74.3.0`
- 次: `v74.4.0`
