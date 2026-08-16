# v72.2.0 実装計画 — AI エラーアシスタント（`fav ai explain` / `fav ai fix`）

Date: 2026-08-12

---

## 依存関係

```
Step 1（driver.rs コア関数追加）
  └→ Step 2（main.rs CLI サブコマンド追加）
       └→ Step 3（v722000_tests 追加）
            └→ Step 4（Cargo.toml バージョン更新）
                 └→ Step 5（cargo test v722000 確認）
                      └→ Step 6（cargo test 全体確認）
                           └→ Step 7（CHANGELOG.md 更新）
                                └→ Step 8（versions/current.md 更新）
```

---

## 実装ステップ

### Step 1: `driver.rs` — コア関数追加

既存の `pub fn cmd_*` 関数群の末尾付近（v72000_tests の前）に追加する。

```rust
// ── v72.2.0: AI エラーアシスタント ──────────────────────────────────────────

/// 既知エラーコードの静的ヒントを返す。API 不要・テスト可能。
pub fn get_ai_hint(error_code: &str) -> Option<&'static str> {
    match error_code {
        "E0374" => Some(
            "!IO エフェクト構文は廃止されました。\
             `ctx: AppCtx` を第1引数として渡す方式に変更してください。\
             例: `fn f() -> T !IO` → `fn f(ctx: AppCtx) -> T`",
        ),
        "E0001" => Some(
            "未定義の変数が参照されています。\
             スペルミスまたはスコープ外の変数ではないか確認してください。",
        ),
        _ => None,
    }
}

/// `!IO` 構文を `ctx: AppCtx` パターンに変換するテキスト変換。
pub fn apply_ctx_migration(src: &str) -> String {
    src.replace("!IO", "/* ctx: AppCtx */")
        .replace("IO.println(", "ctx.io.println(")
        .replace("IO.write_file(", "ctx.io.write_file_raw(")
        .replace("IO.read_file(", "ctx.io.read_file_raw(")
}

/// エラーコードの AI ヒントを表示する。
pub fn cmd_ai_explain(path: &str, error_code: &str) {
    let hint = get_ai_hint(error_code)
        .unwrap_or("このエラーコードのヒントはまだ登録されていません。");
    println!("[AI Explanation for {error_code} in {path}]\n{hint}");
}

/// `!IO` 構文を自動修正してファイルに書き戻す。
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

### Step 2: `main.rs` — CLI サブコマンド追加

`fav ai` サブコマンドと `fav check --ai-explain` フラグを追加する。

既存の `match args` ブロックに `"ai"` アームを追加:

```rust
// fav ai explain <path> [--error-code <code>]
// fav ai fix <path>
["ai", "explain", path] => {
    crate::driver::cmd_ai_explain(path, "E0374");
}
["ai", "explain", path, "--error-code", code] => {
    crate::driver::cmd_ai_explain(path, code);
}
["ai", "fix", path] => {
    if let Err(e) = crate::driver::cmd_ai_fix(path) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

`fav check` のハンドラに `--ai-explain` フラグ対応を追加:

```rust
["check", path, "--ai-explain"] => {
    // 通常の check を実行し、エラーがあれば AI ヒントも表示
    let result = crate::driver::cmd_check(path);
    // ...エラーコード抽出後に cmd_ai_explain を呼ぶ
}
```

> **注意**: `main.rs` の既存の引数パースロジックを確認してから追加すること。
> `args` のパターンマッチ形式が異なる場合は既存パターンに合わせる。

### Step 3: `v722000_tests` 追加（`driver.rs`）

`v721000_tests` モジュールの直後に追加:

```rust
// ── v72.2.0: AI エラーアシスタント ──────────────────────────────────────────
#[cfg(test)]
mod v722000_tests {
    use super::{apply_ctx_migration, get_ai_hint};

    #[test]
    fn ai_explain_e0374_returns_hint() {
        let hint = get_ai_hint("E0374").expect("E0374 should have a hint");
        assert!(
            hint.contains("ctx: AppCtx") || hint.contains("!IO"),
            "E0374 hint should mention ctx:AppCtx or !IO migration"
        );
    }

    #[test]
    fn ai_fix_applies_ctx_migration() {
        let src = "fn main() -> Unit !IO { IO.println(\"hello\") }";
        let fixed = apply_ctx_migration(src);
        assert!(
            fixed.contains("ctx.io.println("),
            "apply_ctx_migration should replace IO.println with ctx.io.println"
        );
    }
}
```

> **注意**:
> - `#[cfg(test)]` を付けること（v721000_tests と同パターン: `// コメント\n#[cfg(test)]\nmod v722000_tests {`）。
> - `get_ai_hint` と `apply_ctx_migration` は driver.rs トップレベルに `pub fn` として定義するため、`use super::{apply_ctx_migration, get_ai_hint}` で直接アクセス可能（既存の `use super::{builtin_primitives, ...}` パターンと同じ）。

### Step 4: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- `Cargo.toml`: `72.1.0` → `72.2.0`
- `driver.rs` 内の `version = \"72.1.0\"` 文字列を `version = \"72.2.0\"` に replace_all
- エラーメッセージ内の `72.1.0` 参照も同様に更新（`Cargo.toml version should be 72.1.0` → `72.2.0`）

### Step 5: `cargo test v722000` — 2 件 pass 確認

### Step 6: `cargo test` 全体 — 3616 tests pass 確認

### Step 7: `CHANGELOG.md` に v72.2.0 エントリ追加

先頭に `## [v72.2.0]` エントリを追加。

### Step 8: `versions/current.md` 更新

- 進行中: v72.2.0（AI エラーアシスタント）
- 次: v72.3.0

---

## 注意事項

- `get_ai_hint` / `apply_ctx_migration` は `pub fn` として実装する（テストで `use super::` でアクセスするため）
- `cmd_ai_fix` は実ファイルを書き換えるため、テストでは呼ばない（`apply_ctx_migration` の単体テストのみ）
- `main.rs` の args パターンは既存実装に合わせる（スライスパターンか Vec か確認必要）
- replace_all でエラーメッセージも含めて `72.1.0` → `72.2.0` に更新する
