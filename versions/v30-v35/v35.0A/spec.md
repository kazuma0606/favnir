# v35.0A spec — ドキュメント ctx 構文統一 + Production Ready 宣言

**バージョン**: v35.6.0
**日付**: 2026-07-05
**前提**: v34.9A (v35.5.0) COMPLETE

---

## 目的

`!Effect` が言語から完全に削除された今、
ドキュメントサイトの全コードサンプルを ctx 構文に統一する。
ユーザーが「副作用のある処理は ctx: AppCtx を渡す」という唯一のパターンを学べるようにする。

合わせて v35.0 Production Ready マイルストーンを宣言する。

---

## 変更対象

| 対象 | 件数 | 内容 |
|---|---|---|
| `site/content/docs/` MDX | 73 件 | `!Effect` コードサンプル → ctx 構文 |
| `site/content/cookbook/` MDX | 48 件 | 同上 |
| `site/content/learn/` MDX | 2 件 | 入門チュートリアルの書き換え |
| `README.md` | 1 件 | エフェクト節を ctx 構文に書き換え |
| `site/content/docs/ctx-syntax-guide.mdx` | 1 件（新規 or 更新） | ctx パターンの公式ガイドページ |

合計: **125 件**

---

## ctx 構文ガイドのメッセージ

サイトドキュメントで一貫して伝えるメッセージ:

> **Favnir では副作用（DB・HTTP・IO・ストリーム等）はすべて `ctx: AppCtx` を通じて行う。**
> `AppCtx` は Capability Context で、「何に接続でき、何ができるか」を型で表現する。
> `!Effect` アノテーション構文は v35.4.0 で削除された。

コードサンプルのパターン統一:
```favnir
// ❌ 古い構文（v35.4.0 以前 — 現在はコンパイルエラー）
fn fetch_orders(conn_str: String) -> List<Order> !Postgres {
    Postgres.query[Order](conn_str, "SELECT * FROM orders")
}

// ✅ 現在の構文
fn fetch_orders(ctx: AppCtx, conn_str: String) -> List<Order> {
    Postgres.query[Order](conn_str, "SELECT * FROM orders")
}
```

---

## 移行スクリプト方針

MDX ファイル内の fenced code block（` ```favnir ` ブロック）のみを対象に
Python スクリプトで一括変換する。

変換ルール（コードブロック内のみ）:
1. `stage Name: A -> B !Effect = |arg| {` → `stage Name: A -> B = |arg| {`
2. `fn name(params) -> T !Effect {` → `fn name(ctx: AppCtx, params) -> T {`（ctx がなければ追加）
3. `fn name(ctx: AppCtx, params) -> T !Effect {` → `fn name(ctx: AppCtx, params) -> T {`（trailing Effect のみ除去）
4. 通常テキスト行（コードブロック外）には手を加えない

---

## ctx-syntax-guide.mdx の内容

`site/content/docs/ctx-syntax-guide.mdx` として完成版を作成する。
（既に存在する場合は更新。v34.5A で作成されているが !Effect への言及が古い可能性あり）

必須セクション:
1. **概要** — Capability Context とは何か
2. **基本パターン** — `fn f(ctx: AppCtx, ...) -> T { ... }`
3. **stage との違い** — stage は ctx を受け取らない（IO は fn で行う）
4. **AppCtx のフィールド** — `ctx.io` / `ctx.db` / `ctx.http` / `ctx.stream` 等
5. **移行前後の対比** — ~~`!Effect`~~ vs `ctx: AppCtx`（旧構文はコンパイルエラーになることを明示）
6. **テスト時** — `Ctx.test_ctx_raw()` の使い方

---

## Production Ready 宣言（v35.0）

`MILESTONE.md` に v35.0 Production Ready 宣言を追記する。

宣言文:
> **v35.0 — Production Ready（2026-07-05）**
> Favnir は本番データエンジニアリング案件で選択できる状態に到達した。
> - `!Effect` アノテーション構文を完全廃止（v35.4.0）し、Capability Context に統一
> - 全 Rune・examples・ドキュメントが ctx 構文に移行済み
> - 2600+ テスト、0 failures

---

## 完了条件

- `site/content/` 全 MDX の fenced code block に `!Effect` が残存しないこと
- `site/content/docs/ctx-syntax-guide.mdx` が上記 6 セクションを含むこと
- `README.md` が ctx 構文を説明していること
- `MILESTONE.md` に v35.0 宣言が追記されていること
- `cargo test` 全件 PASS
