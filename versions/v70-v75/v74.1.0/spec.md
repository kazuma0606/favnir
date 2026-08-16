# v74.1.0 仕様書 — Rune マーケットプレイス（バージョン管理・依存解決）

Date: 2026-08-13

---

## Background

Favnir の Rune エコシステムを拡大するために、Rune の公開・インストール・依存管理を行う
公式マーケットプレイス機能を実装する。
`fav publish rune` / `fav add rune` / `fav rune list` コマンドの基盤構造を
`driver.rs` に追加し、セマンティックバージョニング対応の依存解決を提供する。

本バージョンは基盤データ構造と関数のみを実装する。CLI コマンドおよびレジストリ通信は後続バージョンで実装する。

---

## Goals

1. Rune パッケージのメタデータ構造（`RunePackage`）を定義する
2. `format_rune_publish_manifest` — パッケージ公開メタデータを JSON 形式で生成する
3. `parse_rune_dep_entry` — `fav.toml` の `[rune.deps]` エントリをパースする
4. `v741000_tests` モジュール（2 件）を追加する

---

## API / コマンド例

```bash
# 公式マーケットプレイスへの公開
$ fav publish rune ./runes/mycompany-crm
Published: mycompany/crm@1.0.0

# インストール
$ fav add rune mycompany/crm@^1.0
# fav.toml に [rune.deps] として記録

# 依存関係一覧
$ fav rune list
  mycompany/crm  1.0.2  (latest: 1.0.2)
  favnir/json    9.0.0  (latest: 9.0.0)
  favnir/postgres 5.1.0 (latest: 5.2.0) ← update available
```

### `RunePackage` 構造体

```rust
#[derive(Debug, Clone)]
pub struct RunePackage {
    pub name: String,       // "mycompany/crm"
    pub version: String,    // "1.0.0"
    pub description: String,
    pub author: String,
}
```

### `format_rune_publish_manifest`

```rust
pub fn format_rune_publish_manifest(pkg: &RunePackage) -> String {
    // JSON 形式で出力
    // {"name":"mycompany/crm","version":"1.0.0","description":"...","author":"..."}
}
```

### `parse_rune_dep_entry`

```rust
// "mycompany/crm@^1.0" → ("mycompany/crm", "^1.0")
pub fn parse_rune_dep_entry(entry: &str) -> Result<(String, String), String>
```

前提: パッケージ名（`name` 部分）に `@` 文字は含まれない。`rfind('@')` で最後の `@` を区切り文字として使用する。

---

## Success Criteria

1. `rune_marketplace_publish_format` テストが pass する
   - `RunePackage` を構築して `format_rune_publish_manifest` を呼び出し、name / version / description / author を含む JSON が返る
2. `rune_marketplace_add_updates_toml` テストが pass する
   - `parse_rune_dep_entry("mycompany/crm@^1.0")` が `("mycompany/crm", "^1.0")` を返す
   - バージョン指定なし / 不正フォーマットで `Err` を返す
3. `cargo test` で 3671 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `RunePackage` / `format_rune_publish_manifest` / `parse_rune_dep_entry` + `v741000_tests` 追加（`include_str!` パス: `../Cargo.toml` = `fav/`、`../../CHANGELOG.md` = `favnir/`） |
| `fav/Cargo.toml` | `version = "74.1.0"` に更新 |
| `CHANGELOG.md` | v74.1.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
