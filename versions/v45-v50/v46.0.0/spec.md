# Spec: v46.0.0 — Language Refinement 宣言

Date: 2026-07-16
Status: TODO

---

## 概要

v45.1〜v45.9 で実装した全機能（`return`・`match` 完全網羅・型エイリアス・エラーメッセージ改善・数値リテラル・examples 更新）を統合確認し、Language Refinement マイルストーンを宣言する。

宣言文:

> 「`return` によるガード節・`match` 完全網羅・型エイリアスの明確な境界・
>  改善されたエラーメッセージが揃い、Favnir の構文が成熟した。
>
>  これが Favnir v46.0 — Language Refinement の姿である。」

---

## 調査結果（実装前に確認済み）

### 現状

- `cargo test` 2988 passed（v45.9.0 完了時点）
- `MILESTONE.md`: v45.0.0「Precision & Flow」エントリは存在する。v46.0.0「Language Refinement」エントリは**未追記**
- `README.md`: 「Language Refinement」の文字列は**未記載**
- `v46000_tests` モジュール: **未追加**

---

## 変更対象

### §1 — `MILESTONE.md` 更新

v46.0.0「Language Refinement」エントリを追加する。`"Language Refinement"` という文字列を必ず含めること（`milestone_has_language_refinement` テストが依存）。

```markdown
## v46.0.0 — Language Refinement（2026-07-16）

> 「`return` によるガード節・`match` 完全網羅・型エイリアスの明確な境界・
>  改善されたエラーメッセージが揃い、Favnir の構文が成熟した。
>
>  これが Favnir v46.0 — Language Refinement の姿である。」

v46.0.0 をもって、Favnir の **Language Refinement** を正式に宣言する。

### 達成コンポーネント（v45.1〜v45.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `return` 構文 AST + parser | v45.1 | ReturnStmt ノード・parser 解析 |
| `return` 型チェック + E0415 | v45.2 | 戻り型不一致エラー |
| `return` compiler + VM | v45.3 | Return opcode・早期脱出実行 |
| `match` 網羅性 + W034/E0416 | v45.4 | 非網羅 match の警告・エラー |
| 型エイリアス完全化 | v45.5 | 透過的互換性・opaque 非互換性 |
| エラーメッセージ改善 Phase 1 | v45.6 | E0101〜E0200 suggestion 追加 |
| エラーメッセージ改善 Phase 2 + 数値リテラル `_` | v45.7 | E0201〜E0413 suggestion・`1_000_000` |
| examples 更新 Phase 1 | v45.8 | !Effect 除去確認・return ガード節 |
| examples 更新 Phase 2 + v46.0 前調整 | v45.9 | stage_seq_demo 修正・overview 作成 |
```

### §2 — `README.md` 更新

「Language Refinement」を含む v46.0 達成の一言を追記する（`readme_mentions_language_refinement` テストが依存）。既存の記述スタイルを崩さず、バージョン履歴または機能説明の自然な場所に追記する。

### §3 — `driver.rs`: v46000_tests 追加

`v459000_tests` の直後に `v46000_tests` モジュールを追加（4件）:

```rust
#[cfg(test)]
mod v46000_tests {
    #[test]
    fn cargo_toml_version_is_46_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"46.0.0\""),
            "Cargo.toml version should be 46.0.0"
        );
    }

    #[test]
    fn changelog_has_v46_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v46.0.0]"),
            "CHANGELOG.md should have v46.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_language_refinement() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Language Refinement"),
            "MILESTONE.md should mention 'Language Refinement'"
        );
    }

    #[test]
    fn readme_mentions_language_refinement() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Language Refinement"),
            "README.md should mention 'Language Refinement'"
        );
    }
}
```

### §4 — `cargo clean` ★クリーンアップ

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

クリーン後に `cargo test` を再実行し、全テスト通過を確認する（テスト数 ≥ 2989）。

### §5 — `fav/tmp/hello.fav` 復元（cargo clean 後の必須作業）

`cargo clean` で `fav/tmp/` 以下のファイルが消える。`bootstrap_c2_artifact_roundtrip` テストが依存する `fav/tmp/hello.fav` を復元する:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

## 変更しないファイル

- `ast.rs` / `checker.rs` / `compiler.rs` / `vm.rs` / `lexer.rs` / `parser.rs`（コードフリーズ）
- `error_catalog.rs`
- `examples/` 以下の .fav ファイル

---

## 完了条件

- `cargo test` 全通過（failures=0 かつテスト数 ≥ **2992**、推定: 2988 + 4 = 2992）
- `cargo clippy -- -D warnings` クリーン
- `v46000_tests` 4 件すべて pass
- `MILESTONE.md` に `"Language Refinement"` が含まれる
- `README.md` に `"Language Refinement"` が含まれる
- `cargo clean` 完了
- `CHANGELOG.md` に v46.0.0 エントリ追加
- `versions/current.md` を v46.0.0（2992 tests）に更新
- `fav/Cargo.toml` version → `46.0.0`
