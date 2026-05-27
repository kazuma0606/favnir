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

## テスト手順

```bash
# Rust テスト（全 1043 件）
cd fav
cargo test

# 特定テストのみ
cargo test validate
cargo test bootstrap

# サンプルコードの型チェック
fav check examples/basic/hello.fav
fav check examples/pipeline/pipeline.fav
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

## ライセンス

コントリビューションは MIT ライセンスに同意したものとみなします。
