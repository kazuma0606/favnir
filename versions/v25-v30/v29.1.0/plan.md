# v29.1.0 Plan — `fav publish` 実装（Rune Registry 本番稼働）

## 実装順序

```
T1 Cargo.toml bump
  ↓
T2 driver.rs: cmd_publish に FAVNIR_REGISTRY_URL API 呼び出し追加
T3 driver.rs: cmd_search に FAVNIR_REGISTRY_URL フォールバック追加
T4 driver.rs: pub fn cmd_info 新規追加
T5 main.rs: Some("info") アーム追加
T6 driver.rs: cmd_login に GitHub OAuth URL 生成追加
  ↓（並行可）
T7 CHANGELOG.md 更新
T8 benchmarks/v29.1.0.json 新規作成
  ↓
T9 driver.rs: v291000_tests 6 件追加
  ↓
T9.5 cargo test --bin fav v291000 — 6/6 PASS
  ↓
T10 cargo test --bin fav 全体 — 2318 PASS
  ↓
T11 fav publish --dry-run / fav info postgres 動作確認
  ↓
T12 tasks.md COMPLETE 更新
```

## ファイル変更一覧

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `"29.0.0"` → `"29.1.0"` |
| `fav/src/driver.rs` | 更新 | `cmd_publish` API 呼び出し / `cmd_search` フォールバック / `cmd_info` 新規追加 / `cmd_login` OAuth URL / `v291000_tests` 6 件 |
| `fav/src/main.rs` | 更新 | `Some("info")` アーム追加 |
| `CHANGELOG.md` | 更新 | `[v29.1.0]` セクション追加 |
| `benchmarks/v29.1.0.json` | 新規 | `{"version": "29.1.0", "test_count": 2318}` |

## 注意事項

- `cmd_publish` の既存ローカルレジストリ書き込みは**削除しない**（`FAVNIR_REGISTRY_URL` 未設定時のフォールバック）
- `cmd_info` は `driver.rs` に `pub fn cmd_info(pkg_name: &str)` として追加する
- `main.rs` の `Some("info")` アームは `Some("search")` / `Some("add")` の近くに追加
- GitHub OAuth URL の `client_id` は `FAV_GITHUB_CLIENT_ID` 環境変数から取得（未設定時は `"favnir-registry"` デフォルト）
- `infra/registry/` は既存ファイルであり変更不要（T6 はテストで存在確認のみ）
- `v291000_tests` は `// ── v290000_tests` コメント行の直前に追加
