# v62.9.0 Plan — 安定化・AOT E2E デモ

Version: 62.9.0
Status: 未着手

---

## 実装順序

### Step 1: `infra/e2e-demo/aot/` — デモ環境ファイル作成

以下の3ファイルを新規作成する：

1. `infra/e2e-demo/aot/src/pipeline.fav`
   - AOT 互換の純粋変換パイプライン（emit なし）
   - `OrderRow` → `SummaryRow` 変換 + `fn main() -> Bool`
2. `infra/e2e-demo/aot/scripts/build-aot.sh`
   - `fav build --link / --docker / --validate` の3ステップを実行するシェルスクリプト
   - `chmod +x` で実行権限付与
3. `infra/e2e-demo/aot/README.md`
   - デモの目的・使い方を記述

`cargo build` でエラーなし確認（まだ `include_str!` が追加されていないためコンパイルエラーは起きない）。

### Step 2: `site/content/docs/runtime/aot.mdx` — ドキュメント作成

`site/content/docs/runtime/` に `aot.mdx` を新規作成：
- フロントマター: `title`, `description`
- `fav build` コマンド一覧テーブル
- E0427 エラーの説明
- コード例 + bash 例

### Step 3: `driver.rs` — `v62900_tests` 追加

`v62800_tests` の直前（ファイル先頭方向）に挿入：

```rust
// -- v62900_tests (v62.9.0) -- AOT E2E デモ構造 + docs/runtime/aot.mdx --
#[cfg(test)]
mod v62900_tests {
    #[test]
    fn aot_e2e_demo_structure() {
        let src = include_str!("../../../infra/e2e-demo/aot/src/pipeline.fav");
        assert!(src.contains("pipeline") || src.contains("OrderRow"),
            "aot e2e demo pipeline.fav should define pipeline types");
        assert!(src.contains("SummaryRow"),
            "aot e2e demo pipeline.fav should define SummaryRow");
    }

    #[test]
    fn docs_aot_mdx_exists() {
        let mdx = include_str!("../../site/content/docs/runtime/aot.mdx");
        assert!(mdx.contains("AOT Compilation"),
            "aot.mdx should contain 'AOT Compilation'");
        assert!(mdx.contains("fav build"),
            "aot.mdx should mention 'fav build'");
        assert!(mdx.contains("E0427"),
            "aot.mdx should reference E0427");
    }
}
```

`cargo test v62900` で 2 件 PASS 確認。

### Step 4: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3402 tests passed, 0 failed を確認（実測ベース + 2）。

### Step 5: ドキュメント更新

roadmap / current.md / CHANGELOG.md / tasks.md を更新。

---

## 設計メモ

### E2E デモの最小性

`infra/e2e-demo/aot/` は「スクリプトが実際に動く」ことは要求しない（CI 環境依存）。
`include_str!` でファイルの存在と最低限の内容を確認することが目的。
Dockerfile / terraform は v62.9.0 スコープ外。

### `pipeline.fav` の設計方針

- `emit` を含まない → AOT 互換（`cmd_build_aot_validate` が "AOT compatibility check passed." を返す）
- 型定義 `OrderRow` / `SummaryRow` を含む → 型システムのデモとして意味のある内容
- `fn main() -> Bool { ... }` を含む → fav テスト実行可能

### ロードマップとの乖離

- ベーステスト数: ロードマップ記載 3398 → 実際 3400（v62.8.0 code-reviewer 対応 +2）
- ターゲット: 3400 + 2 = 3402（ロードマップ記載 3400 より +2）
- `fav build --validate` フラグの実際統合はスコープ外（スクリプトは `|| true` で失敗を無視）
