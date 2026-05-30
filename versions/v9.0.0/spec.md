# Favnir v9.0.0 Spec

Date: 2026-05-30
Theme: セルフホスト完成宣言（Self-Hosting Completion）

---

## 背景

v8.11.0 完了時点で、`fav run` の全ケース（単一ファイル・rune import・fav.toml プロジェクト）が
Favnir pipeline（checker.fav + compiler.fav）で動作するようになった。

v9.0.0 は「セルフホスト完成」のマイルストーン宣言バージョン。
新機能の追加ではなく、状態の確定・整理・宣言が主目的。

---

## セルフホスト完成の定義

### 完成条件（すべて v8.x で達成済み）

| 条件 | 達成バージョン |
|---|---|
| `fav check` が Favnir pipeline（checker.fav）経由 | v8.1.0 |
| `fav run`（単一ファイル）が Favnir pipeline（compiler.fav）経由 | v8.5.0 |
| `fav run`（rune import あり）が Favnir pipeline 経由 | v8.6.0 |
| `fav run`（fav.toml プロジェクト）が Favnir pipeline 経由 | v8.11.0 |
| 型チェックが checker.fav（E0001〜E0009）で動作 | v8.9.0〜v8.10.0 |

### 残存 Rust 依存（恒久・許容）

| コンポーネント | 理由 |
|---|---|
| VM（実行エンジン） | 設計上 Rust のまま（フルセルフホストは対象外） |
| ファイルパス解決（rune/project ソース収集） | OS ファイルシステム操作は Rust で担う |
| stdlib コア（List.map 等の Rust 実装） | パフォーマンス上の設計判断 |

---

## v9.0.0 で行うこと

### Phase A: `--legacy` フラグの非推奨化

`fav run --legacy` はセルフホスト完成後は不要になる。
ただし即座に削除はせず、**deprecated** として扱う:
- `--legacy` 使用時に deprecation 警告を stderr に出力
- `--help` テキストに `[deprecated]` を明記

### Phase B: バージョン定数・CHANGELOG 更新

- `fav/Cargo.toml` のバージョン: `"5.0.0"` → `"9.0.0"`（または CHANGELOG 追記のみ）
- `versions/v9.0.0/` にドキュメント一式作成

### Phase C: self-hosting 完成検証テスト追加

```rust
/// v9.0.0 到達の証明: 全パスが Favnir pipeline で動作することを確認。
#[test]
fn v900_self_hosting_complete() {
    // 1. 単一ファイル → compile_file_to_bytes_rune が機能する
    // 2. rune import → collect_merged_sources が機能する
    // 3. プロジェクトモード → compile_project_to_bytes が機能する
    // → 既存 dispatch_* テストが代替するため、このテストは宣言的コメントのみ
}
```

実際には既存の dispatch テスト群（v8.5〜v8.11）がすべてをカバーしているため、
新規テストは最小限（宣言的なドキュメントテスト）。

### Phase D: ドキュメント更新

- `README`（または `CHANGELOG.md`）にセルフホスト完成を記録
- `MEMORY.md` を v9.0.0 完了状態に更新

---

## `--legacy` 非推奨化の詳細

### 現状

```rust
pub fn cmd_run(file: Option<&str>, db_url: Option<&str>, legacy: bool) {
    ...
    let use_favnir = !legacy;
    if use_favnir { ... } else { /* Rust pipeline */ }
}
```

### v9.0.0 変更

```rust
if legacy {
    eprintln!("warning: --legacy is deprecated and will be removed in a future version.");
    eprintln!("         The Favnir pipeline is now the default for all modes.");
}
```

`--help` テキスト（`main.rs`）:
```
--legacy    [deprecated] Use the Rust compiler pipeline instead of Favnir
```

---

## テスト目標

- `cargo test` — 1135+ tests passing（既存テスト全件）
- `cargo test v900` — 新規宣言テスト通過

---

## 注意事項

### セルフホスト「完成」の意味

「セルフホスト完成」とは「Favnir で書いたコードが Favnir 自身のコンパイル・型チェックを担う」状態。
VM（Rust）や OS インターフェース（ファイルI/O）は引き続き Rust が担うが、
**言語処理系のロジック**（型チェック・コンパイル）は Favnir 実装経由で動作する。

この状態は「Favnir が自分自身を型チェック・コンパイルできる」ことを意味し、
bootstrap（v6.2.0）の証明と合わせて、Favnir が自己完結した言語実装であることを示す。

### v9.0.0 以降のロードマップ

セルフホスト完成後は、言語機能・エコシステム強化フェーズに移行:
- Rune 拡充（HTTP rune, Parquet rune 等）
- OSS 公開準備（GitHub Public 化）
- エンタープライズ向け機能（スキーマ推論、fav explain 強化）
