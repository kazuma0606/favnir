# Plan: v92.5.0 — W060 N+1 lint ルール追加

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4105 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92400_tests` が存在することを確認
- `runes/sap-odata/query_builder.fav` に `public type Page` が含まれることを確認（v92.4.0 完了済みの証拠）
- `runes/sap-odata/query_builder.fav` に `public fn fetch_all_pages` が含まれることを確認（v92.4.0 完了済みの証拠）
- `fav/tmp/hello.fav` が存在することを確認
- `fav/src/lint.rs` を Read し、既存 lint ルール（W001〜W059）の実装パターンを確認する
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `lint.rs` に W060 ルールを追加

既存 lint ルールの最後（W059 の直後）に W060 を追加する。

実装パターン（既存ルールに合わせて調整すること）：

```rust
// W060: N+1 クエリ検出（List.map / List.flat_map コールバック内の ctx.sap.* 呼び出し）
// lint.rs の check 関数 / メッセージ文字列を既存パターンに従って追加
// キーワード: "W060", "N+1"
```

W060 の検出ロジック：
1. AST を走査し `List.map` / `List.flat_map` の呼び出し式を探す
2. そのコールバック引数（`fn(_) { ... }` 形式）の本体に `ctx.sap.` へのアクセスがあるか検索する
3. 検出した場合 W060 警告を発行する

警告メッセージに `"W060"` と `"N+1"` を含めること（テストが文字列を検証するため）。

### Step 2: `driver.rs` に `mod v92500_tests` を追加

`mod v92400_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92500_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn w060_lint_rule_defined() {
        let content = std::fs::read_to_string("src/lint.rs")
            .expect("src/lint.rs should exist");
        assert!(
            content.contains("W060"),
            "lint.rs should define W060 lint rule"
        );
    }
    #[test]
    fn w060_lint_message_mentions_n_plus_1() {
        let content = std::fs::read_to_string("src/lint.rs")
            .expect("src/lint.rs should exist");
        assert!(
            content.contains("N+1"),
            "lint.rs W060 message should mention N+1"
        );
    }
}
```

注意: `lint.rs` は `fav/src/` にあるため、パスは `"src/lint.rs"`（`../` は不要）。

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4107 passed; 0 failed`

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック + lint.rs 構造確認）
  → Step 1（lint.rs に W060 追加）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
