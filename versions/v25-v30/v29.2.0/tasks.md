# v29.2.0 Tasks — mlflow Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.1.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2318 passed` を含むこと
- [x] `driver.rs` に `mod v292000_tests` が存在しないこと
- [x] `runes/mlflow/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.1.0` → `29.2.0` | [x] |
| T2 | `runes/mlflow/rune.toml` 作成（`[rune]` セクションのみ）| [x] |
| T3 | `runes/mlflow/mlflow.fav` 作成（8 関数）| [x] |
| T4 | `CHANGELOG.md` に `[v29.2.0]` セクション追加 | [x] |
| T5 | `benchmarks/v29.2.0.json` 作成（test_count: 2324）| [x] |
| T6 | `site/content/docs/runes/mlflow.mdx` 作成 | [x] |
| T7 | `driver.rs` に `v292000_tests` 6 件追加 | [x] |
| T8 | `cargo test --bin fav v292000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2324 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T7）

```rust
// ── v292000_tests (v29.2.0) — mlflow Rune 追加 ──────────────────────────────────────────────────────
#[cfg(test)]
mod v292000_tests {
    #[test]
    fn mlflow_rune_file_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(
            src.contains("start_run"),
            "runes/mlflow/mlflow.fav must define start_run"
        );
    }
    #[test]
    fn mlflow_end_run_and_artifact_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(
            src.contains("end_run") && src.contains("log_artifact"),
            "mlflow.fav must define end_run and log_artifact"
        );
    }
    #[test]
    fn mlflow_log_metric_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("log_metric"), "mlflow.fav must define log_metric");
    }
    #[test]
    fn mlflow_log_param_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("log_param"), "mlflow.fav must define log_param");
    }
    #[test]
    fn mlflow_register_model_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("register_model"), "mlflow.fav must define register_model");
    }
    #[test]
    fn changelog_has_v29_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.2.0]") || src.contains("## v29.2.0"),
            "CHANGELOG.md must contain '[v29.2.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.2.0"
- [x] `runes/mlflow/mlflow.fav` に `start_run` / `log_metric` / `log_param` / `log_artifact` / `end_run` / `register_model` / `load_model` / `list_experiments` が存在する
- [x] `runes/mlflow/rune.toml` が存在する（`[rune]` セクションのみ）
- [x] `CHANGELOG.md` に `[v29.2.0]` セクションあり
- [x] `benchmarks/v29.2.0.json` 存在（test_count: 2324）
- [x] `site/content/docs/runes/mlflow.mdx` 存在
- [x] `cargo test --bin fav v292000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2324 tests PASS

---

## コードレビュー指摘対応

### spec-reviewer 指摘（実装前）
- [MED] `rune.toml` の `[connection]` セクションは非標準 → `[rune]` のみに統一
- [LOW] `end_run`/`log_artifact` 未検証 → `mlflow_end_run_and_artifact_fn_exists` テストに差し替え
- [LOW] spec.md 日付 → `2026-06-30` に更新

### トラブルシュート（cargo clean 後）
- `cargo clean` で `fav/tmp/hello.fav` が消失 → bootstrap_c2_artifact_roundtrip FAILED
- `hello.fav` 復元時の注意: compiler.fav は `= expr` 構文非対応、`{ body }` ブロック構文が必須
- 正しい hello.fav: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`

### code-reviewer 指摘（実装後）
- [MED] `load_model` のクエリパラメータ未エンコード → `Http.get_json` + クエリ文字列結合を `Http.post_json` + JSON body に変更（他7関数と一貫、v292000_tests 6/6 PASS 維持）
- [LOW] `Env.get_or(...)` 8関数で重複 → HTTP 有効化時にヘルパー関数化予定（現スコープ外）
- [LOW] v292000_tests コメント行の Unicode が `?` に化け → 機能に影響なし（次回は Write ツールで直接挿入）
