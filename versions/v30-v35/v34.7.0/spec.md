# v34.7.0 — Spec

## 概要

**テーマ**: ドキュメント・examples ctx 移行

**方針**: ctx 構文の完全リファレンスガイドを新規作成し、
`getting-started.mdx` と `README.md` に AppCtx 構文の説明を追加する。
既存の全 MDX コードサンプル書き換えは **スコープ外**
（影響ファイルが多いため v34.8 以降で個別対応）。

---

## 背景

v34.5.0 で W022 lint ルールと IoCtx インターフェースを追加。
v34.6.0 で DbCtx / HttpClient / StreamClient インターフェースを追加。
v34.7.0 ではこれらの設計思想と移行手順を開発者向けドキュメントとして整備する。

### 既存実装の確認

| 機能 | 状態 | 備考 |
|---|---|---|
| `fav migrate --from-effects` | 実装済み（v13.10.0） | driver.rs |
| `runes/ctx/io.fav` (IoCtx) | 実装済み（v34.5.0） | |
| `runes/ctx/db.fav` (DbCtx) | 実装済み（v34.6.0） | |
| `runes/ctx/http.fav` (HttpClient) | 実装済み（v34.6.0） | |
| `runes/ctx/stream.fav` (StreamClient) | 実装済み（v34.6.0） | |
| `site/content/docs/ctx-syntax-guide.mdx` | **未実装 → 本バージョンで新規作成** | |
| `getting-started.mdx` の AppCtx 説明 | **未記載 → 本バージョンで追加** | |
| `README.md` の v34.5〜v34.7 ctx 移行言及 | **未記載 → 本バージョンで追加** | |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| MDX コードサンプル全書き換え | `!Effect` → ctx 構文に更新 | **スコープ外** — v34.8 以降で対応 |
| examples/.fav 全ファイル書き換え | `fav migrate --from-effects --dir` | **スコープ外** — v34.8 以降で対応 |
| ctx 構文リファレンスガイド | 新規作成 | **本バージョンで追加** |
| getting-started AppCtx 説明 | 入門記事に追加 | **本バージョンで追加（additive）** |
| README ctx 移行シリーズ言及 | v34.5-v34.7 系列の記録 | **本バージョンで追加（additive）** |

---

## 実装スコープ

### 新規ファイル

```
site/content/docs/ctx-syntax-guide.mdx   ctx 構文完全リファレンスガイド
```

### 変更ファイル（additive のみ）

1. `fav/Cargo.toml` — version `34.6.0` → `34.7.0`
2. `site/content/learn/getting-started.mdx` — AppCtx を使ったパイプライン例を末尾に追加
3. `README.md` — v34.5〜v34.7 ctx 移行シリーズの記録を追加
4. `fav/src/driver.rs` — `cargo_toml_version_is_34_6_0` をスタブ化、`v347000_tests` 5 件追加
5. `benchmarks/v34.7.0.json` — 新規作成
6. `CHANGELOG.md` — `[v34.7.0]` セクション先頭追記
7. `versions/current.md` — 最新安定版を v34.7.0 に更新

---

## site/content/docs/ctx-syntax-guide.mdx 仕様

タイトル: `ctx 構文リファレンス`

含むべき内容:
- AppCtx の設計思想（capability 引数による副作用の明示）
- `bind { field } <- ctx` 分解構文の説明
- IoCtx / DbCtx / HttpClient / StreamClient の使い方例
- Before / After 対比（`!Effect` → ctx）
- テストでの `Ctx.mock(...)` 利用例

**含むべきキーワード**: `"AppCtx"` / `"bind"`（アサーション対象）

---

## getting-started.mdx 追加内容仕様

末尾に「Capability Context を使う」セクションを追加:

```markdown
## Capability Context を使う

v34.5.0 以降、副作用は `AppCtx` パラメータで表現します:

```favnir
import runes/ctx

fn fetch_and_save(ctx: AppCtx, url: String) -> Unit {
    bind { http, io } <- ctx
    bind data <- http.get(url)
    io.println(data)
}
```

詳細は [ctx 構文リファレンス](/docs/ctx-syntax-guide) を参照。
```

**含むべきキーワード**: `"AppCtx"`（アサーション対象）

---

## README.md 追加内容仕様

既存の v34.0 マイルストーン行の直後に追加:

```markdown
v34.5.0〜v34.7.0 で、`!Effect` アノテーションを廃止し Capability Context（AppCtx）に一本化しました。
`fav migrate --from-effects` で既存コードを自動移行できます。
```

**含むべきキーワード**: `"v34.5"`（アサーション対象）

---

## テスト仕様（v347000_tests）

```rust
// ── v34.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v347000_tests {
    #[test]
    fn cargo_toml_version_is_34_7_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.7.0"), "Cargo.toml must contain '34.7.0'");
    }

    #[test]
    fn ctx_syntax_guide_exists() {
        let src = include_str!("../../site/content/docs/ctx-syntax-guide.mdx");
        assert!(
            src.contains("AppCtx"),
            "ctx-syntax-guide.mdx must document AppCtx"
        );
    }

    #[test]
    fn ctx_syntax_guide_covers_bind() {
        let src = include_str!("../../site/content/docs/ctx-syntax-guide.mdx");
        assert!(
            src.contains("bind"),
            "ctx-syntax-guide.mdx must show bind destructure syntax"
        );
    }

    #[test]
    fn getting_started_updated() {
        let src = include_str!("../../site/content/learn/getting-started.mdx");
        assert!(
            src.contains("AppCtx"),
            "getting-started.mdx must mention AppCtx"
        );
    }

    #[test]
    fn readme_has_ctx_migration_ref() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("v34.5"),
            "README.md must reference v34.5 ctx migration series"
        );
    }
}
```

### 設計注記

- `use super::*` は**不要**（`include_str!` のみ使用）
- WASM ゲートなし
- `readme_has_ctx_migration_ref` の `include_str!` パス: `"../../README.md"`
  （`fav/src/` → `../../` = `favnir/README.md`）
- v347000_tests は v346000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.7.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.7.0"`
- [ ] `cargo_toml_version_is_34_6_0` が空スタブになっていること
- [ ] `cargo test --bin fav v347000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2571 件想定 = 2566 + 5、0 failures）
- [ ] `site/content/docs/ctx-syntax-guide.mdx` が存在し `"AppCtx"` と `"bind"` を含むこと
- [ ] `site/content/learn/getting-started.mdx` が `"AppCtx"` を含むこと
- [ ] `README.md` が `"v34.5"` を含むこと
- [ ] `CHANGELOG.md` に `[v34.7.0]` セクション
- [ ] `benchmarks/v34.7.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.7.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
