# Plan: v52.9.0 — 安定化・コードフリーズ（Data Quality 2.0 前調整）

---

## ステップ 1: clippy クリーン確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1
```

0 エラー・0 警告であればそのまま次へ。
clippy 指摘がある場合は最小限の修正（追加機能なし）を行う。

---

## ステップ 2: `site/content/docs/data-quality-overview.mdx` 作成

既存の `site/content/docs/data-quality.mdx`（v36.x 向け）とは別ファイル。

内容構成:
- title: "Data Quality & Observability 2.0 — 概要"
- 概要セクション（v52.x 機能群の位置づけ）
- 機能一覧セクション（assert_schema / lineage --with-schema / --format html / SLA / audit-log / OTel 強化）
- 各機能へのリンク
- 「Data Quality」「Observability」「assert_schema」「audit-log」キーワードを含む（テスト要件）

---

## ステップ 3: `fav/src/driver.rs` — `v52900_tests` 追加

`v52800_tests` モジュールの直前に `v52900_tests` を追加:

```rust
#[cfg(test)]
mod v52900_tests {
    #[test]
    fn cargo_toml_version_is_52_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"52.9.0\""), "Cargo.toml must be version 52.9.0");
    }

    #[test]
    fn dq_overview_doc_exists() {
        let src = include_str!("../../site/content/docs/data-quality-overview.mdx");
        assert!(
            src.contains("Data Quality") && src.contains("Observability"),
            "data-quality-overview.mdx must mention Data Quality and Observability"
        );
        assert!(
            src.contains("assert_schema"),
            "data-quality-overview.mdx must mention assert_schema"
        );
        assert!(
            src.contains("audit-log") || src.contains("audit_log"),
            "data-quality-overview.mdx must mention audit-log"
        );
    }
}
```

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "52.8.0"` → `version = "52.9.0"`

**注意**: `cargo_toml_version_is_52_9_0` テストが Cargo.toml 更新後に pass することを確認する。
ステップ 3（driver.rs 更新）→ ステップ 4（Cargo.toml 更新）の順序を守ること。

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3156 passed, 0 failed

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v52.9.0 エントリ追加
- `versions/current.md` を v52.9.0（3156 tests）に更新
- `roadmap-v52.1-v53.0.md` の v52.9.0 実績欄を更新
  - v53.0.0 の推定値を 3158（v52.9.0 実績 + 4）ではなく ≥ **3157**（ロードマップ記載条件）に注意
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）
