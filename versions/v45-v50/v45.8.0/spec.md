# Spec: v45.8.0 — examples 更新 Phase 1

Date: 2026-07-16
Status: TODO

---

## 概要

`examples/` 以下のサンプルを最新構文に統一する第一フェーズ。
旧 `!Effect` 記法の完全除去を確認し、`examples_no_legacy_effect_syntax` テストを追加する。

---

## 調査結果（実装前に確認済み）

### `bind` / `ctx` 構文への統一状況

ロードマップに「`bind` / `ctx` 構文への統一」が記載されているが、
`examples/pipeline/pipeline.fav` をはじめ主要 examples はすでに `bind` / `ctx` 構文を使用している。
調査の結果、非レガシーファイルへの移行は完了済みのため**本バージョンでの変更は不要**。
（`import rune "..."` を使う 4 ファイルは W035 予告対象として将来バージョンで対応予定）

### `!Effect` 記法を含むファイル（3件）

| ファイル | 状態 | 対応 |
|---|---|---|
| `examples/async/async_main_demo.fav` | コメント内参照のみ、実コードは既にクリーン | 変更不要 |
| `examples/pipeline/custom_effects.fav` | 意図的レガシーデモ（NOTE + `--legacy` 明記） | 変更しない（レガシーデモとして保持） |
| `examples/pipeline/effect_errors.fav` | エラーケース説明用（意図的に旧構文を示す） | 変更しない（エラー説明ファイルとして保持） |

→ 非レガシーファイルにはすでに `!Effect` アノテーション構文が存在しない。

### `import rune "..."` を含むファイル（4件）

`auth_demo/src/main.fav` / `env_demo/src/main.fav` / `gen2_demo/src/main.fav` / `log_demo/src/main.fav`

→ W035 警告の対象として**将来バージョンで予告**（本バージョンのスコープ外）。

---

## 変更対象

### §1 — `examples/pipeline/pipeline.fav` に `return` ガード節パターン追加

`return` 構文（v45.1〜v45.3 で実装）を活用したサンプルとして、
`examples/pipeline/pipeline.fav` に `return` ガード節を使う関数を追加する。

追加する関数例:
```favnir
// return guard pattern example (v45.8.0)
fn validate_amount(amount: Float) -> Result<Float, String> {
    if amount <= 0.0 { return Err("amount must be positive") }
    if amount > 1_000_000.0 { return Err("amount exceeds maximum") }
    Ok(amount)
}
```

数値リテラル `_`（v45.7.0）も合わせて活用する。

### §2 — `driver.rs`: `examples_no_legacy_effect_syntax` テスト追加

`v458000_tests` モジュールを追加（1件）。

テストの目的: 非レガシー examples ファイルに `!Effect` アノテーション構文（`-> Type !Effect`）が含まれないことを確認する。

**実装方針**:
`walkdir` を使って `examples/` 以下の `.fav` ファイルを再帰スキャンし、
レガシーファイルをスキップリストで除外した後、各ファイルの内容に
`-> \w+ !\w+` パターン（戻り値型の後に `!Effect`）が含まれないことをチェックする。

レガシースキップリスト（意図的に旧構文を含むファイル）:
- `pipeline/custom_effects.fav`
- `pipeline/effect_errors.fav`

```rust
#[cfg(test)]
mod v458000_tests {
    use walkdir::WalkDir;
    use std::path::Path;

    #[test]
    fn examples_no_legacy_effect_syntax() {
        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let skip_list = &[
            "pipeline/custom_effects.fav",
            "pipeline/effect_errors.fav",
        ];

        let mut violations: Vec<String> = Vec::new();

        for entry in WalkDir::new(&examples_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "fav").unwrap_or(false))
        {
            let rel = entry.path()
                .strip_prefix(&examples_dir)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            if skip_list.iter().any(|s| rel.ends_with(s)) {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            // detect "-> RetType !EffectName" annotation in function signatures
            if regex_like_match(&content) {
                violations.push(rel.to_string());
            }
        }

        assert!(
            violations.is_empty(),
            "legacy !Effect annotation found in examples: {:?}",
            violations
        );
    }

    /// Simple check: a non-comment line contains "!<UpperIdent>" after "->"
    fn regex_like_match(content: &str) -> bool {
        content.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .any(|l| {
                if let Some(pos) = l.find("->") {
                    let after = &l[pos..];
                    // pattern: -> ... !UpperCaseName (effect annotation)
                    after.contains('!') && after
                        .chars()
                        .skip_while(|&c| c != '!')
                        .nth(1)
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                } else {
                    false
                }
            })
    }
}
```

`regex` crate はすでに `Cargo.toml` に存在するが、シンプルな文字列検索で代替する。

**`walkdir` の依存関係**:
`walkdir = "2"` は `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に登録済みだが
`[dev-dependencies]` には未登録。テストモジュールに `#[cfg(not(target_arch = "wasm32"))]`
を付与することで WASM ビルドとの衝突を避ける。native ターゲットのテストでは正常に使用できる。

**`regex_like_match` の誤検知対策**:
インラインコメント（`//` 以降）を除去してから `!` をスキャンすることで、
`fn f() -> String // !Effect legacy` 等の誤検知を防ぐ。
文字列リテラル内の `!` については現在の examples では発生しないため対象外とする。

改善版の `regex_like_match`:
```rust
fn regex_like_match(content: &str) -> bool {
    content.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| {
            // strip inline comment before scanning
            let code = if let Some(i) = l.find("//") { &l[..i] } else { l };
            if let Some(pos) = code.find("->") {
                let after = &code[pos..];
                after.contains('!') && after
                    .chars()
                    .skip_while(|&c| c != '!')
                    .nth(1)
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            } else {
                false
            }
        })
}
```

**スキップリストの照合**:
`ends_with` で照合するため、将来的な同名ファイル作成時の誤スキップを防ぐため
`rel == s || rel.ends_with(&format!("/{}", s))` パターンを使う。

**W017 lint への対応**:
`validate_amount` を `main` から呼び出すサンプルに含めて未使用関数の lint を回避する。

---

## 変更しないファイル

- `ast.rs` / `checker.rs` / `compiler.rs` / `vm.rs`
- `error_catalog.rs`
- `lexer.rs` / `parser.rs`
- `site/`（ドキュメント更新は v45.9.0 以降）
- レガシーデモファイル（`custom_effects.fav` / `effect_errors.fav`）

---

## 完了条件

- `cargo test` 全通過（2986 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v45.8.0 エントリ追加
- `versions/current.md` を v45.8.0（2986 tests）に更新
- `fav/Cargo.toml` version → `45.8.0`
