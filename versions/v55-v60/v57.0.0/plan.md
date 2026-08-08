# Plan — v57.0.0 — Language Power 2.0 宣言 ★クリーンアップ

## 実装順序

```
Cargo.toml → MILESTONE.md → README.md → CHANGELOG.md
→ driver.rs（テスト追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（current.md + 両ロードマップ更新）→ cargo clean（★クリーンアップ）
```

依存関係:
- `MILESTONE.md` / `README.md` / `CHANGELOG.md` の更新は互いに独立（並行可）
- `driver.rs` の `include_str!` テストは各ドキュメントの更新後でないとビルドエラーになる
  （`changelog_has_v57_0_0` は CHANGELOG の更新後、`milestone_has_language_power2` は MILESTONE の更新後）
- `cargo clean` は `cargo test` 全通過確認の**後**

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.0.0"
```

---

## Step 2: `MILESTONE.md` — Language Power 2.0 エントリ追加

`# Favnir Milestones` の直後（`## v56.0.0` エントリの前）に以下を挿入する。

```markdown
## v57.0.0（2026-07-26）— Language Power 2.0

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

**Language Power 2.0** の宣言バージョン。v56.1〜v56.9 の全機能統合を経て、
境界付きジェネリクス・行多相レコード・エフェクト推論 LSP・OR パターン・
as-パターン・モジュール名前空間の成熟を宣言する。

**v56.1〜v56.9 達成内容:**
- v56.1（境界付きジェネリクス本番品質化）: `where T: Interface` 正式化・E0422
- v56.2（複数 constraint・coherence 強化）: `T with Ord with Serialize`・E0423
- v56.3（行多相レコード活用拡張）: `{ field: Type | r }` 行変数明示・LSP ホバー
- v56.4（エフェクト推論 LSP 統合）: inlay hints・`fav check --show-types`
- v56.5（OR パターン + パターンガード強化）: `Ok(x) | Err(x)`・W037
- v56.6（パターンエイリアス）: `head @ { id, amount }` as-パターン
- v56.7（モジュール名前空間）: `import "path" as alias.*`・W038
- v56.8（ドキュメント）: bounded-generics / row-polymorphism / effect-inference MDX
- v56.9（安定化）: language-power2-overview.mdx 骨子・コードフリーズ

---
```

---

## Step 3: `README.md` — Language Power 2.0 宣言の追記

v56.0 宣言エントリを探してその直後に追記する。

```markdown
**v57.0（2026-07-26）で、[Language Power 2.0](./MILESTONE.md) マイルストーンを宣言しました。**
`where T: Interface` 本番品質化・行変数 `{ field: Type | r }` 明示・エフェクト推論 inlay hints・OR パターン・as-パターン・モジュール名前空間（`import "path" as alias.*`）が揃い、Favnir の型システムで開発者の意図を正確に表現できる状態になりました。
```

v56.0 の宣言エントリは README.md 内の `**v56.0（` で検索して見つける。

---

## Step 4: `CHANGELOG.md` — v57.0.0 エントリ追加

先頭（`## [v56.9.0]` の前）に追加する。

```markdown
## [v57.0.0] — 2026-07-26 — Language Power 2.0 宣言

### Changed
- `Cargo.toml`: version → `57.0.0`

### Added
- `MILESTONE.md`: Language Power 2.0 宣言エントリ追加
- `README.md`: Language Power 2.0 マイルストーン宣言リンク追加
- `v57000_tests` 追加（4 件）— 3252 tests
  - `cargo_toml_version_is_57_0_0`
  - `changelog_has_v57_0_0`
  - `milestone_has_language_power2`
  - `readme_mentions_language_power2`

---
```

---

## Step 5: `fav/src/driver.rs` — `v57000_tests` 追加

`v56900_tests` モジュールの直前に挿入する。

```rust
// -- v57000_tests (v57.0.0) -- Language Power 2.0 宣言 --
#[cfg(test)]
mod v57000_tests {
    #[test]
    fn cargo_toml_version_is_57_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"57.0.0\""),
            "Cargo.toml version should be 57.0.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn changelog_has_v57_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("[v57.0.0]"),
            "CHANGELOG.md should have a [v57.0.0] entry"
        );
    }

    #[test]
    fn milestone_has_language_power2() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Language Power 2.0"),
            "MILESTONE.md should declare Language Power 2.0"
        );
    }

    #[test]
    fn readme_mentions_language_power2() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Language Power 2.0"),
            "README.md should mention Language Power 2.0"
        );
    }
}
```

---

## Step 6: `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を更新:

```rust
// 変更前
cargo_toml.contains("version = \"56.9.0\"")
"Cargo.toml version should be 56.9.0, got: {}"

// 変更後
cargo_toml.contains("version = \"57.0.0\"")
"Cargo.toml version should be 57.0.0, got: {}"
```

---

## Step 7: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3252 tests passed, 0 failed を確認する。`v57000_tests` の 4 件が全通過することを確認する。

---

## Step 8: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 9: ポスト処理

`cargo test` / `cargo clippy` 全通過後に実行する。

1. `versions/current.md` を v57.0.0 / 3252 tests に更新
2. `versions/roadmap/roadmap-v56.1-v57.0.md` の v57.0.0 実績を COMPLETE に更新（`3248 + 4 = 3252 tests passed, 0 failed（2026-07-26）` を追記）
3. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.0.0 実績欄も COMPLETE に更新（`~3250` → `3252` に精緻化）

---

## Step 10: ★クリーンアップ（`cargo clean`）

ポスト処理完了後に実行する。

```bash
cargo clean
```

次スプリント（v57.1〜v58.0）はクリーンなビルド状態から開始する。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `MILESTONE.md` に `"Language Power 2.0"` が既存（v33.0 の Language Power と誤検知） | T0 で `"Language Power 2.0"` の完全一致文字列が存在しないことを確認（`"Language Power"` はあるが `"2.0"` はない） |
| `README.md` に `"Language Power 2.0"` が既存（同上） | T0 で確認 |
| `include_str!` で CHANGELOG/MILESTONE/README は `../../` 経由 | `fav/src/driver.rs` → `fav/` → プロジェクトルート の 2 段上がりで正しい |
| `changelog_has_v57_0_0` が CHANGELOG 更新前にコンパイルされてビルドエラー | CHANGELOG を先に更新してから driver.rs のテストを追加する（Step 4 → Step 5 の順序） |
| `cargo clean` 後の再ビルドで問題が発覚する | `cargo clean` 前に `cargo test` / `cargo clippy` を完全通過させる |
