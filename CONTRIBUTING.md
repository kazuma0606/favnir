# Contributing to Favnir

Favnir へのコントリビューションを歓迎します。

---

## 前提条件

| ツール | バージョン | 用途 |
|--------|-----------|------|
| Rust | stable (推奨: 最新) | コンパイラ・VM・CLI |
| Node.js | 22+ | リファレンスサイト |
| wasm-pack | 最新 | WASM バックエンド（任意） |

---

## ビルド手順

```bash
git clone https://github.com/kazuma0606/favnir
cd favnir/fav
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

サイトのビルド:

```bash
cd site
npm ci
npm run build
```

---

## 環境診断

PR を開く前に `fav doctor` で環境が正常かを確認してください:

```bash
./target/debug/fav doctor
# [OK]   fav version: 54.6.0
# [OK]   Rust toolchain: stable
# [OK]   fav.toml: valid
# [OK]   .fav-cache: intact
```

---

## テスト手順

```bash
# Rust テスト（全 3197 件）
cd fav
cargo test -j 8 -- --test-threads=8

# 特定テストのみ
cargo test bootstrap           # Bootstrap 検証（bytecode_A == bytecode_B）
cargo test checker_fav         # checker.fav セルフチェック

# Self-hosted コンポーネントの型チェック
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
./target/debug/fav check self/cli.fav

# Lint・フォーマット確認
./target/debug/fav lint self/compiler.fav
./target/debug/fav fmt --check self/compiler.fav
```

## ベンチマーク・パフォーマンス確認

パフォーマンスに影響する変更（VM・コンパイラ最適化等）を行った場合は、
`fav bench` でリグレッションがないことを確認してください:

```bash
cd fav
# ベンチマーク実行（全 bench_ テスト）
cargo test bench_ -- --nocapture

# ベースラインとの比較（benchmarks/baseline.json が基準値）
./target/debug/fav bench --compare ../benchmarks/baseline.json --fail-on-regression
```

---

## ブランチ命名規則

```
feat/<内容>     新機能
fix/<内容>      バグ修正
docs/<内容>     ドキュメントのみの変更
refactor/<内容> 動作を変えないリファクタリング
```

例: `feat/string-split`, `fix/vm-jump-offset`, `docs/duckdb-rune`

---

## コミットメッセージ形式

```
<type>: <概要>（50 文字以内）

<詳細（任意）>
```

`type` は `feat` / `fix` / `docs` / `refactor` / `test` / `chore` のいずれか。

---

## PR ガイドライン

1. `master` から作業ブランチを切る
2. `cargo test` が全件通ることを確認してから PR を開く
3. `cargo clippy -- -D warnings` でlint エラーがないことを確認する
4. PR の説明に「何を・なぜ変えたか」を記載する
5. 新機能には統合テストを追加する（`fav/src/backend/vm_stdlib_tests.rs` 等）

PR を開く前に **[INVARIANTS.md](./INVARIANTS.md) のチェックリスト**を必ず確認してください。
特に Bootstrap 検証（`cargo test bootstrap_full_self_hosting`）は必須です。

---

## Rune 追加ガイド

Favnir の Rune は **VM primitive（Rust）+ Favnir 層** の二層構造です。

### 1. VM primitive を追加（`fav/src/backend/vm.rs`）

```rust
// call_builtin の match アームに追加
("MyRune", "some_raw") => {
    // ...
    push_value(result);
}
```

### 2. 型シグネチャを追加（`fav/src/middle/checker.rs`）

```rust
("MyRune", "some_raw") => Some(FnSig {
    params: vec![Type::Str],
    ret: Type::Result(Box::new(Type::Str), Box::new(Type::Unknown)),
    effect: Some(Effect::Network),
}),
```

### 3. Favnir 層を実装（`runes/my-rune/my-rune.fav`）

```favnir
// VM primitive を薄くラップし、意味のある操作を提供する
public fn some_operation(arg: String) -> Result<String, MyError> !Network {
    MyRune.some_raw(arg)
}
```

### 4. テストを追加（`runes/my-rune/my-rune.test.fav`）

```favnir
test "some_operation returns expected value" {
    // ...
}
```

### 5. ドキュメントを追加（`site/content/docs/runes/my-rune.mdx`）

---

## セルフホスト一貫性

新機能を Rust 側に追加したら `fav/self/compiler.fav` への反映を忘れずに。
Bootstrap テストを常に通してください:

```bash
cargo test bootstrap
```

---

## コミュニティ Rune 開発ガイド

コミュニティ Rune の審査は以下の 5 条件（connect / read / write / error / test）で行います。

### 5 条件（connect / read / write / error / test）

| 条件 | 内容 |
|---|---|
| connect | `RUNE_NAME_URL` 等の環境変数でサービスに接続できる |
| read | データを取得する関数が 1 つ以上ある |
| write | データを書き込む関数が 1 つ以上ある |
| error | エラーを `Result.err` で返す（クラッシュしない）|
| test | `cargo test` で 3 件以上 PASS する |

### rune.toml の形式

各コミュニティ Rune は `rune.toml`（`[rune]` セクション）と `.fav` ファイルで構成します。

```toml
[rune]
name = "my-rune"
version = "0.1.0"
description = "My community Rune"
author = "Your Name"
license = "MIT"
```

### 投稿方法

1. `runes/<rune-name>/` ディレクトリを作成
2. `rune.toml` と `<rune-name>.fav` を追加（5 条件を満たすこと）
3. PR を開く — レビュー後 `runes/` にマージされます

### 第 1 回 Favnir Rune コンテスト（2026-07）

優秀な Rune には公式 README への掲載・グッズ等の特典あり。
詳細は [/community](/community) ページを参照。

---

## ライセンス

コントリビューションは MIT ライセンスに同意したものとみなします。
