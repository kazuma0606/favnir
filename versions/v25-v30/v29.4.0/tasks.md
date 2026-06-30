# v29.4.0 Tasks — vertex-ai / sagemaker Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.3.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2330 passed` を含むこと
- [x] `driver.rs` に `mod v294000_tests` が存在しないこと
- [x] `runes/vertex-ai/` ディレクトリが存在しないこと
- [x] `runes/sagemaker/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.3.0` → `29.4.0` | [x] |
| T2 | `runes/vertex-ai/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T3 | `runes/vertex-ai/vertex-ai.fav` 作成（4 関数）| [x] |
| T4 | `runes/sagemaker/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T5 | `runes/sagemaker/sagemaker.fav` 作成（3 関数）| [x] |
| T6 | `CHANGELOG.md` に `[v29.4.0]` セクション追加 | [x] |
| T7 | `benchmarks/v29.4.0.json` 作成（test_count: 2336）| [x] |
| T8 | `site/content/docs/runes/vertex-ai.mdx` 作成 | [x] |
| T9 | `site/content/docs/runes/sagemaker.mdx` 作成 | [x] |
| T10 | `driver.rs` に `v294000_tests` 6 件追加 | [x] |
| T11 | `cargo test --bin fav v294000` — 6/6 PASS 確認 | [x] |
| T12 | `cargo test --bin fav` — 2336 tests PASS 確認 | [x] |
| T13 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T10）

```rust
// v294000_tests (v29.4.0) -- vertex-ai / sagemaker Rune
#[cfg(test)]
mod v294000_tests {
    #[test]
    fn vertex_ai_rune_file_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("predict"),
            "runes/vertex-ai/vertex-ai.fav must define predict"
        );
    }
    #[test]
    fn vertex_ai_batch_and_deploy_fn_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("batch_predict") && src.contains("deploy_model"),
            "vertex-ai.fav must define batch_predict and deploy_model"
        );
    }
    #[test]
    fn vertex_ai_list_endpoints_fn_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("list_endpoints"),
            "vertex-ai.fav must define list_endpoints"
        );
    }
    #[test]
    fn sagemaker_rune_file_exists() {
        let src = include_str!("../../runes/sagemaker/sagemaker.fav");
        assert!(
            src.contains("invoke"),
            "runes/sagemaker/sagemaker.fav must define invoke"
        );
    }
    #[test]
    fn sagemaker_endpoint_fns_exist() {
        let src = include_str!("../../runes/sagemaker/sagemaker.fav");
        assert!(
            src.contains("create_endpoint") && src.contains("delete_endpoint"),
            "sagemaker.fav must define create_endpoint and delete_endpoint"
        );
    }
    #[test]
    fn changelog_has_v29_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.4.0]") || src.contains("## v29.4.0"),
            "CHANGELOG.md must contain '[v29.4.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.4.0"
- [x] `runes/vertex-ai/vertex-ai.fav` に `predict` / `batch_predict` / `deploy_model` / `list_endpoints` が存在する
- [x] `runes/vertex-ai/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `runes/sagemaker/sagemaker.fav` に `invoke` / `create_endpoint` / `delete_endpoint` が存在する
- [x] `runes/sagemaker/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `CHANGELOG.md` に `[v29.4.0]` セクションあり
- [x] `benchmarks/v29.4.0.json` 存在（test_count: 2336）
- [x] `site/content/docs/runes/vertex-ai.mdx` 存在
- [x] `site/content/docs/runes/sagemaker.mdx` 存在
- [x] `cargo test --bin fav v294000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2336 tests PASS
