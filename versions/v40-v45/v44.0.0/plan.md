# v44.0.0 Plan — Language Expressiveness 宣言 ★クリーンアップ

## 前提

- 現行バージョン: `43.13.0`（2937 tests）
- 追加テスト数: 4 件
- 目標テスト数: 2941
- スタブ化対象: なし（`v431300_tests` に `cargo_toml` テストなし）

---

## ステップ

### Step 1: MILESTONE.md 更新

`# Favnir Milestones` タイトル行の直後、`## v43.0.0 — Real-Time Power` セクションの直前に以下を挿入:

```markdown
## v44.0.0 — Language Expressiveness（2026-07-13）

> 「戻り値型は省略でき、ジェネリクスは呼び出し側から推論される。
>  ラムダ引数はパイプライン上流の型から確定し、
>  `opaque type` で型の境界を守れる。
>
>  これが Favnir v44.0 — Language Expressiveness の姿である。」

v44.0.0 をもって、Favnir の **Language Expressiveness** を正式に宣言する。

### 達成コンポーネント（v43.1〜v43.13）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 戻り値型推論 | v43.1 | Return type omission |
| fav check 統合・E0410/E0411 | v43.2 | 推論失敗エラー |
| ジェネリック型引数推論 | v43.3 | Call-site generic inference |
| E0412 曖昧型変数検出 | v43.4 | Ambiguous type variable |
| ラムダ引数型推論 | v43.5 | Contextual lambda inference |
| パイプライン型伝播 | v43.6 | Pipeline stage typing |
| 構造体リテラル推論 | v43.7 | Structural inference |
| 双方向型推論 | v43.8 | Bidirectional / top-down |
| fav check --show-inference | v43.9 | 推論型の注釈表示 |
| fav check --explain 統合 | v43.10 | 静的解説テキスト |
| opaque type 完全化 | v43.11 | opaque keyword + E0413 |
| W031/W032 lint | v43.12 | 冗長型注釈の警告 |
| Language Expressiveness cookbook | v43.13 | ドキュメント安定化 |

**宣言日**: 2026-07-13

---
```

### Step 2: README.md 更新

README.md の line 114（`CEP...Real-Time Power 基盤が完成しました。`）の直後、空行を挟んで以下を挿入:

```markdown
**v44.0（2026-07-13）で、[Language Expressiveness](./MILESTONE.md) マイルストーンを宣言しました。**
型推論 6 カテゴリ（戻り値型・ジェネリクス・ラムダ・パイプライン・構造体・双方向）/ opaque type / W031/W032 lint が揃い、型注釈を最小化しながら型安全性を維持できる Language Expressiveness 基盤が完成しました。
```

`"Language Expressiveness"` および `"v44.0"` の両方が含まれるよう挿入する。

### Step 3: driver.rs に `v44000_tests` 追加 / Cargo.toml バンプ

`v431300_tests` の直前に挿入:

```rust
// -- v44000_tests (v44.0.0) -- Language Expressiveness 宣言 --
#[cfg(test)]
mod v44000_tests {
    #[test]
    fn cargo_toml_version_is_44_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.0.0\""), "Cargo.toml must contain version 44.0.0");
    }
    #[test]
    fn changelog_has_v44_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v44.0.0]"), "CHANGELOG.md must contain [v44.0.0]");
    }
    #[test]
    fn milestone_has_language_expressiveness() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Language Expressiveness"),
            "MILESTONE.md must contain 'Language Expressiveness'"
        );
    }
    #[test]
    fn readme_mentions_language_expressiveness() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Language Expressiveness") || src.contains("v44.0"),
            "README.md must mention Language Expressiveness or v44.0"
        );
    }
}
```

スタブ化: `v431300_tests` に `cargo_toml` テストがないため不要。

`fav/Cargo.toml` version: `43.13.0` → `44.0.0`

### Step 4: CHANGELOG.md に v44.0.0 エントリ追加

### Step 5: テスト実行（2941 passed; 0 failed）

### Step 6: ★クリーンアップ（`cargo clean`）

`cargo clean` はビルドアーティファクトを削除するのみでソースに影響なし。テスト実行後に実施する。

### Step 7: バージョン管理ドキュメント更新

---

## 注意事項

- `v431300_tests` にスタブ化対象の `cargo_toml_version_is_43_13_0` が存在しないため、スタブ化ステップは不要
- `cargo clean` はテスト完了後に実施（テスト再実行なし）
- `MILESTONE.md` への挿入: `# Favnir Milestones` の直後、`## v43.0.0` の直前
- README.md への挿入: line 114（v43.0 記述末尾）の直後に空行 + 2 行追加
- `readme_mentions_language_expressiveness` テストは OR 条件だが、実装では両方（`"Language Expressiveness"` と `"v44.0"`）を含む文字列を挿入するため実質 AND を保証
