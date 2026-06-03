# Favnir v10.0.0 仕様書 — OSS 公開準備完了

作成日: 2026-06-03

---

## 概要

v9.x シリーズで積み上げたエコシステム（セルフホスト・型チェック・fmt/lint/doc・par 並列実行等）を
「外部に見せられる状態」に整えて GitHub Public 化するマイルストーン。
機能追加は最小限（`fav new` スキャフォールディングのみ）。

---

## 1. `fav new` — プロジェクトスキャフォールディング

### コマンド

```
fav new <name>
```

### 生成されるファイル構造

```
<name>/
  fav.toml        # プロジェクト設定
  src/
    main.fav      # エントリポイントテンプレート
  .gitignore      # *.fvc / .fav_cache/
```

### fav.toml テンプレート

```toml
[project]
name = "<name>"
version = "0.1.0"
src = "src"
```

### src/main.fav テンプレート

```favnir
type Order = { id: Int  item: String  amount: Float }

stage ParseOrder: String -> Order = |s| {
  Order { id: 1  item: s  amount: 0.0 }
}

stage FormatOrder: Order -> String = |o| {
  "Order#" + Int.to_string(o.id) + ": " + o.item
}

seq ProcessOrder = ParseOrder |> FormatOrder
```

最小の stage + seq 例。`fav run src/main.fav` で即実行できる。

### .gitignore テンプレート

```
*.fvc
.fav_cache/
```

### 実装方式

- Rust 変更なし
- `cli.fav` に `fn cmd_new(name: String) -> Unit !Io` を追加
- ファイル生成には `IO.write_file_raw` を使用
- ディレクトリ作成には `IO.make_dir_raw` primitive が必要（vm.rs に追加）
  - `IO.make_dir_raw(path: String) -> Unit !Io`
  - 内部: `std::fs::create_dir_all`（既存なら成功扱い）

---

## 2. GitHub Actions CI

### ワークフロー: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [master]
  pull_request:

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            fav/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('fav/Cargo.lock') }}

      - name: Build
        working-directory: fav
        run: cargo build --release

      - name: Test
        working-directory: fav
        run: cargo test

      - name: fav check (self)
        working-directory: fav
        run: |
          ./target/release/fav check self/compiler.fav
          ./target/release/fav check self/checker.fav
          ./target/release/fav check self/cli.fav

      - name: fav lint (self)
        working-directory: fav
        run: |
          ./target/release/fav lint self/compiler.fav
          ./target/release/fav lint self/checker.fav

      - name: fav fmt --check (self)
        working-directory: fav
        run: |
          ./target/release/fav fmt --check self/compiler.fav
          ./target/release/fav fmt --check self/checker.fav
```

### CI の目的

- `cargo test` — Rust ユニット・統合テスト全件
- `fav check self/` — Favnir pipeline の self-check（型エラーなし）
- `fav lint self/` — W001〜W005 ルールで警告なし
- `fav fmt --check self/` — コードフォーマット冪等性確認（差分なし）

---

## 3. CONTRIBUTING.md

リポジトリルートに配置。OSS コントリビューターへのガイド。

### 内容

1. **開発環境セットアップ**
   - Rust stable（`rustup update stable`）
   - `cd fav && cargo build`
2. **テスト実行**
   - `cargo test` — 全テスト
   - `cargo test bootstrap` — bootstrap 検証
   - `cargo test checker_fav` — checker.fav self-check
3. **PR ガイドライン**
   - 1 PR = 1 機能/バグ修正
   - テストを必ず追加
   - `fav fmt` + `fav lint` をパスさせること
   - bootstrap 維持（`cargo test bootstrap` 通過必須）
4. **Favnir のセルフホスト構成の説明**
   - `fav/self/compiler.fav` — セルフホストコンパイラ
   - `fav/self/checker.fav` — セルフホスト型チェッカー
   - `fav/self/cli.fav` — セルフホスト CLI
5. **バージョン管理**
   - `versions/` に仕様・計画・タスクを管理
   - バージョン番号は `fav/Cargo.toml` と `fav/self/cli.fav` の両方で管理

---

## 4. CHANGELOG.md

[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) 形式。
リポジトリルートに配置。

### 構成

- `[Unreleased]` セクション（将来の変更を記録する場所）
- v10.0.0 → v9.0.0 → v8.0.0 → v7.0.0 → ... の降順

各バージョンは 1〜3 行のサマリー（詳細は `versions/` ディレクトリを参照）。

### 形式例

```markdown
## [10.0.0] - 2026-06-03
### Added
- `fav new <name>` — プロジェクトスキャフォールディング
- GitHub Actions CI（cargo test / fav check / fav lint / fav fmt --check）
- CONTRIBUTING.md / CHANGELOG.md

## [9.13.0] - 2026-06-03
### Added
- `par [A, B] |> Merge` — 並列 stage 実行（VM スレッド並列化）
- E0016（par 入力型不一致）/ E0017（par 未定義 stage）
```

---

## 5. LICENSE 確認

MIT ライセンスをリポジトリルートに配置。

```
MIT License

Copyright (c) 2026 Yoshi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, ...
```

---

## 積み残し確認

### W004 lint ルール（TooManyArgs）

v9.4.0 の実装ログには "+ W004 lint ルール" と記録されているが、
`memory/MEMORY.md` の未実装タスク欄に残存している不整合を解消する。

- `compiler.fav` に `lint_fn_w004` 関数が存在すれば **実装済み** として記録整理
- 存在しなければ **v10.0.0 で実装する**（`compiler.fav` の lint セクションに追加）

### compiler.fav par stack overflow（既知制限）

v9.13.0 の F-1e テストにて、`compile_file_to_bytes`（compiler.fav pipeline）で
par を含む seq をコンパイルすると Rust stack overflow が発生することが確認されている。

根本原因: `List.fold` + lambda を含む stage body の再帰コンパイルが Favnir VM の
コールスタックを深くしすぎる（Rust の stack frame 深度制限）。

対応方針: **v10.0.0 では修正しない**。`known-limitations.md` に文書化する。

---

## スコープ外（v10.1.0 以降）

- macOS / Windows クロスプラットフォーム CI
- Playground（WASM）の更新
- サイトドキュメント大規模更新
- compiler.fav par stack overflow の根本修正
- `fav publish` コマンド（Rune レジストリへの公開）
- GitHub Releases 自動化
