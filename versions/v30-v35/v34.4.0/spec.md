# v34.4.0 — Spec

## 概要

**テーマ**: セキュリティ審査 v2

**方針**: v24.6.0（セキュリティ審査 v1）を Production Ready の観点で更新する。
4 つの審査項目（エフェクトシステム検証・OSS ライセンス・認証情報の扱い・実行サンドボックス）を
ドキュメントとして公開し、`site/content/docs/tools/` 配下に追加する。

---

## 背景

v24.6.0（セキュリティ審査 v1）で以下を確立した:
- `SECURITY_MODEL.md` — Capability 公理 4 条 + 推論規則
- `SECURITY.md` — CVE 対応プロセス（90日 responsible disclosure）
- W021 `pure_fn_calls_effectful` lint ルール

v34.4.0 では v34.x 系で追加された機能（ctx 構文・OSS 50+ Rune・実案件デモ）を
踏まえてセキュリティ状態を再確認し、4 項目の審査結果を Web で公開する。

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| エフェクトシステム形式検証 | W021 lint が機能しているか | `security-audit-v2.mdx` に W021 動作確認セクション追加 |
| OSS 依存ライセンス確認 | Cargo.toml の全依存が MIT/Apache-2.0 互換か | `oss-licenses.mdx` を新規作成（主要依存 20+ 件のライセンス表） |
| 認証情報の扱い | 環境変数経由のみか、コードに埋め込めないか | `security-audit-v2.mdx` に認証情報ガイドラインセクション追加 |
| 実行サンドボックス確認 | `fav run` の実行サンドボックス | `security-audit-v2.mdx` にサンドボックス確認セクション追加 |
| `SECURITY_MODEL.md` 更新 | 未指定 | v34 系の ctx 移行方針とその影響を追記（"v34" キーワード含む） |

---

## 実装スコープ

### 新規ファイル

```
site/content/docs/tools/
├── security-audit-v2.mdx   セキュリティ審査 v2 レポート
└── oss-licenses.mdx        OSS 依存ライセンス一覧
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.3.0` → `34.4.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_3_0` をスタブ化、`v344000_tests` 5 件追加
3. `SECURITY_MODEL.md` — v34.4 セクション追加（ctx 移行に伴う公理への影響）
4. `benchmarks/v34.4.0.json` — 新規作成
5. `CHANGELOG.md` — `[v34.4.0]` セクション先頭追記
6. `versions/current.md` — 最新安定版を v34.4.0 に更新

---

## site/content/docs/tools/security-audit-v2.mdx 仕様

```markdown
---
title: "セキュリティ審査 v2"
description: "Favnir v34.x のセキュリティ状態確認レポート（v24.6 以降の更新）"
---

# セキュリティ審査 v2

v34.4.0 時点での Favnir セキュリティ状態確認レポート。
v24.6.0（セキュリティ審査 v1）からの変更点を中心に記述する。

## 1. エフェクトシステム検証（W021）

W021 `pure_fn_calls_effectful` lint は v24.6.0 で導入済みで、引き続き有効。
...
## 2. 認証情報の扱い

環境変数ガイドライン: Rune の接続情報はすべて環境変数または `fav.toml [env]` セクション経由で渡す。
...
## 3. 実行サンドボックス（sandbox）

`fav run` はホスト OS の Rust 実行環境上で動作する。
sandbox 境界: `!Io` / `!Http` 等のエフェクト宣言なしでは I/O は発生しない。
...
## 4. OSS ライセンス

依存ライブラリ一覧は [oss-licenses](./oss-licenses) を参照。
...
```

**含むべきキーワード**:
- `"W021"` （アサーション対象）
- `"サンドボックス"` または `"sandbox"` （アサーション対象）

---

## site/content/docs/tools/oss-licenses.mdx 仕様

```markdown
---
title: "OSS ライセンス"
description: "Favnir が依存する OSS ライブラリのライセンス一覧"
---

# OSS ライセンス

| クレート | バージョン | ライセンス |
|---|---|---|
| serde | 1.x | MIT / Apache-2.0 |
| tokio | 1.x | MIT |
| clap | 4.x | MIT / Apache-2.0 |
| ...（主要依存 20+ 件）
```

**含むべきキーワード**:
- `"MIT"` （アサーション対象）

---

## SECURITY_MODEL.md 追記仕様

ファイル末尾に以下のセクションを追加:

```markdown
## v34.x Context 移行との関係

v34.5 以降で `!Effect` アノテーションを廃止し Capability Context（ctx パラメータ）に移行する。
ctx 移行後も公理 1〜4 は変形なく成立する:

- ctx フィールドへのアクセスが「capability を保有する」条件に相当
- ctx を持たない関数は引き続き純粋（公理 1）
- W021 は ctx ベースの実装に対しても適用可能（v34.5 で更新予定）

v34: 本審査時点では `!Effect` 構文が現役。ctx 移行完了後に本セクションを更新する。
```

**含むべきキーワード**:
- `"v34"` （アサーション対象）

---

## テスト仕様（v344000_tests）

```rust
#[cfg(test)]
mod v344000_tests {
    #[test]
    fn cargo_toml_version_is_34_4_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.4.0"), "Cargo.toml must contain '34.4.0'");
    }

    #[test]
    fn security_audit_v2_page_exists() {
        let src = include_str!("../../site/content/docs/tools/security-audit-v2.mdx");
        assert!(
            src.contains("W021"),
            "security-audit-v2.mdx must mention W021"
        );
    }

    #[test]
    fn oss_licenses_page_exists() {
        let src = include_str!("../../site/content/docs/tools/oss-licenses.mdx");
        assert!(
            src.contains("MIT"),
            "oss-licenses.mdx must mention MIT license"
        );
    }

    #[test]
    fn security_model_has_v34_section() {
        let src = include_str!("../../SECURITY_MODEL.md");
        assert!(
            src.contains("v34"),
            "SECURITY_MODEL.md must have a v34 section"
        );
    }

    #[test]
    fn security_audit_v2_covers_sandbox() {
        let src = include_str!("../../site/content/docs/tools/security-audit-v2.mdx");
        assert!(
            src.contains("sandbox") || src.contains("サンドボックス"),
            "security-audit-v2.mdx must cover sandbox / execution boundary"
        );
    }
}
```

### 設計注記

- `use super::*` なし（`include_str!` のみ使用）
- WASM ゲートなし
- v344000_tests は v343000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.4.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.4.0"`
- [ ] `cargo_toml_version_is_34_3_0` が空スタブになっていること
- [ ] `cargo test --bin fav v344000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2556 件想定 = 2551 + 5、0 failures）
- [ ] `site/content/docs/tools/security-audit-v2.mdx` が存在し `"W021"` を含むこと
- [ ] `site/content/docs/tools/security-audit-v2.mdx` が `"sandbox"` または `"サンドボックス"` を含むこと
- [ ] `site/content/docs/tools/security-audit-v2.mdx` が認証情報ガイドライン（`"環境変数"` 等）を含むこと【手動確認。自動テストなし — 4 審査項目のうち W021 と sandbox のみ機械検証し、認証情報・OSS リンクは手動確認とする】
- [ ] `site/content/docs/tools/oss-licenses.mdx` が存在し `"MIT"` を含むこと
- [ ] `SECURITY_MODEL.md` に `"v34"` 言及があること
- [ ] `CHANGELOG.md` に `[v34.4.0]` セクション
- [ ] `benchmarks/v34.4.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.4.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
