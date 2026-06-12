# v14.8.0 Tasks — Rune ファイル --legacy 明示化 + fs.fav バグ修正

Date: 2026-06-12
Branch: master

---

## Phase A — `runes/fs/fs.fav` バグ修正

- [ ] A-1: `glob` 関数内の非 Result `bind` をインライン化
  - 削除: `bind sep <- "/"` / `bind filtered <- ...` / `bind paths <- ...`
  - 修正後: `List.filter` + `List.map` をネストしてインライン化
  - 本文は `plan.md` Phase A-1 参照

---

## Phase B — ambient rune ファイルに `--legacy compatible` コメントを追加

- [ ] B-1: `runes/cache/cache.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-2: `runes/fs/fs.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-3: `runes/log/emitter.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-4: `runes/log/metric.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-5: `runes/queue/queue.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-6: `runes/gen/output.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-7: `runes/http/request.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-8: `runes/graphql/client.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-9: `runes/grpc/server.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-10: `runes/duckdb/query.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-11: `runes/duckdb/io.fav` — ヘッダに `--legacy compatible` コメント追加
- [ ] B-12: `runes/db/connection.fav` — ヘッダに `--legacy compatible` コメント追加

---

## Phase C — `CHANGELOG.md` 更新

- [ ] C-1: `## [v14.8.0]` エントリを追加（`## [v14.7.0]` の直前）
  - Changed: fs.fav バグ修正、12 rune ファイルに --legacy コメント追加
  - 本文は `plan.md` Phase C 参照

---

## Phase D — `README.md` 更新

- [ ] D-1: 「現在の状態」見出しを `v14.8.0` に更新
  - テスト件数: `1540+` に更新
- [ ] D-2: ロードマップ表に `v14.8.0` 行を追記（`v14.7.0` 行の直後）

---

## Phase E — `fav/src/driver.rs`: v148000_tests + バージョンバンプ

- [ ] E-1: `v148000_tests` モジュールを追加（`v147000_tests` の直前）
  - [ ] `version_is_14_8_0` — `CARGO_PKG_VERSION == "14.8.0"` 確認
  - [ ] `fs_rune_glob_no_bind_string` — `runes/fs/fs.fav` に `"bind sep"` が含まれない
  - [ ] `ambient_runes_have_legacy_comment` — cache/emitter/queue の 3 件にコメント存在確認

  テスト本文は `plan.md` Phase E-1 参照。

- [ ] E-2: `v147000_tests` の `version_is_14_7_0` を `>=` 比較に修正

- [ ] E-3: `fav/Cargo.toml` バージョンを `"14.8.0"` にバンプ

- [ ] E-4: `cargo test v148000` で 3 件全パス確認

---

## Phase F — 全テスト + コミット

- [ ] F-1: `cargo test v148000` 全 3 件パス
- [ ] F-2: `cargo test` 全件パス（リグレッションなし）
- [ ] F-3: `git commit -m "feat: v14.8.0 — rune ファイル整備（--legacy 明示化 + fs.fav バグ修正）"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `runes/fs/fs.fav` の `glob` で `bind sep` が存在しない | [ ] |
| ambient rune ファイル 12 件に `--legacy compatible` コメントが存在 | [ ] |
| `CHANGELOG.md` に `[v14.8.0]` エントリが存在 | [ ] |
| `README.md` に `v14.8.0` の記述が存在 | [ ] |
| `cargo test v148000` 全 3 件パス | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `CARGO_PKG_VERSION == "14.8.0"` | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.8.0/spec.md` | 仕様・スコープ |
| `versions/v14.8.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/v14.7.0/tasks.md` | 積み残しテーブル（精査結果） |
| `versions/roadmap-v14.1-v15.0.md` | v14.8.0 の位置づけ・v15.0.0 との関係 |
