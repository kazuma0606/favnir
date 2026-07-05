# v34.5.0 — Spec

## 概要

**テーマ**: `!Effect` 廃止・コンテキスト構文統一

**方針**: `!Effect` アノテーションを非推奨化する lint ルール（W022）を追加し、
Capability Context への移行をガイドする。`fav migrate --from-effects` は v13.10.0 で実装済みのため、
本バージョンは「警告の追加」と「移行ガイドの整備」に集中する。

---

## 背景

v34.4.0（セキュリティ審査 v2）で `SECURITY_MODEL.md` に v34.x ctx 移行方針を追記した。
v34.5.0 では実際のコード変更として W022 lint ルールを追加し、
開発者が `!Effect` を使用したときに移行を促す警告を出す。

### 既存実装の確認

| 機能 | 実装バージョン | 状態 |
|---|---|---|
| `migrate_effects_in_source` 関数 | v13.10.0 | 実装済み（driver.rs）|
| `fav migrate --from-effects` コマンド | v13.10.0 | 実装済み（main.rs）|
| `AppCtx` 型 | v13.5.0 | 実装済み（driver.rs + checker）|
| `runes/ctx/mock_db.fav` | v13.2.0 | 実装済み |
| `runes/ctx/io.fav` (IoCtx interface) | 未実装 | **本バージョンで新規作成** |
| W022 `deprecated_effect_annotation` | 未実装 | **本バージョンで追加** |
| 移行ガイド MDX | 未実装 | **本バージョンで新規作成** |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| W0XX deprecated_effect_annotation lint | W022 を追加 | lint.rs に `check_w022_deprecated_effect_annotation` を追加し `lint_program()` に組み込む |
| IoCtx interface 定義 | `ctx.io` フィールド用の型 | `runes/ctx/io.fav` に `IoCtx` interface を定義 |
| AppCtx に `io: IoCtx` フィールド | v34.5 で追加 | `runes/ctx/io.fav` の `IoCtx` interface として実現。driver.rs の AppCtx 実装変更はスコープ外（`io_ctx_rune_exists` テストで存在を確認）|
| compiler.fav / checker.fav 書き換え | `!Effect` → ctx 構文 | **スコープ外** — 影響範囲が大きいため v34.6〜v34.7 で対応 |
| checker.rs ctx 型チェック優先化 | checker.rs 変更 | **スコープ外** — リスクが高いため v34.6 で対応 |

---

## 実装スコープ

### 新規ファイル

```
runes/ctx/io.fav                          IoCtx interface 定義
site/content/docs/tools/migration-effects.mdx  !Effect → ctx 移行ガイド
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.4.0` → `34.5.0`
2. `fav/src/lint.rs` — `check_w022_deprecated_effect_annotation` 追加 + `lint_program()` に組み込み
3. `fav/src/driver.rs` — `cargo_toml_version_is_34_4_0` をスタブ化、`v345000_tests` 5 件追加
4. `benchmarks/v34.5.0.json` — 新規作成
5. `CHANGELOG.md` — `[v34.5.0]` セクション先頭追記
6. `versions/current.md` — 最新安定版を v34.5.0 に更新

---

## runes/ctx/io.fav 仕様

```favnir
// runes/ctx/io.fav — IoCtx interface（v34.5.0）
// !Io エフェクトの Capability Context 移行用インターフェース。
// `!Io` を使用している関数は ctx.io を通じて IO 操作を行う。

interface IoCtx {
    fn println(ctx: IoCtx, msg: String) -> Unit
    fn read_line(ctx: IoCtx) -> Result<String, String>
    fn read_file(ctx: IoCtx, path: String) -> Result<String, String>
    fn write_file(ctx: IoCtx, path: String, content: String) -> Result<Unit, String>
    fn env(ctx: IoCtx, key: String) -> Option<String>
}
```

**含むべきキーワード**:
- `"IoCtx"` （アサーション対象）

---

## W022: `deprecated_effect_annotation` 仕様

### 検出条件

`!Effect` アノテーション（`Pure` 以外）を持つ `fn` 定義を検出する。

**警告メッセージ**:
```
W022: function `name` uses deprecated `!Effect` annotation — migrate to Capability Context using `fav migrate --from-effects`
```

### 実装方針

```rust
pub fn check_w022_deprecated_effect_annotation(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let has_real_effect = fd.effects.iter().any(|e| e != &Effect::Pure);
            if has_real_effect {
                errors.push(LintError::new(
                    "W022",
                    format!(
                        "function `{}` uses deprecated `!Effect` annotation \
                         — migrate to Capability Context using `fav migrate --from-effects`",
                        fd.name
                    ),
                    fd.span.clone(),
                ));
            }
        }
    }
}
```

### 抑制方法

```bash
fav lint --allow W022 src/pipeline.fav        # 単一ファイル
fav lint --allow W022 --dir src/              # ディレクトリ全体
```

セルフホスト済みファイル（compiler.fav / checker.fav）は W022 を suppress する慣習を適用。

### 既存テストへの影響

W022 を `lint_program()` に追加すると、`!Effect` を使用した **既存の lint テスト** の
`warnings.len()` カウントが増加する可能性がある。
**実装前に `grep -n "fav_lint\|lint_program" fav/src/driver.rs | grep -v "//"`で
影響する既存テストを洗い出し、修正またはソースを純粋関数に変更すること**。

---

## site/content/docs/tools/migration-effects.mdx 仕様

タイトル: `!Effect から Capability Context への移行ガイド`

含むべき内容:
- W022 警告の説明
- `fav migrate --from-effects` コマンドの使い方
- Before/After コード例（`!Http` → `ctx: AppCtx`）
- `AppCtx` / `IoCtx` 各フィールドの対応表

**含むべきキーワード**:
- `"W022"` （アサーション対象）
- `"AppCtx"` または `"ctx"` （アサーション対象）

---

## テスト仕様（v345000_tests）

```rust
#[cfg(test)]
mod v345000_tests {
    #[test]
    fn cargo_toml_version_is_34_5_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.5.0"), "Cargo.toml must contain '34.5.0'");
    }

    #[test]
    fn w022_deprecated_effect_annotation_fires() {
        let src = "fn fetch(url: String) -> String !Http { url }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        let mut warnings = Vec::new();
        crate::lint::check_w022_deprecated_effect_annotation(&prog, &mut warnings);
        assert!(
            warnings.iter().any(|w| w.code == "W022"),
            "W022 must fire when !Http is used: {:?}", warnings
        );
    }

    #[test]
    fn io_ctx_rune_exists() {
        let src = include_str!("../../runes/ctx/io.fav");
        assert!(
            src.contains("IoCtx"),
            "runes/ctx/io.fav must define IoCtx interface"
        );
    }

    #[test]
    fn migration_guide_page_exists() {
        let src = include_str!("../../site/content/docs/tools/migration-effects.mdx");
        assert!(
            src.contains("W022"),
            "migration-effects.mdx must mention W022"
        );
    }

    #[test]
    fn migration_guide_covers_ctx_syntax() {
        let src = include_str!("../../site/content/docs/tools/migration-effects.mdx");
        assert!(
            src.contains("AppCtx") || src.contains("ctx"),
            "migration-effects.mdx must cover ctx-based syntax"
        );
    }
}
```

### 設計注記

- `w022_deprecated_effect_annotation_fires` は `crate::frontend::lexer::Lexer` / `crate::frontend::parser::Parser` / `crate::lint::check_w022_deprecated_effect_annotation` を絶対パスで呼び出す（v246000_tests の W021 テストと同一パターン）
- **`use super::*` は不要**（絶対 `crate::` パスを使用）
- WASM ゲートなし
- v345000_tests は v344000_tests 直後・`// ── v31.7.0 tests` の前に挿入

### 既存 lint テストへの W022 影響について

W022 を `lint_program()` に追加すると、`lint_program()` を呼び出すテストで !Effect fn を含む
ソースを渡している場合に W022 が追加発火する。ただし:
- W021 テスト（v246000_tests）は `check_w021_pure_fn_calls_effectful` を **直接呼び出す** ため影響なし
- 影響を受けるのは `lint_program()` 経由で全ルールを実行しているテストのみ
- **実装前に `grep -n "lint_program" fav/src/driver.rs | grep -v "//"` で全箇所を確認し、
  `fav_lint(src, &["W022"])` 相当の抑制または純粋関数ソースへの変更を行うこと**

---

## 完了条件

- [ ] `cargo clean` 不要（x.5.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.5.0"`
- [ ] `cargo_toml_version_is_34_4_0` が空スタブになっていること
- [ ] `cargo test --bin fav v345000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2561 件想定 = 2556 + 5、0 failures）
- [ ] `lint.rs` に `check_w022_deprecated_effect_annotation` が追加され `lint_program()` に組み込まれていること
- [ ] `runes/ctx/io.fav` が存在し `"IoCtx"` を含むこと
- [ ] `site/content/docs/tools/migration-effects.mdx` が存在し `"W022"` を含むこと
- [ ] `CHANGELOG.md` に `[v34.5.0]` セクション
- [ ] `benchmarks/v34.5.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.5.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
