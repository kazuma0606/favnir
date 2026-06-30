# vX.Y.Z — 実装計画

## 前提確認

- [ ] vX.(Y-1).Z のテストが全件 PASS していること
- [ ] `fav/Cargo.toml` のバージョンが `X.(Y-1).Z` になっていること
- [ ] spec.md の依存関係が満たされていること

---

## 実装ステップ

### Step 0 — Cargo.toml バージョン更新

```toml
version = "X.Y.Z"
```

### Step 1 — [メイン実装]

**対象ファイル:**
- `fav/src/[path]`
- `runes/[rune-name]/`

**実装内容:**
- [具体的な変更点]

### Step 2 — テスト追加

**対象ファイル:**
- `fav/src/driver.rs`（`vXYZ00_tests` モジュール）

```rust
#[cfg(test)]
mod vXYZ00_tests {
    #[test]
    fn test_name() {
        // テストコード
    }
}
```

### Step 3 — ドキュメント更新

- [ ] `CHANGELOG.md` に `[vX.Y.Z]` エントリ追加
- [ ] `benchmarks/vX.Y.Z.json` 追加
- [ ] `site/content/docs/` 該当ページ更新（必要な場合）

### Step 4 — tasks.md 更新

全チェックボックスを `[x]` にして COMPLETE にする。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

---

## コードレビューチェックリスト

- [ ] セキュリティ: SQL インジェクション / XSS リスクなし
- [ ] エラーハンドリング: 外部 API 失敗時に型付きエラーを返す
- [ ] テスト: モックを使った自動テストが通る
- [ ] ドキュメント: 公開 API に使用例あり
