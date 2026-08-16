# v72.2.0 spec — AI エラーアシスタント（`fav ai explain` / `fav ai fix`）

Date: 2026-08-12

---

## Background

Favnir のコンパイルエラーは E0374・E0001 等のエラーコードで分類されているが、
初見のユーザーにとってエラーメッセージだけでは修正方法が分かりにくい。
v72.2.0 では、エラーコードと該当ソースコードを元に自然言語の説明と修正案を
提供する AI エラーアシスタント機能を実装する。

**設計方針**:
- コア実装はローカル静的ヒントマップ（`get_ai_hint`）— API 不要・テスト可能
- オプションで Claude API に送信して詳細説明を取得（`--ai-explain` フラグ）
- `fav ai fix` は `apply_ctx_migration` によるテキスト変換を適用

---

## Goals

1. `cmd_ai_explain(path, error_code)` — エラーコード + ソースを基に説明を生成・表示（v72.2.0 は静的ヒントマップのみ。Claude API 統合は v72.3.0 以降）
2. `cmd_ai_fix(path)` — `!IO` 構文 → `ctx: AppCtx` 変換を適用（diff プレビュー付き）
3. `fav check --ai-explain` フラグとの統合
4. `fav ai explain <path>` / `fav ai fix <path>` CLI サブコマンドを追加
5. `v722000_tests` 2 件を `driver.rs` に追加
6. `CHANGELOG.md` に v72.2.0 エントリを追加
7. `versions/current.md` を更新（進行中: v72.2.0、次: v72.3.0）

---

## CLI 使用例

```bash
# fav check に --ai-explain フラグを追加
$ fav check pipeline.fav --ai-explain
E0374 detected at line 43.

[AI Explanation]
このエラーは `!IO` というエフェクトアノテーション構文が使われているために
発生しています。v35.4.0 でこの構文は廃止され、代わりに `ctx: AppCtx` を
関数の第1引数として渡す方式に変わりました。

[Suggested Fix]
Before: fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO
After:  fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String>

さらに `IO.write_file(...)` → `ctx.io.write_file_raw(...)` への変更も必要です。

Apply this fix? [y/N]:   ← 確認プロンプト・diff プレビューは v72.2.0 スコープ外（v72.3.0 以降）

# または自動修正のみ（v72.2.0: 確認プロンプトなし即書き込み）
$ fav ai fix pipeline.fav
```

---

## 実装詳細

### `get_ai_hint(error_code: &str) -> Option<&'static str>`

既知エラーコードの静的ヒントマップ:

```rust
pub fn get_ai_hint(error_code: &str) -> Option<&'static str> {
    match error_code {
        "E0374" => Some(
            "!IO エフェクト構文は廃止されました。\
             `ctx: AppCtx` を第1引数として渡す方式に変更してください。\
             例: `fn f() -> T !IO` → `fn f(ctx: AppCtx) -> T`"
        ),
        "E0001" => Some(
            "未定義の変数が参照されています。\
             スペルミスまたはスコープ外の変数ではないか確認してください。"
        ),
        _ => None,
    }
}
```

### `apply_ctx_migration(src: &str) -> String`

`!IO` 構文を `ctx: AppCtx` パターンに変換する:

```rust
pub fn apply_ctx_migration(src: &str) -> String {
    src.replace("!IO", "/* ctx: AppCtx */")
       .replace("IO.println(", "ctx.io.println(")
       .replace("IO.write_file(", "ctx.io.write_file_raw(")
       .replace("IO.read_file(", "ctx.io.read_file_raw(")
}
```

### `cmd_ai_explain(path: &str, error_code: &str)`

```rust
pub fn cmd_ai_explain(path: &str, error_code: &str) {
    let hint = get_ai_hint(error_code)
        .unwrap_or("このエラーコードのヒントはまだ登録されていません。");
    println!("[AI Explanation]\n{hint}");
}
```

### `cmd_ai_fix(path: &str)`

```rust
pub fn cmd_ai_fix(path: &str) -> Result<(), String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {path}: {e}"))?;
    let fixed = apply_ctx_migration(&src);
    if fixed == src {
        println!("No automatic fixes available for {path}.");
        return Ok(());
    }
    std::fs::write(path, &fixed)
        .map_err(|e| format!("cannot write {path}: {e}"))?;
    println!("Applied ctx migration to {path}.");
    Ok(())
}
```

---

## テスト詳細

```rust
// v722000_tests — AI アシスタント静的ヒント + 変換ロジック確認

fn ai_explain_e0374_returns_hint() {
    let hint = get_ai_hint("E0374").expect("E0374 should have a hint");
    assert!(
        hint.contains("ctx: AppCtx") || hint.contains("!IO"),
        "E0374 hint should mention ctx:AppCtx or !IO migration"
    );
}

fn ai_fix_applies_ctx_migration() {
    let src = "fn main() -> Unit !IO { IO.println(\"hello\") }";
    let fixed = apply_ctx_migration(src);
    assert!(
        fixed.contains("ctx.io.println("),
        "apply_ctx_migration should replace IO.println with ctx.io.println"
    );
}
```

---

## Success Criteria

- `cargo test v722000` で 2 件 pass（0 failures）
  - `ai_explain_e0374_returns_hint` pass
  - `ai_fix_applies_ctx_migration` pass
- `cargo test` 全体で 3616 tests pass（3614 + 2）
- `fav/Cargo.toml` のバージョンが `72.2.0`
- `cargo build` が通ること（`fav ai explain` / `fav ai fix` / `fav check --ai-explain` がビルドに含まれる）
- `get_ai_hint("E0374")` が `Some(...)` を返すこと
- `apply_ctx_migration` が `IO.println(` → `ctx.io.println(` に変換すること

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `get_ai_hint` / `apply_ctx_migration` / `cmd_ai_explain` / `cmd_ai_fix` 追加 + `v722000_tests` モジュール（2 テスト）+ version アサーション更新 |
| `fav/src/main.rs` | `fav ai explain <path>` / `fav ai fix <path>` サブコマンド追加、`fav check --ai-explain` フラグ追加 |
| `fav/Cargo.toml` | version `72.1.0` → `72.2.0` |
| `CHANGELOG.md` | `## [v72.2.0]` エントリ追加 |
| `versions/current.md` | 進行中: v72.2.0、次: v72.3.0 |

---

## スコープ外

- 実際の Claude API 呼び出し（HTTP リクエスト）: 別タスク（v72.3.0 以降の拡張）
- `cmd_ai_fix` の diff プレビュー・対話的確認プロンプト: 別タスク（v72.3.0 以降）
- E0374 以外の全エラーコードのヒント登録: 段階的に追加
- `fav check --ai-explain` の完全統合（エラーコード自動抽出）: 別タスク（v72.3.0 以降）
- `site/content/docs/cli/ai.mdx` 追加: v72.3.0 以降
- `site/` MDX 更新: 別タスク
