# v45.0.0 Plan — Precision & Flow 宣言 ★クリーンアップ

## 前提

- 現行バージョン: `44.9.0`（2962 tests）
- 追加テスト数: 4 件
- 目標テスト数: 2966
- スタブ化対象: `v44900_tests::cargo_toml_version_is_44_9_0`

---

## ステップ

### Step 0: 事前確認（v44.1〜v44.9 全機能動作確認）

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8
```

**2962 passed; 0 failed** を確認してから次ステップへ進む。v44100_tests 〜 v44900_tests が全件 pass していることが宣言の前提条件。

### Step 1: MILESTONE.md 更新

`# Favnir Milestones` タイトル行の直後、`## v44.0.0 — Language Expressiveness` セクションの直前に以下を挿入:

```markdown
## v45.0.0 — Precision & Flow（2026-07-15）

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

v45.0.0 をもって、Favnir の **Precision & Flow** を正式に宣言する。

### 達成コンポーネント（v44.1〜v44.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type × Streaming 統合 | v44.1 | collect_refinement_stream_bindings |
| CEP × Refinement type | v44.2 | collect_cep_refinement_event_refs |
| Stream join × Opaque type | v44.3 | collect_opaque_alias_groups |
| 型推論 × パイプライン lineage | v44.4 | collect_annotated_lineage_bindings |
| Back-pressure × fav policy 統合 | v44.5 | collect_stage_max_inflight_annotations |
| Precision & Flow E2E デモ | v44.6 | infra/e2e-demo/precision-flow/ |
| ドキュメントサイト概要ページ | v44.7 | precision-and-flow.mdx |
| パフォーマンス最終調整 | v44.8 | collect_bench_stream_notes + CHANGELOG |
| v45.0 前調整・安定化 | v44.9 | precision-and-flow-overview.mdx |

**宣言日**: 2026-07-15

---
```

### Step 2: README.md 更新

README.md の `v44.0（2026-07-13）` 記述行（line 116 付近）の直後に以下を挿入:

```markdown

**v45.0（2026-07-15）で、[Precision & Flow](./MILESTONE.md) マイルストーンを宣言しました。**
Refinement type × Streaming / CEP × Opaque type / Back-pressure / E2E デモが揃い、最小限の型注釈で安全なリアルタイムパイプラインを記述できる Precision & Flow 基盤が完成しました。
```

`"Precision & Flow"` および `"v45.0"` の両方が含まれるよう挿入する。

### Step 3: driver.rs に `v45000_tests` 追加 / スタブ化 / Cargo.toml バンプ

`v44900_tests` の直前に挿入:

```rust
// -- v45000_tests (v45.0.0) -- Precision & Flow 宣言 --
#[cfg(test)]
mod v45000_tests {
    #[test]
    fn cargo_toml_version_is_45_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"45.0.0\""), "Cargo.toml must contain version 45.0.0");
    }
    #[test]
    fn changelog_has_v45_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v45.0.0]"), "CHANGELOG.md must contain [v45.0.0]");
    }
    #[test]
    fn milestone_has_precision_and_flow() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Precision & Flow"),
            "MILESTONE.md must contain 'Precision & Flow'"
        );
    }
    #[test]
    fn readme_mentions_precision_and_flow() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Precision & Flow") || src.contains("v45.0"),
            "README.md must mention Precision & Flow or v45.0"
        );
    }
}
```

スタブ化: `v44900_tests::cargo_toml_version_is_44_9_0` の `assert!` 行のみを削除し以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 45.0.0 in v45.0.0.
```

`fav/Cargo.toml` version: `44.9.0` → `45.0.0`

### Step 4: CHANGELOG.md に v45.0.0 エントリ追加

`[v45.0.0]` を含む先頭エントリを追加。Precision & Flow 宣言・v44.1〜v44.9 全機能完成・★クリーンアップを記録。

### Step 5: テスト実行（2966 passed; 0 failed）

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8
```

### Step 6: ★クリーンアップ（`cargo clean`）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

テスト完了後に実施（テスト再実行なし）。

### Step 7: バージョン管理ドキュメント更新

- `versions/current.md` → v45.0.0 最新安定版（2966 tests）、次版（未確定の場合は `TBD` と記入）
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v45.0.0 を `✅ COMPLETE（2026-07-15）`
- `versions/v40-v45/v45.0.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 注意事項

- コードフリーズ: `collect_*` 系ヘルパーは追加しない
- `v44900_tests` にスタブ化対象 `cargo_toml_version_is_44_9_0` が存在することを事前確認すること
- `cargo clean` はテスト完了後に実施（テスト再実行なし）
- MILESTONE.md 挿入: `# Favnir Milestones` H1 の直後、`## v44.0.0` H2 の直前
- `readme_mentions_precision_and_flow` テストは OR 条件（`||`）。README.md には `"Precision & Flow"` と `"v45.0"` の両方を含む文字列を挿入するため実際には両方に合致するが、テストコードは OR のまま維持する（v44.0.0 の実績パターンと一致）
