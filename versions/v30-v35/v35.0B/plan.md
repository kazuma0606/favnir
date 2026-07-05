# v35.7.0 (v35.0B) 実装計画

## Phase 1: docs_server.rs 修正

### 1-1. IO_FUNCTIONS の signature 修正

対象行（`fav/src/docs_server.rs`）：

```
before: signature: "String -> Unit !Io",   // println
after:  signature: "String -> Unit",

before: signature: "String -> Unit !Io",   // print
after:  signature: "String -> Unit",

before: signature: "() -> String !Io",     // read_line
after:  signature: "() -> String",
```

### 1-2. IO_FUNCTIONS の effects 修正

```
before: effects: &["Io"],   // println
after:  effects: &[],

before: effects: &["Io"],   // print
after:  effects: &[],

before: effects: &["Io"],   // read_line
after:  effects: &[],
```

## Phase 2: バージョン管理

### 2-1. Cargo.toml バージョン更新

`35.6.0` → `35.7.0`

### 2-2. CHANGELOG.md 追記

```markdown
## [35.7.0] — 2026-07-05

### Changed
- `docs_server.rs`: IO 関数シグネチャから `!Io` エフェクト表記を除去
- `docs_server.rs`: IO 関数の `effects` フィールドを空配列に統一
- これにより Favnir コードベースから `!Effect` 表記が**完全に除去**された

### Migration
- `fav doc` の `/api/stdlib` エンドポイントが返す IO 関数シグネチャが変更
  - 旧: `"String -> Unit !Io"` / 新: `"String -> Unit"`
  - `effects` 配列は空になるが、フィールド自体は後方互換のため残存
```

## Phase 3: テスト追加

`driver.rs` に `v35700_tests` モジュールを追加：

```rust
#[cfg(test)]
mod v35700_tests {
    // 1. バージョン確認
    // 2. IO signature に ! がない
    // 3. IO effects が空
    // 4. CHANGELOG エントリ
    // 5. docs_server.rs ソースに !Io 等がない（include_str! で確認）
}
```

## 実装順序

1. `docs_server.rs` 修正（6行）
2. `Cargo.toml` バージョン更新
3. `CHANGELOG.md` 追記
4. `driver.rs` テスト追加
5. `cargo test` で全テスト pass 確認
6. `/review wasm` スキップ（Rust コードのみ変更）
7. `/review code` でコードレビュー
