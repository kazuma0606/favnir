# Spec: v50.1.0 — エラー診断統一 Phase 1（全コード suggestion 補完）

Date: 2026-07-18
Status: Draft

---

## 概要

`error_catalog.rs` に登録されたエラーコードのうち `suggestion: None` が残る 34 件すべてに
有意義な修正提案テキストを追加する。
カバレッジテスト（全エントリに `suggestion: Some` が存在することを assert）で完備を保証する。

---

## 背景

v45.6.0 でエラーカタログに `suggestion: Option<&'static str>` フィールドが追加され、
主要な E0001・E0003・E0007・E0415 等に did-you-mean / 修正提案が実装された。
しかし v50.0.0 時点で `suggestion: None` のままのコードが 34 件残っており、
それらのエラーに遭遇したユーザーは次の一手がわからない状態となっている。

本バージョンでは **全エラーコードへの suggestion 統一適用** を完了させ、
「エラーを見れば次の一手がわかる」状態を確立する。

---

## 仕様

### 対象ファイル

`fav/src/error_catalog.rs`

### 変更内容

`suggestion: None` となっている全 34 エントリに対して、
そのエラーコードの意味に対応した英語��修正提案テキストを設定する。

各 suggestion テキストの要件:
- 修正の方向性を具体的に示す（例: 「Check the spelling.」だけでなく「Did you mean `X`?」）
- 1〜3 文程度に収める
- 既存の `suggestion: Some(...)` エントリと文体を統一する（英語・命令調）

主な対象エラーコードと提案文の方針（実装時に error_catalog.rs の各エントリを参照して確定）:

| 行範囲 | 対象コード群 | suggestion 方針 |
|---|---|---|
| 行 50〜150 付近 | E0004〜E0016 系 | 型・スコープ・引数・エフェクト関連の修正手順を記述 |
| 行 663〜856 付近 | E0380〜E0384 / E0420 / E0500〜E0505 / E0580〜E0581 / E0601〜E0605 / E0901〜E0903 系 | スキーマ・CEP・モジュール・循環 import・DB ランタイム・廃止キーワード系の提案 |

### テスト仕様

`v501000_tests` モジュールを `v50000_tests` の直前に追加（2 件）:

1. `error_suggestion_all_covered`
   - `error_catalog.rs` の `CATALOG` 定数（または全 `FavError` 構築関数）��ループし、
     `suggestion` フィールドが `None` のエントリが存在しないことを assert
   - 実装方針: `driver.rs` 内で `error_catalog::all_errors()` を呼び出し、
     各エントリの `suggestion.is_some()` を確認する
   - 実際には `include_str!` で `error_catalog.rs` を読み込み、
     `"suggestion: None"` が含まれないことを文字列レベルで確認する簡易実装でも可

2. `error_suggestion_e0018_text`
   - `error_catalog.rs` を `include_str!` で読み込み
   - E0018 の suggestion テキストに `"no longer needed"` が含まれることを確認
   - `"no longer needed"` は他のエントリには含まれない E0018 固有のフレーズ

### テスト配置

`v501000_tests` モジュールを `driver.rs` の `v50000_tests` の直前に挿入する。

```rust
#[cfg(test)]
mod v501000_tests {
    #[test]
    fn error_suggestion_all_covered() {
        let content = include_str!("error_catalog.rs");
        assert!(
            !content.contains("suggestion: None"),
            "error_catalog.rs should have no suggestion: None entries"
        );
    }

    #[test]
    fn error_suggestion_e0018_text() {
        let content = include_str!("error_catalog.rs");
        assert!(
            content.contains("no longer needed"),
            "E0018 suggestion should contain 'no longer needed'"
        );
    }
}
```

> 注: `include_str!("error_catalog.rs")` のパスは `driver.rs` と同一ディレクトリ（`fav/src/`）への参照。
> `fav/src/driver.rs` と `fav/src/error_catalog.rs` は同階層のため、ファイル名のみで解決できる。

---

## 完了条件

- `cargo test` 3093 tests passed, 0 failed（3091 + 2 件）
- `error_catalog.rs` に `suggestion: None` が 0 件
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.1.0 エントリ追加
- `versions/current.md` を v50.1.0 に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.1.0 実績を記入
