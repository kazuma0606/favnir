# v18.0.0 — Language Power マイルストーン宣言 タスク

## ステータス: 完了

---

## タスク一覧

### T1: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `17.8.0` → `18.0.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T2: CHANGELOG.md 更新

- [x] `CHANGELOG.md`（リポジトリルート）に v18.0.0 エントリを先頭に追加
- [x] v17.1.0〜v17.8.0 の全エントリを追加（各バージョンの概要を 1〜3 行で）

追加するエントリ形式：
```markdown
## v18.0.0 — Language Power（2026-06-16）

- 境界付きジェネリクス・パターンマッチ拡張・内包表記・プロパティテスト・パッケージシステムの完成を宣言

## v17.8.0 — パッケージシステム成熟（2026-06-16）

- `fav add` / `fav update` / `fav remove` / `fav login` / `fav publish` CLI 追加
- `fav.toml` に `[dev-dependencies]` / `[registry]` セクション追加
- `fav.lock` に `checksum` / `source` フィールド追加
- `registry/resolver.rs`（SemVer/VersionReq/resolve_best）/ `registry/client.rs`（RegistryClient）追加

## v17.7.0 — `forall` プロパティベーステスト（2026-06-15）

- `forall x: Type [where { guard }] { body }` 構文追加
- `__forall_gen_int/str/bool/float` VM primitive（xorshift64 固定シード）
- `--cases N` CLI オプション

## v17.6.0 — `fav bench` 統計強化（2026-06-15）

- `bench "name" { ... }` 構文追加
- avg / p50 / p95 / min / max 統計出力
- `--runs` / `--warmup` / `--json` オプション

## v17.5.0 — REPL 品質向上（2026-06-15）

- `:doc` / `:load` / `:save` / `:history` / `:paste` コマンド追加
- タブ補完（モジュール名・関数名・`:` コマンド）

## v17.4.0 — `let` バインディング除去（2026-06-15）

- 誤実装の `let` キーワードを除去。`bind x <- expr` に統一

## v17.3.0 — コレクション内包表記（2026-06-15）

- `[x * 2 | x <- nums]` list-comp、`[? f(x) | x <- xs]` result-comp 追加
- `List.collect_result` builtin 追加

## v17.2.0 — パターンマッチ拡張（2026-06-15）

- or-pattern（`"a" | "b" => ...`）/ list-pattern（`[head, ..tail]`）/ guard（`if cond`）追加
- `DotDot` トークン、`ListLen` / `ListGet` / `ListDrop` VM opcodes 追加

## v17.1.0 — 境界付きジェネリクス（2026-06-15）

- `fn f<T with Ord>(a: T, b: T) -> T` 構文追加
- 組み込み bounds: Ord / Eq / Serialize / Display / Hash / Clone
- E0325: bound 不満足エラー
```

### T3: README.md 更新

- [x] README.md の「現在のバージョン」を `v18.0.0 — Language Power` に更新
- [x] 主要機能リストに以下を追加：
  - `Bounded Generics` (`fn f<T with Ord>(...)`)
  - `Pattern matching` (or-pattern, list-pattern, guard)
  - `Collection comprehensions` (`[x * 2 | x <- list]`)
  - `Property-based testing` (`forall`)
  - `Package system` (`fav add`, `fav publish`)
- [x] バージョン履歴表に v17.1.0〜v18.0.0 のエントリを追加

### T4: `site/content/docs/language/patterns.mdx` 作成

- [x] or-pattern 構文と例を記載
- [x] list-pattern（`[]` / `[x]` / `[head, ..tail]`）を記載
- [x] guard 条件（`if expr`）を記載
- [x] 実用例（パイプラインのステータス分岐）を記載

### T5: `site/content/docs/language/comprehensions.mdx` 作成

- [x] 基本 map 構文 `[expr | x <- src]` を記載
- [x] フィルタ付き `[expr | x <- src, guard]` を記載
- [x] 複数ソース直積 `[Pair(a,b) | a <- as, b <- bs]` を記載
- [x] Result 内包 `[? f(x) | x <- xs]` を記載
- [x] `List.map + List.filter` との Before/After 比較を記載

### T6: `site/content/docs/language/bind.mdx` 作成

- [x] `bind x <- expr` の基本構文を記載
- [x] Result 値・非 Result 値の両方で使えることを説明
- [x] `let` キーワードが存在しない理由を説明
- [x] パイプライン内での活用例を記載

### T7: `site/content/docs/packages/publishing.mdx` 作成

- [x] `fav.toml` の `[rune]` セクション（name / version / description）を記載
- [x] `fav publish --dry-run` で確認する手順を記載
- [x] `fav login` の認証フローを記載
- [x] `fav publish` で公開する手順を記載

### T8: `driver.rs` — `v180000_tests` 追加

- [x] `v178000_tests` の `version_is_17_8_0` テストを削除
- [x] `v180000_tests` モジュールを追加（5件）

```rust
// ── v180000_tests (v18.0.0) — Language Power マイルストーン ─────────────────
#[cfg(test)]
mod v180000_tests {
    #[test]
    fn version_is_18_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"18.0.0\""), "Cargo.toml should have version 18.0.0");
    }

    #[test]
    fn changelog_has_v17_entries() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(changelog.contains("v17."), "CHANGELOG.md should contain v17.x entries");
    }

    #[test]
    fn readme_mentions_bounded_generics() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.to_lowercase().contains("bounded generics"),
            "README.md should mention bounded generics"
        );
    }

    #[test]
    fn readme_mentions_package_system() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("fav add") || readme.to_lowercase().contains("package system"),
            "README.md should mention the package system"
        );
    }

    #[test]
    fn docs_generics_exists() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("site/content/docs/language/generics.mdx");
        assert!(path.exists(), "site/content/docs/language/generics.mdx should exist");
    }
}
```

---

## テスト（v180000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_0_0` | Cargo.toml に "18.0.0" が含まれる |
| `changelog_has_v17_entries` | CHANGELOG.md に "v17." が含まれる |
| `readme_mentions_bounded_generics` | README.md に "bounded generics" が含まれる |
| `readme_mentions_package_system` | README.md に "fav add" または "package system" が含まれる |
| `docs_generics_exists` | `site/content/docs/language/generics.mdx` が存在する |

---

## 完了条件チェックリスト

- [x] `fav/Cargo.toml` のバージョンが `18.0.0`
- [x] `CHANGELOG.md` に v17.1.0〜v18.0.0 の全エントリが存在する
- [x] `README.md` に `v18.0.0`・`bounded generics`・`fav add` の記載がある
- [x] `site/content/docs/language/patterns.mdx` が存在する
- [x] `site/content/docs/language/comprehensions.mdx` が存在する
- [x] `site/content/docs/language/bind.mdx` が存在する
- [x] `site/content/docs/packages/publishing.mdx` が存在する
- [x] `cargo test v180000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（Cargo.toml）
T2（CHANGELOG.md）    ← T1 と並列可
T3（README.md）       ← T1/T2 と並列可
T4（patterns.mdx）    ← T1〜T3 と並列可
T5（comprehensions.mdx）← 並列可
T6（bind.mdx）        ← 並列可
T7（publishing.mdx）  ← 並列可
→ T8（v180000_tests） ← T1〜T7 すべて完了後（include_str! がファイルを参照するため）

T1〜T7 はすべて並列実施可能。T8 のみ最後に実施。

---

## 補足

- `site/content/docs/language/generics.mdx` は v17.1.0 で作成済み（`docs_generics_exists` テストが確認）
- `site/content/docs/tools/property-testing.mdx` は v17.7.0 で作成済み
- `site/content/docs/packages/getting-started.mdx` は v17.8.0 で作成済み
- v18.0.0 はこれらを整合させてマイルストーン宣言するだけ
