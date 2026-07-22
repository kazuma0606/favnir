# Plan: v53.0.0 — Data Quality & Observability 2.0 宣言

---

## ステップ 1: `MILESTONE.md` 更新

先頭に v53.0.0 エントリを追加:

```markdown
## v53.0.0（2026-07-22）— Data Quality & Observability 2.0

> 「スキーマはランタイムで検証され、データの来歴はグラフで見え、
>  SLA 違反は即座に検知され、アクセスはすべて記録される。
>  Favnir のパイプラインは信頼できるデータを届ける。
>
>  これが Favnir v53.0 — Data Quality & Observability 2.0 の姿である。」

**Data Quality & Observability 2.0** の宣言バージョン。v52.1〜v52.9 の全機能統合を経て、
assert_schema・リネージ可視化・SLA 監視・audit-log・OTel 強化の成熟を宣言する。
```

`milestone_has_data_quality` テストが `"Data Quality & Observability 2.0"` を参照するため、
この文字列が必ず含まれていることを確認する。

---

## ステップ 2: `README.md` 更新

`readme_mentions_data_quality` テストが `"Data Quality"` を確認する。
README.md に v53.0.0 の言及を追加:

- バージョン記載箇所（例: 機能一覧・最新バージョン欄）に `v53.0` または `Data Quality` を追記する。

---

## ステップ 3: `driver.rs` — `v53000_tests` 追加

`v52900_tests` モジュールの直前に `v53000_tests` を追加:

```rust
// -- v53000_tests (v53.0.0) -- Data Quality & Observability 2.0 宣言 --
#[cfg(test)]
mod v53000_tests {
    #[test]
    fn cargo_toml_version_is_53_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"53.0.0\""), "Cargo.toml must be version 53.0.0");
    }

    #[test]
    fn changelog_has_v53_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v53.0.0"), "CHANGELOG.md must contain v53.0.0 entry");
    }

    #[test]
    fn milestone_has_data_quality() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Data Quality & Observability 2.0"),
            "MILESTONE.md must contain 'Data Quality & Observability 2.0'"
        );
    }

    #[test]
    fn readme_mentions_data_quality() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("Data Quality"),
            "README.md must mention 'Data Quality'"
        );
    }
}
```

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "52.9.0"` → `version = "53.0.0"`

**注意**: ステップ 3 後の `cargo build` 時点では `cargo_toml_version_is_53_0_0` および
`changelog_has_v53_0_0` の両テストが FAIL する（Cargo.toml / CHANGELOG.md が未更新のため）。
ステップ 4 + ステップ 5 を完了してから `cargo test` を実行すること（tasks.md T4 に対応）。

---

## ステップ 5: `CHANGELOG.md` 更新（tasks.md T4 の後半部分）

v53.0.0 エントリを追加（`changelog_has_v53_0_0` テストが `"v53.0.0"` を確認する）。
`changelog_has_v53_0_0` テストは `include_str!("../../CHANGELOG.md")` を参照するため、
この変更後に `changelog_has_v53_0_0` テストが pass できる状態になる。
tasks.md T4 では Cargo.toml 更新と CHANGELOG 更新を同一タスクにまとめており、
ステップ 4 と 5 を完了した後にテスト実行（ステップ 6）を行う。

---

## ステップ 6: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3160 passed, 0 failed（3156 + 4件追加）≥ 3157 ✓

---

## ステップ 7: ★クリーンアップ（`cargo clean`）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

**重要**: `cargo clean` 後は `fav/tmp/hello.fav` が消えるため必ず復元する。

復元内容（1行目・2行目）:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

復元後に再度 `cargo test` を実行して `bootstrap_c2_artifact_roundtrip` が pass することを確認する。

---

## ステップ 8: 後処理

- `versions/current.md` を v53.0.0（3160 tests）に更新
- `roadmap-v52.1-v53.0.md` の v53.0.0 実績欄を更新
- `tasks.md` を COMPLETE に更新（T0〜T6 全 `[x]`）
