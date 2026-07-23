# Plan: v46.9.0 — Developer Experience ドキュメント + v47.0 前調整

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/tools/fav-test.mdx` | 新規作成 |
| `site/content/docs/tools/developer-experience.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v469000_tests` モジュール追加 |
| `fav/Cargo.toml` | version → `"46.9.0"` |
| `CHANGELOG.md` | v46.9.0 エントリ追加 |
| `versions/current.md` | v46.9.0（3012 tests）に更新 |
| `versions/v45-v50/v46.9.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### `site/content/docs/tools/fav-test.mdx`

```mdx
---
title: fav test — Inline Testing
description: fav test コマンドと #[test] アノテーションによるインラインテストの使い方
---

# fav test — Inline Testing

Favnir では `#[test]` アノテーションを使ってソースファイル内にテストを記述できます。

## 基本構文

```favnir
stage Double: Int -> Int = |x| { x * 2 }

#[test]
fn test_double() {
  assert_eq(Double(3), 6)
  assert_eq(Double(0), 0)
}
```

## コマンド

```bash
fav test main.fav                     # すべての #[test] fn を実行
fav test main.fav --filter test_add   # 名前でフィルタ
```

## アサーション関数

| 関数 | 説明 |
|---|---|
| `assert_eq(actual, expected)` | 値が等しいことを確認 |
| `assert_ne(a, b)` | 値が異なることを確認 |
| `assert_ok(result)` | `Ok(...)` であることを確認 |
| `assert_err(result)` | `Err(...)` であることを確認 |

## 出力フォーマット

```
PASS test_double (2ms)
PASS test_validate_order (5ms)
FAIL test_edge_case
  assert_eq failed: expected 0, got -1

2 passed, 1 failed
```
```

### `site/content/docs/tools/developer-experience.mdx`

```mdx
---
title: Developer Experience — v46.x
description: fav test / LSP クイックフィックス / fav explain 2.0 による開発体験の改善
---

# Developer Experience — v46.x

v46.x シリーズでは Favnir の開発体験を大幅に改善する機能を追加しました。

## fav test — インラインテスト

`#[test]` アノテーションでソースファイル内にテストを記述できます。
詳細は [fav test](/docs/tools/fav-test) を参照してください。

## LSP クイックフィックス

エディタ上でエラーに対して自動修正アクションを提供します。

- **E0102（未定義変数）**: did-you-mean `quickFix` — スペルミスに最も近い変数名を提案
- **E0101（引数数不一致）**: 引数追加提案アクション

VSCode で `Ctrl+.` (Mac: `Cmd+.`) を押してクイックフィックスを適用できます。

## fav explain 2.0

### fav explain --types

パイプライン各ステージの宣言型を一覧表示します。

```bash
fav explain --types main.fav
```

出力例:
```
stage ParseCsv: String -> List<Row>
stage FilterRows: List<Row> -> List<Row>
stage SaveToDb: List<Row> -> Result<Int>
```

### fav explain --lineage --show-dead

リネージグラフで早期脱出パス（`return` 文を持つステージ）を dead path として表示します。

```bash
fav explain --lineage --show-dead --format mermaid main.fav
```

### fav explain --format mermaid（dead path）

パイプライン図で早期脱出パスを点線（`-.->`)、エラーパスを赤線で表示します。

```bash
fav explain --format mermaid main.fav
```

## v47.0 Developer Experience 宣言に向けて

v46.1〜v46.9 の全機能が揃い次第、v47.0 で「Developer Experience」マイルストーンを宣言します。
```

---

### `fav/src/driver.rs` — `v469000_tests`

```rust
// -- v469000_tests (v46.9.0) -- DX ドキュメント存在確認 --
#[cfg(test)]
mod v469000_tests {
    #[test]
    fn fav_test_doc_exists() {
        let content = include_str!("../../site/content/docs/tools/fav-test.mdx");
        assert!(content.contains("#[test]"), "fav-test.mdx should mention #[test]");
        assert!(content.contains("assert_eq"), "fav-test.mdx should mention assert_eq");
        assert!(content.contains("fav test"), "fav-test.mdx should mention fav test command");
    }

    #[test]
    fn developer_experience_doc_exists() {
        let content = include_str!("../../site/content/docs/tools/developer-experience.mdx");
        assert!(content.contains("fav explain --types"), "doc should mention fav explain --types");
        assert!(content.contains("--show-dead"), "doc should mention --show-dead");
        assert!(content.contains("quickFix"), "doc should mention quickFix");
    }
}
```

---

## 実装順序

1. `site/content/docs/tools/fav-test.mdx` 新規作成
2. `site/content/docs/tools/developer-experience.mdx` 新規作成
3. `driver.rs`: `v469000_tests` を追加（`v468000_tests` の直前）
4. `cargo test` で 3012 passed 確認
5. `cargo clippy -- -D warnings` クリーン確認
6. `Cargo.toml` version → `"46.9.0"`
7. `CHANGELOG.md` エントリ追加
8. `versions/current.md` 更新
9. `tasks.md` COMPLETE に更新

---

## 注意事項

- MDX ファイルは `include_str!("../../site/content/docs/tools/fav-test.mdx")` でアクセスする
  （`driver.rs` は `fav/src/` にあり、`../../` で `fav/` の親ディレクトリ = `favnir/` に到達）
- `site/` は `fav/` と同じ親ディレクトリ下（`favnir/site/`）にある
- コードブロック内のバッククォートはエスケープ不要（MDX のコードフェンス内）
- `versions/current.md` 更新時はサブスプリントの参照先も
  `roadmap-v45.1-v46.0.md` → `roadmap-v46.1-v47.0.md` に変更すること
- `developer-experience.mdx` 作成時はコードフェンスのネストに注意
  （MDX 内のコードブロックと plan.md のコードブロックを混同しないこと）
