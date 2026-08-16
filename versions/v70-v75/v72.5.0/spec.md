# v72.5.0 仕様 — Playground 2.0

Date: 2026-08-12
Status: 計画中

---

## Background

Favnir リファレンスサイトのブラウザ内 Playground を全面強化する。
本バージョンでは Rust 側に次の 2 機能を追加し、ユニットテストで検証する:

1. **Playground テンプレートギャラリー** — 最低 5 エントリの静的定義（`PlaygroundTemplate` + `PLAYGROUND_TEMPLATES`）
2. **共有リンク生成** — コードを base64 エンコードして `/playground?code=<base64>` 形式の URL を生成する `playground_share_url` 関数

サイト側（TypeScript / Next.js）の強化（Monaco エディタ統合・AI 補完・可視化等）は別タスク（v73.x 以降）。
WASM ビルド対応は既存の WASM 設定を利用するため本バージョンでの追加作業はない。

---

## Goals

1. `PlaygroundTemplate` 構造体を `driver.rs` に追加する（`name: &'static str`, `description: &'static str`, `code: &'static str`）
2. `PLAYGROUND_TEMPLATES: &[PlaygroundTemplate]` を定義し 5 エントリ以上を含める
3. `playground_share_url(code: &str) -> String` を `pub fn` で追加する
   - コードを base64 エンコード（標準ライブラリ + `base64` crate、または手動 URL-safe エンコード）
   - `/playground?code=<base64>` 形式の文字列を返す
4. テスト 2 件を `v725000_tests` モジュールとして追加する

---

## API 例

### テンプレートギャラリー

```rust
assert!(PLAYGROUND_TEMPLATES.len() >= 5);
assert_eq!(PLAYGROUND_TEMPLATES[0].name, "Hello World");
```

### 共有リンク

```rust
let url = playground_share_url("fn main() -> Unit { }");
assert!(url.starts_with("/playground?code="));
assert!(url.len() > "/playground?code=".len());
```

---

## 実装詳細

### `PlaygroundTemplate`

```rust
pub struct PlaygroundTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub code: &'static str,
}
```

### `PLAYGROUND_TEMPLATES` — 5 エントリ

| 名前 | 説明 |
|---|---|
| Hello World | 最小の Favnir プログラム |
| CSV ETL | CSV 読み込み → スキーマ検証 → 変換 |
| AI Generate | `fav ai generate` 生成コードのサンプル |
| Distributed Par | `par` 並列ステージのデモ |
| Data Quality | `Schema.validate_all` を使ったデータ品質パイプライン |

### `playground_share_url(code: &str) -> String`

base64 外部クレートを追加せず、Rust 標準の手法で実装:

```rust
pub fn playground_share_url(code: &str) -> String {
    let encoded = BASE64_ENCODE(code.as_bytes()); // URL-safe base64
    format!("/playground?code={encoded}")
}
```

base64 crate は追加しない（WASM 互換性リスクを避けるため）。
実装は hex エンコード（`format!("{b:02x}")` の連結）のみとし、URL-safe base64 は v73.x 以降に本格実装する。

---

## 成功条件

- `playground2_template_gallery_has_5_entries`: `PLAYGROUND_TEMPLATES.len() >= 5` を assert
- `playground2_share_url_format`: `playground_share_url("fn main() -> Unit { }")` が `/playground?code=` で始まり、かつ空でないコード部分を含むことを assert
- `cargo test v725000` で 2 件 pass
- `cargo test` 全体で 3627 tests pass（v72.4.0 完了時点 3625 + 2）
- `CHANGELOG.md` に `## [v72.5.0]` エントリが存在する
- `versions/current.md` の進行中バージョンが `v72.5.0` である

**WASM への影響**: `base64` crate を追加しない（hex エンコードを使用するため）。

---

## エラーコード

新規エラーコードなし。

---

## 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `PlaygroundTemplate` / `PLAYGROUND_TEMPLATES` / `playground_share_url` / `v725000_tests` 追加 |
| `fav/Cargo.toml` | `version = "72.4.0"` → `"72.5.0"` |
| `CHANGELOG.md` | `## [v72.5.0]` エントリ追加 |
| `versions/current.md` | 進行中バージョンを v72.5.0 に更新 |

---

## スコープ外（明示的除外）

- サイト側（TypeScript / Next.js）の Monaco エディタ統合・AI 補完（v73.x 以降）
- 実行結果の可視化（List<Record> → テーブル表示、List<Float> → グラフ）（v73.x 以降）
- `base64` crate 追加（WASM 互換性リスクを避け本バージョンでは hex エンコードを使用）
- 共有リンクの永続化・サーバー側ストレージ（v73.x 以降）
- WASM ビルドへの新規変更（既存設定で対応済み）
- `site/content/playground/` MDX 更新（v73.x 以降）
- `rustyline` 統合・`~/.fav_history` 永続化・Rune メソッド補完（v72.4.0 から延期、v72.6.0 以降に実施）
