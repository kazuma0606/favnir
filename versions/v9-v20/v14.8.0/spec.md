# v14.8.0 Spec — Rune ファイル --legacy 明示化 + fs.fav バグ修正

Date: 2026-06-12

---

## 目的

v14.7.0 の精査結果（12 ファイルで E0025 リスクあり）を受けて、v15.0.0 CrossCloud E2E デモの前に rune ライブラリの状態を整理する。

完全な ctx-aware 移行は影響範囲が大きく v15.0 後に行うため、本バージョンでは以下に絞る:

1. **fs.fav の `bind` 非 Result 誤用バグを修正**（高優先度）
2. **全 ambient rune ファイルに `// NOTE: --legacy compatible` コメントを追加**（現状を明文化）
3. CHANGELOG / README を v14.8.0 に更新
4. `v148000_tests` + バージョンバンプ

---

## 現状（v14.7.0 精査結果）

### fs.fav の バグ

```fav
// glob 関数内の問題箇所（v14.7.0 精査で発見）
bind sep      <- "/"                          // ← String は Result<_, _> でない → 非 Result bind
bind filtered <- List.filter(...)             // ← 同上
bind paths    <- List.map(...)                // ← 同上
```

`bind x <- expr` は `expr: Result<T, E>` に対して使う構文。
`String` や `List<T>` に対して使うのは誤り。`let` に変更すべきだが、
rune ファイル内で `let` を使うとパースエラーになるため、インライン化で対処する。

### ambient rune ファイル一覧（コメント追加対象）

| ファイル | 使用エフェクト |
|---|---|
| `runes/cache/cache.fav` | `!Cache` |
| `runes/fs/fs.fav` | `!IO` |
| `runes/log/emitter.fav` | `!Io` |
| `runes/log/metric.fav` | `!Io` |
| `runes/queue/queue.fav` | `!Queue` |
| `runes/gen/output.fav` | `!Io` / `!Db` |
| `runes/http/request.fav` | `!Network` / `!Http` |
| `runes/graphql/client.fav` | `!Http` |
| `runes/grpc/server.fav` | `!Io` / `!Rpc` |
| `runes/duckdb/query.fav` | `!Db` |
| `runes/duckdb/io.fav` | `!Db` |
| `runes/db/connection.fav` | `!Db` |

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `runes/fs/fs.fav` バグ修正 | `glob` 関数内の非 Result `bind` をインライン化 |
| 全 ambient rune コメント追加 | ファイル先頭に `// NOTE: --legacy compatible` を追記 |
| CHANGELOG.md 更新 | `## [v14.8.0]` エントリ追加 |
| README.md 更新 | 「現在の状態」を v14.8.0 に更新 |
| `v148000_tests` (3 件) | バージョン・fs.fav・rune コメント検証 |
| `Cargo.toml` バージョン `14.8.0` | |

### Out of Scope

- rune ファイルの ctx-aware 全面移行（影響範囲大 → v15.0 後に計画）
- 新 Capability interface の定義（`CacheCtx` / `QueueCtx` 等）
- site/ docs の追加更新

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `runes/fs/fs.fav` の `glob` で非 Result `bind` が存在しない | ✅ |
| 全 ambient rune ファイルに `--legacy compatible` コメントが存在する | ✅ |
| `CHANGELOG.md` に `[v14.8.0]` エントリが存在する | ✅ |
| `README.md` に `v14.8.0` の記述が存在する | ✅ |
| `cargo test v148000` 全 3 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.8.0"` | ✅ |
